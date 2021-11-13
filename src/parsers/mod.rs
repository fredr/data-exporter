// TODO(fredr): create our own Value type that we can parse into from different formats
pub trait Parser {
    fn parse(&self, data: &str) -> serde_json::Value;
}

pub struct JsonParser {}

impl Parser for JsonParser {
    fn parse(&self, data: &str) -> serde_json::Value {
        serde_json::from_str(data).unwrap()
    }
}
