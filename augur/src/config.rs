use augur_emitter_kafka::KafkaConfig;
use serde::Deserialize;

fn default_metrics_address() -> String {
    "0.0.0.0".to_owned()
}

fn default_metrics_port() -> u16 {
    4243
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct GeneralConfig {
    pub frequency: u32,
    pub period: u32,
    #[serde(default)]
    pub debug: DebugMode,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_address")]
    pub addr: String,
    #[serde(default = "default_metrics_port")]
    pub port: u16,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            addr: default_metrics_address(),
            port: default_metrics_port(),
        }
    }
}

#[derive(Default, Clone, Debug, Deserialize)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub general: GeneralConfig,
    #[serde(default = "Default::default")]
    pub metrics: MetricsConfig,
}

#[derive(Clone, Copy, Debug)]
pub enum DebugMode {
    /// Output gathered samples to the configured kafka topic
    Production,
    /// Output gathered samples to the terminal
    Terminal,
    /// Don't output samples anywhere
    Quiet,
}

impl Default for DebugMode {
    fn default() -> Self {
        Self::Production
    }
}

impl<'de> Deserialize<'de> for DebugMode {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        Ok(match <&str>::deserialize(de)? {
            "none" | "prod" => DebugMode::Production,
            "terminal" => DebugMode::Terminal,
            "quiet" => DebugMode::Quiet,
            x => {
                return Err(Error::unknown_variant(
                    x,
                    &["none", "prod", "terminal", "quiet"],
                ))
            }
        })
    }
}
