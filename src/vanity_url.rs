use crate::constants::{VANITY_API, VANITY_CONCURRENT_REQUESTS};
use crate::steam_id::SteamId;

use std::str::FromStr;

use futures::{Stream, StreamExt};
use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error("invalid vanity-url: {0}")]
    InvalidVanityUrl(String),
    #[error("invalid steam-id: {0}")]
    InvalidSteamId(String),
}
pub type Result<T> = std::result::Result<T, VanityUrlError>;

#[derive(Deserialize, Debug)]
struct ResponseInner {
    #[serde(rename = "steamid")]
    steam_id: Option<String>,
}
#[derive(Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

#[derive(Debug)]
pub struct VanityUrl<'a> {
    pub vanity_url: &'a str,
    pub steam_id: Result<SteamId>,
}

impl TryFrom<(Response, &str)> for SteamId {
    type Error = VanityUrlError;
    fn try_from((response, url): (Response, &str)) -> Result<Self> {
        let id = response.response.steam_id;
        let id = id.ok_or(VanityUrlError::InvalidVanityUrl(url.to_owned()))?;
        SteamId::from_str(&id).map_err(|_| VanityUrlError::InvalidSteamId(id.to_owned()))
    }
}

impl<'a> From<(Response, &'a str)> for VanityUrl<'a> {
    fn from((response, url): (Response, &'a str)) -> Self {
        Self {
            vanity_url: url,
            steam_id: SteamId::try_from((response, url)),
        }
    }
}

/// Resolve a Vanity-URL using [`this endpoint`](https://partner.steamgames.com/doc/webapi/ISteamUser#ResolveVanityURL).
pub async fn resolve_vanity_url<'a>(
    client: &'a Client,
    api_key: &'a str,
    vanity_url: &'a str,
) -> reqwest::Result<VanityUrl<'a>> {
    let query = [("key", api_key), ("vanityurl", vanity_url)];
    let req = client.get(VANITY_API).query(&query);
    let resp = crate::request_helper::send_request::<Response>(req, true, true).await?;
    Ok(VanityUrl::from((resp, vanity_url)))
}

/// Returns a [`futures::stream::Buffered`]
pub fn resolve_vanity_urls<'a>(
    client: &'a Client,
    api_key: &'a str,
    urls: impl Iterator<Item = &'a str>,
) -> impl Stream<Item = reqwest::Result<VanityUrl<'a>>> {
    let buffer_cap = match urls.size_hint() {
        (_, Some(upper)) => VANITY_CONCURRENT_REQUESTS.min(upper),
        _ => VANITY_CONCURRENT_REQUESTS,
    };
    futures::stream::iter(urls)
        .map(|url| resolve_vanity_url(client, api_key, url))
        .buffered(buffer_cap)
}

#[cfg(test)]
mod tests {
    use super::resolve_vanity_url;
    use futures::StreamExt;

    #[tokio::test]
    async fn to_steam_id_string_works() {
        // https://steamcommunity.com/id/GabeLoganNewell
        // https://steamcommunity.com/id/john
        // https://steamcommunity.com/id/4in50ayimf
        let urls = ["GabeLoganNewell", "john", "4in50ayimf", "a", "b", "c", "d"];

        dotenv::dotenv().unwrap();
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(urls.iter().map(|&u| u))
            .map(|url| resolve_vanity_url(&client, &api_key, url))
            .buffered(10);

        while let Some(res) = stream.next().await {
            let inner = match res {
                Err(err) => {
                    println!("Request failed: {}", err);
                    continue;
                }
                Ok(val) => val,
            };
            match inner.steam_id {
                Ok(steam_id) => println!("{} => {}", inner.vanity_url, steam_id),
                Err(err) => println!("{} => {}", inner.vanity_url, err),
            };
        }
    }
}
