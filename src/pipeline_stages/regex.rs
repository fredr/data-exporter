use super::PipelineStage;

pub struct Stage {
    pub regex: regex::Regex,
    pub replace: String,
}

impl PipelineStage for Stage {
    fn transform(&self, value: &str) -> String {
        self.regex.replace(value, &self.replace).to_string()
    }
}
