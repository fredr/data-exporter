pub struct Stage {
    pub expression: String,
}
impl super::PipelineStage for Stage {
    fn transform(&self, value: &str) -> String {
        let mut compiled = jq_rs::compile(&self.expression).unwrap();
        compiled.run(value).unwrap()
    }
}
