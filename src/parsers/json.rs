pub struct Parser;

impl super::Parser for Parser {
    fn parse(&self, data: &str) -> serde_json::Value {
        serde_json::from_str(data).unwrap()
    }
}
