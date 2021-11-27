use std::collections::HashMap;

use super::ParseError;

pub struct Parser {
    pub labels: Vec<String>,
    pub value: Option<String>,
}

impl super::Parser for Parser {
    fn parse(&self, data: &str) -> Result<Vec<super::Parsed>, ParseError> {
        match serde_json::from_str(data)? {
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|v| match v {
                    serde_json::Value::Object(obj) => self.handle_obj(obj),
                    _ => {
                        return Err(ParseError::IncorrectType(String::from(
                            "exepcted object or array of objects",
                        )))
                    }
                })
                .collect(),
            serde_json::Value::Object(obj) => Ok(vec![self.handle_obj(&obj)?]),
            _ => {
                return Err(ParseError::IncorrectType(String::from(
                    "exepcted object or array of objects",
                )))
            }
        }
    }
}

impl Parser {
    fn handle_obj(
        &self,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<super::Parsed, ParseError> {
        let mut parsed_labels = HashMap::new();

        for label in &self.labels {
            let value =
                obj.get(label)
                    .map(|v| v.as_str())
                    .flatten()
                    .ok_or(ParseError::MissingField(String::from(
                        "expected field missing",
                    )))?;

            parsed_labels.insert(label.clone(), value.to_string());
        }

        let parsed_value = self
            .value
            .as_ref()
            .map(|key| obj.get(key).map(|val| val.as_f64()).flatten())
            .flatten();

        Ok(super::Parsed {
            labels: parsed_labels,
            value: parsed_value,
        })
    }
}
