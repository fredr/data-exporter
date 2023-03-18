use std::{collections::HashMap, num::ParseFloatError, str::Utf8Error};

use bytes::Bytes;

pub mod json;
pub mod regex;

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
    IncorrectType(String),
    MissingField(String),
    ParseFloat(ParseFloatError),
    InvalidInput(Utf8Error),
}

impl From<Utf8Error> for ParseError {
    fn from(e: Utf8Error) -> Self {
        ParseError::InvalidInput(e)
    }
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
    fn parse(&self, data: Bytes) -> Result<Vec<Parsed>, ParseError>;
}
