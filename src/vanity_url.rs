use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::client::Client;
use crate::constants::VANITY_API;
use crate::parse_response::{ParseJsonResponse, ParseResponse};
use crate::steam_id::SteamId;

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

impl ParseJsonResponse for Response {
    type Error = VanityUrlError;
    type Output = Option<SteamId>;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        let Some(id) = self.response.steam_id else {
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
        let json = self.get_json::<Response>(VANITY_API, &query).await?;
        json.parse_steam_json()
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::parse_response::ParseJsonResponse;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("vanity_url.json");
        let url = json.parse_steam_json().unwrap();
        assert_eq!(url, Some(crate::SteamId(76561197960287930)))
    }
}
