use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use rdkafka::producer::ProducerContext;
use rdkafka::ClientContext;
use rustcommon_metrics::{Counter, DynBoxedMetric, Gauge};

macro_rules! declare_gauges {
    (
        $self:expr,
        $stats:expr,
        format $(!)? ($fmt:literal $( , $arg:expr )* $(,)? ),
        $first:ident $( , $rest:ident )* $(,)?
    ) => {
        $self.set_gauge(
            format!($fmt $(, $arg )*, name = stringify!($first)),
            $stats.$first as i64
        );
        declare_gauges!($self, $stats, format!($fmt $( , $arg )*), $( $rest ),*);
    };
    ( $self:expr, $stats:expr, format $(!)? ($fmt:literal $( , $arg:expr )* $(,)? ) $(,)?) => {}
}

/// Special producer context that records rdkafka metrics.
#[derive(Default)]
pub struct AugurProducerContext {
    exporter: Mutex<MetricsExporter>,
}

impl ClientContext for AugurProducerContext {
    fn stats(&self, statistics: rdkafka::Statistics) {
        let mut exporter = self.exporter.lock().expect("Mutex was poisioned");

        exporter.export(statistics);
    }
}

impl ProducerContext for AugurProducerContext {
    type DeliveryOpaque = ();

    fn delivery(
        &self,
        _delivery_result: &rdkafka::producer::DeliveryResult<'_>,
        _delivery_opaque: Self::DeliveryOpaque,
    ) {
    }
}

#[derive(Default)]
struct MetricsExporter {
    gauges: HashMap<String, DynBoxedMetric<Gauge>>,
    counters: HashMap<String, DynBoxedMetric<Counter>>,
    seen: HashSet<String>,
}

impl MetricsExporter {
    fn set_gauge(&mut self, name: String, value: i64) {
        if !self.seen.contains(&name) {
            self.seen.insert(name.clone());
        }

        self.gauges
            .entry(name)
            .or_insert_with_key(|key| DynBoxedMetric::new(Gauge::new(), key.clone()))
            .set(value);
    }

    fn export(&mut self, stats: rdkafka::Statistics) {
        declare_gauges!(
            self,
            stats,
            format!("rdkafka/{name}"),
            replyq,
            msg_cnt,
            msg_size,
            simple_cnt,
            metadata_cache_cnt,
            msg_max,
            msg_size_max,
            tx,
            tx_bytes,
            rx,
            rx_bytes,
            txmsgs,
            txmsg_bytes,
            rxmsgs,
            rxmsg_bytes
        );

        for (_, topic) in &stats.topics {
            let mut txbytes = 0;
            let mut txmsgs = 0;
            let mut rxbytes = 0;
            let mut rxmsgs = 0;
            let mut msgs = 0;

            for (_, partition) in &topic.partitions {
                declare_gauges!(
                    self,
                    partition,
                    format!(
                        "rdkafka/topic/{}/partition/{}/{name}",
                        topic.topic, partition.partition
                    ),
                    msgq_cnt,
                    msgq_bytes,
                    xmit_msgq_cnt,
                    xmit_msgq_bytes,
                    fetchq_cnt,
                    fetchq_size,
                    msgs_inflight,
                    next_ack_seq,
                    next_err_seq,
                    broker,
                    leader,
                    txmsgs,
                    txbytes,
                    rxmsgs,
                    rxbytes,
                    msgs,
                    rx_ver_drops,
                    acked_msgid
                );

                txbytes += partition.txbytes;
                rxbytes += partition.rxbytes;
                txmsgs += partition.txmsgs;
                rxmsgs += partition.rxmsgs;
                msgs += partition.msgs;
            }

            self.set_gauge(
                format!("rdkafka/topic/{}/txbytes", topic.topic),
                txbytes as _,
            );
            self.set_gauge(
                format!("rdkafka/topic/{}/rxbytes", topic.topic),
                rxbytes as _,
            );
            self.set_gauge(format!("rdkafka/topic/{}/txmsgs", topic.topic), txmsgs as _);
            self.set_gauge(format!("rdkafka/topic/{}/rxmsgs", topic.topic), rxmsgs as _);
            self.set_gauge(format!("rdkafka/topic/{}/msgs", topic.topic), msgs as _);
        }

        let ref seen = self.seen;
        self.gauges.retain(|name, _| seen.contains(name));
        self.counters.retain(|name, _| seen.contains(name));
        self.seen.clear();
    }
}
