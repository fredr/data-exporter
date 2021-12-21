use futures::StreamExt;
use log::warn;
use prometheus::core::Collector;
use prometheus::{opts, GaugeVec};
use std::collections::HashMap;

use crate::parsers;
use crate::targets;

#[tokio::main]
pub async fn collect(metrics: &[Metric]) -> Vec<prometheus::proto::MetricFamily> {
    futures::stream::iter(metrics)
        .map(|m| async {
            match m.collect().await {
                Ok(v) => {
                    crate::COLLECT_SUCCESSES.with_label_values(&[&m.name]).inc();
                    v
                }
                Err(err) => {
                    crate::COLLECT_FAILURES.with_label_values(&[&m.name]).inc();
                    warn!("Failed collecting metric {}, error: {:?}", m.name, err);
                    vec![]
                }
            }
        })
        .buffer_unordered(100)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

#[derive(Debug)]
enum CollectError {
    ParseError(parsers::ParseError),
    MissingValue(String),
    TargetError(targets::TargetError),
}
impl From<parsers::ParseError> for CollectError {
    fn from(e: parsers::ParseError) -> Self {
        CollectError::ParseError(e)
    }
}
impl From<targets::TargetError> for CollectError {
    fn from(e: targets::TargetError) -> Self {
        CollectError::TargetError(e)
    }
}

pub struct MetricBuilder {
    name: String,
    help: String,
    value: Option<f64>,
    targets: Vec<targets::Target>,
    parser: Option<Box<dyn crate::parsers::Parser + Sync + Send>>,
    labels: Vec<String>,
    pipeline_stages: Vec<Box<dyn crate::pipeline_stages::PipelineStage + Sync + Send>>,
}
impl MetricBuilder {
    pub fn new(name: String, help: String, labels: Vec<String>) -> MetricBuilder {
        MetricBuilder {
            name,
            help,
            labels,
            value: None,
            targets: Vec::new(),
            parser: None,
            pipeline_stages: Vec::new(),
        }
    }
    pub fn value(&mut self, v: f64) {
        self.value = Some(v)
    }
    pub fn targets(&mut self, t: Vec<targets::Target>) {
        self.targets.extend(t.into_iter())
    }
    pub fn parser(&mut self, p: Box<dyn crate::parsers::Parser + Sync + Send>) {
        self.parser = Some(p)
    }
    pub fn pipeline_stages(
        &mut self,
        ps: Vec<Box<dyn crate::pipeline_stages::PipelineStage + Sync + Send>>,
    ) {
        self.pipeline_stages.extend(ps.into_iter())
    }

    pub fn build(self) -> Metric {
        let mut label_names = self.labels.iter().map(|s| &**s).collect::<Vec<&str>>();
        label_names.push("target");

        let gauge = GaugeVec::new(
            opts!(self.name.clone(), self.help.clone()),
            label_names.as_slice(),
        )
        .unwrap();

        Metric {
            name: self.name,
            value: self.value,
            targets: self.targets,
            parser: self.parser.unwrap(),
            pipeline_stages: self.pipeline_stages,
            gauge,
        }
    }
}

pub struct Metric {
    pub name: String,
    pub value: Option<f64>,
    pub targets: Vec<targets::Target>,
    pub parser: Box<dyn crate::parsers::Parser + Sync + Send>,
    pub pipeline_stages: Vec<Box<dyn crate::pipeline_stages::PipelineStage + Sync + Send>>,
    pub gauge: GaugeVec,
}

impl Metric {
    async fn collect(&self) -> Result<Vec<prometheus::proto::MetricFamily>, CollectError> {
        self.gauge.reset();

        for target in &self.targets {
            let resp = target.fetch().await?;
            let resp = self
                .pipeline_stages
                .iter()
                .fold(resp, |acc, stage| stage.transform(&acc));

            for parsed in self.parser.parse(&resp)? {
                let mut labels: HashMap<&str, &str> = parsed
                    .labels
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                labels.insert("target", target.describe());

                let value = match (parsed.value, self.value) {
                    (Some(value), _) => Ok(value),
                    (_, Some(value)) => Ok(value),
                    (None, None) => Err(CollectError::MissingValue(String::from(
                        "expected either a constant or a parsed value",
                    ))),
                }?;

                self.gauge.with(&labels).set(value);
            }
        }

        Ok(self.gauge.collect())
    }
}
