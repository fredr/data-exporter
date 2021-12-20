use std::{collections::HashMap, num::ParseFloatError};

pub mod json;
pub mod regex;

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
    IncorrectType(String),
    MissingField(String),
    ParseFloat(ParseFloatError),
}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::InvalidJson(e)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(e: ParseFloatError) -> Self {
        ParseError::ParseFloat(e)
    }
}

pub struct Parsed {
    pub value: Option<f64>,
    pub labels: HashMap<String, String>,
}

impl Parsed {
    fn new() -> Self {
        Parsed {
            labels: HashMap::new(),
            value: None,
        }
    }
}

pub trait Parser {
    fn parse(&self, data: &str) -> Result<Vec<Parsed>, ParseError>;
}
