use futures::StreamExt;
use prometheus::core::Collector;
use prometheus::{opts, GaugeVec};
use std::collections::HashMap;

use crate::parsers;

#[tokio::main]
pub async fn run(probes: &[Probe]) -> Vec<prometheus::proto::MetricFamily> {
    futures::stream::iter(probes)
        .map(|p| async {
            p.probe().await.unwrap_or_else(|err| {
                // TODO(fredr): Metric for failed probes increase
                println!("Probe {} failed: {:?}", p.target, err); // TODO(fredr): Replace with real logging
                vec![]
            })
        })
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

#[derive(Debug)]
enum ProbeError {
    IO(std::io::Error),
    Reqwest(reqwest::Error),
    ParseResponse(String),
    ParseError(parsers::ParseError),
}
impl From<std::io::Error> for ProbeError {
    fn from(e: std::io::Error) -> Self {
        ProbeError::IO(e)
    }
}
impl From<reqwest::Error> for ProbeError {
    fn from(e: reqwest::Error) -> Self {
        ProbeError::Reqwest(e)
    }
}
impl From<parsers::ParseError> for ProbeError {
    fn from(e: parsers::ParseError) -> Self {
        ProbeError::ParseError(e)
    }
}

pub struct Probe {
    pub target: String, // This should be an enum for http, file etc
    pub parser: Box<dyn crate::parsers::Parser + Sync + Send>,
    pub pipeline_stages: Vec<Box<dyn crate::pipeline_stages::PipelineStage + Sync + Send>>,
    pub metric: MetricConfig,
}

impl Probe {
    async fn probe(&self) -> Result<Vec<prometheus::proto::MetricFamily>, ProbeError> {
        let target = self.target.clone();
        let resp = reqwest::get(&target).await?.text().await?;

        let resp = self
            .pipeline_stages
            .iter()
            .fold(resp, |acc, ps| ps.transform(&acc));

        self.metric.gauge.reset();
        self.set_metrics_from_response(&resp)?;
        Ok(self.metric.gauge.collect())
    }

    fn set_metrics_from_response(&self, resp: &str) -> Result<(), ProbeError> {
        match self.parser.parse(resp)? {
            serde_json::Value::Array(arr) => {
                for val in arr {
                    match val {
                        serde_json::Value::Object(map) => {
                            self.set_metric_from_data(map);
                        }
                        _ => {
                            return Err(ProbeError::ParseResponse(String::from(
                                "unexpected pared data in array, expected object",
                            )))
                        }
                    }
                }
            }
            serde_json::Value::Object(map) => {
                self.set_metric_from_data(map);
            }
            _ => {
                return Err(ProbeError::ParseResponse(String::from(
                    "unexpected parsed data, expected object or array of objects",
                )))
            }
        }

        Ok(())
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

        self.metric.gauge.with(&labels).set(value);
    }
}

pub enum MetricValue {
    FromData(String),
    Vector(f64),
}

pub struct MetricConfig {
    pub gauge: GaugeVec,
    pub labels: HashMap<String, String>,
    pub value: MetricValue,
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

        let gauge = GaugeVec::new(
            opts!(name, help).namespace("data_exporter"),
            label_names.as_slice(),
        )
        .unwrap();

        Self {
            gauge,
            labels,
            value,
        }
    }
}
