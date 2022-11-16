use std::os::unix::ffi::OsStrExt;

use augur_common::{Annotator, Sample};
use bstr::BString;

/// Annotates a sample with the hostname of the current machine.
pub struct Hostname {
    hostname: BString,
}

impl Hostname {
    pub fn new() -> anyhow::Result<Self> {
        let hostname = nix::unistd::gethostname()?.as_bytes().into();

        Ok(Self { hostname })
    }
}

#[async_trait]
impl Annotator for Hostname {
    fn name(&self) -> &str {
        "hostname"
    }

    async fn annotate(&self, sample: &mut Sample) {
        sample.hostname = Some(self.hostname.clone());
    }
}
