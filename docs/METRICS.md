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




## CPU Idle

Provides system-wide telemetry for CPU idle states. The following are taken from
`/sys/devices/system/cpu/...`

* `cpuidle/state0` - the number of nanoseconds the CPUs have spent in State 0
* `cpuidle/state1` - the number of nanoseconds the CPUs have spent in State 1
* `cpuidle/state2` - the number of nanoseconds the CPUs have spent in State 2
* `cpuidle/state3` - the number of nanoseconds the CPUs have spent in State 3

## Disk

Provides system-wide telemetry for block storage. The following are taken from
`/sys/class/block/...`

* `disk/bandwidth/read` - the number of bytes read from disk
* `disk/bandwidth/write` - the number of bytes written to disk
* `disk/operations/read` - the number of IOs servicing reads
* `disk/operations/write` - the number of IOs servicing writes

## Rezolus

The following capture the resource utilization of Rezolus

* `rezolus/memory/virtual` - the virtual address space in bytes
* `rezolus/memory/resident` - the number of bytes of RAM occupied
* `rezolus/cpu/kernel` - the amount of time, in nanoseconds, spent in
  kernel-mode
* `rezolus/cpu/user` - the amount of time, in nanoseconds, spent in user-space

## eBPF

All of the following subsections are eBPF telemetry

### Block

Captures system-wide telemetry about block IO

* `block/device_latency/read` - distribution of device latency for read
* `block/device_latency/write` - distribution of device latency for write
* `block/latency/read` - distribution of end-to-end latency for read
* `block/latency/write` - distribution of end-to-end latency for write
* `block/queue_latency/read` - distribution of queue latency for read
* `block/queue_latency/write` - distribution of queue latency for write
* `block/size/read` - distribution of sizes in kilobytes for reads
* `block/size/write` - distribution of sizes in kilobytes for writes


### EXT4

Capture system-wide filesystem latency for EXT4

* `ext4/read` - distribution of latency for read operations in nanoseconds
* `ext4/write` - distribution of latency for write operations in nanoseconds
* `ext4/fsync` - distribution of latency for fsync operations in nanoseconds
* `ext4/open` - distribution of latency for open operations in nanoseconds

### Network

Provides additional system-wide telemetry for networking

* `network/receive/size` - distribution of received packet sizes in bytes
* `network/transmit/size` - distribution of transmitted packet sizes in bytes

### Perf

 * `cpu/cache/misses`
 * `cpu/cache/references`
 * `cpu/branch_prediction/instructions`
 * `cpu/branch_prediction/misses`
 * `cpu/cycles`
 * `cpu/instructions`
 * `scheduler/cpu_migrations`
 * `scheduler/context_switches`
 * `cpu/reference_cycles`
 * `cpu/dtlb/load`
 * `cpu/dtlb/load/misses`
 * `cpu/dtlb/store`
 * `cpu/dtlb/store/misses`
 * `memory/load`
 * `memory/store`
 * `memory/store/misses`
 * `memory/load/misses`
 * `cpu/page_fault`
 * `cpu/stalled_cycles/frontend`
 * `cpu/stalled_cycles/backend`

### Rezolus

 * `rezolus/cpu/user` - amount of user cpu time used by Rezolus
 * `rezolus/cpu/system` - amount of system cpu time used by Rezolus
 * `rezolus/memory/virtual` - amount of virtual memory allocated to Rezolus
 * `rezolus/memory/resident` - amount of memory occupied by Rezolus

### Scheduler

#### eBPF

* `scheduler/runqueue/latency` - amount of time runnable tasks are waiting in
  the runqueue before being scheduled (nanoseconds)

### Softnet

 * `softnet/cpu_collision` - number of times collision occurred on obtaining device
   lock while transmitting
 * `softnet/dropped` - number of frames dropped due to no room on processing
   queue
 * `softnet/flow_limit_count` - number of times the flow limit has been reached
 * `softnet/processed` - total number of frames processed
 * `softnet/received_rps` - number of times CPU has been woken up to process
   packets via inter-processor interrupt
 * `softnet/time_squeezed` - number of times net_rx_action had more work, but
   budget or time exhausted

### TCP

 * `tcp/receive/checksum_errors` - tcp segments received with checksum errors
 * `tcp/receive/collapsed` - tcp packets collapsed in receive queue due to low
   socket buffer
 * `tcp/receive/ofo_pruned` - TCP packets dropped from out-of-order queue due to
   low socket buffer
 * `tcp/receive/prune_called` - number of times pruning has been run on the
   receive queue
 * `tcp/receive/pruned` - TCP packets pruned from receive queue
 * `tcp/receive/segments` - tcp segments received
 * `tcp/receive/errors` - tcp segments received in error
 * `tcp/syncookies/failed` - number of failed SYN cookies
 * `tcp/syncookies/received` - number of received SYN cookies
 * `tcp/syncookies/sent` - number of sent SYN cookies
 * `tcp/transmit/delayed_acks` - number of delayed ACKs sent
 * `tcp/receive/listen_drops` - number of SYNs to LISTEN sockets dropped
 * `tcp/receive/listen_overflows` - number of times the listen queue of a socket
   overflowed
 * `tcp/transmit/resets` - tcp segments transmitted with the RST flag
 * `tcp/transmit/retransmits` - tcp segments retransmitted
 * `tcp/transmit/segments` - tcp segments transmitted

#### eBPF

 * `tcp/connect/latency` - latency of active TCP connect

## UDP

 * `udp/receive/datagrams` - UDP datagrams received
 * `udp/receive/errors` - UDP datagrams that were not delivered to valid port
 * `udp/transmit/datagrams` - UDP datagrams transmitted

### XFS

#### eBPF

* `xfs/fsync/latency` - XFS fsync latency (nanoseconds)
* `xfs/open/latency` - XFS open latency (nanoseconds)
* `xfs/read/latency` - XFS read latency (nanoseconds)
* `xfs/write/latency` - XFS write latency (nanoseconds)

## Network

Capture system-wide telemetry for network interfaces and protocols. Reads from
`/sys/class/net/...`, `/proc/net/snmp`, and `/proc/net/netstat`

### Interface telemetry

* `network/receive/bytes` - `rx_bytes` number of bytes received
* `network/receive/errors/crc` - `rx_crc_errors` number of packets with CRC
  error. Specific meaning may vary depending on the MAC layer, but could mean
  there is packet corruption.
* `network/receive/errors/discards_phy` - `rx_discards_phy` number of packets
  dropped due to lack of buffer space on the NIC. Implies the adapter is
  congested and cannot absorb the traffic coming from the network.
* `network/receive/dropped` - `rx_dropped_errors` number of packets dropped and
  not forwarded to the upper layers for packet processing. Exact meaning varies
  with network driver.
* `network/receive/errors/total` - `rx_errors` the number of errors on receive
* `network/reveive/errors/fifo` - `rx_fifo_errors` Indicates number of receive
  FIFO errors seen by this network device. Applies to: `mlx4`
* `network/receive/errors/misses` - `rx_missed_errors` Indicates number of
  packets which have been missed due to lack of capacity in the receive side.
  Applies to: `ixgbe`
* `network/receive/packets` - `rx_packets` the total number of packets received
* `network/transmit/bytes` - `tx_bytes` the number of bytes transmitted
* `network/transmit/errors/discards_phy` - `tx_discards_phy` the number of
  packets dropped due to lack of buffers on transmit. Implies the adapter is
  congested and cannot absorb the traffic. Applies to: `mlx5`
* `network/transmit/dropped` - `tx_dropped` number of packets dropped on
  transmit
* `network/transmit/errors/total` - `tx_errors` number of errors on transmit
* `network/transmit/errors/fifo` - `tx_fifo_errors` Indicates number of
  transmit FIFO errors seen by this network device. Applies to: `mlx4`
* `network/transmit/packets` - `tx_packets` number of packets transmitted

### Protocol telemetry

#### TCP Telemetry

* `network/tcp/receive/segments` - `tcp_in_segs` number of TCP segments
  received
* `network/tcp/transmit/segments` - `tcp_out_segs` number of TCP segments sent
* `network/tcp/receive/prune_called` - `tcp_prune_called` indicates extreme
  memory pressure on the TCP buffers and that the kernel is dropping packets.
  This is very bad.
* `network/tcp/receive/collapsed` - `tcp_rcv_collapsed` indicates memory
  pressure on the TCP buffers
* `network/tcp/transmit/retransmits` - `tcp_retrans_segs` indicates number of
  segments which have been retransmitted

#### UDP Telemetry

* `network/udp/receive/datagrams` - `udp_in_datagrams` indicates number of
  datagrams received
* `network/udp/receive/errors` - `udp_in_errors` indicates number of errors on
  incoming datagrams
* `network/udp/transmit/datagrams` - `udp_out_datagrams` indicates number of
  datagrams transmitted

## Perf

The following telemetry is gathered from the perf events subsystem and provides
a view into hardware and software performance system-wide

* `perf/cache/dtlb/read/references` - `dtlb_loads` total number of read
  references to the dTLB
* `perf/cache/dtlb/read/misses` - `dtlb_load_misses` number of dTLB reads
  resulting in miss
* `perf/cache/dtlb/write/references` - `dtlb_stores` total number of write
  references to the dTLB
* `perf/cache/dtlb/write/misses` - `dtlb_store_misses` number of dTLB writes
  resulting in miss
* `perf/cache/misses` - `cache_misses` number of cache references resulting in
  miss
* `perf/cache/references` - `cache_references` total number of cache references
* `perf/cpu/branch_instructions` - `cpu_branch_instruction` total number of
  branch instructions
* `perf/cpu/branch_misses` - `cpu_branch_misses` number of branch predictions
  missed
* `perf/cpu/cycles` - `cpu_cycles` number of cycles **may not be accurate with
  frequency scaling**
* `perf/cpu/cycles/stalled/backend` - `stalled_cycles_backend` number of cycles
  stalled waiting on backend
* `perf/cpu/cycles/stalled/frontend` - `stalled_cycles_frontend` number of
  cycles stalled waiting on frontend
* `perf/cpu/instructions` - `cpu_instructions` number of instructions retired
* `perf/cpu/reference_cycles` - `cpu_ref_cycles` number of cycles **accurate**
* `perf/memory/read/references` - `memory_loads` number of memory read accesses
* `perf/memory/read/misses` - `memory_load_misses` number of memory reads
  resulting in miss
* `perf/memory/write/references` - `memory_stores` number of memory write
  accesses
* `perf/memory/write/misses` - `memory_store_misses` number of memory writes
  resulting in miss
* `perf/system/context_switches` - `context_switches` number of context switches
* `perf/system/cpu_migrations` - `cpu_migrations` number of times a task
  migrated between cores
* `perf/system/page_faults` - `page_faults` number of page faults

## Softnet

Provides system-wide telemetry about packet processing gathered from
`/proc/net/softnet_stat`

* `softnet/processed` - total number of packets processed by the kernel network
  stack
* `softnet/dropped` - number of packets dropped by the kernel network stack
* `softnet/time_squeezed` - number of times the kernel network stack could not
  complete its work within its working interval. Indicates that the network
  stack is overwhelmed or unable to get sufficient CPU time.

