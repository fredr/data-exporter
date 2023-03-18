use bytes::Bytes;

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
