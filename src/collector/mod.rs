use bytes::Bytes;
use futures::StreamExt;
use log::warn;
use metrics::{gauge, increment_counter};

use crate::parsers::{self, Parser};
use crate::pipeline_stages::{PipelineError, Service};
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

pub struct NoParser;
impl Parser for NoParser {
    fn parse(&self, _data: bytes::Bytes) -> Result<Vec<parsers::Parsed>, parsers::ParseError> {
        Ok(Vec::new())
    }
}

pub struct NoStages;
impl Service for NoStages {
    type Error = PipelineError;

    fn call(&self, _input: bytes::Bytes) -> Result<bytes::Bytes, Self::Error> {
        Ok(Bytes::new())
    }
}

pub struct MetricBuilder<P, S> {
    name: String,
    help: String,
    value: Option<f64>,
    targets: Vec<targets::Target>,
    parser: P,
    pipeline_stages: S,
}

impl MetricBuilder<NoParser, NoStages> {
    pub fn new(name: String, help: String) -> MetricBuilder<NoParser, NoStages> {
        MetricBuilder {
            name,
            help,
            value: None,
            targets: Vec::new(),
            parser: NoParser,
            pipeline_stages: NoStages,
        }
    }
}

impl<P> MetricBuilder<P, NoStages> {
    pub fn pipeline_stages<S>(self, stages: S) -> MetricBuilder<P, S>
    where
        S: Service<Error = PipelineError>,
    {
        MetricBuilder {
            name: self.name,
            help: self.help,
            value: self.value,
            targets: self.targets,
            parser: self.parser,
            pipeline_stages: stages,
        }
    }
}

impl<S> MetricBuilder<NoParser, S> {
    pub fn parser<P>(self, parser: P) -> MetricBuilder<P, S>
    where
        S: Service<Error = PipelineError>,
    {
        MetricBuilder {
            name: self.name,
            help: self.help,
            value: self.value,
            targets: self.targets,
            parser,
            pipeline_stages: self.pipeline_stages,
        }
    }
}

impl<P, S> MetricBuilder<P, S> {
    pub fn value(self, value: Option<f64>) -> Self {
        Self {
            name: self.name,
            help: self.help,
            value,
            targets: self.targets,
            parser: self.parser,
            pipeline_stages: self.pipeline_stages,
        }
    }

    pub fn targets(self, t: Vec<targets::Target>) -> Self {
        Self {
            name: self.name,
            help: self.help,
            value: self.value,
            targets: t,
            parser: self.parser,
            pipeline_stages: self.pipeline_stages,
        }
    }
}

impl<P, S> MetricBuilder<P, S>
where
    P: Parser + Send + Sync + 'static,
    S: Service<Error = PipelineError> + Send + Sync + 'static,
{
    pub fn build(self) -> Metric {
        Metric {
            name: self.name,
            help: self.help,
            value: self.value,
            targets: self.targets,
            parser: Box::new(self.parser),
            pipeline_stages: Box::new(self.pipeline_stages),
        }
    }
}

pub struct Metric {
    pub name: String,
    pub help: String,
    pub value: Option<f64>,
    pub targets: Vec<targets::Target>,
    pub parser: Box<dyn Parser + Send + Sync>,
    pub pipeline_stages: Box<dyn Service<Error = PipelineError> + Send + Sync>,
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
