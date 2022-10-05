use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use reqwest::StatusCode;
use serde::de::DeserializeOwned;

pub struct Client {
    retry_timeout: Duration,
    max_retries: usize,
    dont_retry: Vec<StatusCode>,
    session_id: String,
    api_keys: Vec<String>,
    client: reqwest::Client,
    retries: AtomicUsize,
}

// TODO: Make getting session_id optional since it requires a request on construction
pub struct ClientOptions {
    retry_timeout: Option<Duration>,
    max_retries: Option<usize>,
    api_keys: Vec<String>,
    dont_retry: Vec<StatusCode>,
}

impl Default for ClientOptions {
    fn default() -> Self {
        ClientOptions::new()
    }
}

impl ClientOptions {
    #[must_use]
    pub fn new() -> Self {
        Self {
            retry_timeout: None,
            max_retries: None,
            api_keys: Vec::new(),
            dont_retry: Vec::new(),
        }
    }
    #[must_use]
    pub fn retry_timeout(mut self, dur: Duration) -> Self {
        self.retry_timeout = Some(dur);
        self
    }
    #[must_use]
    pub fn retry_timeout_ms(self, ms: u64) -> Self {
        self.retry_timeout(Duration::from_millis(ms))
    }
    #[must_use]
    pub fn retries(mut self, retries: usize) -> Self {
        self.max_retries = Some(retries);
        self
    }
    #[must_use]
    pub fn api_key(mut self, key: String) -> Self {
        self.api_keys.push(key);
        self
    }
    #[must_use]
    pub fn api_keys(mut self, keys: Vec<String>) -> Self {
        self.api_keys.extend(keys);
        self
    }
    #[must_use]
    pub fn dont_retry(mut self, code: StatusCode) -> Self {
        self.dont_retry.push(code);
        self
    }
    #[must_use]
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

        const SESSION_ID: &str = "sessionid=";

        let url = Url::from_str("https://steamcommunity.com/").ok()?;

        let jar = Arc::new(Jar::default());
        let builder = reqwest::Client::builder().cookie_provider(Arc::clone(&jar));
        let client = builder.build().ok()?;

        let resp = client.get(USER_SEARCH_API).send().await.ok()?;
        if resp.status() != StatusCode::UNAUTHORIZED {
            // Every status-code other than 401 should be an error
            resp.error_for_status().ok()?;
        }

        let cookies = jar.cookies(&url)?;
        let cookie_str = cookies.to_str().ok()?;
        let session_id = cookie_str
            .split(';')
            .map(str::trim_start)
            .find(|&str| str.starts_with(SESSION_ID))
            .map(|str| &str[SESSION_ID.len()..])?;

        Some((client, session_id.to_string()))
    }
    /// # Panics
    /// Panics, if no api-key has been set.
    pub async fn build(self) -> Client {
        assert!(!self.api_keys.is_empty(), "no api-key has been set");

        let (client, session_id) = Self::client_with_session_id().await.unwrap();

        let mut dont_retry = self.dont_retry;

        // TODO: Is it really a good idea to add this here? (NOIDONTTHINKSO)
        // if !dont_retry.contains(&StatusCode::UNAUTHORIZED) {
        //     dont_retry.push(StatusCode::UNAUTHORIZED);
        // }

        dont_retry.sort_unstable();
        dont_retry.dedup();

        Client {
            retry_timeout: self.retry_timeout.unwrap_or(Duration::from_millis(1000)),
            max_retries: self.max_retries.unwrap_or(3),
            dont_retry,
            session_id,
            api_keys: self.api_keys,
            client,
            retries: AtomicUsize::new(0),
        }
    }
}

impl Client {
    pub async fn get_json<T>(&self, url: &str, query: &[(&str, &str)]) -> reqwest::Result<T>
    where
        T: DeserializeOwned,
    {
        let mut retries = 0_usize;
        let result = loop {
            let err = match self.client.get(url).query(query).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(resp) => break Ok(resp.json().await?),
                    Err(err) => err,
                },
                Err(err) => err,
            };
            if retries == self.max_retries {
                break Err(err);
            }
            if let Some(status) = err.status() {
                if self.dont_retry.contains(&status) {
                    break Err(err);
                }
            }
            retries += 1;
            tokio::time::sleep(self.retry_timeout).await;
        };
        if retries > 0 {
            self.retries.fetch_add(retries, Ordering::Relaxed);
        }
        result
    }
    pub fn api_key(&self) -> &str {
        self.api_keys[0].as_str()
    }
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }
    pub fn retries(&self) -> usize {
        self.retries.load(Ordering::Relaxed)
    }
    pub fn reset_retries(&self) {
        self.retries.store(0, Ordering::Relaxed);
    }
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
