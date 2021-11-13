pub trait PipelineStage {
    fn transform(&self, value: &str) -> String;
}

pub struct JqStage {
    pub expression: String,
}
impl PipelineStage for JqStage {
    fn transform(&self, value: &str) -> String {
        let mut compiled = jq_rs::compile(&self.expression).unwrap();
        compiled.run(value).unwrap()
    }
}
