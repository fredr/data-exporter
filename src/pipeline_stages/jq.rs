use std::str::Utf8Error;

use bytes::Bytes;
use thiserror::Error;

use super::Service;

pub struct JqStage<S> {
    service: S,
    expression: String,
}

impl<S> JqStage<S> {
    pub fn new(service: S, expression: String) -> Self {
        Self {
            service,
            expression,
        }
    }
}

#[derive(Error, Debug)]
pub enum JqStageError {
    #[error("executing jq failed")]
    Jq(#[from] jq_rs::Error),
    #[error("invalid input")]
    Input(#[from] Utf8Error),
}

impl<S> Service for JqStage<S>
where
    S: Service,
    S::Error: From<JqStageError>,
{
    type Error = S::Error;

    fn call(&self, input: bytes::Bytes) -> Result<bytes::Bytes, Self::Error> {
        // TODO: measure how long this take and move out so it doent need to compile on every request (if it is slow).
        // JqProgram is not Send, so need some kind of actor or similar

        let input = std::str::from_utf8(&input).map_err(Into::into)?;
        let mut compiled = jq_rs::compile(&self.expression).map_err(Into::into)?;

        let resp = compiled
            .run(input)
            .map_err(JqStageError::Jq)
            .map(Bytes::from)
            .map_err(Into::into)?;

        self.service.call(resp)
    }
}
