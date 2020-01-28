// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use metrics::Statistic;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
    Clone, Copy, Debug, Deserialize, EnumString, Eq, IntoStaticStr, PartialEq, Hash, Serialize,
)]
#[serde(deny_unknown_fields, try_from = "&str", into = "&str")]
pub enum MemoryStatistic {
    #[strum(serialize = "memory/total")]
    Total,
    #[strum(serialize = "memory/free")]
    Free,
    #[strum(serialize = "memory/available")]
    Available,
    #[strum(serialize = "memory/buffers")]
    Buffers,
    #[strum(serialize = "memory/cached")]
    Cached,
    #[strum(serialize = "memory/swap/cached")]
    SwapCached,
    #[strum(serialize = "memory/active/total")]
    Active,
    #[strum(serialize = "memory/inactive/total")]
    Inactive,
    #[strum(serialize = "memory/active/anon")]
    ActiveAnon,
    #[strum(serialize = "memory/inactive/anon")]
    InactiveAnon,
    #[strum(serialize = "memory/active/file")]
    ActiveFile,
    #[strum(serialize = "memory/inactive/file")]
    InactiveFile,
    #[strum(serialize = "memory/unevictable")]
    Unevictable,
    #[strum(serialize = "memory/mlocked")]
    Mlocked,
    #[strum(serialize = "memory/swap/total")]
    SwapTotal,
    #[strum(serialize = "memory/swap/free")]
    SwapFree,
    #[strum(serialize = "memory/dirty")]
    Dirty,
    #[strum(serialize = "memory/writeback")]
    Writeback,
    #[strum(serialize = "memory/anon_pages")]
    AnonPages,
    #[strum(serialize = "memory/mapped")]
    Mapped,
    #[strum(serialize = "memory/shmem")]
    Shmem,
    #[strum(serialize = "memory/slab/total")]
    SlabTotal,
    #[strum(serialize = "memory/slab/reclaimable")]
    SlabReclaimable,
    #[strum(serialize = "memory/slab/unreclaimable")]
    SlabUnreclaimable,
    #[strum(serialize = "memory/kernel_stack")]
    KernelStack,
    #[strum(serialize = "memory/page_tables")]
    PageTables,
    #[strum(serialize = "memory/nfs_unstable")]
    NFSUnstable,
    #[strum(serialize = "memory/bounce")]
    Bounce,
    #[strum(serialize = "memory/writeback_temp")]
    WritebackTmp,
    #[strum(serialize = "memory/commit/limit")]
    CommitLimit,
    #[strum(serialize = "memory/commit/committed")]
    CommittedAS,
    #[strum(serialize = "memory/vmalloc/total")]
    VmallocTotal,
    #[strum(serialize = "memory/vmalloc/used")]
    VmallocUsed,
    #[strum(serialize = "memory/vmalloc/chunk")]
    VmallocChunk,
    #[strum(serialize = "memory/percpu")]
    Percpu,
    #[strum(serialize = "memory/hardware_corrupted")]
    HardwareCorrupted,
    #[strum(serialize = "memory/anon_hugepages")]
    AnonHugePages,
    #[strum(serialize = "memory/shmem_hugepages")]
    ShmemHugePages,
    #[strum(serialize = "memory/shmem_pmd_mapped")]
    ShmemPmdMapped,
    #[strum(serialize = "memory/hugepages/total")]
    HugePagesTotal,
    #[strum(serialize = "memory/hugepages/free")]
    HugePagesFree,
    #[strum(serialize = "memory/hugepages/reserved")]
    HugePagesRsvd,
    #[strum(serialize = "memory/hugepages/surplus")]
    HugePagesSurp,
    #[strum(serialize = "memory/hugepage_size")]
    Hugepagesize,
    #[strum(serialize = "memory/hugetlb")]
    Hugetlb,
    #[strum(serialize = "memory/directmap/4k")]
    DirectMap4k,
    #[strum(serialize = "memory/directmap/2M")]
    DirectMap2M,
    #[strum(serialize = "memory/directmap/1G")]
    DirectMap1G,
}

impl TryFrom<&str> for MemoryStatistic {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        MemoryStatistic::from_str(s)
    }
}

impl Statistic for MemoryStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> metrics::Source {
        metrics::Source::Gauge
    }
}
