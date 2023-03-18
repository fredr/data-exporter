use std::fmt::Debug;

use bytes::Bytes;
use thiserror::Error;

use self::{jq::JqStageError, regex::RegexStageError};

mod jq;
mod regex;
mod service;

pub use self::regex::*;
pub use jq::*;
pub use service::Service;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("jq stage failed")]
    Jq(#[from] JqStageError),
    #[error("regex stage failed")]
    Regex(#[from] RegexStageError),
}

pub struct Pipeline;
impl Pipeline {
    pub fn new() -> Self {
        Self
    }
}
impl Service for Pipeline {
    type Error = PipelineError;

    fn call(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        Ok(input)
    }
}
