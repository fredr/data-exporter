use super::{ParseError, Parsed};

pub struct JsonParser {
    labels: Vec<String>,
    value: Option<String>,
}

impl super::Parser for JsonParser {
    fn parse(&self, data: &str) -> Result<Vec<Parsed>, ParseError> {
        match serde_json::from_str(data)? {
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|v| match v {
                    serde_json::Value::Object(obj) => self.handle_obj(obj),
                    _ => Err(ParseError::IncorrectType(
                        "exepcted object or array of objects".into(),
                    )),
                })
                .collect(),
            serde_json::Value::Object(obj) => Ok(vec![self.handle_obj(&obj)?]),
            _ => Err(ParseError::IncorrectType(
                "exepcted object or array of objects".into(),
            )),
        }
    }
}

impl JsonParser {
    pub fn new(labels: Vec<String>, value: Option<String>) -> JsonParser {
        JsonParser { labels, value }
    }

    fn handle_obj(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Parsed, ParseError> {
        let mut parsed = Parsed::new();

        for label in &self.labels {
            let value = obj
                .get(label)
                .map(|v| v.as_str())
                .flatten()
                .ok_or_else(|| ParseError::MissingField("expected field missing".into()))?;

            parsed.labels.insert(label.clone(), value.to_string());
        }

        if let Some(key) = &self.value {
            let value = obj
                .get(key)
                .ok_or_else(|| ParseError::MissingField("expected field missing".into()))?
                .as_f64()
                .ok_or_else(|| ParseError::IncorrectType("expected a float64".into()))?;
            parsed.value = Some(value);
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Parser;

    #[test]
    fn test_parse_labels_object() {
        let data = r#"{"label": "value"}"#;
        let p = JsonParser::new(vec![String::from("label")], None);
        let parsed = p.parse(data).expect("could not parse data");
        assert_eq!(parsed[0].labels.get("label"), Some(&String::from("value")));
    }
    #[test]
    fn test_parse_value_object() {
        let data = r#"{"val": 100}"#;
        let p = JsonParser::new(Vec::new(), Some(String::from("val")));
        let parsed = p.parse(data).expect("could not parse data");
        assert_eq!(parsed[0].value, Some(100f64));
    }
    #[test]
    fn test_error_parse_labels_object_missing_field() {
        let data = r#"{"label": "value"}"#;
        let p = JsonParser::new(vec![String::from("other")], None);
        let parsed = p.parse(data);
        assert!(matches!(parsed, Err(ParseError::MissingField(..))))
    }
    #[test]
    fn test_error_parse_labels_object_missing_value() {
        let data = r#"{"label": "value"}"#;
        let p = JsonParser::new(vec![String::from("label")], Some(String::from("val")));
        let parsed = p.parse(data);
        assert!(matches!(parsed, Err(ParseError::MissingField(..))))
    }
    #[test]
    fn test_error_parse_labels_object_incorrect_value_type() {
        let data = r#"{"label": "value", "val": "string"}"#;
        let p = JsonParser::new(vec![String::from("label")], Some(String::from("val")));
        let parsed = p.parse(data);
        assert!(matches!(parsed, Err(ParseError::IncorrectType(..))))
    }
    #[test]
    fn test_error_invalid_json() {
        let p = JsonParser::new(Vec::new(), None);
        assert!(matches!(
            p.parse("not json"),
            Err(ParseError::InvalidJson(..))
        ));
    }
    #[test]
    fn test_error_incorrect_type() {
        let p = JsonParser::new(Vec::new(), None);
        assert!(matches!(
            p.parse(r#""json string""#),
            Err(ParseError::IncorrectType(..))
        ));
    }
}
