use futures::StreamExt;
use log::warn;
use metrics::{gauge, increment_counter};

use crate::parsers;
use crate::pipeline_stages::PipelineError;
use crate::targets;

pub async fn collect(metrics: &[Metric]) {
    futures::stream::iter(metrics)
        .for_each_concurrent(25, |m| async {
            match m.collect().await {
                Ok(()) => {
                    increment_counter!(crate::COLLECT_SUCCESSES, "metric" => m.name.clone());
                }
                Err(err) => {
                    increment_counter!(crate::COLLECT_FAILURES, "metric" => m.name.clone());
                    warn!("Failed collecting metric {}, error: {:?}", m.name, err);
                }
            }
        })
        .await;
}

#[derive(Debug)]
enum CollectError {
    ParseError(parsers::ParseError),
    MissingValue(String),
    TargetError(targets::TargetError),
    TransformerError(PipelineError),
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

    // TODO: maybe these can be non-option
    parser: Option<Box<dyn crate::parsers::Parser + Sync + Send>>,
    pipeline_stages:
        Option<Box<dyn crate::pipeline_stages::Service<Error = PipelineError> + Sync + Send>>,
}
impl MetricBuilder {
    pub fn new(name: String, help: String) -> MetricBuilder {
        MetricBuilder {
            name,
            help,
            value: None,
            targets: Vec::new(),
            parser: None,
            pipeline_stages: None,
        }
    }
    pub fn value(&mut self, v: f64) {
        self.value = Some(v);
    }
    pub fn targets(&mut self, t: Vec<targets::Target>) {
        self.targets.extend(t.into_iter());
    }
    pub fn parser(&mut self, p: Box<dyn crate::parsers::Parser + Sync + Send>) {
        self.parser = Some(p);
    }
    pub fn pipeline_stages(
        &mut self,
        stages: Box<dyn crate::pipeline_stages::Service<Error = PipelineError> + Sync + Send>,
    ) {
        self.pipeline_stages = Some(stages);
    }

    pub fn build(self) -> Metric {
        Metric {
            name: self.name,
            help: self.help,
            value: self.value,
            targets: self.targets,

            // TODO: this should not be unwrap
            parser: self.parser.unwrap(),
            pipeline_stages: self.pipeline_stages.unwrap(),
        }
    }
}

pub struct Metric {
    pub name: String,
    pub help: String,
    pub value: Option<f64>,
    pub targets: Vec<targets::Target>,
    pub parser: Box<dyn crate::parsers::Parser + Sync + Send>,
    pub pipeline_stages:
        Box<dyn crate::pipeline_stages::Service<Error = PipelineError> + Sync + Send>,
}

impl Metric {
    async fn collect(&self) -> Result<(), CollectError> {
        for target in &self.targets {
            let resp = target.fetch().await?;
            let resp = self
                .pipeline_stages
                .call(resp)
                .map_err(CollectError::TransformerError)?;

            for parsed in self.parser.parse(resp)? {
                let mut labels: Vec<metrics::Label> = parsed
                    .labels
                    .into_iter()
                    .map(|(k, v)| metrics::Label::from(&(k, v)))
                    .collect();

                labels.push(metrics::Label::from(&(
                    "target".to_owned(),
                    target.describe().to_owned(),
                )));

                labels.sort();

                let value = match (parsed.value, self.value) {
                    (Some(value), _) | (_, Some(value)) => Ok(value),
                    (None, None) => Err(CollectError::MissingValue(String::from(
                        "expected either a constant or a parsed value",
                    ))),
                }?;

                gauge!(self.name.clone(), value, labels);
            }
        }

        Ok(())
    }
}
