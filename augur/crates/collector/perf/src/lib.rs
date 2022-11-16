//! This is a sample collector that gathers samples by running an instance of
//! `perf record` and periodically parsing the perf.data files that it outputs
//! using `perf script`.
//!
//! Beyond that, it has some special handling for finding `perf-[pid].map`
//! that are emitted in non-standard locations and adding symlinks of those
//! under `/tmp`.

mod input;

use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::prelude::OsStringExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use augur_common::{Dso, Frame, FrameMemmapInfo, FrameSymbolInfo, Sample};
use futures::StreamExt;
use inotify::{EventStream, Inotify, WatchDescriptor, WatchMask};
use nix::unistd::Pid;
use rustcommon_metrics::{metric, Counter};
use tempdir::TempDir;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::input::PerfEvent;

#[macro_use]
extern crate log;

const SCRIPT_SRC: &[u8] = include_bytes!("perf-script.py");
const SCRIPT_PATH: &str = "perf-script.py";

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;

#[metric(
    name = "collectors/perf/script_runs",
    description = "the number of times that `perf script` has been run by augur"
)]
static PERF_SCRIPT_RUNS: Counter = Counter::new();

#[metric(
    name = "collectors/perf/script_successes",
    description = "the number of times that `perf script` exited succesfully"
)]
static PERF_SCRIPT_SUCCESSES: Counter = Counter::new();

#[metric(
    name = "collectors/perf/script_failures",
    description = "the number of types that `perf script` exited with an error"
)]
static PERF_SCRIPT_FAILURES: Counter = Counter::new();

#[metric(
    name = "collectors/perf/samples",
    description = "the number of samples collected by the perf sample collector"
)]
static SAMPLES: Counter = Counter::new();

// MacOS doesn't have CLOCK_MONOTONIC_RAW. To allow this to at least compile on
// macos we use CLOCK_MONOTONIC here when the OS is not linux.
cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        const CLOCK: i32 = libc::CLOCK_MONOTONIC_RAW;
        const CLOCK_NAME: &str = "CLOCK_MONOTONIC_RAW";
    } else {
        const CLOCK: u32 = libc::CLOCK_MONOTONIC;
        const CLOCK_NAME: &str = "CLOCK_MONOTONIC";
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PerfConfig {
    /// The frequency at which perf will gather samples on each core, in Hz.
    pub frequency: u32,

    /// The period between successive emissions of perf data files, in seconds.
    pub period: u32,
}

pub struct PerfCollector {
    perf: Child,
    workdir: TempDir,
    inotify: Inotify,
    script_wd: WatchDescriptor,
    stream: EventStream<Vec<u8>>,
    events: Vec<Sample>,
    config: PerfConfig,
}

impl PerfCollector {
    pub fn new(config: PerfConfig) -> anyhow::Result<Self> {
        // Limit frequency to 100Hz max to prevent misconfigurations from
        // causing too much overhead within a server.
        assert!(config.frequency <= 100);

        let workdir =
            TempDir::new("augur").context("Failed to create temporary working directory")?;

        let mut inotify =
            Inotify::init().context("Failed to setup inotify watch on temporary directory")?;
        let script_wd = inotify
            .add_watch(workdir.path(), WatchMask::CLOSE_WRITE)
            .with_context(|| format!("Failed to start watching `{}`", SCRIPT_PATH))?;

        let stream = inotify
            .event_stream(vec![0; 1024])
            .context("Failed to create inotify event stream")?;

        let mut command = Command::new("perf");
        command
            .arg("record")
            .arg("--quiet")
            .arg("--all-cpus")
            .arg("-g") // Enable call-graph recording
            .arg("--timestamp")
            .arg("--clockid")
            .arg(CLOCK_NAME)
            .arg("--sample-cpu")
            .arg("--period")
            .arg(format!("--switch-output={}s", config.period))
            .arg("-F")
            .arg(format!("{}", config.frequency))
            .arg("-o")
            .arg(workdir.path().join("sample"));

        let perf = command
            .spawn()
            .context("Failed to spawn `perf record` command")?;

        let mut me = Self {
            workdir,
            inotify,
            stream,
            perf,
            script_wd,
            events: Vec::new(),
            config,
        };

        me.regen_perf_script()?;

        Ok(me)
    }

    fn symlink_perf_maps(&self) -> anyhow::Result<()> {
        for file in glob::glob("/tmp/perf-*.map")? {
            let file = match file {
                Ok(file) => file,
                Err(_) => continue,
            };

            if is_broken_symlink(&file).unwrap_or(false) {
                let _ = std::fs::remove_file(file);
            }
        }

        let mut out = PathBuf::new();
        out.push("/tmp/dummy");

        for path in glob::glob(
            "/var/lib/mesos/slaves/*/frameworks/*/executors/*/runs/*/sandbox/perf-*.map",
        )? {
            let path = match path {
                Ok(path) => path,
                Err(_) => continue,
            };

            let basename = match path.file_name() {
                Some(name) => name,
                None => continue,
            };

            out.set_file_name(basename);
            if out.exists() {
                continue;
            }

            if let Err(e) = std::os::unix::fs::symlink(&path, &out) {
                warn!(
                    "Unable to symlink perf-map `{}` to `{}`: {}",
                    path.display(),
                    out.display(),
                    e
                );
            }
        }

        Ok(())
    }

    fn regen_perf_script(&mut self) -> anyhow::Result<()> {
        let script_path = self.workdir.path().join(SCRIPT_PATH);
        let mut script = File::create(&script_path)
            .with_context(|| format!("Failed to create perf `{SCRIPT_PATH}`"))?;

        script
            .write_all(SCRIPT_SRC)
            .with_context(|| format!("Failed to write script out to `{SCRIPT_PATH}`"))?;

        debug!("Wrote perf python script to `{}`", script_path.display());

        // We need to close the script file before we setup the inotify watcher as
        // otherwise we'll get the CLOSE_WRITE event for the script and try to
        // process it as a perf.data file.
        drop(script);

        self.script_wd = self
            .inotify
            .add_watch(
                self.workdir.path().join(SCRIPT_PATH),
                WatchMask::DELETE_SELF,
            )
            .with_context(|| format!("Failed to create watch on `{SCRIPT_PATH}`"))?;

        Ok(())
    }

    async fn parse_perf_data(&mut self, file: &Path) -> anyhow::Result<Vec<Sample>> {
        if let Err(e) = self.symlink_perf_maps() {
            error!("Unable to symlink new perf-[pid].map files: {e}");
        }

        let mut command = Command::new("perf");
        command
            .arg("script")
            .arg("--input")
            .arg(file)
            .arg("--script")
            .arg(self.workdir.path().join(SCRIPT_PATH))
            .stdout(Stdio::piped())
            .kill_on_drop(true);

        debug!("Running command {:?}", command.as_std());

        let mut child = command
            .spawn()
            .context("Unable to spawn perf script process")?;

        PERF_SCRIPT_RUNS.increment();

        let stdout = child
            .stdout
            .take()
            .expect("Perf script process instance did not contain a stdout pipe");
        let offset = clock_offset();

        let mut events = Vec::new();
        let mut lines = BufReader::new(stdout).lines();
        while let Some(line) = lines.next_line().await? {
            let de = &mut serde_json::Deserializer::from_str(&line);
            let mut perf_sample: PerfEvent = serde_path_to_error::deserialize(de)
                .map_err(|e| {
                    let field = e.path().to_string();
                    let inner = e.into_inner();

                    anyhow::anyhow!(inner).context(format!("while deserializing `{field}`"))
                })
                .context("perf script produced a sample with an invalid format")?;

            // Convert the timestamp to something close to its unix time. This will not be
            // exactly accurate (and there's no way to really do that) but it should be
            // close enough to be usable for a number of use cases.
            perf_sample.sample.time += offset;

            let frames = perf_sample
                .callchain
                .into_iter()
                .map(|perf_frame| {
                    let mut frame = Frame::new(perf_frame.ip);

                    if let Some(sym) = &perf_frame.sym {
                        let mut info = FrameSymbolInfo::new(sym.start, false);
                        info.mangled = sym.name.as_deref().map(|x| x.to_owned());
                        info.demangled = sym.name.as_deref().map(|x| x.to_owned());

                        frame.symbol = Some(info);
                    }

                    let dso = match perf_frame.dso.as_ref().map(|dso| -> &[u8] { dso }) {
                        Some(b"[kernel.kallsyms]") => Dso::Kernel,
                        Some(symbol) => Dso::File {
                            path: OsString::from_vec(symbol.to_vec()),
                            offset: None,
                        },
                        None => Dso::Anonymous, // TODO: vdso
                    };

                    frame.mmap = Some(FrameMemmapInfo::new(dso));
                    frame
                })
                .collect::<Vec<_>>();

            let time = SystemTime::UNIX_EPOCH
                + Duration::new(
                    perf_sample.sample.time / NANOSECONDS_PER_SECOND,
                    (perf_sample.sample.time % NANOSECONDS_PER_SECOND) as _,
                );

            let mut sample = Sample::new(
                perf_sample.sample.pid,
                perf_sample.sample.tid,
                perf_sample.sample.cpu,
                time,
                NANOSECONDS_PER_SECOND / self.config.frequency as u64,
            );

            sample.thread_name = Some(perf_sample.comm.into_owned());
            sample.frames = frames;

            events.push(sample);
        }

        let status = child
            .wait()
            .await
            .context("Failed to wait for perf script command to finish")?;

        if status.success() {
            PERF_SCRIPT_SUCCESSES.increment();
        } else {
            PERF_SCRIPT_FAILURES.increment();
            warn!(
                "`perf script` exited with status {}",
                status.code().or_else(|| status.signal()).unwrap_or(-1)
            );
        }

        events.reverse();

        SAMPLES.add(events.len() as _);

        Ok(events)
    }

    /// Get the next group of events emitted by `perf record`.
    async fn get_events(&mut self) -> anyhow::Result<Vec<Sample>> {
        let event = match self.stream.next().await {
            Some(event) => event?,
            None => {
                warn!("inotify returned end-of-file");
                return Err(anyhow::Error::new(io::Error::from(
                    io::ErrorKind::UnexpectedEof,
                )));
            }
        };

        if event.wd == self.script_wd {
            self.regen_perf_script()?;
            return Ok(Vec::new());
        }

        let file = match &event.name {
            Some(name) if name == SCRIPT_PATH => return Ok(Vec::new()),
            Some(name) => self.workdir.path().join(name),
            None => return Ok(Vec::new()),
        };

        let result = self.parse_perf_data(&file).await;

        if let Err(e) = std::fs::remove_file(&file) {
            error!("Unable to remove `{}`: {e}", file.display());
        }

        let events = match result {
            Ok(events) => events,
            Err(e) => {
                PERF_SCRIPT_FAILURES.increment();
                error!("Failed to parse perf data file `{}`: {e:#}", file.display());
                return Ok(Vec::new());
            }
        };

        Ok(events)
    }

    /// Get the next event, waiting until `perf record` generates more events
    /// if necessary.
    pub async fn next_event(&mut self) -> anyhow::Result<Sample> {
        loop {
            if let Some(event) = self.events.pop() {
                return Ok(event);
            }

            self.events = self.get_events().await?;
        }
    }
}

impl Drop for PerfCollector {
    fn drop(&mut self) {
        if let Some(pid) = self.perf.id() {
            // We need to do this using the syscall since Command only
            // supports sending SIGKILL.
            let _ = nix::sys::signal::kill(Pid::from_raw(pid as _), nix::sys::signal::SIGINT);
        }
    }
}

fn is_broken_symlink(path: &Path) -> std::io::Result<bool> {
    use std::io::ErrorKind;

    let sym = match std::fs::symlink_metadata(path) {
        Ok(sym) => sym,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };

    if !sym.file_type().is_symlink() {
        return Ok(false);
    }

    // A symlink exists at the path location but no file exists -> symlink is broken
    Ok(!path.exists())
}

// Get an approximate offset between monotonic and realtime clocks in
// nanoseconds.
fn clock_offset() -> u64 {
    let mut realtime = unsafe { std::mem::zeroed() };
    let mut monotonic = unsafe { std::mem::zeroed() };

    unsafe { libc::clock_gettime(libc::CLOCK_REALTIME, &mut realtime) };
    unsafe { libc::clock_gettime(CLOCK, &mut monotonic) };

    let monotonic = monotonic.tv_sec as u64 * NANOSECONDS_PER_SECOND + monotonic.tv_nsec as u64;
    let realtime = realtime.tv_sec as u64 * NANOSECONDS_PER_SECOND + realtime.tv_nsec as u64;

    realtime - monotonic
}
