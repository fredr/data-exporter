#[derive(Debug)]
pub struct Config {
    pub url: String,
}

impl Config {
    pub async fn fetch(&self) -> reqwest::Result<String> {
        Ok(reqwest::get(&self.url).await?.text().await?)
    }
}
