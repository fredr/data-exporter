pub mod collector;
pub mod config;
pub mod parsers;
pub mod pipeline_stages;
pub mod targets;

use std::sync::Arc;

use collector::collect;
use metrics::{describe_counter, describe_gauge, register_counter};

const COLLECT_FAILURES: &str = "data_exporter_collect_failures_total";
const COLLECT_SUCCESSES: &str = "data_exporter_collect_successes_total";

pub fn init_metrics(metrics: &DataMetrics) {
    let metrics = metrics.metrics.clone();
    for metric in metrics.iter() {
        describe_gauge!(metric.name.clone(), metric.help.clone());

        register_counter!(COLLECT_FAILURES, "metric" => metric.name.clone());
        register_counter!(COLLECT_SUCCESSES, "metric" => metric.name.clone());
    }

    describe_counter!(COLLECT_FAILURES, "Number of failed collects");
    describe_counter!(COLLECT_SUCCESSES, "Number of succeeded collects");
}

#[derive(Clone)]
pub struct DataMetrics {
    metrics: Arc<Vec<collector::Metric>>,
}

impl DataMetrics {
    pub fn new(metrics: Vec<collector::Metric>) -> Self {
        DataMetrics {
            metrics: Arc::new(metrics),
        }
    }

    pub async fn collect(&self) {
        let metrics: Arc<Vec<collector::Metric>> = self.metrics.clone();
        collect(&metrics).await;
    }
}
