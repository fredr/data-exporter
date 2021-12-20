use super::PipelineStage;

pub struct Stage {
    pub regex: regex::Regex,
    pub replace: String,
}

impl PipelineStage for Stage {
    fn transform(&self, value: &str) -> String {
        self.regex.replace_all(value, &self.replace).to_string()
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

        assert_eq!(stage.transform(text), "This is text that is wrong");
    }
}
