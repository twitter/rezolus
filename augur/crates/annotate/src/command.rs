use augur_common::{Annotator, Sample};
use bstr::BString;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Default)]
pub struct Command(());

impl Command {
    pub fn new() -> Self {
        Self::default()
    }

    async fn read_process_command(pid: u32) -> anyhow::Result<BString> {
        let mut file = File::open(format!("/proc/{}/comm", pid)).await?;
        let mut comm = Vec::new();
        file.read_to_end(&mut comm).await?;

        comm.pop();

        Ok(comm.into())
    }
}

#[async_trait]
impl Annotator for Command {
    fn name(&self) -> &str {
        "command"
    }

    async fn annotate(&self, sample: &mut Sample) {
        sample.command = Self::read_process_command(sample.pid)
            .await
            .ok()
            .or_else(|| sample.thread_name.clone());
    }
}
