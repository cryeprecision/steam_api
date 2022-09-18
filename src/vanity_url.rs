use crate::constants::VANITY_API;
use crate::request_helper::send_request;
use crate::steam_id::SteamId;

use std::convert::TryFrom;
use std::str::FromStr;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
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
    pub steam_id: SteamId,
}

impl<'a> TryFrom<(Response, &'a str)> for VanityUrl<'a> {
    type Error = VanityUrlError;
    fn try_from((response, url): (Response, &'a str)) -> Result<Self> {
        let id = response.response.steam_id;
        let id = id.ok_or(VanityUrlError::InvalidVanityUrl(url.to_owned()))?;
        let id = SteamId::from_str(&id).map_err(|_| VanityUrlError::InvalidSteamId(id.to_owned()));
        Ok(Self {
            vanity_url: url,
            steam_id: id?,
        })
    }
}

/// Resolve a Vanity-URL using [`this endpoint`](https://partner.steamgames.com/doc/webapi/ISteamUser#ResolveVanityURL).
pub async fn resolve_vanity_url<'a>(
    client: &'a Client,
    api_key: &'a str,
    vanity_url: &'a str,
) -> Result<VanityUrl<'a>> {
    let query = [("key", api_key), ("vanityurl", vanity_url)];
    let req = client.get(VANITY_API).query(&query);
    let resp = send_request::<Response>(req, true, true).await?;
    VanityUrl::try_from((resp, vanity_url))
}

#[cfg(test)]
mod tests {
    use super::resolve_vanity_url;
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        // https://steamcommunity.com/id/GabeLoganNewell
        // https://steamcommunity.com/id/john
        // https://steamcommunity.com/id/4in50ayimf
        let urls = ["GabeLoganNewell", "john", "4in50ayimf", "a", "b", "c", "d"];

        dotenv::dotenv().unwrap();
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(urls.iter().map(|&u| u))
            .map(|url| resolve_vanity_url(&client, &api_key, url))
            .buffer_unordered(2);

        while let Some(res) = stream.next().await {
            let inner = match res {
                Err(err) => {
                    println!("Request failed: {}", err);
                    continue;
                }
                Ok(val) => val,
            };
            println!("{} => {}", inner.vanity_url, inner.steam_id);
        }
    }
}
