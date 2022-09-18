/// client_error <=> status is within 400-499.
/// server_error <=> status is within 500-599.
pub async fn send_request<T: serde::de::DeserializeOwned>(
    builder: reqwest::RequestBuilder,
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
