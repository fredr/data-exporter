const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug)]
pub struct Config {
    pub url: String,
}

impl Config {
    pub async fn fetch(&self) -> reqwest::Result<String> {
        let client = reqwest::Client::new();
        let req = client
            .request(reqwest::Method::GET, &self.url)
            .header(reqwest::header::USER_AGENT, format!("{}/{}", NAME, VERSION));
        let resp = req.send().await?;

        resp.text().await
    }
}
