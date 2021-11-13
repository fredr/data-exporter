pub mod json;

// TODO(fredr): create our own Value type that we can parse into from different formats
pub trait Parser {
    fn parse(&self, data: &str) -> serde_json::Value;
}
