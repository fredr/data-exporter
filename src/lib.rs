pub mod config;

use std::collections::HashMap;

use prometheus::core::Collector;
use prometheus::{opts, GaugeVec};

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

// TODO(fredr): create our own Value type that we can parse into from different formats
pub trait Parser {
    fn parse(&self, data: &str) -> serde_json::Value;
}

pub struct JsonParser {}

impl Parser for JsonParser {
    fn parse(&self, data: &str) -> serde_json::Value {
        serde_json::from_str(data).unwrap()
    }
}

pub trait PipelineStep {
    fn transform(&self, value: &str) -> String;
}

pub struct JqStep {
    pub expression: String,
}
impl PipelineStep for JqStep {
    fn transform(&self, value: &str) -> String {
        let mut compiled = jq_rs::compile(&self.expression).unwrap();
        compiled.run(value).unwrap()
    }
}

pub struct Probe {
    pub target: String, // This should be an enum for http, file etc
    pub parser: Box<dyn Parser + Sync + Send>,
    pub pipeline_stages: Vec<Box<dyn PipelineStep + Sync + Send>>,
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

        let parsed = self.parser.parse(&resp);
        self.metric.metric.reset();

        // TODO(fredr): handle both array an object
        if let serde_json::Value::Array(arr) = parsed {
            for val in arr {
                let labels: HashMap<_, _> = self
                    .metric
                    .labels
                    .iter()
                    .map(|(k, t)| (k.as_str(), val.get(t).unwrap().as_str().unwrap()))
                    .collect();

                let value = match &self.metric.value {
                    MetricValue::FromData(label) => val.get(label).unwrap().as_f64().unwrap(),
                    MetricValue::Vector(v) => *v,
                };
                self.metric.metric.with(&labels).set(value);
            }
        }

        Ok(self.metric.metric.collect())
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
