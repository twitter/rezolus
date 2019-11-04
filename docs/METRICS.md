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

## Container

Instruments the container Rezolus is running within. Providing enhanced
telemetry for containerized environments like Mesos by running as a sidecar
process.

* `container/cpu/system` - the amount of time, in nanoseconds, the container has
  spent in kernel-space
* `container/cpu/total` - the amount of time, in nanoseconds, the container has
  been running
* `container/cpu/user` - the amount of time, in nanoseconds, the container has
  spent in user-space

## CPU

Provides system-wide CPU utilization telemetry. The following are taken from
`/proc/stat`

* `cpu/user` - the amount of time, in nanoseconds, spent in user-space
* `cpu/nice` - the amount of time, in nanoseconds, spent on lower-priority
  tasks
* `cpu/system` - the amount of time, in nanoseconds, spent in kernel-space
* `cpu/idle` - the amount of time, in nanoseconds, where nothing is running
* `cpu/irq` - the amount of time, in nanoseconds, handling interrupts
* `cpu/softirq` - the amount of time, in nanoseconds, handling soft interrupts
* `cpu/steal` - the amount of time, in nanoseconds, stolen by the hypervisor
* `cpu/guest` - the amount of time, in nanoseconds, running a guest VM
* `cpu/guest_nice` - the amount of time, in nanoseconds, running a low-priority
  guest VM

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

### Scheduler

Captures system-wide scheduler telemetry

* `scheduler/runqueue_latency_ns` - distribution of the amount of time in
  nanoseconds that runnable tasks are waiting to be scheduled onto a core

### TCP

Captures additional system-wide telemetry for TCP

* `network/tcp/connect/latency` - distribution of latency for establishing
active (outbound) connections

### XFS

Capture system-wide filesystem latency for XFS

* `xfs/read` - distribution of latency for read operations in nanoseconds
* `xfs/write` - distribution of latency for write operations in nanoseconds
* `xfs/fsync` - distribution of latency for fsync operations in nanoseconds
* `xfs/open` - distribution of latency for open operations in nanoseconds

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

