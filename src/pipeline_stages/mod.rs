pub mod jq;
pub mod regex;

pub trait PipelineStage {
    fn transform(&self, value: &str) -> String;
}
