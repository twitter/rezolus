// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::config::*;

use samplers::cpu::CpuConfig;
use samplers::disk::DiskConfig;
use samplers::ext4::Ext4Config;
use samplers::memcache::MemcacheConfig;
use samplers::memory::MemoryConfig;
use samplers::network::NetworkConfig;
use samplers::rezolus::RezolusConfig;
use samplers::scheduler::SchedulerConfig;
use samplers::softnet::SoftnetConfig;
use samplers::tcp::TcpConfig;
use samplers::udp::UdpConfig;
use samplers::xfs::XfsConfig;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Samplers {
    #[serde(default)]
    cpu: CpuConfig,
    #[serde(default)]
    disk: DiskConfig,
    #[serde(default)]
    ext4: Ext4Config,
    #[serde(default)]
    memcache: MemcacheConfig,
    #[serde(default)]
    memory: MemoryConfig,
    #[serde(default)]
    network: NetworkConfig,
    #[serde(default)]
    rezolus: RezolusConfig,
    #[serde(default)]
    scheduler: SchedulerConfig,
    #[serde(default)]
    softnet: SoftnetConfig,
    #[serde(default)]
    tcp: TcpConfig,
    #[serde(default)]
    udp: UdpConfig,
    #[serde(default)]
    xfs: XfsConfig,
}

impl Samplers {
    pub fn cpu(&self) -> &CpuConfig {
        &self.cpu
    }

    pub fn disk(&self) -> &DiskConfig {
        &self.disk
    }

    pub fn ext4(&self) -> &Ext4Config {
        &self.ext4
    }

    pub fn memcache(&self) -> &MemcacheConfig {
        &self.memcache
    }

    pub fn memory(&self) -> &MemoryConfig {
        &self.memory
    }

    pub fn network(&self) -> &NetworkConfig {
        &self.network
    }

    pub fn rezolus(&self) -> &RezolusConfig {
        &self.rezolus
    }

    pub fn scheduler(&self) -> &SchedulerConfig {
        &self.scheduler
    }

    pub fn softnet(&self) -> &SoftnetConfig {
        &self.softnet
    }

    pub fn tcp(&self) -> &TcpConfig {
        &self.tcp
    }

    pub fn udp(&self) -> &UdpConfig {
        &self.udp
    }

    pub fn xfs(&self) -> &XfsConfig {
        &self.xfs
    }
}
