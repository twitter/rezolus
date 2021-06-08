// Copyright 2021 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use serde_derive::Deserialize;
use std::collections::BTreeMap;

use crate::config::SamplerConfig;

use super::stat::UsercallStatistic;

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct LibraryProbeConfig {
    pub name: String,
    pub path: Option<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UsercallConfig {
    #[serde(default)]
    bpf: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    interval: Option<usize>,
    #[serde(default)]
    percentiles: Vec<f64>,
    #[serde(default)]
    libraries: Vec<LibraryProbeConfig>,
}

impl UsercallConfig {
    pub fn libraries(&self) -> Vec<LibraryProbeConfig> {
        let mut lib_map: BTreeMap<String, BTreeMap<Option<String>, LibraryProbeConfig>> =
            BTreeMap::new();
        for lib_conf in self.libraries.iter() {
            if lib_conf.name == "".to_string() {
                warn!("Skipping library config without a name: {:?}", lib_conf);
                continue;
            }

            match lib_map.get_mut(&lib_conf.name) {
                Some(file_map) => {
                    if let Some(conf) = file_map.get(&None) {
                        warn!(
                            "Removing duplicate user call library search path: {:?}",
                            conf
                        );
                        file_map.remove(&None);
                    }
                    if lib_conf.path.is_some() && !file_map.contains_key(&lib_conf.path) {
                        file_map.insert(lib_conf.path.clone(), lib_conf.clone());
                    } else {
                        warn!("Removing duplicate user call config: {:?}", lib_conf);
                    }
                }
                None => {
                    let mut to_add = BTreeMap::new();
                    to_add.insert(lib_conf.path.clone(), lib_conf.clone());
                    lib_map.insert(lib_conf.name.clone(), to_add);
                }
            };
        }
        lib_map
            .values()
            .map(|m| m.values())
            .flatten()
            .map(|x| x.clone())
            .collect()
    }
}

impl SamplerConfig for UsercallConfig {
    type Statistic = UsercallStatistic;

    fn bpf(&self) -> bool {
        self.bpf
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn interval(&self) -> Option<usize> {
        self.interval
    }

    fn percentiles(&self) -> &[f64] {
        &self.percentiles
    }

    fn statistics(&self) -> Vec<<Self as SamplerConfig>::Statistic> {
        let mut stats = Vec::new();
        for lib_conf in self.libraries().iter() {
            for func in lib_conf.functions.iter() {
                stats.push(UsercallStatistic::new(&lib_conf.name, &func));
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! dedup_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, expected) : (Vec<LibraryProbeConfig>, Vec<LibraryProbeConfig>) = $value;
                let mut config = UsercallConfig::default();
                config.libraries = input;
                assert_eq!(expected, config.libraries());
            }
        )*
        }
    }

    macro_rules! vals {
        ($lt:literal) => {
            match ($lt) {
                "cTmp" => LibraryProbeConfig {
                    path: Some("/tmp".into()),
                    name: "c".into(),
                    functions: vec![],
                },
                "cUsr" => LibraryProbeConfig {
                    path: Some("/usr".into()),
                    name: "c".into(),
                    functions: vec![],
                },
                "cSearch" => LibraryProbeConfig {
                    path: None,
                    name: "c".into(),
                    functions: vec![],
                },
                _ => LibraryProbeConfig::default(),
            }
        };
    }

    dedup_tests! {
        dedup_1: (vec![vals!("")], vec![]),
        dedup_2: (vec![vals!("cTmp")], vec![vals!("cTmp")]),
        dedup_3: (vec![vals!("cUsr"), vals!("cTmp")], vec![vals!("cTmp"), vals!("cUsr")]),
        dedup_4: (vec![vals!("cUsr"), vals!("cTmp"), vals!("cSearch")], vec![vals!("cTmp"), vals!("cUsr")]),
        dedup_5: (vec![vals!("cSearch"), vals!("cTmp"), vals!("cUsr")], vec![vals!("cTmp"), vals!("cUsr")]),
        dedup_6: (vec![vals!("cTmp"), vals!("cTmp"), vals!("cUsr")], vec![vals!("cTmp"), vals!("cUsr")]),
    }
}
