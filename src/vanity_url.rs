use crate::constants::VANITY_API;
use crate::constants::{RETRIES, WAIT_DURATION};
use crate::parse_response::ParseResponse;
use crate::request_helper::send_request_with_reties;
use crate::steam_id::SteamId;

use std::str::FromStr;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
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

impl ParseResponse<Response> for Option<SteamId> {
    type Error = VanityUrlError;
    fn parse_response(value: Response) -> Result<Self> {
        let id = match value.response.steam_id {
            Some(id) => id,
            None => return Ok(None),
        };
        match SteamId::from_str(&id) {
            Err(_) => Err(VanityUrlError::InvalidSteamId(id.to_owned())),
            Ok(id) => Ok(Some(id)),
        }
    }
}

/// Resolve a Vanity-URL using [`this endpoint`](https://partner.steamgames.com/doc/webapi/ISteamUser#ResolveVanityURL).
pub async fn resolve_vanity_url(
    client: &Client,
    api_key: &str,
    vanity_url: &str,
) -> Result<Option<SteamId>> {
    let query = [("key", api_key), ("vanityurl", vanity_url)];

    let resp = send_request_with_reties(
        client,
        VANITY_API,
        &query,
        true,
        true,
        RETRIES,
        WAIT_DURATION,
    )
    .await?;

    Option::<SteamId>::parse_response(resp)
}

#[cfg(test)]
mod tests {
    use super::resolve_vanity_url;
    use futures::{FutureExt, StreamExt};

    #[tokio::test]
    async fn it_works() {
        // https://steamcommunity.com/id/GabeLoganNewell
        // https://steamcommunity.com/id/john
        // https://steamcommunity.com/id/4in50ayimf
        let urls = ["GabeLoganNewell", "john", "4in50ayimf"];

        dotenv::dotenv().unwrap();
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(urls.iter().map(|&u| u))
            .map(|url| resolve_vanity_url(&client, &api_key, url).map(move |r| (r, url)))
            .buffer_unordered(2);

        while let Some((res, url)) = stream.next().await {
            let id = match res {
                Err(err) => {
                    println!("[{}] err: {}", url, err);
                    continue;
                }
                Ok(id) => id,
            };
            println!("[{}] {:?}", url, id);
        }
    }
}
