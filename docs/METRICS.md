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
* `cpu/usage/guest` - nanoseconds spent running a guest VM
* `cpu/usage/guestnice` - nanoseconds spent running a low-priority guest VM
* `cpu/usage/idle` - nanoseconds spent idle
* `cpu/usage/irq` - nanoseconds spent handling interrupts
* `cpu/usage/nice` - nanoseconds spent on lower-priority tasks
* `cpu/usage/softirq` - nanoseconds spent handling soft interrupts
* `cpu/usage/steal` - nanoseconds stolen by the hypervisor
* `cpu/usage/system` - nanoseconds spent in kernel-space
* `cpu/usage/user` - nanoseconds spent in user-space

### perf_events
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

### eBPF

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

### eBPF

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
* `interrupt/nvme` - interrupts for servicing NVMe queues
* `interrupt/performance_monitoring` - interrupts generated when a performance
  counter overflows or PEBS interrupt threshold is reached
* `interrupt/rescheduling` - interrupts used to notify a core to schedule a
  thread
* `interrupt/rtc` - interrupts caused by the realtime clock
* `interrupt/serial` - interrupts caused by serial ports
* `interrupt/spurious` - interrupts which were marked spurious and not handled
* `interrupt/thermal_event` - interrupts caused by thermal events, like
  throttling
* `interrupt/timer` - interrupts related to the system timer (PIT/HPET)
* `interrupt/tlb_shootdowns` - interrupts caused to trigger TLB shootdowns

## Memory

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

### eBPF

* `network/receive/size` - size distribution, in bytes, of received packets
* `network/transmit/size` - size distribution, in bytes, of transmitted packets

## Rezolus

Provides telemetry about Rezolus itself

### Basic
* `rezolus/cpu/user` - nanoseconds spent in user mode running Rezolus
* `rezolus/cpu/system` - nanoseconds spent in system mode running Rezolus
* `rezolus/memory/virtual` - total virtual memory allocated to Rezolus
* `rezolus/memory/resident` - amount of memory actually used by Rezolus


## Scheduler

Provides telemetry about the Linux Scheduler

### Basic

* `scheduler/context_switches` - number of context switches
* `scheduler/processes/created` - number of processes created
* `scheduler/processes/running` - number of processes currently running
* `scheduler/processes/blocked` - number of processes currently blocked

### perf_events

* `scheduler/cpu_migrations` - number of times processes have been migrated
  across CPUs

### eBPF

* `scheduler/cfs/throttled` - the distribution of time cgroups spent throttled
* `scheduler/runqueue/latency` - the distribution of time that runnable tasks
  were waiting on the runqueue

## Softnet

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

## Basic

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

### eBPF

* `tcp/connect/latency` - end-to-end latency, in nanoseconds, from an active
  outbount `connect()` until the socket is established

## UDP

* `udp/receive/datagrams` - number of datagrams received
* `udp/receive/errors` - number of errors on receive
* `udp/transmit/datagrams` - number of datagrams transmitted


## XFS

* `xfs/fsync/latency` - latency distribution, in nanoseconds, for `fsync()` on
  xfs filesystems
* `xfs/open/latency` - latency distribution, in nanoseconds, for `open()` on
  xfs filesystems
* `xfs/read/latency` - latency distribution, in nanoseconds, for `read()` on
  xfs filesystems
* `xfs/write/latency` - latency distribution, in nanoseconds, for `write()` on
  xfs filesystems
