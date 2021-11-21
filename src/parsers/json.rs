use super::ParseError;

pub struct Parser;

impl super::Parser for Parser {
    fn parse(&self, data: &str) -> Result<serde_json::Value, ParseError> {
        Ok(serde_json::from_str(data)?)
    }
}
