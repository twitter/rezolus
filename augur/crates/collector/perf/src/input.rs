//! Format for JSON samples emitted by perf-script.py

use std::borrow::Cow;

use bstr::BStr;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct PerfFrameSym<'a> {
    pub start: u64,
    pub binding: u32,
    pub end: u64,
    pub name: Option<Cow<'a, BStr>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct PerfFrame<'a> {
    pub ip: u64,
    pub sym: Option<PerfFrameSym<'a>>,
    pub dso: Option<Cow<'a, BStr>>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PerfSample {
    pub pid: u32,
    pub tid: u32,
    #[allow(dead_code)]
    pub period: u64,
    pub time: u64,
    pub cpu: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub(crate) struct PerfEvent<'a> {
    pub sample: PerfSample,
    pub comm: Cow<'a, BStr>,
    pub callchain: Vec<PerfFrame<'a>>,
}

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE: &str = include_str!("example-sample.json");

    #[test]
    fn test_deserialize() {
        let event: PerfEvent =
            serde_json::from_str(SAMPLE) //
                .expect("Unable to deserialize PerfEvent from test sample");

        assert_eq!(*event.comm, "perf");
    }
}
