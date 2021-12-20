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
                        .name(label)
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

#[cfg(test)]
mod tests {
    use crate::parsers::Parser;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_regex_parser() {
        let text = r#"a=1,b=2,c=3,d=4"#;
        let pattern = r#"(?P<key>[a-z])=(?P<val>\d)"#;

        let parser = RegexParser::new(pattern, vec!["key".to_string()], Some("val".to_string()));
        let parsed = parser.parse(text).unwrap();

        assert_eq!(parsed.len(), 4);

        assert_eq!(
            parsed[0].labels,
            HashMap::from([("key".to_string(), "a".to_string())])
        );
        assert_eq!(parsed[0].value, Some(1f64));

        assert_eq!(
            parsed[1].labels,
            HashMap::from([("key".to_string(), "b".to_string())])
        );
        assert_eq!(parsed[1].value, Some(2f64));

        assert_eq!(
            parsed[2].labels,
            HashMap::from([("key".to_string(), "c".to_string())])
        );
        assert_eq!(parsed[2].value, Some(3f64));

        assert_eq!(
            parsed[3].labels,
            HashMap::from([("key".to_string(), "d".to_string())])
        );
        assert_eq!(parsed[3].value, Some(4f64));
    }
}
