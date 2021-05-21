use async_trait::async_trait;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use walkdir::WalkDir;

use crate::common::bpf::BPF;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};

mod config;
mod stat;

pub use config::LibCallConfig;
pub use stat::LibCallStatistic;

#[allow(dead_code)]
pub struct LibCall {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    statistics: Vec<LibCallStatistic>,
    probe_funcs: HashMap<String, Vec<String>>,
    lib_paths: Vec<String>,
}

const _PROBE_TEMPLATE: &str = r#"
int {}(void *ctx) {
    int loc = {};
    u64 *val = counts.lookup(&loc);
    if (!val) {
        return 0;   // Should never happen, # of locations is known
    }
    (*val)++;
    return 0;
}
"#;

fn path_match(lib_names: &[String], path: &Path) -> bool {
    if let Some(file_name) = path.file_name() {
        if let Some(file_str) = file_name.to_str() {
            let parts: Vec<&str> = file_str.split('.').collect();
            if parts.len() < 2 {
                return false;
            }
            let mut stem_str: String = parts[0].to_string();
            let mut ext_str: String = parts[1].to_string();
            let end_index = parts.len() - 1;
            if parts[end_index] == "so" {
                stem_str = parts[..end_index].join(".");
                ext_str = "so".into();
            }
            let to_test = match stem_str.starts_with("lib") {
                true => stem_str[3..].into(),
                false => stem_str,
            };
            return lib_names.contains(&to_test) && "so".eq(&ext_str[..]);
        }
    }
    false
}

impl LibCall {
    fn init_bpf(&self) {
        let default_paths = vec![
            "/lib64".into(),
            "/usr/lib64".into(),
            "/usr/local/lib64".into(),
            "/lib".into(),
            "/usr/lib".into(),
            "/usr/local/lib".into(),
        ];
        let to_search: Vec<String> = [&self.lib_paths[..], &default_paths[..]].concat();
        // let bpf_prog = String::new();
        for path in to_search.iter() {
            for (lib, funcs) in &self.probe_funcs {
                info!("Searching for {}, {:?}", lib, funcs);
                for entry in WalkDir::new(path)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    path_match(&to_search, entry.path());
                }
            }
        }
    }
}

#[async_trait]
impl Sampler for LibCall {
    type Statistic = LibCallStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().libcall().statistics();
        let probe_funcs = common.config().samplers().libcall().probe_funcs();
        let lib_paths = common.config().samplers().libcall().lib_paths();
        let sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics,
            lib_paths,
            probe_funcs,
        };
        if sampler.sampler_config().enabled() {
            sampler.init_bpf();
            sampler.register();
        }
        Ok(sampler)
    }

    fn spawn(common: Common) {
        if common.config().samplers().libcall().enabled() {
            if let Ok(mut sampler) = Self::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = sampler.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize libcall sampler");
            } else {
                error!("failed to initialize libcall sampler");
            }
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().libcall()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }
        for statistic in self.statistics.iter() {
            let _ = self.metrics().record_counter(statistic, Instant::now(), 1);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! path_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (to_test, path, expected) = $value;
                let p = Path::new(path);
                let test_vec : Vec<String> = to_test.iter().map(|s: &&str| s.to_string()).collect();
                assert_eq!(expected, path_match(&test_vec, p));
            }
        )*
        }
    }

    path_tests! {
        path_0: (vec![], "test.so", false),
        path_1: (vec!["test"], "test.so", true),
        path_2: (vec!["one", "two", "three", "test"], "test.so", true),
        path_3: (vec!["pam"], "test.so", false),
        path_4: (vec!["test", "pam"], "libpam.so", true),
        path_5: (vec!["test", "krb5"], "libkrb5.so.3.3", true),
        path_6: (vec!["test", "pthread"], "libpthread-2.17.so", false),
        path_7: (vec!["test", "pthread-2.17"], "libpthread-2.17.so", true),
    }
}
