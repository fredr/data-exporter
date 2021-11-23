pub mod config;
pub mod parsers;
pub mod pipeline_stages;
pub mod probe;

use lazy_static::lazy_static;
use prometheus::core::Collector;
use prometheus::{register_int_counter_vec, IntCounterVec};
use std::sync::Arc;

lazy_static! {
    pub static ref PROBE_FAILURES: IntCounterVec = register_int_counter_vec!(
        "probe_failures_total",
        "Number of failed probes",
        &["type", "probe"]
    )
    .unwrap();
    pub static ref PROBE_SUCCESSES: IntCounterVec = register_int_counter_vec!(
        "probe_successes_total",
        "Number of succeeded probes",
        &["type", "probe"]
    )
    .unwrap();
}

pub fn init_metrics() {
    // needs to be initialized before use, otherwise they'll be initialiezed during gather, causing deadlock
    PROBE_FAILURES.reset();
    PROBE_SUCCESSES.reset();
}

pub struct DataMetrics {
    probes: Arc<Vec<probe::Probe>>,
}

impl DataMetrics {
    pub fn new(probes: Vec<probe::Probe>) -> Self {
        DataMetrics {
            probes: Arc::new(probes),
        }
    }
}

impl Collector for DataMetrics {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.probes
            .iter()
            .flat_map(|p| p.metric.gauge.desc())
            .collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let probes = self.probes.clone();
        std::thread::spawn(move || probe::run(probes.as_ref()))
            .join()
            .unwrap()
    }
}
