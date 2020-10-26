// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::time::*;

use async_trait::async_trait;

use crate::common::*;
use crate::config::SamplerConfig;
use crate::samplers::Common;
use crate::Sampler;

mod config;
mod stat;

pub use config::*;
pub use stat::*;

#[allow(dead_code)]
pub struct Ntp {
    common: Common,
    statistics: Vec<NtpStatistic>,
}

#[async_trait]
impl Sampler for Ntp {
    type Statistic = NtpStatistic;

    fn new(common: Common) -> Result<Self, anyhow::Error> {
        let statistics = common.config().samplers().ntp().statistics();
        #[allow(unused_mut)]
        let mut sampler = Self {
            common,
            statistics,
        };

        if sampler.sampler_config().enabled() {
            sampler.register();
        }

        Ok(sampler)
    }

    fn spawn(common: Common) {
        debug!("spawning");
        if common.config().samplers().ntp().enabled() {
            debug!("sampler is enabled");
            if let Ok(mut ntp) = Ntp::new(common.clone()) {
                common.runtime().spawn(async move {
                    loop {
                        let _ = ntp.sample().await;
                    }
                });
            } else if !common.config.fault_tolerant() {
                fatal!("failed to initialize ntp sampler");
            } else {
                error!("failed to initialize ntp sampler");
            }
        }
    }

    fn common(&self) -> &Common {
        &self.common
    }

    fn common_mut(&mut self) -> &mut Common {
        &mut self.common
    }

    fn sampler_config(&self) -> &dyn SamplerConfig<Statistic = Self::Statistic> {
        self.common.config().samplers().ntp()
    }

    async fn sample(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut delay) = self.delay() {
            delay.tick().await;
        }

        if !self.sampler_config().enabled() {
            return Ok(());
        }

        debug!("sampling");

        let r = self.sample_ntp_adjtime().await;
        self.map_result(r)?;

        Ok(())
    }
}

impl Ntp {
    async fn sample_ntp_adjtime(&mut self) -> Result<(), std::io::Error> {
        let mut timeval = default_ntptimeval();
        let time = Instant::now();
        let status = unsafe { libc::ntp_gettime(&mut timeval) };
        if status == 0 {
            let _ = self.metrics().record_gauge(&NtpStatistic::MaximumError, time, timeval.maxerror as u64 * MICROSECOND);

            #[cfg(all(not(target_os = "macos"), not(target_os = "ios"), unix))]
            let _ = self.metrics().record_gauge(&NtpStatistic::EstimatedError, time, timeval.esterror as u64 * MICROSECOND);
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn default_ntptimeval() -> libc::ntptimeval {
    libc::ntptimeval {
        time: libc::timespec { tv_sec: 0, tv_nsec: 0 },
        maxerror: 0,
        esterror: 0,
        tai: 0,
        time_state: 0,
    }
}

#[cfg(all(not(target_os = "macos"), not(target_os = "ios"), unix))]
fn default_ntptimeval() -> libc::ntptimeval {
    libc::ntptimeval {
        time: libc::timeval { tv_sec: 0, tv_usec: 0 },
        maxerror: 0,
        esterror: 0,
        tai: 0,
        __glibc_reserved1: 0,
        __glibc_reserved2: 0,
        __glibc_reserved3: 0,
        __glibc_reserved4: 0,
    }
}
