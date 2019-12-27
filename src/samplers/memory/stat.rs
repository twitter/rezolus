// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use metrics::Statistic;
use serde_derive::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Hash)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum MemoryStatistic {
    MemTotal,
    MemFree,
    MemAvailable,
    Buffers,
    Cached,
    SwapCached,
    Active,
    Inactive,
    ActiveAnon,
    InactiveAnon,
    ActiveFile,
    InactiveFile,
    Unevictable,
    Mlocked,
    SwapTotal,
    SwapFree,
    Dirty,
    Writeback,
    AnonPages,
    Mapped,
    Shmem,
    Slab,
    SReclaimable,
    SUnreclaim,
    KernelStack,
    PageTables,
    NFSUnstable,
    Bounce,
    WritebackTmp,
    CommitLimit,
    CommittedAS,
    VmallocTotal,
    VmallocUsed,
    VmallocChunk,
    Percpu,
    HardwareCorrupted,
    AnonHugePages,
    ShmemHugePages,
    ShmemPmdMapped,
    HugePagesTotal,
    HugePagesFree,
    HugePagesRsvd,
    HugePagesSurp,
    Hugepagesize,
    Hugetlb,
    DirectMap4k,
    DirectMap2M,
    DirectMap1G,
}

impl std::str::FromStr for MemoryStatistic {
    type Err = MemoryStatisticParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let stat = match s {
            "MemTotal" => Self::MemTotal,
            "MemFree" => Self::MemFree,
            "MemAvailable" => Self::MemAvailable,
            "Buffers" => Self::Buffers,
            "Cached" => Self::Cached,
            "SwapCached" => Self::SwapCached,
            "Active" => Self::Active,
            "Inactive" => Self::Inactive,
            "Active(anon)" => Self::ActiveAnon,
            "Inactive(anon)" => Self::InactiveAnon,
            "Active(file)" => Self::ActiveFile,
            "Inactive(file)" => Self::InactiveFile,
            "Unevictable" => Self::Unevictable,
            "Mlocked" => Self::Mlocked,
            "SwapTotal" => Self::SwapTotal,
            "SwapFree" => Self::SwapFree,
            "Dirty" => Self::Dirty,
            "Writeback" => Self::Writeback,
            "AnonPages" => Self::AnonPages,
            "Mapped" => Self::Mapped,
            "Shmem" => Self::Shmem,
            "Slab" => Self::Slab,
            "SReclaimable" => Self::SReclaimable,
            "SUnreclaim" => Self::SUnreclaim,
            "KernelStack" => Self::KernelStack,
            "PageTables" => Self::PageTables,
            "NFS_Unstable" => Self::NFSUnstable,
            "Bounce" => Self::Bounce,
            "WritebackTmp" => Self::WritebackTmp,
            "CommitLimit" => Self::CommitLimit,
            "Committed_AS" => Self::CommittedAS,
            "VmallocTotal" => Self::VmallocTotal,
            "VmallocUsed" => Self::VmallocUsed,
            "VmallocChunk" => Self::VmallocChunk,
            "Percpu" => Self::Percpu,
            "HardwareCorrupted" => Self::HardwareCorrupted,
            "AnonHugePages" => Self::AnonHugePages,
            "ShmemHugePages" => Self::ShmemHugePages,
            "ShmemPmdMapped" => Self::ShmemPmdMapped,
            "HugePages_Total" => Self::HugePagesTotal,
            "HugePages_Free" => Self::HugePagesFree,
            "HugePages_Rsvd" => Self::HugePagesRsvd,
            "HugePages_Surp" => Self::HugePagesSurp,
            "Hugepagesize" => Self::Hugepagesize,
            "Hugetlb" => Self::Hugetlb,
            "DirectMap4k" => Self::DirectMap4k,
            "DirectMap2M" => Self::DirectMap2M,
            "DirectMap1G" => Self::DirectMap1G,
            _ => return Err(MemoryStatisticParseError),
        };

        Ok(stat)
    }
}

impl Statistic for MemoryStatistic {
    fn name(&self) -> &str {
        match self {
            Self::MemTotal => "memory/total",
            Self::MemFree => "memory/free",
            Self::MemAvailable => "memory/available",
            Self::Buffers => "memory/buffers",
            Self::Cached => "memory/cached",
            Self::SwapCached => "memory/swapcached",
            Self::Active => "memory/active",
            Self::Inactive => "memory/inactive",
            Self::ActiveAnon => "memory/active/anon",
            Self::InactiveAnon => "memory/inactive/anon",
            Self::ActiveFile => "memory/active/file",
            Self::InactiveFile => "memory/inactive/file",
            Self::Unevictable => "memory/unevictable",
            Self::Mlocked => "memory/mlocked",
            Self::SwapTotal => "memory/swap/total",
            Self::SwapFree => "memory/swap/free",
            Self::Dirty => "memory/dirty",
            Self::Writeback => "memory/writeback",
            Self::AnonPages => "memory/anonpages",
            Self::Mapped => "memory/mapped",
            Self::Shmem => "memory/shmem",
            Self::Slab => "memory/slab/total",
            Self::SReclaimable => "memory/slab/reclaimable",
            Self::SUnreclaim => "memory/slab/unreclaimable",
            Self::KernelStack => "memory/kernelstack",
            Self::PageTables => "memory/pagetables",
            Self::NFSUnstable => "memory/nfs_unstable",
            Self::Bounce => "memory/bounce",
            Self::WritebackTmp => "memory/writebacktmp",
            Self::CommitLimit => "memory/commitlimit",
            Self::CommittedAS => "memory/committed",
            Self::VmallocTotal => "memory/vmalloc/total",
            Self::VmallocUsed => "memory/vmalloc/used",
            Self::VmallocChunk => "memory/vmalloc/chunk",
            Self::Percpu => "memory/percpu",
            Self::HardwareCorrupted => "memory/hardware_corrupted",
            Self::AnonHugePages => "memory/hugepages/anon",
            Self::ShmemHugePages => "memory/hugepages/shmem",
            Self::ShmemPmdMapped => "memory/hugepages/memory_mapped",
            Self::HugePagesTotal => "memory/hugepages/total",
            Self::HugePagesFree => "memory/hugepages/free",
            Self::HugePagesRsvd => "memory/hugepages/reserved",
            Self::HugePagesSurp => "memory/hugepages/surp",
            Self::Hugepagesize => "memory/hugepagesize",
            Self::Hugetlb => "memory/hugetlb",
            Self::DirectMap4k => "memory/directmap/4k",
            Self::DirectMap2M => "memory/directmap/2M",
            Self::DirectMap1G => "memory/directmap/1G",
        }
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Gauge
    }
}

#[derive(Debug)]
pub struct MemoryStatisticParseError;

impl std::fmt::Display for MemoryStatisticParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid meminfo field")
    }
}

impl std::error::Error for MemoryStatisticParseError {
    fn description(&self) -> &str {
        "Error parsing MemInfoStat"
    }
}
