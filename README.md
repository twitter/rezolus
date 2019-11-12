# Rezolus

Rezolus is a tool for collecting detailed systems performance telemetry and
exposing burst patterns through high-resolution telemetry. Rezolus provides
instrumentation of basic systems metrics, performance counters, and support for
eBPF (extended Berkeley Packet Filter) telemetry. Measurement is the first step
toward improved performance.

Per-metric documentation can be found in the [METRICS](docs/METRICS.md)
documentation.

## Overview

Rezolus collects telemetry from several different sources. Currently, Rezolus
collects telemetry from traditional sources (procfs, sysfs), the perf_events
subsystem, and from eBPF. Each sampler implements a consistent set of functions
so that new ones can be easily added to further extend the capabilities of
Rezolus.

Each telemetry source is oversampled so that we can build a histogram across a
time interval. This histogram allows us to capture variations which will appear
in the far upper and lower percentiles. This oversampling approach is one of
the key differentiators of Rezolus when compared to other telemetry agents.

With its support for eBPF as well as more common telemetry sources, Rezolus is
a very sophisticated tool for capturing performance anomalies, profiling
systems performance, and conducting performance diagnostics.

More detailed information about the underlying metrics library and sampler
design can be found in the [DESIGN](docs/DESIGN.md) documentation.

### Features

* traditional telemetry sources (procfs, sysfs, ...)
* perf_events support for hardware performance counters
* eBPF support to instrument kernel and user space activities
* oversampling and percentile metrics to capture bursts

### Traditional Telemetry Sources

Rezolus collects metrics from traditional sources (procfs, sysfs) to provide
basic telemetry for CPU, disk, and network. Rezolus exports CPU utilization,
disk bandwidth, disk IOPs, network bandwidth, network packet rate, network
errors, as well as TCP and UDP protocol counters.

These basic telemetry sources, when coupled with the approach of oversampling
to capture their bursts, often provide a high-level view of systems performance
and may readily indicate areas where resources are saturated or errors are
occurring.

### Perf Events

Perf Events allow us to report on both hardware and software events. Typical
software events are things like page faults, context switches, and CPU
migrations. Typical hardware events are things like CPU cycles, instructions
retired, cache hits, cache misses, and a variety of other detailed metrics
about how a workload is running on the underlying hardware.

These metrics are typically used for advanced performance debugging, as well as
for tuning and optimization efforts.

### eBPF

There is an expansive amount of performance information that can be exposed
through eBPF, which allows us to have the Linux Kernel perform telemetry
capture and aggregation at very fine-grained levels.

Rezolus comes with samplers that capture block IO size distribution, EXT4 and
XFS operation latency distribution, and scheduler run queue latency
distribution. You'll see that here we are mainly exposing distributions of
sizes and latencies The kernel is recording the appropriate value for each
operation into a histogram. Rezolus then accesses this histogram from
user-space and transfers the values over to its own internal storage where it
is then exposed to external aggregators.

By collecting telemetry in-kernel, we're able to gather data about events that
happen at extremely high rates - e.g., task scheduling - with minimal
performance overhead for collecting the telemetry. The eBPF samplers can be
used to both capture runtime performance anomalies as well as characterize
workloads.

### Sampling rate and resolution

In order to accurately reflect the intensity of a burst, the sampling rate must
be at least twice the duration of the shortest burst to record accurately. This
ensures that at least 1 sample completely overlaps the burst section of the
event. With a traditional minutely time series, this means that a spike must
least 120 seconds or more to be accurately recorded in terms of intensity.
Rezolus allows for sampling rate to be configured, allowing us to make a
trade-off between resolution and resource consumption. At 10Hz sampling, 200ms
or more of consecutive burst is enough to be accurately reflected in the pMax.
Contrast that with minutely metrics requiring 120_000ms, or secondly requiring
2000ms of consecutive burst to be accurately recorded.

## Getting Started

### Building

Rezolus is built with the standard Rust toolchain which can be installed and
managed via [rustup](https://rustup.rs) or by following the directions on the
Rust [website](https://www.rust-lang.org/).

**NOTE:** at this time, Rezolus needs to be built with the nightly toolchain
or a locally built development toolchain which has nightly features enabled.
This is because Rezolus requires language features that have not been fully
stabilized in the language. These features are required to get support for
performance counters.

The rest of the guide assumes you've chosen to install the toolchain via rustup.

**NOTE:** Rezolus is intended to be built and deployed on Linux systems.

#### Install the nightly toolchain
```bash
rustup toolchain install nightly
```
#### Clone and build Rezolus from source
```bash
git clone https://github.com/twitter/rezolus
cd rezolus

# create an unoptimized development build
cargo build

# run the unoptimized binary and display help
cargo run -- --help

# create an optimized release build
cargo build --release

# run the optimized binary and display help
cargo run --release -- --help

# run the optimized binary with the example config (needs sudo for perf_events)
cargo build --release && \
sudo target/release/rezolus --config configs/example.toml

# metrics can be viewed in human-readable form with curl
curl --silent http://localhost:4242/vars
```

### Building with eBPF Support

By default, eBPF support is not compiled in. If you wish to produce a build with
eBPF support enabled, follow the steps below:

#### Prerequisites

eBPF support requires that we link against the [BPF Compiler Collection]. You
may either use the version provided by your distribution, or can build BCC and
install from source. It is critical to know which version of BCC you have
installed. Rezolus supports multiple versions by utilizing different feature
flags at build time.

#### Building

As mentioned above, we provide different feature flags to map to various
supported BCC versions. The `ebpf` feature will map to the newest version
supported by the [rust-bcc] project. For most users, this will be the right flag
to use. However, if you must link against an older BCC version, you will need
to use a more specific form of the feature flag. These version-specific flags
take the form of `ebpf_v0_10_0` with the BCC version being reflected in the name
of the feature. You can find a complete list of the feature flags in the
[cargo manifest] for this project.

```bash
# create an optimized release build with eBPF support
cargo build --release --features ebpf
sudo target/release/rezolus --config configs/example.toml

# metrics can be viewed in human-readable form with curl
curl --silent http://localhost:4242/vars
```

### HTTP Exposition

Rezolus exposes metrics over HTTP, with different paths corresponding to
different exposition formats.

* human-readable: `/vars`
* JSON: `/vars.json`, `/metrics.json`, `/admin/metrics.json`
* Prometheus: `/metrics`

**NOTE:** currently, JSON exposition is provided by default for any other path.
This behavior may change in the future and should not be relied on.

Additionally, you can get the running version on the root-level path `/`

## Support

Create a [new issue](https://github.com/twitter/rezolus/issues/new) on GitHub.

## Contributing

We feel that a welcoming community is important and we ask that you follow
Twitter's [Open Source Code of Conduct] in all interactions with the community.

## Authors

* Brian Martin <bmartin@twitter.com>

A full list of [contributors] can be found on GitHub.

Follow [@TwitterOSS](https://twitter.com/twitteross) on Twitter for updates.

## License

Copyright 2019 Twitter, Inc.

Licensed under the Apache License, Version 2.0:
https://www.apache.org/licenses/LICENSE-2.0

## Security Issues?

Please report sensitive security issues via Twitter's bug-bounty program
(https://hackerone.com/twitter) rather than GitHub.

[cargo manifest]: https://github.com/twitter/rezolus/blob/master/Cargo.toml
[contributors]: https://github.com/twitter/rezolus/graphs/contributors?type=a
[rust-bcc]: https://github.com/rust-bpf/rust-bcc
[BPF Compiler Collection]: https://github.com/iovisor/bcc
[Open Source Code of Conduct]: https://github.com/twitter/code-of-conduct/blob/master/code-of-conduct.md
