use std::str::Utf8Error;

use bytes::Bytes;
use thiserror::Error;

use super::Service;

pub struct RegexStage<S> {
    service: S,
    regex: regex::Regex,
    replace: String,
}

impl<S> RegexStage<S> {
    pub fn new(service: S, regex: regex::Regex, replace: String) -> Self {
        Self {
            service,
            regex,
            replace,
        }
    }
}

#[derive(Error, Debug)]
pub enum RegexStageError {
    #[error("invalid input")]
    Input(#[from] Utf8Error),
}

impl<S> Service for RegexStage<S>
where
    S: Service,
    S::Error: From<RegexStageError>,
{
    type Error = S::Error;

    fn call(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        let input = std::str::from_utf8(&input).map_err(Into::into)?;
        let result = self.regex.replace_all(input, &self.replace);

        let bytes = Bytes::from(result.into_owned());

        self.service.call(bytes)
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline_stages::Pipeline;

    use super::*;

    #[test]
    fn test_replace() {
        let text = Bytes::from(r#"This are text that are wrong"#);
        let stage = RegexStage::new(
            Pipeline::new(),
            regex::Regex::new("are").unwrap(),
            "is".into(),
        );

        assert_eq!(stage.call(text).unwrap(), "This is text that is wrong");
    }
}
