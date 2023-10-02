use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use reqwest::cookie::Jar;
use reqwest::header::{HeaderValue, SET_COOKIE};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::constants::USER_SEARCH_API;

pub struct Client {
    retry_timeout: Duration,
    max_retries: usize,
    dont_retry: Vec<StatusCode>,
    session_id: String,
    api_keys: Vec<String>,
    client: reqwest::Client,
    retries: AtomicUsize,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("builder configuration is invalid: {0}")]
    ClientConfig(reqwest::Error),
    #[error("unexpected status code: {0}")]
    Status(reqwest::Error),
    #[error("couldn't make request to get session id: {0}")]
    Request(reqwest::Error),
    #[error("response is missing set-cookie header for session id")]
    SetCookieMissing,
    #[error("set-cookie header for session-id is not valid utf-8")]
    HeadersUtf8,
    #[error("session id in set-cookie header has invalid length")]
    SetCookieLen,
    #[error("builder is missing api-key")]
    ApiKey,
}
type Result<T> = std::result::Result<T, Error>;

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

    pub fn retries(&mut self, retries: usize) -> &mut Self {
        self.max_retries = Some(retries);
        self
    }
    pub fn retry_timeout(&mut self, dur: Duration) -> &mut Self {
        self.retry_timeout = Some(dur);
        self
    }
    pub fn retry_timeout_ms(&mut self, ms: u64) -> &mut Self {
        self.retry_timeout = Some(Duration::from_millis(ms));
        self
    }
    pub fn dont_retry(&mut self, code: StatusCode) -> &mut Self {
        self.dont_retry.push(code);
        self
    }
    pub fn dont_retries(&mut self, codes: Vec<StatusCode>) -> &mut Self {
        self.dont_retry.extend(codes);
        self
    }
    pub fn dont_retry_unauthorized(&mut self) -> &mut Self {
        self.dont_retry.push(StatusCode::UNAUTHORIZED);
        self
    }

    pub fn api_key(&mut self, key: String) -> &mut Self {
        self.api_keys.push(key);
        self
    }
    pub fn api_keys(&mut self, keys: Vec<String>) -> &mut Self {
        self.api_keys.extend(keys);
        self
    }

    fn client_with_cookie_store() -> Result<reqwest::Client> {
        let builder = reqwest::Client::builder().cookie_provider(Arc::new(Jar::default()));
        let client = builder.build().map_err(Error::ClientConfig)?;
        Ok(client)
    }
    async fn get_session_id(client: &reqwest::Client) -> Result<String> {
        fn find_cookie(v: &HeaderValue) -> Option<&str> {
            let str = v.to_str().ok()?;
            str.strip_prefix(SESSION_ID_PREFIX)?
                .split_once(';')
                .map(|(id, _)| id)
        }

        // Header value looks like this
        // sessionid=a0a0a0a0a0a0a0a0a0a0a0a0; Path=/; Secure; SameSite=None
        const SESSION_ID_PREFIX: &str = "sessionid=";

        // Using the USER_SEARCH_API URL because it returns very little data
        let resp = client
            .get(USER_SEARCH_API)
            .send()
            .await
            .map_err(Error::Request)?;

        // We expect this status code to be returned
        if resp.status() != StatusCode::UNAUTHORIZED {
            resp.error_for_status_ref().map_err(Error::Status)?;
        }

        let set_cookies = resp.headers().get_all(SET_COOKIE);
        let session_id = set_cookies
            .iter()
            .filter_map(find_cookie)
            .next()
            .ok_or(Error::SetCookieMissing)?;

        // let session_id = cookie.split

        Ok(session_id.to_string())
    }

    /// # Panics
    /// - If no api-key has been set
    /// - If session_id but no cookie_store
    pub async fn build(&self) -> Result<Client> {
        if self.api_keys.is_empty() {
            return Err(Error::ApiKey);
        }

        let client = Self::client_with_cookie_store()?;
        let session_id = Self::get_session_id(&client).await?;

        let mut dont_retry = self.dont_retry.clone();

        // TODO: Is it really a good idea to add this here? (NOIDONTTHINKSO)
        // if !dont_retry.contains(&StatusCode::UNAUTHORIZED) {
        //     dont_retry.push(StatusCode::UNAUTHORIZED);
        // }

        dont_retry.sort_unstable();
        dont_retry.dedup();

        Ok(Client {
            retry_timeout: self.retry_timeout.unwrap_or(Duration::from_millis(1000)),
            max_retries: self.max_retries.unwrap_or(3),
            dont_retry,
            session_id,
            api_keys: self.api_keys.clone(),
            client,
            retries: AtomicUsize::new(0),
        })
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
            self.retries.fetch_add(retries, Ordering::SeqCst);
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
        self.retries.load(Ordering::SeqCst)
    }
    pub fn reset_retries(&self) {
        self.retries.store(0, Ordering::SeqCst);
    }
    /// Clone the inner [`reqwest::Client`], which is just a call to `Arc::clone`
    /// to share the connection pool with other program parts that need one.
    pub fn clone_client(&self) -> reqwest::Client {
        self.client.clone()
    }
    pub fn options() -> ClientOptions {
        ClientOptions::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    #[tokio::test]
    async fn get_session_id() {
        let client = Client::options()
            .api_key("invalid".to_string())
            .build()
            .await
            .unwrap();

        println!("{}", client.session_id());
    }
}
