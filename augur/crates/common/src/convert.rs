use std::path::Path;

use augur_thrift::{StackFrame, StackSample};
use bstr::{BString, ByteVec};

use crate::{Dso, Frame, Sample};

impl From<Frame> for StackFrame {
    fn from(frame: Frame) -> Self {
        let mut thrift = StackFrame::default();

        thrift.ip = Some(frame.ip as _);

        if let Some(symbol) = &frame.symbol {
            thrift.symbol = symbol
                .demangled
                .clone()
                .or_else(|| symbol.mangled.clone())
                .map(|symbol| Vec::from(symbol).into_string_lossy());
        }

        if let Some(mmap) = &frame.mmap {
            thrift.dso = match &mmap.source.dso {
                Dso::Anonymous => None,
                Dso::Vdso => Some("[vdso]".to_owned()),
                Dso::Kernel => Some("[kernel.kallsyms]".to_owned()),
                Dso::File { path, .. } => Some(Path::new(path).display().to_string()),
            };
            thrift.start = mmap.source.bounds.as_ref().map(|bounds| bounds.start as _);
        }

        thrift
    }
}

impl From<Sample> for StackSample {
    fn from(sample: Sample) -> Self {
        let mut thrift = StackSample::with_required(sample.time.into());

        thrift.pid = Some(sample.pid as _);
        thrift.tid = Some(sample.tid as _);
        thrift.cpu = Some(sample.cpu as _);
        thrift.period = Some(sample.weight as _);
        thrift.command = sample.command.map(bstr_to_str);
        thrift.thread_name = sample.thread_name.map(bstr_to_str);
        thrift.hostname = sample.hostname.map(bstr_to_str);

        if let Some(aurora) = sample.aurora {
            thrift.service_name = aurora.service_name.map(bstr_to_str);
            thrift.instance_id = aurora.instance_id.map(|x| x as _);
            thrift.source = Some(bstr_to_str(aurora.source));
        } else if let Some(systemd) = sample.systemd {
            thrift.source = systemd.unit.map(bstr_to_str);
        }

        thrift.frames = Some(sample.frames.into_iter().map(From::from).collect());

        thrift
    }
}

fn bstr_to_str(x: BString) -> String {
    Vec::from(x).into_string_lossy()
}
