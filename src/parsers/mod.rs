use std::collections::HashMap;

pub mod json;

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
    IncorrectType(String),
    MissingField(String),
}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::InvalidJson(e)
    }
}

pub struct Parsed {
    pub value: Option<f64>,
    pub labels: HashMap<String, String>,
}

pub trait Parser {
    fn parse(&self, data: &str) -> Result<Vec<Parsed>, ParseError>;
}
