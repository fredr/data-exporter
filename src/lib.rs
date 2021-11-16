pub mod config;
pub mod parsers;
pub mod pipeline_stages;

use prometheus::core::Collector;
use prometheus::{opts, GaugeVec};
use std::collections::HashMap;

pub struct DataMetrics {
    probes: Vec<Probe>,
}

impl DataMetrics {
    pub fn new(probes: Vec<Probe>) -> Self {
        DataMetrics { probes }
    }
}

impl Collector for DataMetrics {
    fn desc(&self) -> Vec<&prometheus::core::Desc> {
        self.probes
            .iter()
            .flat_map(|p| p.metric.metric.desc())
            .collect()
    }

    fn collect(&self) -> Vec<prometheus::proto::MetricFamily> {
        // TODO(fredr): Make this parallel via some async thread or something
        self.probes
            .iter()
            .flat_map(|p| p.probe().unwrap())
            .collect()
    }
}

pub struct Probe {
    pub target: String, // This should be an enum for http, file etc
    pub parser: Box<dyn parsers::Parser + Sync + Send>,
    pub pipeline_stages: Vec<Box<dyn pipeline_stages::PipelineStage + Sync + Send>>,
    pub metric: MetricConfig,
}

impl Probe {
    fn probe(&self) -> std::io::Result<Vec<prometheus::proto::MetricFamily>> {
        let target = self.target.clone();
        let resp = std::thread::spawn(move || {
            let resp = reqwest::blocking::get(&target).unwrap();
            resp.text().unwrap()
        })
        .join()
        .unwrap();

        let resp = self
            .pipeline_stages
            .iter()
            .fold(resp, |acc, ps| ps.transform(&acc));

        self.metric.metric.reset();
        self.set_metrics_from_response(&resp);
        Ok(self.metric.metric.collect())
    }

    // TODO(fredr): return errors instead of panic
    fn set_metrics_from_response(&self, resp: &str) {
        match self.parser.parse(&resp) {
            serde_json::Value::Array(arr) => {
                for val in arr {
                    match val {
                        serde_json::Value::Object(map) => {
                            self.set_metric_from_data(map);
                        }
                        _ => panic!("unexpected pared data in array, expected object"),
                    }
                }
            }
            serde_json::Value::Object(map) => {
                self.set_metric_from_data(map);
            }
            _ => panic!("unexpected parsed data, expected object or array of objects"),
        }
    }
    fn set_metric_from_data(&self, data: serde_json::Map<String, serde_json::Value>) {
        let labels: HashMap<_, _> = self
            .metric
            .labels
            .iter()
            .map(|(k, t)| (k.as_str(), data.get(t).unwrap().as_str().unwrap()))
            .collect();

        let value = match &self.metric.value {
            MetricValue::FromData(label) => data.get(label).unwrap().as_f64().unwrap(),
            MetricValue::Vector(v) => *v,
        };

        self.metric.metric.with(&labels).set(value);
    }
}

pub enum MetricValue {
    FromData(String),
    Vector(f64),
}

pub struct MetricConfig {
    metric: GaugeVec, // TOOD(fredr): Should it be possible to create other types as well?
    labels: HashMap<String, String>,
    value: MetricValue,
}

impl MetricConfig {
    pub fn new(
        name: String,
        help: String,
        labels: HashMap<String, String>,
        value: MetricValue,
    ) -> Self {
        let label_names = labels
            .iter()
            .map(|(k, _v)| k.as_str())
            .collect::<Vec<&str>>();

        let metric = GaugeVec::new(
            opts!(name, help).namespace("data_exporter"),
            label_names.as_slice(),
        )
        .unwrap();

        Self {
            metric,
            labels,
            value,
        }
    }
}
