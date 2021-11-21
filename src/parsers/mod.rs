pub mod json;

#[derive(Debug)]
pub enum ParseError {
    InvalidJson(serde_json::Error),
}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::InvalidJson(e)
    }
}

// TODO(fredr): create our own Value type that we can parse into from different formats
pub trait Parser {
    fn parse(&self, data: &str) -> Result<serde_json::Value, ParseError>;
}
