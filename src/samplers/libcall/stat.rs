use rustcommon_metrics::{AtomicU32, AtomicU64, Source, Statistic};

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LibCallStatistic {
    stat_path: String,
}

impl LibCallStatistic {
    pub fn new(library: &str, func: &str) -> Self {
        Self {
            stat_path: format!("{}/{}", library, func),
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for LibCallStatistic {
    fn name(&self) -> &str {
        &self.stat_path
    }

    fn source(&self) -> Source {
        Source::Counter
    }
}
