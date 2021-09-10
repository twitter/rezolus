mod heatmap;

pub use self::heatmap::{HeatmapSummarizedCounter, HeatmapSummarizedGauge, SampledHeatmap};
pub use rustcommon_metrics_v2::{metric, Counter, DynBoxedMetric, Gauge, Heatmap};

/// A short form for a sequence of if statements.
/// 
/// # Example
/// ```
/// # let i = 0;
/// if_block! {
///     if i % 2 == 0 => println!("divisible by 2");
///     if i % 3 == 0 => println!("divisible by 3");
///     if i % 4 == 0 => println!("divisible by 4");
///     // etc..
/// }
/// ```
macro_rules! if_block {
    { if let $pat:pat = $val:expr => $then:expr ; $( $rest:tt )* } => {{
    if let $pat = $val { $then; }
    if_block! { $( $rest )* }
    }};
    { if $cond:expr => $then:expr ; $( $rest:tt )* } => {{
        if $cond { $then; }
        if_block! { $( $rest )* }
    }};
    {} => {};
}
