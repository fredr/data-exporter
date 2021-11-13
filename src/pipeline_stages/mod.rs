pub mod jq;

pub trait PipelineStage {
    fn transform(&self, value: &str) -> String;
}
