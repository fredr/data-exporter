use super::PipelineError;

pub struct Stage {
    pub expression: String,
}

pub enum Error {
    JqError(jq_rs::Error),
}

impl super::PipelineStage for Stage {
    type Error = Error;
    fn transform(&self, value: &str) -> Result<String, Self::Error> {
        // TODO: measure how long this take and move out so it doent need to compile on every request (if it is slow).
        // JqProgram is not Send, so need some kind of actor or similar
        let mut compiled = jq_rs::compile(&self.expression).map_err(Error::JqError)?;
        compiled.run(value).map_err(Error::JqError)
    }
}

impl From<Error> for PipelineError {
    fn from(_err: Error) -> Self {
        PipelineError::Jq
    }
}
