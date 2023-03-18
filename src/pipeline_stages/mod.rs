use std::{convert::Infallible, fmt::Debug};

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

pub trait PipelineStage {
    type Error;

    fn transform(&self, value: &str) -> Result<String, Self::Error>;
}

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("jq stage failed")]
    Jq(#[from] JqStageError),
    #[error("regex stage failed")]
    Regex(#[from] RegexStageError),
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

/*

#[derive(Debug)]
enum JqStuffError {}
struct JqStuff {
    _query: String,
}
impl PPipelineStage for JqStuff {
    type Error = JqStuffError;

    fn transform(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        Ok([input, Bytes::from("-JQSTUFF-")].concat().into())
    }
}

struct WrapperStage<P1, P2> {
    inner: P1,
    outer: P2,
}
impl<P1, P2> WrapperStage<P1, P2> {
    fn new(inner: P1, outer: P2) -> Self {
        Self { inner, outer }
    }
}

impl<P1, P2> PPipelineStage for WrapperStage<P1, P2>
where
    P1: PPipelineStage,
    P2: PPipelineStage,
{
    type Error = P1::Error;

    fn transform(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        let inner = self.inner.transform(input)?;
        let outer = self.outer.transform(inner)?;

        Ok(outer)
    }
}

impl<P> PPipelineStageWrap<P> for JqStuff {
    type PPipelineStage = WrapperStage<P, JqStuff>;

    fn wrap(self, inner: P) -> Self::PPipelineStage {
        WrapperStage::new(inner, self)
    }
}

struct InfallibleStuff {}

impl PPipelineStage for InfallibleStuff {
    type Error = Infallible;

    fn transform(&self, input: Bytes) -> Result<Bytes, Self::Error> {
        Ok([input, Bytes::from("-INFSTUFF-")].concat().into())
    }
}
trait PPipelineStage {
    type Error;

    fn transform(&self, input: Bytes) -> Result<Bytes, Self::Error>;
}

trait PPipelineStageWrap<P> {
    type PPipelineStage;

    fn wrap(self, inner: P) -> Self::PPipelineStage;
}

#[test]
fn compose() {
    let stage = JqStuff {
        _query: "hello".to_string(),
    };

    let stage = stage.wrap(InfallibleStuff {});

    let output = stage.transform(Bytes::from("outer-input")).unwrap();

    assert_eq!(output, Bytes::from("hello"))
}
*/

// TODO: refactor PipelineStage into something like this, that wrap each other, like tower layers and services, but simpler
// should return futures so they can be called async, and they should all wrap each other.
// What happens with errors? :think:? do we wrap all of the in a map error layer?
//
/*
trait PipelineStage {
    type Error;

    fn transform(input: &str) -> Result<String, Self::Error>;
    fn transform(input) -> Future<Output = Result<String, Self::Error>>
}

trait PipelineLayer<P> {
    type PipelineStage;

    fn layer(&self, inner: P) -> Self::PipelineStage
}

let stage = MyStage::new().layer(MyOtherStage::new());



*/
