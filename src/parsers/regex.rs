use super::{ParseError, Parsed};

pub struct RegexParser {
    regex: regex::Regex,
    labels: Vec<String>,
    value: Option<String>,
}

impl RegexParser {
    pub fn new(pattern: &str, labels: Vec<String>, value: Option<String>) -> Self {
        RegexParser {
            regex: regex::Regex::new(pattern).unwrap(),
            labels,
            value,
        }
    }
}

impl super::Parser for RegexParser {
    fn parse(&self, data: &str) -> Result<Vec<Parsed>, ParseError> {
        self.regex
            .captures_iter(data)
            .try_fold(Vec::new(), |mut acc, cap| {
                let mut parsed = Parsed::new();

                for label in &self.labels {
                    let value = cap
                        .name(&label)
                        .map(|m| m.as_str())
                        .ok_or_else(|| ParseError::MissingField("expected field missing".into()))?;

                    parsed.labels.insert(label.clone(), value.to_string());
                }

                if let Some(key) = &self.value {
                    let value = cap
                        .name(key)
                        .map(|m| m.as_str())
                        .ok_or_else(|| ParseError::MissingField("expected field missing".into()))?
                        .parse::<f64>()?;

                    parsed.value = Some(value);
                }

                acc.push(parsed);
                Ok(acc)
            })
    }
}
