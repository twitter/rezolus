# [Unreleased]

# [2.15.0] - 2020-12-03
## Changed
- Allow selective enablement of various BPF metrics. (#254)
- Support up to BCC 0.23.0 and makes it the new default version. (#256)
- Removed ssl support in http sampler to remove dependency on openssl. (#257)

## Added
- Adds TCP jitter and connections accepted and initiated using BPF. (#247)
- Adds TCP packet drops counter using BPF. (#250)
- Adds Pelikan specific stats to memcache sampled. (#249)
- Adds TCP tail loss recovery and retransmit timeout using BPF. (#253)
- Adds TCP duplicate segment and out-of-order segment counters using BPF. (#255)

## Fixed
- Improved handling of BPF initialization errors so that samplers will continue
  to initialize remaining BPF probes if fault tolerant error handling is
  enabled. (#259)

# [2.14.0] - 2020-09-16
## Added
- Adds new `SRTT` metric for TCP sampler using BPF. (#238)
- Adds new `krb5kdc` sampler to get telemetry on MIT Kerberos. (#241)

# [2.13.0] - 2020-07-12
## Fixed
- Interrupt sampler failed to sample all interrupts if it encountered an
  unexpected keyword. (#225)
- Interrupt sampler incorrectly initialized per-NUMA node counts for NVMe and
  network interrupts. (#226)
- Memory sampler failed to report some stats. (#227)
- CPU c-state sampling now handles older style c-state names. (#229)
- Prometheus metric exposition now includes type annotations and changes the
  format for percentiles to be encoded as a label value. This fixes collection
  with OpenTelemetry. (#230)

## Changed
- Removed unused `interrupt/serial` metric from the interrupt sampler. (#228)

# [2.12.0] - 2020-06-10
## Fixed
- NTP sampler failed to build with musl toolchain. (#216)

## Added
- New `usercall` sampler for probing arbitrary userspace functions in shared
  libraries.

# [2.11.1] - 2021-02-24
## Fixed
- HTTP and Memcache samplers reporting incorrect percentiles.

# [2.11.0] - 2021-01-11
## Added
- Nvidia GPU sampler which uses the Nvidia Management Library (NVML) to gather
  telemetry for GPU utilization and health.
- NTP sampler to gather telemetry about NTP synchronization.

## Fixed
- Disk BPF sampling now compatible with newer kernels.
- Bug introduced in 2.8.0 caused sample rates greater than 1000ms to cause
  errors.

# [2.10.0] - 2020-10-26
## Changed
- Updates tokio to 0.3.1 from 0.2.x
- Reduces syscall load by reusing filehandles in memory, interupt, and network
  samplers.

# [2.9.0] - 2020-10-15
## Added
- Page Cache sampler which uses BPF to instrument Page Cache hit/miss.

## Fixed
- Updated rustcommon dependencies to get some runtime performance benefits.
- Added proper core -> NUMA node mapping to address issues with per-node metrics
  for interrupt sampler.
- Reduce the cost of disabled samplers by skipping all initialization of
  samplers which are not enabled in the config.
- Documentation updates.

# [2.8.0] - 2020-09-23
## Changed
- Metrics library has been replaced with a new version which reduces memory
  footprint.
- Samplers have been optimized to reduce number of system calls and temporary
  allocations.
- Arbitrary percentiles may now be expressed in the configuration.
- Percentile exposition format has changed to allow arbitrary percentiles. They
  are now expressed in a decimal format padded to 2 digits before the decimal.
  For example, the 5th percentile is now `p05` and the 99.9th percentile is now
  `p99.9`.

# [2.7.1] - 2020-08-31
## Fixed
- Fixed memcache sampler causing tokio worker to panic due to issues registering
  the tcp stream with the tokio runtime.

# [2.7.0] - 2020-08-25
## Changed
- Perf event sampling now implemented with BPF. Now requires building with BPF
  support.
- Renamed worker threads and set limit for total number of runtime threads
  instead of just core threads.

## Added
- CPU sampler now includes CPU frequency.

## Fixed
- BPF probes are now dropped properly on program termination. Previously, on
  some kernel versions, BPF probes might remain after exit.
- Memcache sampler was not being initialized. It's now re-enabled.

# [2.6.0] - 2020-08-11
## Added
- Expanded memory sampler coverage to include telemetry related to NUMA access
  patterns, transparent hugepages, and compaction.

## Fixed
- Disk sampler was not reporting stats for all disks on some multi-disk systems.

# [2.5.0] - 2020-07-24
## Added
- Interrupt sampler now has BPF sampling of time distribution of hardirq/softirq
handlers.

## Fixed
- Replaced remaining uses of chashmap with dashmap which has better performance
characteristics.
- Statically linking bcc/bpf has fixes in upstream crates.

# [2.4.0] - 2020-07-06
## Added
- HTTP sampler to poll JSON endpoint and provide summary metrics
- Added support for bcc 0.15.0, making it the new default version

# [2.3.0] - 2020-06-15
## Added
- TCP abort metrics added to `tcp` sampler
- Increased max for context switch histogram to prevent clipping

## Fixed
- Fixed bug where percentiles could get stuck at the max value if they hit it

# [2.2.0] - 2020-05-29
## Added
- Interrupt sampler can now export network, nvme, and total interrupts per NUMA
  node

# [2.1.0] - 2020-05-26
## Added
- Interrupt sampler to gather system-level telemetry about interrupts

## Fixed
- Improved error handling in samplers that read from multiple sources to prevent
  errors reading from earlier sources from preventing the collection from
  sources which are sampled afterwards

# [2.0.0] - 2020-03-24
## Changed
- Many metrics have been renamed to improve consistency
- Config format updated to be more flexible in configuring individual samplers
- Moved BPF and perf functionality into each sampler so that samplers focus on
  particular aspects of performance and not method of gathering telemetry
- Runtime is now async and samplers rewritten to use async/await
- Changed the default version of bcc to 0.13.0

## Added
- Push-based exposition of metrics to Kafka

# [1.3.0] - 2019-12-20
## Added
- Support for bcc 0.11.0, making it the new default version
- Block device telemetry now includes nvme devices and discard ops/bandwidth
- Memcache sampler now logs successful connections

## Fixed
- JSON output was improperly formatted in memcache proxy mode

# [1.2.0] - 2019-11-06
## Added
- Configuration flag to disable fault tolerance, enabling proper smoke tests of
  sampler initialization in CI
- Network eBPF sampler which provides packet size distribution

## Fixed
- Fixes build issue when `perf` feature is disabled

# [1.1.0] - 2019-10-15
## Added
- Container sampler to use within an application container for telemetry
- Allow for per-sampler collection intervals
- Adds a TCP eBPF sampler which provides latencies for establishing active TCP
  connections

## Fixed
- Allows memcache sampler to reconnect to the cache instance which helps to make
  the sampler more resilient to transient errors
- Softnet sampler now disabled by default to be consistent with other samplers
- Updates bcc version to pull-in bugfixes
- Fixes an issue where network percentiles may be reported incorrectly if the
  primary NIC has an operstate of `unknown`

# [1.0.1] - 2019-08-22
## Fixed
- Fixes interaction between command line arguments and config file so that
  logging level can be set in the config

# [1.0.0] - 2019-08-20

Initial release.

[Unreleased]: https://github.com/twitter/rezolus/compare/v2.15.0...HEAD
[2.15.0]: https://github.com/twitter/rezolus/compare/v2.14.0...v2.15.0
[2.14.0]: https://github.com/twitter/rezolus/compare/v2.13.0...v2.14.0
[2.13.0]: https://github.com/twitter/rezolus/compare/v2.12.0...v2.13.0
[2.12.0]: https://github.com/twitter/rezolus/compare/v2.11.1...v2.12.0
[2.11.1]: https://github.com/twitter/rezolus/compare/v2.11.0...v2.11.1
[2.11.0]: https://github.com/twitter/rezolus/compare/v2.10.0...v2.11.0
[2.10.0]: https://github.com/twitter/rezolus/compare/v2.9.0...v2.10.0
[2.9.0]: https://github.com/twitter/rezolus/compare/v2.8.0...v2.9.0
[2.8.0]: https://github.com/twitter/rezolus/compare/v2.7.1...v2.8.0
[2.7.1]: https://github.com/twitter/rezolus/compare/v2.7.0...v2.7.1
[2.7.0]: https://github.com/twitter/rezolus/compare/v2.6.0...v2.7.0
[2.6.0]: https://github.com/twitter/rezolus/compare/v2.5.0...v2.6.0
[2.5.0]: https://github.com/twitter/rezolus/compare/v2.4.0...v2.5.0
[2.4.0]: https://github.com/twitter/rezolus/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/twitter/rezolus/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/twitter/rezolus/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/twitter/rezolus/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/twitter/rezolus/compare/v1.3.0...v2.0.0
[1.3.0]: https://github.com/twitter/rezolus/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/twitter/rezolus/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/twitter/rezolus/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/twitter/rezolus/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/twitter/rezolus/releases/tag/v1.0.0