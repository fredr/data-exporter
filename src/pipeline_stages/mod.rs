use std::fmt::Debug;

use bytes::Bytes;
use thiserror::Error;

use self::{jq::JqStageError, regex::RegexStageError};

pub trait Service {
    type Error;

    fn call(&self, input: Bytes) -> Result<Bytes, Self::Error>;
}

impl<S> Service for Box<S>
where
    S: Service + ?Sized,
{
    type Error = S::Error;

    fn call(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        (**self).call(input)
    }
}

pub mod jq;
pub mod regex;

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
