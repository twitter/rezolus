//! Utilities for generating serialized pprof profiles.
//!
//! This crate has two main parts:
//! - Generated protobuf bundings under the [`proto`] module, and
//! - The [`ProfileBuilder`] type for incrementally assembling a pprof profile.

use std::hash::Hash;
use std::time::SystemTime;

use augur_common::{Frame, Sample};
use bstr::{BString, ByteSlice, ByteVec};
use fxhash::FxHashMap;

#[allow(rustdoc::broken_intra_doc_links)]
mod gen;

pub extern crate protobuf;

/// Generated
pub mod proto {
    pub use crate::gen::pprof::*;
}

struct Interner<T> {
    values: Vec<T>,
    map: FxHashMap<T, usize>,
}

impl<T> Interner<T>
where
    T: Default + Clone + Hash + Eq,
{
    pub fn new() -> Self {
        let mut this = Self::empty();
        this.intern(T::default());
        this
    }

    pub fn empty() -> Self {
        Self {
            values: Vec::new(),
            map: Default::default(),
        }
    }
}

impl<T> Interner<T>
where
    T: Clone + Hash + Eq,
{
    pub fn intern(&mut self, value: T) -> usize {
        *self.map.entry(value).or_insert_with_key(|key| {
            let index = self.values.len();
            self.values.push(key.clone());
            index
        })
    }

    pub fn into_vec(self) -> Vec<T> {
        self.values
    }
}

impl Interner<String> {
    pub fn intern_string(&mut self, string: impl Into<BString>) -> usize {
        self.intern(Vec::from(string.into()).into_string_lossy())
    }
}

/// Builder for assembling [`Profile`]s.
///
/// [`Profile`]: crate::proto::Profile
pub struct ProfileBuilder {
    strings: Interner<String>,
    locations: Interner<Location>,
    functions: Interner<Function>,
    start: Option<SystemTime>,
    last: SystemTime,

    samples: Vec<proto::Sample>,
}

impl ProfileBuilder {
    /// Create a new, empty, profile.
    pub fn new() -> Self {
        Self {
            strings: Interner::new(),
            locations: Interner::new(),
            functions: Interner::new(),
            samples: Vec::new(),
            last: SystemTime::UNIX_EPOCH,
            start: None,
        }
    }

    /// Get the number of samples that have been inserted into this builder.
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Add a new sample.
    pub fn add(&mut self, sample: Sample) {
        let mut proto = proto::Sample::new();

        // Sample weight is in nanoseconds, convert it to microseconds.
        proto.value.push((sample.weight / 1000) as _);

        self.start.get_or_insert(sample.time);
        self.last = sample.time;

        // proto
        //     .label
        //     .push(self.label_num("pid", sample.pid as _, "id"));
        // proto
        //     .label
        //     .push(self.label_num("tid", sample.tid as _, "id"));
        // proto
        //     .label
        //     .push(self.label_num("cpu", sample.cpu as _, "id"));

        let ts = sample
            .time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        proto
            .label
            .push(self.label_num("timestamp", ts.as_micros() as u64, "microseconds"));

        if let Some(hostname) = &sample.hostname {
            proto.label.push(self.label_string("hostname", hostname));
        }
        if let Some(command) = &sample.command {
            proto.label.push(self.label_string("command", command))
        }
        if let Some(thread_name) = &sample.thread_name {
            proto
                .label
                .push(self.label_string("thread_name", thread_name));
        }

        if let Some(aurora) = &sample.aurora {
            if let Some(service) = &aurora.service_name {
                proto.label.push(self.label_string("job", service));
            }

            if let Some(instance) = aurora.instance_id {
                proto
                    .label
                    .push(self.label_string("instance_id", instance.to_string().as_bytes()));
            }
        }

        if let Some(systemd) = &sample.systemd {
            if let Some(unit) = &systemd.unit {
                proto.label.push(self.label_string("systemd.unit", &unit));
            }

            if let Some(slice) = &systemd.slice {
                proto.label.push(self.label_string("systemd.slice", &slice));
            }
        }

        let aurora_source = sample.aurora.as_ref().map(|aurora| &aurora.source);
        let systemd_unit = sample
            .systemd
            .as_ref()
            .and_then(|systemd| systemd.unit.as_ref());

        if let Some(source) = aurora_source.or(systemd_unit) {
            proto.label.push(self.label_string("source", source));
        }

        // Frame 0 is supposed to be the leaf node so reverse the order
        proto.location_id = sample
            .frames
            .into_iter()
            .map(|frame| self.location(frame))
            .collect();

        self.samples.push(proto);
    }

    /// Build the final pprof [`Profile`] object suitable for serializing
    /// to a file/over the network/wherever.
    ///
    /// [`Profile`]: crate::proto::Profile
    pub fn build(mut self) -> proto::Profile {
        let mut profile = proto::Profile::new();

        profile.sample_type = vec![proto::ValueType {
            type_: self.strings.intern_string("samples") as _,
            unit: self.strings.intern_string("count") as _,
            ..Default::default()
        }];
        profile.period_type.0 = Some(Box::new(proto::ValueType {
            type_: self.strings.intern_string("wall") as _,
            unit: self.strings.intern_string("microseconds") as _,
            ..Default::default()
        }));

        profile.sample = self.samples;
        profile.string_table = self.strings.into_vec();

        if let Some(start) = self.start.take() {
            profile.time_nanos = start
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as _;
            profile.duration_nanos = self
                .last
                .duration_since(start)
                .unwrap_or_default()
                .as_nanos() as _;
        }

        profile.location = self
            .locations
            .into_vec()
            .into_iter()
            .enumerate()
            .map(|(index, location)| {
                let mut proto = proto::Location::new();

                proto.id = index as _;
                proto.address = location.ip;
                proto.line = vec![proto::Line {
                    function_id: location.function as _,
                    ..Default::default()
                }];

                proto
            })
            .collect();

        profile.function = self
            .functions
            .into_vec()
            .into_iter()
            .enumerate()
            .map(|(index, function)| {
                let mut proto = proto::Function::new();

                proto.id = index as _;
                proto.name = function.name as _;
                proto.system_name = function.system_name as _;

                proto
            })
            .collect();

        profile
    }

    fn location(&mut self, frame: Frame) -> u64 {
        let mut location = Location::default();
        let mut function = Function::default();

        if let Some(symbol) = &frame.symbol {
            function.system_name = symbol
                .mangled
                .as_ref()
                .map(|sym| self.strings.intern_string(sym.clone()))
                .unwrap_or(0);
            function.name = symbol
                .demangled
                .as_ref()
                .map(|sym| self.strings.intern_string(sym.clone()))
                .unwrap_or(0);
        }

        location.ip = frame.ip;
        location.function = self.functions.intern(function);

        self.locations.intern(location) as _
    }

    fn label_string(&mut self, key: &str, value: &[u8]) -> proto::Label {
        let mut label = proto::Label::new();

        label.key = self.strings.intern(key.to_owned()) as _;
        label.str = self.strings.intern(value.to_str_lossy().into_owned()) as _;
        label
    }

    fn label_num(&mut self, key: &str, value: u64, unit: &str) -> proto::Label {
        let mut label = proto::Label::new();

        label.key = self.strings.intern(key.to_owned()) as _;
        label.num = value as _;
        label.num_unit = self.strings.intern(unit.to_owned()) as _;

        label
    }
}

impl Default for ProfileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
struct Location {
    ip: u64,
    function: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
struct Function {
    name: usize,
    system_name: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
struct Mapping {
    filename: usize,
}
