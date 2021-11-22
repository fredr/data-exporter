use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader};

#[derive(Deserialize)]
#[serde(untagged)]
enum Value {
    Vector(f64),
    FromData(String),
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum PipelineStage {
    Jq { query: String },
    Regex { pattern: String, replace: String },
}

#[derive(Deserialize)]
struct Metric {
    name: String,
    help: String,
    labels: HashMap<String, String>,
    value: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Parser {
    Json,
}

#[derive(Deserialize)]
struct Probe {
    target: Target,
    pipeline_stages: Vec<PipelineStage>,
    parser: Parser,
    metric: Metric,
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
    probes: Vec<Probe>,
}

pub fn parse(path: String) -> serde_yaml::Result<crate::DataMetrics> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let config: Config = serde_yaml::from_reader(reader)?;

    let probes: Vec<crate::probe::Probe> = config
        .probes
        .iter()
        .map(|p| {
            let value = match &p.metric.value {
                Value::FromData(s) => crate::probe::MetricValue::FromData(s.clone()),
                Value::Vector(x) => crate::probe::MetricValue::Vector(*x),
            };
            let parser = match &p.parser {
                Parser::Json => crate::parsers::json::Parser {},
            };
            let pipeline_stages = p
                .pipeline_stages
                .iter()
                .map(
                    |s| -> Box<dyn crate::pipeline_stages::PipelineStage + Send + Sync> {
                        match s {
                            PipelineStage::Jq { query } => {
                                Box::new(crate::pipeline_stages::jq::Stage {
                                    expression: query.clone(),
                                })
                            }
                            PipelineStage::Regex { pattern, replace } => {
                                Box::new(crate::pipeline_stages::regex::Stage {
                                    regex: regex::Regex::new(pattern).unwrap(),
                                    replace: replace.to_string(),
                                })
                            }
                        }
                    },
                )
                .collect();

            let target = match &p.target {
                Target::Http { url } => crate::probe::Target::Http {
                    url: String::from(url),
                },
                Target::File { path } => crate::probe::Target::File {
                    path: String::from(path),
                },
            };

            crate::probe::Probe {
                target,
                pipeline_stages,
                parser: Box::new(parser),
                metric: crate::probe::MetricConfig::new(
                    p.metric.name.clone(),
                    p.metric.help.clone(),
                    p.metric.labels.clone(),
                    value,
                ),
            }
        })
        .collect();

    Ok(crate::DataMetrics::new(probes))
}
