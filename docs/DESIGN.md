# Design

The primary goal for Rezolus is to provide rich telemetry with a low resource
utilization. We want to make it so that Rezolus can be run everywhere so that
we have enhanced visibility into performance anomalies. With this visibility,
we will be able to capture data about runtime performance problems, profile
systems performance to identify tuning and optimization opportunities, and
measure how we are using our infrastructure.

To ensure it can be deployed broadly, we are deeply focused on making sure that
the resource footprint is kept to a reasonable level. It can be difficult to
quantify exact utilization, because Rezolus is able to be configured to collect
from a variety of sources with configurable resolution. There parameters
greatly influence the amount of CPU and memory that Rezolus will require.

As a rough estimate, at 1Hz sampling with all samplers enabled, Rezolus will
occupy 125MB of RAM and utilize approximately 0.08 CPUs. With eBPF disabled,
the footprint drops to approximately 20MB RAM and 0.03 CPUs and increasing the
sampling rate to 10Hz with results in approximately 50MB RAM and 0.12 CPUs
utilized. We believe these levels of resource utilization are well-balanced
against the enhanced telemetry that Rezolus is able to provide.

## Samplers

All samplers implement the same set of core functions. This makes it easy to
add new samplers as manage them as a collection without worrying about
implementation details. You may think of them as "plugins", even though they
are compiled with the rest of the code.

At a high-level, a sampler takes a sample and records values into a metrics
library. The sampler must also be able to add and remove metrics from the
metrics registry in addition to specifying what types of telemetry will be
exposed for aggregation. For instance, a sampler may specify specific
percentiles to export for one or more metrics.

We recommend taking a look at the rest of the documentation and at a few of the
samplers within this repository to get a sense of how they can be implemented.

## Metrics

We are using the metrics library provided in the [rustcommon][1] project. This
metrics library is focused on performance and precision.

The metrics library provides all of the core functionalities related to
tracking values and producing the types of telemetry we get from oversampling.
We can simply write consecutive readings of a counter into the metrics library,
and it can generate percentiles across a time interval in addition to tracking
the counters value. We can also directly insert bucketized readings like we get
from BPF samplers to transfer the kernel-space aggregate over to user-space.

Perhaps the most critical aspect of this library to understand in the context
of its usage in Rezolus is how it handles counter measurements with regard to
oversampling and producing percentiles across a time range. The first time a
counter is recorded, it simply stores the current value and time the counter
was read. When this counter is again measured and recorded, it calculates the
delta between the two consecutive measurements in both value and time. It uses
the difference in value and time to calculate a rate which is normalized to a
secondly rate.

Assuming that we have asked the library to track one or more percentiles for
this counter, the secondly rate is recorded into a histogram. In Rezolus, we
use moving histograms which retain values across a rolling window. As values
age-out, they are dropped from the histogram. This means when we poll Rezolus's
exposition endpoint, we are given values which represent secondly rates across
the configured time interval.

For instance, we typically would use a one-minute window, and the p50 value
would tell us the secondly rate for which half of the samples would be at or
below this value and the other half would be at or above this value.
Additionally, the p100 value would represent the highest rate seen between two
consecutive samplings of the counter.

[1]: https://github.com/twitter/rustcommon
