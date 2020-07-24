# [Unreleased]

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

[Unreleased]: https://github.com/twitter/rezolus/compare/v2.5.0...HEAD
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