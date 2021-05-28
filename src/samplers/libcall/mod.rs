use async_trait::async_trait;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::common::bpf::BPF;
use crate::config::SamplerConfig;
use crate::samplers::{Common, Sampler};

#[cfg(feature = "bpf")]
use crate::common::bpf::bpf_hash_char_to_map;
#[cfg(feature = "bpf")]
use rustcommon_metrics::Statistic;

mod config;
mod stat;

pub use config::LibCallConfig;
pub use stat::LibCallStatistic;

use std::path::Path;

#[allow(dead_code)]
pub struct LibCall {
    bpf: Option<Arc<Mutex<BPF>>>,
    bpf_last: Arc<Mutex<Instant>>,
    common: Common,
    probe_funcs: HashMap<String, Vec<String>>,
    statistics: Vec<LibCallStatistic>,
    lib_paths: Vec<String>,
}

#[allow(dead_code)]
const PROBE_PRELUDE: &str = r#"
#include <uapi/linux/ptrace.h>
struct key_t {
    char c[80];
};
BPF_HASH(counts, struct key_t);

"#;

macro_rules! probe_template {
    () => {
        r#"

int probe_{}(void *ctx) {{
    struct key_t key = {{.c = "{}"}};
    u64 zero = 0, *val;
    val = counts.lookup_or_init(&key, &zero);
    (*val)++;
    return 0;
}}
"#
    };
}

#[allow(dead_code)]
fn path_match(lib_name: &str, path: &Path) -> bool {
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
            return to_test.eq(lib_name) && "so".eq(&ext_str[..]);
        }
    }
    false
}

impl LibCall {
    fn init_bpf(&mut self) -> Result<(), anyhow::Error> {
        let default_paths = vec![
            "/lib64".into(),
            "/usr/lib64".into(),
            "/usr/local/lib64".into(),
            "/lib".into(),
            "/usr/lib".into(),
            "/usr/local/lib".into(),
        ];
        let to_search: Vec<String> = [&self.lib_paths[..], &default_paths[..]].concat();
        let entries: Vec<walkdir::DirEntry> = to_search
            .iter()
            .map(|p| {
                walkdir::WalkDir::new(p)
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
            })
            .flatten()
            .collect();

        let mut bpf_probes = String::new();
        let mut found_probes: Vec<(String, &str, &str)> = Vec::new();
        for (lib, funcs) in &self.probe_funcs {
            for entry in &entries {
                if path_match(&lib, entry.path()) {
                    for func in funcs.iter() {
                        bpf_probes.push_str(&format!(
                            probe_template!(),
                            found_probes.len(),
                            format!("{}/{}", lib, func)
                        ));
                        found_probes.push((entry.path().to_string_lossy().to_string(), lib, func));
                    }
                    break;
                }
            }
        }
        #[cfg(feature = "bpf")]
        {
            info!("Registering probes: {:?}", found_probes);
            let bpf_prog = PROBE_PRELUDE.to_string() + &bpf_probes;
            let mut bpf = bcc::BPF::new(&bpf_prog)?;
            let mut i = 0;
            for (path, _lib, func) in found_probes.iter() {
                bcc::Uprobe::new()
                    .handler(&format!("probe_{}", i))
                    .binary(path)
                    .symbol(func)
                    .attach(&mut bpf)?;
                i += 1;
            }

            self.bpf = Some(Arc::new(Mutex::new(BPF { inner: bpf })));
        }

        Ok(())
    }
}

#[async_trait]
impl Sampler for LibCall {
    type Statistic = LibCallStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().libcall().statistics();
        let probe_funcs = common.config().samplers().libcall().probe_funcs();
        let lib_paths = common.config().samplers().libcall().lib_paths();

        let mut sampler = Self {
            bpf: None,
            bpf_last: Arc::new(Mutex::new(Instant::now())),
            common,
            statistics,
            lib_paths,
            probe_funcs,
        };
        if sampler.sampler_config().enabled() {
            sampler.init_bpf().unwrap();
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

        #[cfg(feature = "bpf")]
        if let Some(ref bpf) = self.bpf {
            let bpf = bpf.lock().unwrap();
            let table = (*bpf).inner.table("counts").unwrap();
            let stat_map = bpf_hash_char_to_map(&table);
            for stat in self.statistics.iter() {
                let val = stat_map.get(&stat.name().to_string()).unwrap_or(&0);
                self.metrics()
                    .record_counter(stat, Instant::now(), *val)
                    .unwrap();
            }
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
                assert_eq!(expected, path_match(to_test, p));
            }
        )*
        }
    }

    path_tests! {
        path_1: ("test", "test.so", true),
        path_2: ("pam", "test.so", false),
        path_3: ("pam", "libpam.so", true),
        path_4: ("krb5", "libkrb5.so.3.3", true),
        path_5: ("pthread-2.17", "/usr/bin/libpthread-2.17.so", true),
        path_6: ("krb5", "/usr/lib64/libkrb5.so.3.3", true),
    }
}
