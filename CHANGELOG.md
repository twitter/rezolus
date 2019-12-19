# [Unreleased]

#[1.3.0] - 2019-12-19
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

[Unreleased]: https://github.com/twitter/rezolus/compare/v1.3.0...HEAD
[1.3.0]: https://github.com/twitter/rezolus/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/twitter/rezolus/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/twitter/rezolus/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/twitter/rezolus/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/twitter/rezolus/releases/tag/v1.0.0