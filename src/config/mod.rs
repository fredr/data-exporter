use serde::Deserialize;
use std::{fs::File, io::BufReader};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum PipelineStage {
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
}

#[derive(Deserialize)]
struct Metric {
    name: String,
    help: String,
    value: Option<f64>,
    targets: Vec<Target>,
    pipeline_stages: Vec<PipelineStage>,
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

    let metrics: Vec<crate::metrics::Metric> = config
        .metrics
        .iter()
        .map(|m| {
            let (parser, labels) = match &m.parser {
                Parser::Json { labels, value } => (
                    crate::parsers::json::JsonParser::new(labels.to_vec(), value.to_owned()),
                    labels,
                ),
            };
            let pipeline_stages = m
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

            let targets = m
                .targets
                .iter()
                .map(|t| match t {
                    Target::Http { url } => crate::metrics::Target::Http {
                        url: String::from(url),
                    },
                    Target::File { path } => crate::metrics::Target::File {
                        path: String::from(path),
                    },
                })
                .collect();

            let mut builder =
                crate::metrics::MetricBuilder::new(m.name.clone(), m.help.clone(), labels.to_vec());
            if let Some(v) = m.value {
                builder.value(v);
            }
            builder.targets(targets);
            builder.pipeline_stages(pipeline_stages);
            builder.parser(Box::new(parser));
            builder.build()
        })
        .collect();

    Ok(crate::DataMetrics::new(metrics))
}
