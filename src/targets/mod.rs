use tokio::io::AsyncReadExt;

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
    Http { url: String },
    File { path: String },
}

impl Target {
    pub fn describe(&self) -> &str {
        match self {
            Self::Http { url } => url,
            Self::File { path } => path,
        }
    }
    pub async fn fetch(&self) -> Result<String, TargetError> {
        match &self {
            Self::Http { url } => Ok(reqwest::get(url).await?.text().await?),
            Self::File { path } => {
                let mut file = tokio::fs::File::open(path).await?;
                let mut buffer = String::new();
                file.read_to_string(&mut buffer).await?;
                Ok(buffer)
            }
        }
    }
}
