use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::AsyncReadExt;
pub mod http;

#[derive(Debug)]
pub enum TargetError {
    HTTP(reqwest::Error),
    IO(std::io::Error),
}
impl From<std::io::Error> for TargetError {
    fn from(e: std::io::Error) -> Self {
        TargetError::IO(e)
    }
}
impl From<reqwest::Error> for TargetError {
    fn from(e: reqwest::Error) -> Self {
        TargetError::HTTP(e)
    }
}

#[derive(Debug)]
pub enum Target {
    Http(http::Config),
    File { path: String },
}

impl Target {
    pub fn describe(&self) -> &str {
        match self {
            Self::Http(http::Config { url }) => url,
            Self::File { path } => path,
        }
    }
    pub async fn fetch(&self) -> Result<Bytes, TargetError> {
        match &self {
            Self::Http(config) => Ok(config.fetch().await?),
            Self::File { path } => {
                let mut file = tokio::fs::File::open(path).await?;
                let mut buffer = BytesMut::new().writer();
                file.read(buffer.get_mut()).await?;

                Ok(buffer.into_inner().freeze())
            }
        }
    }
}
