use std::time::Duration;

use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use tokio::time::sleep;

/// client_error <=> status is within 400-499.
/// server_error <=> status is within 500-599.
pub async fn send_request<T: DeserializeOwned>(
    builder: RequestBuilder,
    error_for_client_error: bool,
    error_for_server_error: bool,
) -> reqwest::Result<T> {
    let resp = builder.send().await?;
    if error_for_client_error && resp.status().is_client_error()
        || error_for_server_error && resp.status().is_server_error()
    {
        Err(resp.error_for_status().unwrap_err())
    } else {
        resp.json::<T>().await
    }
}

pub async fn send_request_with_reties<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    query: &[(&str, &str)],
    error_for_client_error: bool,
    error_for_server_error: bool,
    max_retries: usize,
    wait_duration: Duration,
) -> reqwest::Result<T> {
    let mut retries = 0_usize;
    loop {
        let req = client.get(url).query(query);
        let err = match send_request::<T>(req, error_for_client_error, error_for_server_error).await
        {
            Ok(resp) => return Ok(resp),
            Err(err) => err,
        };
        if retries >= max_retries {
            return Err(err.into());
        }
        retries += 1;
        sleep(wait_duration).await;
    }
}
