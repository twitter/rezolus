# [Unreleased]
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

[Unreleased]: https://github.com/twitter/rezolus/compare/v1.0.1...HEAD
[1.0.1]: https://github.com/twitter/rezolus/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/twitter/rezolus/releases/tag/v1.0.0