pub mod config;
pub mod metrics;
pub mod parsers;
pub mod pipeline_stages;

use lazy_static::lazy_static;
use prometheus::core::Collector;
use prometheus::{register_int_counter_vec, IntCounterVec};
use std::sync::Arc;

lazy_static! {
    pub static ref COLLECT_FAILURES: IntCounterVec = register_int_counter_vec!(
        "collect_failures_total",
        "Number of failed collects",
        &["metric"]
    )
    .unwrap();
    pub static ref COLLECT_SUCCESSES: IntCounterVec = register_int_counter_vec!(
        "collect_successes_total",
        "Number of succeeded collects",
        &["metric"]
    )
    .unwrap();
}

pub fn init_metrics() {
    // needs to be initialized before use, otherwise they'll be initialiezed during gather, causing deadlock
    COLLECT_FAILURES.reset();
    COLLECT_SUCCESSES.reset();
}

pub struct DataMetrics {
    metrics: Arc<Vec<metrics::Metric>>,
}

impl DataMetrics {
    pub fn new(metrics: Vec<metrics::Metric>) -> Self {
        DataMetrics {
            metrics: Arc::new(metrics),
        }
    }
}

impl Collector for DataMetrics {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.metrics.iter().flat_map(|m| m.gauge.desc()).collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        let metrics = self.metrics.clone();
        std::thread::spawn(move || metrics::collect(metrics.as_ref()))
            .join()
            .unwrap()
    }
}
