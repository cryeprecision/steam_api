use crate::client::Client;
use crate::constants::VANITY_API;
use crate::parse_response::ParseResponse;
use crate::steam_id::SteamId;

use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("invalid steam-id: {0}")]
    InvalidSteamId(String),
}
type Result<T> = std::result::Result<T, VanityUrlError>;

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
        let Some(id) = value.response.steam_id else {
            return Ok(None);
        };
        let id = SteamId::from_str(&id).map_err(|_| VanityUrlError::InvalidSteamId(id))?;
        Ok(Some(id))
    }
}

impl Client {
    /// Resolve a Vanity-URL using [`this endpoint`](https://partner.steamgames.com/doc/webapi/ISteamUser#ResolveVanityURL).
    pub async fn resolve_vanity_url(&self, vanity_url: &str) -> Result<Option<SteamId>> {
        let query = [("key", self.api_key()), ("vanityurl", vanity_url)];
        let resp = self.get_json::<Response>(VANITY_API, &query).await?;
        Option::<SteamId>::parse_response(resp)
    }
}

#[cfg(test)]
mod tests {
    use futures::{FutureExt, StreamExt};

    use crate::ClientOptions;

    #[tokio::test]
    async fn it_works() {
        // https://steamcommunity.com/id/GabeLoganNewell
        // https://steamcommunity.com/id/john
        // https://steamcommunity.com/id/4in50ayimf
        let urls = ["GabeLoganNewell", "john", "4in50ayimf"];

        dotenv::dotenv().unwrap();

        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let client = ClientOptions::new().api_key(api_key).build().await;

        let mut stream = futures::stream::iter(urls.iter().map(|&u| u))
            .map(|url| client.resolve_vanity_url(url).map(move |r| (r, url)))
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
