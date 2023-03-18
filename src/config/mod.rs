use serde::Deserialize;
use std::{fs::File, io::BufReader};

use crate::{
    collector::MetricBuilder,
    pipeline_stages::{self, Pipeline, PipelineError, Service},
};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum PipelineStageType {
    Jq { query: String },
    Regex { pattern: String, replace: String },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Parser {
    Json {
        labels: Vec<String>,
        value: Option<String>,
    },
    Regex {
        pattern: String,
        labels: Vec<String>,
        value: Option<String>,
    },
}

#[derive(Deserialize)]
struct Metric {
    name: String,
    help: String,
    value: Option<f64>,
    targets: Vec<Target>,
    pipeline_stages: Option<Vec<PipelineStageType>>,
    parser: Parser,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Target {
    Http { url: String },
    File { path: String },
}

#[derive(Deserialize)]
struct Config {
    metrics: Vec<Metric>,
}

pub fn parse(path: String) -> serde_yaml::Result<crate::DataMetrics> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let config: Config = serde_yaml::from_reader(reader)?;

    let metrics: Vec<crate::collector::Metric> = config
        .metrics
        .iter()
        .map(|m| {
            let parser: Box<dyn crate::parsers::Parser + Send + Sync> = match &m.parser {
                Parser::Regex {
                    labels,
                    value,
                    pattern,
                } => Box::new(crate::parsers::regex::RegexParser::new(
                    pattern,
                    labels.clone(),
                    value.clone(),
                )),
                Parser::Json { labels, value } => Box::new(crate::parsers::json::JsonParser::new(
                    labels.clone(),
                    value.clone(),
                )),
            };

            let mut pipeline_stages: Box<dyn Service<Error = PipelineError> + Sync + Send> =
                Box::new(Pipeline::new());

            if let Some(stages) = &m.pipeline_stages {
                for stage in stages {
                    match stage {
                        PipelineStageType::Jq { query } => {
                            pipeline_stages = Box::new(pipeline_stages::JqStage::<
                                Box<dyn Service<Error = PipelineError> + Sync + Send>,
                            >::new(
                                pipeline_stages, query.clone()
                            ));
                        }
                        PipelineStageType::Regex { pattern, replace } => {
                            pipeline_stages = Box::new(pipeline_stages::RegexStage::<
                                Box<dyn Service<Error = PipelineError> + Sync + Send>,
                            >::new(
                                pipeline_stages,
                                regex::Regex::new(pattern).unwrap(),
                                replace.clone(),
                            ));
                        }
                    }
                }
            }

            let targets = m
                .targets
                .iter()
                .map(|t| match t {
                    Target::Http { url } => {
                        crate::targets::Target::Http(crate::targets::http::Config {
                            url: String::from(url),
                        })
                    }
                    Target::File { path } => crate::targets::Target::File {
                        path: String::from(path),
                    },
                })
                .collect();

            MetricBuilder::new(m.name.clone(), m.help.clone())
                .value(m.value)
                .targets(targets)
                .pipeline_stages(pipeline_stages)
                .parser(parser)
                .build()
        })
        .collect();

    Ok(crate::DataMetrics::new(metrics))
}
