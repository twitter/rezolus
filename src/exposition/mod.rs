// Copyright 2019 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use rustcommon_metrics::*;

use crate::metrics::{Counter, Gauge, SampledHeatmap};

mod http;
#[cfg(feature = "push_kafka")]
mod kafka;

pub use self::http::Http;
#[cfg(feature = "push_kafka")]
pub use self::kafka::KafkaProducer;

macro_rules! downcast_match {
    {
        $any:expr => {
            $( $name:ident @ $ty:ty => $stmt:expr, )*
            $( _ => $default:expr )?
        }
    } => {{
        let ref any = $any;

        match () {
            $(
                _ if any.is::<$ty>() => match any.downcast_ref::<$ty>() {
                    Some($name) => { $stmt },
                    None => unreachable!()
                }
            ),*
            () => { $( $default )? }
        }
    }};
}

pub struct MetricsSnapshot {
    metrics: Arc<Metrics<AtomicU64, AtomicU32>>,
    snapshot: HashMap<Metric<AtomicU64, AtomicU32>, u64>,
    refreshed: Instant,
    count_label: Option<String>,
}

impl MetricsSnapshot {
    pub fn new(metrics: Arc<Metrics<AtomicU64, AtomicU32>>, count_label: Option<&str>) -> Self {
        Self {
            metrics,
            snapshot: HashMap::new(),
            refreshed: Instant::now(),
            count_label: count_label.map(std::string::ToString::to_string),
        }
    }

    pub fn refresh(&mut self) {
        self.snapshot = self.metrics.snapshot();
        self.refreshed = Instant::now();
    }

    pub fn prometheus(&self) -> String {
        let mut data = Vec::new();

        for metric in &rustcommon_metrics_v2::metrics() {
            let any = match metric.as_any() {
                Some(any) => any,
                None => continue,
            };

            downcast_match! { any => {
                counter @ Counter => {
                    data.push(format!("# TYPE {0} gauge\n{0} {1}", metric.name(), counter.value()));
                },
                gauge @ Gauge => {
                    data.push(format!("# TYPE {0} gauge\n{0} {1}", metric.name(), gauge.value()));
                },
                heatmap @ SampledHeatmap => {
                    for percentile in heatmap.percentiles() {
                        // SampledHeatmaps are by default named as <thing>/histogram
                        // However, prometheus wants this to be exported as <thing>{{..}}
                        let name = metric.name().trim_end_matches("/histogram");

                        data.push(format!(
                            "# TYPE {0} gauge\n{0}{{percentile=\"{1:02}\"}} {2}",
                            name, percentile, heatmap.percentile(*percentile).unwrap_or(0)
                        ));
                    }
                },
                _ => ()
            }}
        }

        for (metric, value) in &self.snapshot {
            let label = metric.statistic().name();
            let output = metric.output();
            match output {
                Output::Reading => {
                    data.push(format!("# TYPE {} gauge\n{} {}", label, label, value));
                }
                Output::Percentile(percentile) => {
                    data.push(format!(
                        "# TYPE {} gauge\n{}{{percentile=\"{:02}\"}} {}",
                        label, label, percentile, value
                    ));
                }
            }
        }
        data.sort();
        let mut content = data.join("\n");
        content += "\n";
        let parts: Vec<&str> = content.split('/').collect();
        parts.join("_")
    }

    pub fn human(&self) -> String {
        let mut data = Vec::new();

        for metric in &rustcommon_metrics_v2::metrics() {
            let any = match metric.as_any() {
                Some(any) => any,
                None => continue,
            };

            downcast_match! { any => {
                counter @ Counter => {
                    data.push(match &self.count_label {
                        Some(count_label) => format!(
                            "{}/{}: {}",
                            metric.name(),
                            count_label,
                            counter.value()
                        ),
                        None => format!("{}: {}", metric.name(), counter.value())
                    })
                },
                gauge @ Gauge => {
                    data.push(match &self.count_label {
                        Some(count_label) => format!(
                            "{}/{}: {}",
                            metric.name(),
                            count_label,
                            gauge.value()
                        ),
                        None => format!("{}: {}", metric.name(), gauge.value())
                    })
                },
                heatmap @ SampledHeatmap => {
                    for percentile in heatmap.percentiles() {
                        data.push(format!(
                            "{}/p{:02}: {}",
                            metric.name(), percentile, heatmap.percentile(*percentile).unwrap_or(0)
                        ));
                    }
                },
                _ => ()
            }}
        }

        for (metric, value) in &self.snapshot {
            let label = metric.statistic().name();
            let output = metric.output();
            match output {
                Output::Reading => {
                    if let Some(ref count_label) = self.count_label {
                        data.push(format!("{}/{}: {}", label, count_label, value));
                    } else {
                        data.push(format!("{}: {}", label, value));
                    }
                }
                Output::Percentile(percentile) => {
                    data.push(format!("{}/histogram/p{:02}: {}", label, percentile, value));
                }
            }
        }
        data.sort();
        let mut content = data.join("\n");
        content += "\n";
        content
    }

    fn json(&self, pretty: bool) -> String {
        let mut head = "{".to_owned();
        if pretty {
            head += "\n  ";
        }
        let mut data = Vec::new();

        for metric in &rustcommon_metrics_v2::metrics() {
            let any = match metric.as_any() {
                Some(any) => any,
                None => continue,
            };

            downcast_match! { any => {
                counter @ Counter => {
                    data.push(match &self.count_label {
                        Some(count_label) => format!(
                            "\"{}/{}\": {}",
                            metric.name(),
                            count_label,
                            counter.value()
                        ),
                        None => format!("\"{}\": {}", metric.name(), counter.value())
                    })
                },
                gauge @ Gauge => {
                    data.push(match &self.count_label {
                        Some(count_label) => format!(
                            "\"{}/{}\": {}",
                            metric.name(),
                            count_label,
                            gauge.value()
                        ),
                        None => format!("\"{}\": {}", metric.name(), gauge.value())
                    })
                },
                heatmap @ SampledHeatmap => {
                    for percentile in heatmap.percentiles() {
                        data.push(format!(
                            "\"{}/p{:02}\": {}",
                            metric.name(), percentile, heatmap.percentile(*percentile).unwrap_or(0)
                        ));
                    }
                },
                _ => ()
            }}
        }

        for (metric, value) in &self.snapshot {
            let label = metric.statistic().name();
            let output = metric.output();
            match output {
                Output::Reading => {
                    if let Some(ref count_label) = self.count_label {
                        data.push(format!("\"{}/{}\": {}", label, count_label, value));
                    } else {
                        data.push(format!("\"{}\": {}", label, value));
                    }
                }
                Output::Percentile(percentile) => {
                    data.push(format!(
                        "\"{}/histogram/p{:02}\": {}",
                        label, percentile, value
                    ));
                }
            }
        }
        data.sort();
        let body = if pretty {
            data.join(",\n  ")
        } else {
            data.join(",")
        };
        let mut content = head;
        content += &body;
        if pretty {
            content += "\n";
        }
        content += "}";
        content
    }
}
