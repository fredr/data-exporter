use std::convert::Infallible;

pub mod jq;
pub mod regex;

pub trait PipelineStage {
    type Error;

    fn transform(&self, value: &str) -> Result<String, Self::Error>;
}

#[derive(Debug)]
pub enum PipelineError {
    Jq,
    Regex,
}

pub struct PipelineMapErr<T> {
    inner: T,
}

impl<T> PipelineMapErr<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> PipelineStage for PipelineMapErr<T>
where
    T: PipelineStage,
    T::Error: Into<PipelineError>,
{
    type Error = PipelineError;

    fn transform(&self, value: &str) -> Result<String, Self::Error> {
        self.inner.transform(value).map_err(Into::into)
    }
}

impl From<Infallible> for PipelineError {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
