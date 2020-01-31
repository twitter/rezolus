# Metrics

Below you will find a list of possible metrics which Rezolus can export as well
as details on their source and meaning. Metrics below may typically be followed
by:
* `/count` - the value of the counter
* `/histogram/(percentile)` - a percentile of a counter's secondly rate or the
  percentile taken from a distribution
* `/maximum/value` - the maximum of a counter's secondly rate or the maximum
  value taken from a distribution
* `/maximum/offset_ms` - the offset into the minute at which the maximum
  occurred

Sampler configurations will refer to the metrics according to their basenames as
used in the descriptions below.

## CPU

Provides system-wide CPU telemetry.

### Basic
* `cpu/cstate/c0/time` -
* `cpu/cstate/c1/time` -
* `cpu/cstate/c1e/time` -
* `cpu/cstate/c2/time` -
* `cpu/cstate/c3/time` -
* `cpu/cstate/c6/time` -
* `cpu/cstate/c7/time` -
* `cpu/cstate/c8/time` -
* `cpu/usage/guest` - the amount of time, in nanoseconds, running a guest VM
* `cpu/usage/guestnice` - the amount of time, in nanoseconds, running a
  low-priority guest VM
* `cpu/usage/idle` - the amount of time, in nanoseconds, where nothing is
  running
* `cpu/usage/irq` - the amount of time, in nanoseconds, handling interrupts
* `cpu/usage/nice` - the amount of time, in nanoseconds, spent on lower-priority
  tasks
* `cpu/usage/softirq` - the amount of time, in nanoseconds, handling soft
  interrupts
* `cpu/usage/steal` - the amount of time, in nanoseconds, stolen by the
  hypervisor
* `cpu/usage/system` - the amount of time, in nanoseconds, spent in kernel-space
* `cpu/usage/user` - the amount of time, in nanoseconds, spent in user-space

### perf_events
* `cpu/bpu/branch` -
* `cpu/bpu/miss` -
* `cpu/cache/access` -
* `cpu/cache/miss` -
* `cpu/cycles` -
* `cpu/dtlb/load/access` -
* `cpu/dtlb/load/miss` -
* `cpu/dtlb/store/access` -
* `cpu/dtlb/store/miss` -
* `cpu/instructions` -
* `cpu/reference_cycles` -
* `cpu/stalled_cycles/backend` -
* `cpu/stalled_cycles/frontend` -


## Disk

Provides system-wide telemetry for disk devices

### Basic

* `disk/discard/bytes` -
* `disk/discard/operations` -
* `disk/read/bytes` -
* `disk/read/operations` - 
* `disk/write/bytes` -
* `disk/write/operations` -

### eBPF

* `disk/read/device_latency` -
* `disk/read/latency` -
* `disk/read/io_size` -
* `disk/read/queue_latency` -
* `disk/write/device_latency` -
* `disk/write/io_size` -
* `disk/write/latency` -
* `disk/write/queue_latency` -

## EXT4

Provides system-wide telemetry for EXT4 filesystems

### eBPF

* `ext4/fsync/latency` -
* `ext4/open/latency` -
* `ext4/read/latency` -
* `ext4/write/latency` -

## Memory

### Basic

* `memory/active/anon`
* `memory/active/file`
* `memory/active/total`
* `memory/anon_hugepages`
* `memory/anon_pages`
* `memory/available`
* `memory/bounce`
* `memory/buffers`
* `memory/cached`
* `memory/commit/committed`
* `memory/commit/limit`
* `memory/directmap/1G`
* `memory/directmap/2M`
* `memory/directmap/4k`
* `memory/dirty`
* `memory/free`
* `memory/hardware_corrupted`
* `memory/hugepage_size`
* `memory/hugepages/free`
* `memory/hugepages/reserved`
* `memory/hugepages/surplus`
* `memory/hugepages/total`
* `memory/hugetlb`
* `memory/inactive/anon`
* `memory/inactive/file`
* `memory/inactive/total`
* `memory/kernel_stack`
* `memory/mapped`
* `memory/mlocked`
* `memory/nfs_unstable`
* `memory/page_tables`
* `memory/percpu`
* `memory/shmem_hugepages`
* `memory/shmem_pmd_mapped`
* `memory/shmem`
* `memory/slab/reclaimable`
* `memory/slab/total`
* `memory/slab/unreclaimable`
* `memory/swap/cached`
* `memory/swap/free`
* `memory/swap/total`
* `memory/total`
* `memory/unevictable`
* `memory/vmalloc/chunk`
* `memory/vmalloc/total`
* `memory/vmalloc/used`
* `memory/writeback_temp`
* `memory/writeback`

## Network

Provides system-wide network telemetry

### Basic

* `network/receive/bytes`
* `network/receive/compressed`
* `network/receive/drops`
* `network/receive/errors`
* `network/receive/fifo`
* `network/receive/frame`
* `network/receive/multicast`
* `network/receive/packets`
* `network/transmit/bytes`
* `network/transmit/carrier`
* `network/transmit/collisions`
* `network/transmit/compressed`
* `network/transmit/drops`
* `network/transmit/errors`
* `network/transmit/fifo`
* `network/transmit/packets`

### eBPF

* `network/receive/size`
* `network/transmit/size`

## Rezolus

Provides telemetry about Rezolus itself

### Basic
* `rezolus/cpu/user`
* `rezolus/cpu/system`
* `rezolus/memory/virtual`
* `rezolus/memory/resident`


## Scheduler

Provides telemetry about the Linux Scheduler

### Basic

* `scheduler/context_switches`
* `scheduler/processes/created`
* `scheduler/processes/running`
* `scheduler/processes/blocked`

### perf_events

* `scheduler/cpu_migrations`

### eBPF

* `scheduler/runqueue/latency`


## Softnet

### Basic

* `softnet/processed`
* `softnet/dropped`
* `softnet/time_squeezed`
* `softnet/cpu_collision`
* `softnet/received_rps`
* `softnet/flow_limit_count`



## TCP

* `tcp/receive/checksum_error`
* `tcp/receive/collapsed`
* `tcp/receive/error`
* `tcp/receive/listen_drops`
* `tcp/receive/listen_overflows`
* `tcp/receive/ofo_pruned`
* `tcp/receive/prune_called`
* `tcp/receive/pruned`
* `tcp/receive/segment`
* `tcp/syncookies/failed`
* `tcp/syncookies/received`
* `tcp/syncookies/sent`
* `tcp/transmit/delayed_ack`
* `tcp/transmit/reset`
* `tcp/transmit/retransmit`
* `tcp/transmit/segment`

### eBPF

* `tcp/connect/latency`


## UDP

* `udp/receive/datagrams`
* `udp/receive/errors`
* `udp/transmit/datagrams`


## XFS

* `xfs/fsync/latency`
* `xfs/open/latency`
* `xfs/read/latency`
* `xfs/write/latency`
