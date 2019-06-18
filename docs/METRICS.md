# Metrics

Below you will find a list of possible metrics which Rezolus can export as
well as details on their source and meaning. Metrics below may typically be
followed by:
* `/count` - the value of the counter
* `/histogram/(percentile)` - a percentile of a counter's secondly rate or the
percentile taken from a distribution
* `/maximum/value` - the maximum of a counter's secondly rate or the maximum
value taken from a distribution
* `/maximum/offset_ms` - the offset into the minute at which the maximum occured

## CPU

The following are taken from `/proc/stat`

* `cpu/user` - the amount of time, in nanoseconds, spent in user-space
* `cpu/nice` - the amount of time, in nanoseconds, spent on lower-priority tasks
* `cpu/system` - the amount of time, in nanoseconds, spent in kernel-space
* `cpu/idle` - the amount of time, in nanoseconds, where nothing is running
* `cpu/irq` - the amount of time, in nanoseconds, handling interrupts
* `cpu/softirq` - the amount of time, in nanoseconds, handling soft interrupts
* `cpu/steal` - the amount of time, in nanoseconds, stolen by the hypervisor
* `cpu/guest` - the amount of time, in nanoseconds, running a guest VM
* `cpu/guest_nice` - the amount of time, in nanoseconds, running a low-priority
guest VM

## Disk

The following are taken from `/sys/class/block/...`

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

Capture the IO sizes at the block layer

* `block/io_size_kb` - distribution of block IO sizes in kilobytes

### EXT4

Capture filesystem latency for EXT4

* `ext4/read` - distribution of latency for read operations in nanoseconds
* `ext4/write` - distribution of latency for write operations in nanoseconds
* `ext4/fsync` - distribution of latency for fsync operations in nanoseconds
* `ext4/open` - distribution of latency for open operations in nanoseconds

### Scheduler

Captures scheduler telemetry

* `scheduler/runqueue_latency_ns` - distribution of the amount of time in
nanoseconds that runnable tasks are waiting to be scheduled onto a core

### XFS

Capture filesystem latency for XFS

* `xfs/read` - distribution of latency for read operations in nanoseconds
* `xfs/write` - distribution of latency for write operations in nanoseconds
* `xfs/fsync` - distribution of latency for fsync operations in nanoseconds
* `xfs/open` - distribution of latency for open operations in nanoseconds

## Network

Capture telemetry for network interfaces and protocols. Reads from
`/sys/class/net/...`, `/proc/net/snmp`, and `/proc/net/netstat`

### Interface telemetry:

* `network/receive/bytes` - `rx_bytes` number of bytes received
* `network/receive/errors/crc` - `rx_crc_errors` number of packets with CRC
error. Specific meaning may vary depending on the MAC layer, but could mean
there is packet corruption.
* `network/receive/errors/discards_phy` - `rx_discards_phy` number of packets
dropped due to lack of buffer space on the NIC. Implies the adapter is congested
and cannot absorb the traffic coming from the network.
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
congested and acnnot absorb the traffic. Applies to: `mlx5`
* `network/transmit/dropped` - `tx_dropped` number of packets dropped on
transmit
* `network/transmit/errors/total` - `tx_errors` number of errors on transmit
* `network/transmit/errors/fifo` - `tx_fifo_errors` Indicates number of transmit
FIFO errors seen by this network device. Applies to: `mlx4`
* `network/transmit/packets` - `tx_packets` number of packets transmitted

### TCP Telemetry

* `network/tcp/receive/segments` - `Tcp: InSegs` number of TCP segments received
* `network/tcp/transmit/segments` - `Tcp: OutSegs` number of TCP segments sent
* `network/tcp/receive/prune_called` - `TcpExt: PruneCalled` indicates extreme
memory pressure on the TCP buffers and that the kernel is dropping packets. This
is very bad.
* `network/tcp/receive/collapsed` - `TcpExt: RcvCollapsed` indicates memory
pressure on the TCP buffers
* `network/tcp/transmit/retransmits` - `Tcp: RetransSegs` indicates number of
segments which have been retransmitted

### UDP Telemetry

* `network/udp/receive/datagrams` - `Udp: InDatagrams` indicates number of
datagrams received
* `network/udp/receive/errors` - `Udp: InErrors` indicates number of errors on
incoming datagrams
* `network/udp/transmit/datagrams` - `Udp: OutDatagrams` indicates number of
datagrams transmitted

## Perf

The following telemetry is gathered from the perf events subsystem

* `cache/misses` - number of cache references resulting in miss
* `cache/references` - total number of cache references
* `system/context_switches` - number of context switches
* `cpu/branch_instructions` - total number of branch instructions
* `cpu/branch_misses` - number of branch predictions missed
* `cpu/cycles` - number of cycles **may not be accurate with frequency scaling**
* `cpu/instructions` - number of instructions retired
* `cpu/reference_cycles` - number of cycles **accurate**
* `system/cpu_migrations` - number of times a task migrated between cores
* `cache/dtlb/read/references` - total number of read references to the dTLB
* `cache/dtlb/read/misses` - number of dTLB reads resulting in miss
* `cache/dtlb/write/references` - total number of write references to the dTLB
* `cache/dtlb/write/misses` - number of dTLB writes resulting in miss
* `system/page_faults` - number of page faults

## Softnet

This telemetry is gathered from `/proc/net/softnet_stat`

* `softnet/processed` - total number of packets processed by the kernel network
stack
* `softnet/dropped` - number of packets dropped by the kernel network stack
* `softnet/time_squeezed` - number of times the kernel network stack could not
complete its work within its working interval. Indicates that the network stack
is overwhelmed or unable to get sufficient CPU time.

