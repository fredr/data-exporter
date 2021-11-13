use serde::Deserialize;
use std::{collections::HashMap, fs::File, io::BufReader};

#[derive(Deserialize)]
#[serde(untagged)]
enum Value {
    Vector(f64),
    FromData(String),
}

#[derive(Deserialize)]
struct Stage {
    name: String,
    expr: String,
}

#[derive(Deserialize)]
struct Metric {
    name: String,
    help: String,
    labels: HashMap<String, String>,
    value: Value,
}

#[derive(Deserialize)]
struct Probe {
    target: String,
    pipeline_stages: Vec<Stage>,
    parser: String,
    metric: Metric,
}
#[derive(Deserialize)]
struct Config {
    probes: Vec<Probe>,
}

pub fn parse(path: String) -> serde_yaml::Result<crate::DataMetrics> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let config: Config = serde_yaml::from_reader(reader)?;

    let probes: Vec<crate::Probe> = config
        .probes
        .iter()
        .map(|p| {
            let value = match &p.metric.value {
                Value::FromData(s) => crate::MetricValue::FromData(s.clone()),
                Value::Vector(x) => crate::MetricValue::Vector(*x),
            };
            let parser = match p.parser.as_str() {
                "json" => crate::JsonParser {},
                _ => panic!("not a valid parser"),
            };
            let stages = p
                .pipeline_stages
                .iter()
                .map(|s| {
                    let stage: Box<dyn crate::PipelineStep + Send + Sync> = match s.name.as_str() {
                        "jq" => Box::new(crate::JqStep {
                            expression: s.expr.clone(),
                        }),
                        _ => panic!("not a valid pipeline_stage"),
                    };
                    stage
                })
                .collect();

            crate::Probe {
                target: p.target.clone(),
                pipeline_stages: stages,
                parser: Box::new(parser),
                metric: crate::MetricConfig::new(
                    p.metric.name.clone(),
                    p.metric.help.clone(),
                    p.metric.labels.clone(),
                    value,
                ),
            }
        })
        .collect();

    Ok(crate::DataMetrics { probes })
}

/*
    DataMetrics::new(vec![Probe {
        target: String::from("http://0.0.0.0:8000/file.json"),
        pipeline_stages: vec![Box::new(JqStep {
            expression: String::from(
                "[.[] | {parent: .name} + (.children[] | {name: .name, val: .value})]",
            ),
        })],
        parser: Box::new(JsonParser {}),
        metric: MetricConfig::new(
            String::from("metric_name"),
            String::from("help text is here"),
            HashMap::from([
                (String::from("parent"), String::from("parent")),
                (String::from("name"), String::from("name")),
            ]),
            MetricValue::FromData(String::from("val")),
        ),
    }])

*/
