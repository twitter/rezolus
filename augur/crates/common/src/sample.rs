use std::ffi::OsString;
use std::ops::Range;
use std::time::SystemTime;

use bstr::BString;
use serde::Serialize;

use crate::serde::{serialize_bstring, serialize_opt_bstring};

/// Description of the file/location that was mapped.
#[derive(Clone, Debug, Serialize)]
pub enum Dso {
    /// This memory map entry corresponds to a range of data within a file
    /// path on the current file system.
    ///
    /// Note that the file path may no longer exist or the file may have been
    /// replaced.
    File {
        /// The path to the file that this mapping is from.
        ///
        /// There is no guarantee that the file may be the version of the file
        /// that was mapped or that the file even exists.
        path: OsString,

        /// The offset of the mapping within the file.
        offset: Option<u64>,
    },

    /// The kernel-space memory map.
    Kernel,

    /// This memory map entry corresponds to the VDSO
    Vdso,

    /// This memory map entry is an anonymous executable map.
    ///
    /// This usually means that there is some sort of JIT being used in the
    /// process and it is executing code that was generated at runtime.
    Anonymous,
}

/// Description of an executable mapped memory region within a process'
/// address space.
#[derive(Clone, Debug, Serialize)]
pub struct MemmapInfo {
    /// The file or region that was mapped along with its offset.
    pub dso: Dso,

    /// The starting and ending addresses of this map within the process' memory
    /// space.
    pub bounds: Option<Range<u64>>,
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct FrameMemmapInfo {
    /// The memory mapping that the instruction pointer belongs to.
    pub source: MemmapInfo,

    /// The offset of the instruction within the memory map above.
    pub offset: Option<u64>,
}

impl FrameMemmapInfo {
    pub fn new(dso: Dso) -> Self {
        Self {
            source: MemmapInfo { dso, bounds: None },
            offset: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[non_exhaustive]
pub struct FrameSymbolInfo {
    /// The base address of this symbol relative to the process' address space.
    pub base: u64,

    /// Whether this stack frame is an artifical one created for an inlined
    /// stack frame.
    ///
    /// If this is true then all the addresses correspond to the stack frame
    /// that physically existed on the stack. The only difference will be that
    /// the symbol names have been updated to correspond to that of the
    /// inlined stack frame.
    pub inlined: bool,

    /// The mangled symbol name for this stack frame.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub mangled: Option<BString>,

    /// The human readable demangled name for this stack frame.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub demangled: Option<BString>,

    /// The source file name that corresponds to the sampled instruction within
    /// this stack frame.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub file: Option<BString>,

    /// The line number within the above file that corresponds to the sampled
    /// instruction.
    pub line: Option<u32>,
}

impl FrameSymbolInfo {
    pub fn new(base: u64, inlined: bool) -> Self {
        Self {
            base,
            inlined,

            mangled: None,
            demangled: None,
            file: None,
            line: None,
        }
    }
}

/// A stack frame within a sample.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct Frame {
    /// The value of the instruction pointer for this stack frame.
    pub ip: u64,

    /// Which file or memory map the memory pointed to by `ip` came from.
    pub mmap: Option<FrameMemmapInfo>,

    /// Information about the symbol within the symbol within the binary
    /// associated with `ip`.
    pub symbol: Option<FrameSymbolInfo>,
}

impl Frame {
    pub fn new(ip: u64) -> Self {
        Self {
            ip,
            mmap: None,
            symbol: None,
        }
    }
}

/// Information about the aurora container that the process is running in.
#[derive(Clone, Debug, Serialize)]
pub struct AuroraInfo {
    /// The raw name of the aurora container that the process was running in
    /// on this host.
    #[serde(serialize_with = "serialize_bstring")]
    pub source: BString,

    /// The aurora job name, without the datacenter or instance id components.
    /// This will be a string like "io-perf/prod/charlie-cache".
    pub service_name: Option<BString>,

    /// The instance ID of this container within the overall aurora job.
    pub instance_id: Option<u32>,
}

impl AuroraInfo {
    pub fn new(source: BString) -> Self {
        Self {
            source,
            service_name: None,
            instance_id: None,
        }
    }
}

/// Information about the systemd context a sample is running under.
#[derive(Clone, Debug, Serialize, Default)]
pub struct SystemdInfo {
    /// The name of the systemd unit that contains the process.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub unit: Option<BString>,

    /// The name of the systemd slice that the process belonged to.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub slice: Option<BString>,
}

#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct Sample {
    /// The process ID that the sample was taken from.
    pub pid: u32,

    /// The thread ID that the sample was taken from.
    pub tid: u32,

    /// The CPU that the thread was running on when the sample was taken.
    pub cpu: u32,

    /// The time at which the sample is taken.
    pub time: SystemTime,

    /// The frames that make up the call stack.
    pub frames: Vec<Frame>,

    /// The weight of this sample. The meaning of this depends on what
    /// is being sampled.
    /// - For CPU cycles it may be the number of cycles since the last sample on
    ///   this core.
    /// - For memory misses it could be the number of misses.
    /// - For syscalls it may be the time spent blocked in the syscall.
    pub weight: u64,

    /// The name of the host on which the sample was taken.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub hostname: Option<BString>,

    /// The command string of the process.
    ///
    /// By default, this will be the binary part of the command that was used to
    /// invoke the process. However, this is just the name of the main thread so
    /// the process may change it.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub command: Option<BString>,

    /// The name of the thread that was sampled.
    ///
    /// In many cases this will just be the same as `command` but if the process
    /// being sampled chooses to rename its threads then that will show up here.
    #[serde(serialize_with = "serialize_opt_bstring")]
    pub thread_name: Option<BString>,

    /// Information about the container under which the sampled process was
    /// running, if there is one.
    pub aurora: Option<AuroraInfo>,

    /// Information about the systemd context in which the sampled process
    /// was running, if there is one.
    pub systemd: Option<SystemdInfo>,
}

impl Sample {
    /// Create a new sample and initialize the minimum required fields
    pub fn new(pid: u32, tid: u32, cpu: u32, time: SystemTime, weight: u64) -> Self {
        Self {
            pid,
            tid,
            cpu,
            time,
            weight,

            frames: Vec::new(),
            hostname: None,
            command: None,
            thread_name: None,
            aurora: None,
            systemd: None,
        }
    }
}
