# Metrics

Below you will find a list of possible metrics which Rezolus can export as well
as details on their source and meaning. Metrics below may typically be followed
by:
* `/count` - the value of the counter
* `/histogram/(percentile)` - a percentile of a counter's secondly rate, a
  gauge's instantaneous readings, or the percentile taken from a distribution

Sampler configurations will refer to the metrics according to their basenames as
used in the descriptions below.

**Note:** summary metrics taken from underlying distributions use a significant
figure preserving histogram binning. This means that the reported values will be
rounded up to the highest value that still preserves that number of leading
digits. Currently, this is fixed at 2 significant figures to help maintain a low
memory footprint. This means, you may see a percentile like 10999, which implies
the true value is somewhere between 10000 and 10999 (inclusive).

Summary metrics for counters and gauges use a different strategy for percentile
calculation, as we can hold the number of samples to calculate an exact
percentile in memory.

## CPU

Provides telemetry around CPU usage and performance.

### Basic

* `cpu/cstate/c0/time` - nanoseconds spent in c0 state, Active Mode
* `cpu/cstate/c1/time` - nanoseconds spent in c1 state, Auto Halt
* `cpu/cstate/c1e/time` - nanoseconds spent in c1e state, Auto Halt + low
  frequency + low voltage
* `cpu/cstate/c2/time` - nanoseconds spent in c2 state, temporary before c3 with
  memory paths still open
* `cpu/cstate/c3/time` - nanoseconds spent in c3 state, L1/L2 flush + clocks off
* `cpu/cstate/c6/time` - nanoseconds spent in c6 state, save core states before
  shutdown and PLL off
* `cpu/cstate/c7/time` - nanoseconds spent in c7 state, c6 + LLC may flush
* `cpu/cstate/c8/time` - nanoseconds spent in c8 state, c7 + LLC must flush
* `cpu/frequency` - instantaneous cpu frequency in Hz
* `cpu/usage/guest` - nanoseconds spent running a guest VM
* `cpu/usage/guestnice` - nanoseconds spent running a low-priority guest VM
* `cpu/usage/idle` - nanoseconds spent idle
* `cpu/usage/irq` - nanoseconds spent handling interrupts
* `cpu/usage/nice` - nanoseconds spent on lower-priority tasks
* `cpu/usage/softirq` - nanoseconds spent handling soft interrupts
* `cpu/usage/steal` - nanoseconds stolen by the hypervisor
* `cpu/usage/system` - nanoseconds spent in kernel-space
* `cpu/usage/user` - nanoseconds spent in user-space

### Perf Events

* `cpu/bpu/branch` - total branch instructions
* `cpu/bpu/miss` - branch predictions resulting in miss
* `cpu/cache/access` - total cache accesses
* `cpu/cache/miss` - cache accesses resulting in miss
* `cpu/cycles` - cpu cycles elapsed, may not be accurate with frequency scaling.
  consult processor documentation for details and consider using
  `cpu/reference_cycles` metric
* `cpu/dtlb/load/access` - total dtlb loads
* `cpu/dtlb/load/miss` - dtlb loads resulting in miss
* `cpu/dtlb/store/access` - total dtlb stores
* `cpu/dtlb/store/miss` - dtlb stores resulting in miss
* `cpu/instructions` - instructions retired
* `cpu/reference_cycles` - reference number of cpu cycles elapsed, may not be
  present on all processors. consult processor documentation
* `cpu/stalled_cycles/backend` - cycles stalled waiting on backend, eg memory
  access
* `cpu/stalled_cycles/frontend` - cycles stalled waiting on frontend, eg
  instructions

## Disk

Provides system-wide telemetry for disk devices

### Basic

* `disk/discard/bytes` - bytes marked as unused on SSD devices 
* `disk/discard/operations` - total number of discards completed
* `disk/read/bytes` - bytes read from disk devices
* `disk/read/operations` - total number of reads completed
* `disk/write/bytes` - bytes written to disk devices
* `disk/write/operations` - total number of writes completed

### BPF

* `disk/read/device_latency` - latency distribution, in nanoseconds, waiting for
  disk to complete a read operation
* `disk/read/latency` - end-to-end latency distribution, in nanoseconds, for
  read operations
* `disk/read/io_size` - size distribution, in bytes, for read operations
* `disk/read/queue_latency` - latency distribution, in nanoseconds, where read
  was waiting on the device queue
* `disk/write/device_latency` - latency distribution, in nanoseconds, waiting
  for disk to complete a write operation
* `disk/write/io_size` - size distribution, in bytes, for write operations
* `disk/write/latency` - end-to-end latency distribution, in nanoseconds, for
  write operations
* `disk/write/queue_latency` - latency distribution, in nanoseconds, where write
  was waiting on the device queue

## EXT4

Provides system-wide telemetry for EXT4 filesystems

### BPF

* `ext4/fsync/latency` - latency distribution, in nanoseconds, for `fsync()` on
  ext4 filesystems
* `ext4/open/latency` - latency distribution, in nanoseconds, for `open()` on
  ext4 filesystems
* `ext4/read/latency` - latency distribution, in nanoseconds, for `read()` on
  ext4 filesystems
* `ext4/write/latency` - latency distribution, in nanoseconds, for `write()` on
  ext4 filesystems

## Interrupt

Provides system-wide telemetry for IRQs

### Basic

* `interrupt/local_timer` - APIC interrupts which fire on a specific CPU as a
  result of a local timer
* `interrupt/machine_check_exception` - interrupts caused by machine check
  exceptions
* `interrupt/network` - interrupts for servicing network devices (NIC queues)
* `interrupt/nmi` - Non-Maskable Interrupts
* `interrupt/node0/network` - interrupts for servicing network devices which
  were handled on NUMA node 0
* `interrupt/node0/nvme` - interrupts for servicing NVMe devices which
  were handled on NUMA node 0
* `interrupt/node0/total` - total interrupts which were handled on NUMA node 0
* `interrupt/node1/network` - interrupts for servicing network devices which
  were handled on NUMA node 1
* `interrupt/node1/nvme` - interrupts for servicing NVMe devices which
  were handled on NUMA node 1
* `interrupt/node1/total` - total interrupts which were handled on NUMA node 1
* `interrupt/nvme` - interrupts for servicing NVMe queues
* `interrupt/performance_monitoring` - interrupts generated when a performance
  counter overflows or PEBS interrupt threshold is reached
* `interrupt/rescheduling` - interrupts used to notify a core to schedule a
  thread
* `interrupt/rtc` - interrupts caused by the realtime clock
* `interrupt/spurious` - interrupts which were marked spurious and not handled
* `interrupt/thermal_event` - interrupts caused by thermal events, like
  throttling
* `interrupt/timer` - interrupts related to the system timer (PIT/HPET)
* `interrupt/tlb_shootdowns` - interrupts caused to trigger TLB shootdowns
* `interrupt/total` - total interrupts

## Memory

Provides telemetry around memory usage, transparent huge-pages, huge-pages,
compaction, NUMA access, etc.

### Basic

* `memory/active/anon` - the amount of anonymous and tmpfs/shmem memory, in
  bytes, that is in active use, or was in active use since the last time the
  system moved something to swap.
* `memory/active/file` - the amount of file cache memory, in bytes, that is in
  active use, or was in active use since the last time the system reclaimed
  memory.
* `memory/active/total` - the amount of memory, in bytes, that has been used
  more recently and is usually not reclaimed unless absolutely necessary.
* `memory/anon_hugepages` - the total amount of memory, in bytes, used by huge
  pages that are not backed by files and are mapped into userspace page tables.
* `memory/anon_pages` - the total amount of memory, in bytes, used by pages that
  are not backed by files and are mapped into userspace page tables.
* `memory/available` - estimate of the amount of memory, in bytes, available on
  the system to allocate without swapping
* `memory/bounce` - the amount of memory, in bytes, used for the block device
  "bounce buffers".
* `memory/buffers` - the amount, in bytes, of temporary storage for raw disk
  blocks
* `memory/cached` - the amount of physical RAM, in bytes, used as cache memory
* `memory/commit/committed` - the total amount of memory, in bytes, estimated to
  complete the workload. This value represents the worst case scenario value,
  and also includes swap memory.
* `memory/commit/limit` - total amount of memory, inb bytes, currently available
  to be allocated on the system based on the overcommit ratio
* `memory/compact/daemon/free_scanned` - the number of pages kcompactd has
  scanned to potentially free
* `memory/compact/daemon/migrate_scanned` - the number of pages kcompactd has
  scanned to potentially migrate
* `memory/compact/daemon/wake` - the number of times kcompactd has woken
* `memory/compact/fail` - the number of compactions which fail to free a
  hugepage
* `memory/compact/free_scanned` - the number of pages scanned to potentially
  free
* `memory/compact/isolated` - the number of pages isolated by compaction
* `memory/compact/migrate_scanned` - the number of pages scanned to potentially
  migrate
* `memory/compact/stall` - the number of times processes stall to run compaction
* `memory/compact/success` - the number of compactions resulting in successfully
  freeing a hugepage
* `memory/directmap/1G` - the amount of memory, in bytes, mapped into kernel
  address space with 1 GB page mappings.
* `memory/directmap/2M` - the amount of memory, in bytes, mapped into kernel
  address space with 2 MB page mappings.
* `memory/directmap/4k` - the amount of memory, in bytes, mapped into kernel
  address space with 4 kB page mappings.
* `memory/dirty` - the total amount of memory, in bytes, waiting to be written
  back to the disk.
* `memory/free` - the amount of physical RAM, in bytes, left unused by the
  system
* `memory/hardware_corrupted` - the amount of memory, in bytes, with physical
  memory corruption problems, identified by the hardware and set aside by the
  kernel so it does not get used.
* `memory/hugepage_size` - the size for each hugepages unit in bytes.
* `memory/hugepages/free` - the total number of hugepages available for the
  system.
* `memory/hugepages/reserved` - the number of unused huge pages reserved for
  hugetlbfs.
* `memory/hugepages/surplus` - the number of surplus huge pages.
* `memory/hugepages/total` - the total number of hugepages for the system.
* `memory/hugetlb`
* `memory/inactive/anon` - the amount of anonymous and tmpfs/shmem memory, in
  bytes, that is a candidate for eviction.
* `memory/inactive/file` - the amount of file cache memory, in bytes, that is
  newly loaded from the disk, or is a candidate for reclaiming.
* `memory/inactive/total` - the amount of memory, in bytes, that has been used
  less recently and is more eligible to be reclaimed for other purposes.
* `memory/kernel_stack` - the amount of memory, in bytes, used by the kernel
  stack allocations done for each task in the system.
* `memory/mapped` - the memory, in bytes, used for files that have been mmaped,
  such as libraries.
* `memory/mlocked` - the total amount of memory, in bytes, that is not evictable
  because it is locked into memory by user programs.
* `memory/nfs_unstable` - the amount, in bytes, of NFS pages sent to the server
  but not yet committed to the stable storage.
* `memory/numa/foreign` - the number of bytes which had to be allocated on a
  remote node even though the allocation should have been local
* `memory/numa/hit` - the number of bytes successfully allocated on the intended
  node
* `memory/numa/interleave` - the number of bytes allocated on the remote node as
  intended by interleave policy
* `memory/numa/local` - the number of bytes allocated on the node where the
  process was running at time of allocation
* `memory/numa/miss` - the number of bytes which could not be allocated on the
  intended node
* `memory/numa/other` - the number of bytes allocated on a node where the
  process was not running at time of allocation
* `memory/page_tables` - the total amount of memory, in bytes, dedicated to the
  lowest page table level.
* `memory/shmem_hugepages` - the number of hugepages which are used for shared
  memory allocated as transparent hugepages
* `memory/shmem_pmd_mapped` - the number of hugepages which are used for
  application transparent hugepages
* `memory/shmem` - the total amount of memory, in bytes, used by shared memory
  (shmem) and tmpfs.
* `memory/slab/reclaimable` - the part of Slab that can be reclaimed, such as
  caches.
* `memory/slab/total` - the total amount of memory, in bytes, used by the kernel
  to cache data structures for its own use.
* `memory/slab/unreclaimable` - the part of Slab that cannot be reclaimed even
  when lacking memory.
* `memory/swap/cached` - the amount of memory, in bytes, that has once been
  moved into swap, then back into the main memory, but still also remains in the
  swapfile. This saves I/O, because the memory does not need to be moved into
  swap again.
* `memory/swap/free` - the total amount of swap free, in bytes.
* `memory/swap/total` - the total amount of swap available, in bytes.
* `memory/thp/collapse_alloc` - number of times a hugepage was successfully
  allocated to collapse multiple pages
* `memory/thp/collapse_alloc_failed` - number of times the allocation of a
  hugepage failed when attempting to collapse multiple pages
* `memory/thp/deferred_split_page` - number of times a page split was deferred
  by placing it on the split queue. This means the page is partially unmapped
  and splitting will free some memory
* `memory/thp/fault_alloc` - the number of times a huge page was allocated to
  satisfy a page fault
* `memory/thp/fault_fallback` - the number of times a page fault required a
  base page allocation following a failure allocating a huge page
* `memory/thp/split_page` - the number of huge pages which have been split into
  base pages
* `memory/thp/split_page_failed` - the number of times a huge page split failed
* `memory/total` - total amount of usable RAM, in bytes, which is physical RAM
  minus a number of reserved bits and the kernel binary code
* `memory/unevictable` - the amount of memory, in bytes, discovered by the
  pageout code, that is not evictable because it is locked into memory by user
  programs.
* `memory/vmalloc/chunk` - the largest contiguous block of memory, in bytes, of
  available virtual address space.
* `memory/vmalloc/total` -  total amount of memory, in bytes, of total allocated
  virtual address space.
* `memory/vmalloc/used` - total amount of memory, in bytes, of used virtual
  address space.
* `memory/writeback_temp` - the amount of memory, in bytes, used by FUSE for
  temporary writeback buffers.
* `memory/writeback` - the total amount of memory, in bytes, actively being
  written back to the disk.

## Network

Provides system-wide network telemetry

### Basic

* `network/receive/bytes` - number of bytes received on all network interfaces
* `network/receive/compressed` - number of compressed packets received
* `network/receive/drops` - number of received packets which were dropped by the
  device driver
* `network/receive/errors` - number of receive errors detected by the device
  driver
* `network/receive/fifo` - number of FIFO buffer errors on receive
* `network/receive/frame` - number of packets received with framming errors
* `network/receive/multicast` - number of multicast packets received
* `network/receive/packets` - total number of packets received
* `network/transmit/bytes` - number of bytes transmitted on all network
  interfaces
* `network/transmit/carrier` - number of carrier losses detected by the device
 driver
* `network/transmit/collisions` - number of collisions detected
* `network/transmit/compressed` - number of compressed packets transmitted
* `network/transmit/drops` - number of packets to transmit which were dropped by
  the device driver
* `network/transmit/errors` - total number of errors when transmitting packets
* `network/transmit/fifo` - number of FIFO buffer errors on transmit
* `network/transmit/packets` - total number of packets transmitted

### BPF

* `network/receive/size` - size distribution, in bytes, of received packets
* `network/transmit/size` - size distribution, in bytes, of transmitted packets

## NTP

NTP sampler provides some basic stats about time synchronization via NTP.

**NOTE:** this sampler is currently not supported for musl toolchains

### Basic

* `ntp/estimated_error` - the current estimated error of the local clock in
  nanoseconds
* `ntp/maximum_error` - the maximum error of the local clock in nanoseconds

## Nvidia

Telemetry for Nvidia GPUs, collected by using the Nvidia Management Library
(NVML). Unlike other samplers, these stats are fully scoped to specific GPUs
within the system. Exported metrics will have the form: `nvidia/gpu_[id]/...`
where the id is the device identifier as reported by the NVML. The set of
metrics to collect uses the short form of the metric name, as provided below.

### Basic

* `clock/sm/current` - current streaming multiprocessor clock speed in MHz
* `clock/memory/current` - current memory clock speed in MHz
* `decoder/utilization` - video decoder utilization as a percentage
* `encoder/utilization` - video encoder utilization as a percentage
* `energy/consumption` - total energy consumption since boot in Joules
* `gpu/temperature` - current GPU temperature in °C
* `cpu/utilization` - GPU utilzation as a percentage
* `memory/ecc/enabled` - boolean (0 or 1) indicating if ECC is enabled
* `memory/ecc/dbe` - count of double-bit errors (uncorrectable)
* `memory/ecc/sbe` - count of single-bit errors (correctable)
* `memory/fb/free` - framebuffer memory free in bytes
* `memory/fb/total` - total framebuffer memory in bytes
* `memory/fb/used` - framebuffer memory used in bytes
* `memory/retired/sbe` - memory pages retired due to multiple single-bit errors
* `memory/retired/dbe` - memory pages retired due to double-bit error
* `memory/retired/pending` - boolean (0 or 1) indicating that memory pages are
  pending retirement
* `memory/utilization` - memory copy utilization as a percentage
* `pcie/replay` - count of PCIe replays. May indicate link issues.
* `pcie/rx/throughput` - PCIe receive throughput in KB/s
* `pcie/tx/throughput` - PCIe transmit throughput in KB/s
* `power/limit` - enforced power limit in Watts
* `power/usage` - current power usage in Watts
* `processes/compute` - number of processes running in compute context

## Page Cache

The page cache is a transparent cache for pages originating from a secondary
storage. Telemetry about page cache performance can be useful for when tuning
applications which rely on the page cache.

### BPF
* `page_cache/hit` - the number of times a read request was served from the page
  cache
* `page_cache/miss` - the number of times a read request resulted in a page
  cache miss 

## Rezolus

Provides telemetry about Rezolus itself. This can be used to understand the
runtime characteristics of Rezolus for various configurations.

### Basic
* `rezolus/cpu/user` - nanoseconds spent in user mode running Rezolus
* `rezolus/cpu/system` - nanoseconds spent in system mode running Rezolus
* `rezolus/memory/virtual` - total virtual memory allocated to Rezolus
* `rezolus/memory/resident` - amount of memory actually used by Rezolus


## Scheduler

Provides telemetry about the Linux scheduler. Provides insights into
thread/process characteristics. The runqueue latency is useful when performing
scheduler tuning or investigating potential interference between workloads.

### Basic

* `scheduler/context_switches` - number of context switches
* `scheduler/processes/created` - number of processes created
* `scheduler/processes/running` - number of processes currently running
* `scheduler/processes/blocked` - number of processes currently blocked

### Perf Events

* `scheduler/cpu_migrations` - number of times processes have been migrated
  across CPUs

### BPF

* `scheduler/runqueue/latency` - the distribution of time that runnable tasks
  were waiting on the runqueue

## Softnet

Softnet telemetry provides a view into kernel packet processing.

### Basic

* `softnet/processed` - the total number of packets processed in the softnet
  layer
* `softnet/dropped` - the number of packets dropped
* `softnet/time_squeezed` - number of times that packet processing did not
  complete within the time slice
* `softnet/cpu_collision` - collisions occurring obtaining device lock while
  transmitting
* `softnet/received_rps` - number of times cpus woken up for received rps
* `softnet/flow_limit_count` - number of times the flow limit count was reached

## TCP

This sampler provides telemetry about TCP traffic and connections.

## Basic

* `tcp/abort/failed` - failed to send RST on abort due to memory pressure
* `tcp/abort/on_close` - connections reset due to early user close
* `tcp/abort/on_data` - connections reset due to unexpected data
* `tcp/abort/on_linger` - connections reset after user close while in linger
  timeout
* `tcp/abort/on_memory` - connections reset due to memory pressure or too many
  orphaned sockets
* `tcp/abort/on_timeout` - connections reset due to timeout
* `tcp/receive/checksum_error` - segments received with invalid checksum
* `tcp/receive/collapsed` - segments collapsed in the receive queue
* `tcp/receive/error` - total number of errors on receive
* `tcp/receive/listen_drops` - number of SYNs to LISTEN sockets ignored
* `tcp/receive/listen_overflows` - times the listen queue of a socket overflowed
* `tcp/receive/ofo_pruned` - number of packets pruned from the out-of-order
  queue due to socket buffer overrun
* `tcp/receive/prune_called` - number of packets pruned from the receive queue
  because of socket buffer overrun
* `tcp/receive/pruned` - packets pruned from the receive queue
* `tcp/receive/segment` - total number of segments received
* `tcp/syncookies/failed` - number of invalid SYN cookies received
* `tcp/syncookies/received` - number of SYN cookies received
* `tcp/syncookies/sent` - number of SYN cookies sent
* `tcp/transmit/delayed_ack` - number of delayed ACKs sent
* `tcp/transmit/reset` - number of RSTs sent
* `tcp/transmit/retransmit` - number of segments retransmitted
* `tcp/transmit/segment` - number of segments transmitted

### BPF

* `tcp/connect/latency` - end-to-end latency, in nanoseconds, from an active
  outbound `connect()` until the socket is established

## UDP

* `udp/receive/datagrams` - number of datagrams received
* `udp/receive/errors` - number of errors on receive
* `udp/transmit/datagrams` - number of datagrams transmitted

## XFS

Provides telemetry about XFS filesystem performance.

### BPF
* `xfs/fsync/latency` - latency distribution, in nanoseconds, for `fsync()` on
  xfs filesystems
* `xfs/open/latency` - latency distribution, in nanoseconds, for `open()` on
  xfs filesystems
* `xfs/read/latency` - latency distribution, in nanoseconds, for `read()` on
  xfs filesystems
* `xfs/write/latency` - latency distribution, in nanoseconds, for `write()` on
  xfs filesystems
