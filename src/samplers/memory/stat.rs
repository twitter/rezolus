// Copyright 2019-2020 Twitter, Inc.
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use core::convert::TryFrom;
use core::str::FromStr;

use rustcommon_metrics::*;
use serde_derive::{Deserialize, Serialize};
use strum::ParseError;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Hash,
    Serialize,
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
    // NUMA
    #[strum(serialize = "memory/numa/hit")]
    NumaHit,
    #[strum(serialize = "memory/numa/miss")]
    NumaMiss,
    #[strum(serialize = "memory/numa/foreign")]
    NumaForeign,
    #[strum(serialize = "memory/numa/interleave")]
    NumaInterleave,
    #[strum(serialize = "memory/numa/local")]
    NumaLocal,
    #[strum(serialize = "memory/numa/other")]
    NumaOther,
    // THP
    #[strum(serialize = "memory/thp/fault_alloc")]
    ThpFaultAlloc,
    #[strum(serialize = "memory/thp/fault_fallback")]
    ThpFaultFallback,
    #[strum(serialize = "memory/thp/collapse_alloc")]
    ThpCollapseAlloc,
    #[strum(serialize = "memory/thp/collapse_alloc_failed")]
    ThpCollapseAllocFailed,
    #[strum(serialize = "memory/thp/split_page")]
    ThpSplitPage,
    #[strum(serialize = "memory/thp/split_page_failed")]
    ThpSplitPageFailed,
    #[strum(serialize = "memory/thp/deferred_split_page")]
    ThpDeferredSplitPage,
    // Compaction
    #[strum(serialize = "memory/compact/stall")]
    CompactStall,
    #[strum(serialize = "memory/compact/fail")]
    CompactFail,
    #[strum(serialize = "memory/compact/success")]
    CompactSuccess,
    #[strum(serialize = "memory/compact/migrate_scanned")]
    CompactMigrateScanned,
    #[strum(serialize = "memory/compact/free_scanned")]
    CompactFreeScanned,
    #[strum(serialize = "memory/compact/isolated")]
    CompactIsolated,
    #[strum(serialize = "memory/compact/daemon/wake")]
    CompactDaemonWake,
    #[strum(serialize = "memory/compact/daemon/migrate_scanned")]
    CompactDaemonMigrateScanned,
    #[strum(serialize = "memory/compact/daemon/free_scanned")]
    CompactDaemonFreeScanned,
}

impl MemoryStatistic {
    pub fn multiplier(self) -> u64 {
        match self {
            // these are counts of pages or events
            Self::HugePagesTotal
            | Self::HugePagesFree
            | Self::HugePagesRsvd
            | Self::HugePagesSurp
            | Self::ShmemHugePages
            | Self::ShmemPmdMapped
            | Self::ThpFaultAlloc
            | Self::ThpFaultFallback
            | Self::ThpCollapseAllocFailed
            | Self::ThpCollapseAlloc
            | Self::ThpSplitPage
            | Self::ThpSplitPageFailed
            | Self::ThpDeferredSplitPage
            | Self::CompactStall
            | Self::CompactFail
            | Self::CompactSuccess
            | Self::CompactMigrateScanned
            | Self::CompactFreeScanned
            | Self::CompactIsolated
            | Self::CompactDaemonWake
            | Self::CompactDaemonMigrateScanned
            | Self::CompactDaemonFreeScanned => 1,
            // convert from pages to bytes
            Self::NumaHit
            | Self::NumaMiss
            | Self::NumaForeign
            | Self::NumaInterleave
            | Self::NumaLocal
            | Self::NumaOther => 4096,
            // convert from kilobytes to bytes
            _ => 1024,
        }
    }
}

impl Statistic<AtomicU64, AtomicU32> for MemoryStatistic {
    fn name(&self) -> &str {
        (*self).into()
    }

    fn source(&self) -> Source {
        match self {
            Self::NumaHit
            | Self::NumaMiss
            | Self::NumaForeign
            | Self::NumaInterleave
            | Self::NumaLocal
            | Self::NumaOther => Source::Counter,
            _ => Source::Gauge,
        }
    }
}
