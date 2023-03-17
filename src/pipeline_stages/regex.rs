use std::convert::Infallible;

use super::PipelineStage;

pub struct Stage {
    pub regex: regex::Regex,
    pub replace: String,
}

impl PipelineStage for Stage {
    type Error = Infallible;
    fn transform(&self, value: &str) -> Result<String, Self::Error> {
        Ok(self.regex.replace_all(value, &self.replace).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace() {
        let text = r#"This are text that are wrong"#;
        let stage = Stage {
            replace: "is".into(),
            regex: regex::Regex::new("are").unwrap(),
        };

        assert_eq!(stage.transform(text).unwrap(), "This is text that is wrong");
    }
}
