use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;
use std::sync::Arc;

use arc_swap::ArcSwap;
use augur_common::{Annotator, AuroraInfo, Sample};
use bstr::{BStr, BString, ByteSlice};
use reqwest::Client;
use tokio::time::{Duration, Instant};

mod container;
mod state;

#[derive(Debug, Clone)]
struct ContainerInfo {
    pub source: BString,
    pub service: Option<BString>,
    pub instance: Option<u32>,
    #[allow(dead_code)]
    pub container: BString,
}

#[derive(Default, Debug, Clone)]
struct MesosInfo {
    known: HashMap<u32, ContainerInfo>,
}

impl MesosInfo {
    /// Get the cached container info for the given pid.
    fn pid_container_info(&self, pid: u32) -> Option<&ContainerInfo> {
        self.known.get(&pid)
    }

    /// Capture a view of the local thermos executor state
    async fn fetch(client: &Client) -> anyhow::Result<Self> {
        let state = self::state::fetch(client).await?;
        let state = self::state::parse(&state)?;

        let containers = self::container::fetch(client).await?;
        let containers = self::container::parse(&containers)?;

        let mut known = HashMap::new();
        let mut lookups = HashMap::new();

        for framework in &state.frameworks {
            for executor in &framework.executors {
                for task in &executor.tasks {
                    if executor.source.len() <= task.name.len() + 1 {
                        continue;
                    }

                    // The state doesn't have an explicit field for the instance_id
                    // so we instead parse it out from the source.
                    let instance_id = &executor.source[task.name.len() + 1..];
                    let instance_id: u32 = match instance_id.to_str_lossy().parse() {
                        Ok(id) => id,
                        Err(_) => {
                            warn!("Unable to parse instance id from '{}'", instance_id);
                            continue;
                        }
                    };

                    lookups.insert(task.executor_id, (task.name, instance_id));
                }
            }
        }

        for record in &containers {
            let pids = match cgroup_ids(record.container_id) {
                Ok(pids) => pids,
                Err(e) => {
                    warn!(
                        "Unable to read cgroup PIDs for container {}: {}",
                        record.container_id, e
                    );
                    continue;
                }
            };

            for pid in pids {
                let (service, instance) = match lookups.get(&record.executor_id) {
                    Some(&(service, instance)) => (Some(service), Some(instance)),
                    None => (None, None),
                };

                let info = ContainerInfo {
                    source: record.source.to_owned(),
                    service: service.map(|x| x.to_owned()),
                    instance,
                    container: record.container_id.to_owned(),
                };

                known.insert(pid, info);
            }
        }

        Ok(Self { known })
    }
}

#[derive(Debug)]
pub struct Mesos {
    info: Arc<ArcSwap<MesosInfo>>,
    task: tokio::task::JoinHandle<()>,
}

impl Mesos {
    pub fn new() -> Self {
        let info = Arc::default();
        let task = tokio::spawn(Self::background_updater(Arc::clone(&info)));

        Self { info, task }
    }

    /// Get the latest snapshot of the mesos state.
    fn current(&self) -> Arc<MesosInfo> {
        self.info.load_full()
    }

    fn fetch_info(&self, sample: &Sample) -> Option<AuroraInfo> {
        let current = self.current();
        let container = current.pid_container_info(sample.pid)?;

        let mut info = AuroraInfo::new(container.source.clone());
        info.service_name = container.service.clone();
        info.instance_id = container.instance.clone();

        Some(info)
    }

    /// Background task for periodically updating our view of the mesos state.
    // TODO(seanl): The default period should probably be configurable.
    async fn background_updater(info: Arc<ArcSwap<MesosInfo>>) {
        const MAX_PERIOD: u64 = 60 * 60;
        const MIN_PERIOD: u64 = 60;

        let mut period = 60;
        let client = match Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
        {
            Ok(client) => client,
            Err(e) => {
                error!("Unable to build mesos background client: {e}");
                return;
            }
        };

        // Note: In order to avoid spamming the logs we do limited exponential
        //       backoff whenever we fail to gather state from the mesos
        //       executor.
        loop {
            let start = Instant::now();
            match MesosInfo::fetch(&client).await {
                Ok(newinfo) => {
                    info.store(Arc::new(newinfo));
                    period = MIN_PERIOD;
                }
                Err(e) => {
                    warn!("Unable to gather state from mesos executor: {e}");
                    period = (period * 2).min(MAX_PERIOD);
                }
            }

            tokio::time::sleep_until(start + Duration::from_secs(period)).await;
        }
    }
}

#[async_trait]
impl Annotator for Mesos {
    fn name(&self) -> &str {
        "mesos"
    }

    async fn annotate(&self, sample: &mut Sample) {
        if let Some(info) = self.fetch_info(sample) {
            sample.aurora = Some(info);
        }
    }
}

impl Drop for Mesos {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Default for Mesos {
    fn default() -> Self {
        Self::new()
    }
}

fn cgroup_ids(container: &BStr) -> anyhow::Result<Vec<u32>> {
    // Mesos uses several different cgroups for different types of isolation. This
    // uses just the CPU controller.
    let mut path = PathBuf::new();
    path.push("/sys/fs/cgroup/cpu/mesos");
    path.push(OsStr::from_bytes(container));
    path.push("cgroup.procs");

    let mut output = Vec::new();
    for line in BufReader::new(File::open(&path)?).lines() {
        output.push(line?.parse()?);
    }

    Ok(output)
}
