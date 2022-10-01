use std::time::Duration;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use tokio::time::{interval, Interval, MissedTickBehavior};

pub struct Client {
    retry_timeout: Duration,
    retries: usize,
    dont_retry: Vec<StatusCode>,
    session_id: String,
    api_keys: Vec<String>,
    client: reqwest::Client,
}

pub struct ClientOptions {
    retry_timeout: Option<Duration>,
    retries: Option<usize>,
    api_keys: Vec<String>,
    dont_retry: Vec<StatusCode>,
}

impl ClientOptions {
    pub fn new() -> Self {
        Self {
            retry_timeout: None,
            retries: None,
            api_keys: Vec::new(),
            dont_retry: Vec::new(),
        }
    }
    pub fn retry_timeout(mut self, dur: Duration) -> Self {
        self.retry_timeout = Some(dur);
        self
    }
    pub fn retry_timeout_ms(self, ms: u64) -> Self {
        self.retry_timeout(Duration::from_millis(ms))
    }
    pub fn retries(mut self, retries: usize) -> Self {
        self.retries = Some(retries);
        self
    }
    pub fn api_key(mut self, key: String) -> Self {
        self.api_keys.push(key);
        self
    }
    pub fn api_keys(mut self, keys: Vec<String>) -> Self {
        self.api_keys.extend(keys);
        self
    }
    pub fn dont_retry(mut self, code: StatusCode) -> Self {
        self.dont_retry.push(code);
        self
    }
    pub fn dont_retries(mut self, codes: Vec<StatusCode>) -> Self {
        self.dont_retry.extend(codes);
        self
    }
    async fn client_with_session_id() -> Option<(reqwest::Client, String)> {
        use crate::constants::USER_SEARCH_API;
        use reqwest::cookie::{CookieStore, Jar};
        use reqwest::Url;
        use std::str::FromStr;
        use std::sync::Arc;

        let url = Url::from_str("https://steamcommunity.com/").ok()?;

        let jar = Arc::new(Jar::default());
        let builder = reqwest::Client::builder().cookie_provider(Arc::clone(&jar));
        let client = builder.build().ok()?;

        let resp = client.get(USER_SEARCH_API).send().await.ok()?;
        if resp.status() != StatusCode::UNAUTHORIZED {
            // Every status-code other than 401 should be an error
            let _ = resp.error_for_status().ok()?;
        }

        let cookies = jar.cookies(&url)?;
        let cookie_str = cookies.to_str().ok()?;
        let session_id = cookie_str
            .split(';')
            .map(str::trim_start)
            .find(|&str| str.starts_with("sessionid="))
            .map(|str| &str[10..])?;

        Some((client, session_id.to_string()))
    }
    pub async fn build(self) -> Client {
        if self.api_keys.is_empty() {
            panic!("no api-key has been set");
        }
        let (client, session_id) = Self::client_with_session_id().await.unwrap();
        Client {
            retry_timeout: self.retry_timeout.unwrap_or(Duration::from_millis(1000)),
            retries: self.retries.unwrap_or(3),
            dont_retry: self.dont_retry,
            session_id: session_id,
            api_keys: self.api_keys,
            client: client,
        }
    }
}

impl Client {
    pub async fn get_json<T>(&self, url: &str, query: &[(&str, &str)]) -> reqwest::Result<T>
    where
        T: DeserializeOwned,
    {
        let mut retries = 0_usize;
        loop {
            let err = match self.client.get(url).query(query).send().await {
                Ok(resp) => return Ok(resp.json().await?),
                Err(err) => err,
            };
            if retries == self.retries {
                break Err(err);
            }
            if let Some(status) = err.status() {
                if self.dont_retry.contains(&status) {
                    break Err(err);
                }
            }
            retries += 1;
            tokio::time::sleep(self.retry_timeout).await;
            println!("retry({}): {}", retries, err);
        }
    }
    pub fn api_key(&self) -> &str {
        self.api_keys[0].as_str()
    }
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }
}

pub fn limiter(per_sec: u64) -> Interval {
    let delay_ms = (1.0 / per_sec as f64) as u64;
    let mut limiter = interval(Duration::from_millis(delay_ms));
    limiter.set_missed_tick_behavior(MissedTickBehavior::Burst);
    limiter
}

#[cfg(test)]
mod tests {
    use super::ClientOptions;

    #[tokio::test]
    async fn it_works() {
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let _client = ClientOptions::new().api_key(api_key).build().await;
    }
}
