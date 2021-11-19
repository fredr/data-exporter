pub mod config;
pub mod parsers;
pub mod pipeline_stages;
pub mod probe;

use prometheus::core::Collector;
use std::sync::Arc;

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
