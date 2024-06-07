use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::VANITY_API;
use crate::model::SteamIdStr;
use crate::steam_id::SteamId;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("invalid steam-id: {0}")]
    Json(#[from] serde_json::Error),

    #[error("vanity url '{0}' not found")]
    NotFound(String),
}
type Result<T> = std::result::Result<T, VanityUrlError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VanityUrl {
    #[serde(rename = "steamid")]
    pub steam_id: Option<SteamIdStr>,
}

#[derive(Deserialize, Debug)]
struct Response {
    response: VanityUrl,
}

impl From<Response> for VanityUrl {
    fn from(value: Response) -> Self {
        value.response
    }
}

impl Client {
    /// Resolve a Vanity-URL using [`this endpoint`](https://partner.steamgames.com/doc/webapi/ISteamUser#ResolveVanityURL).
    pub async fn resolve_vanity_url(&self, vanity_url: &str) -> Result<SteamId> {
        let query = [("key", self.api_key()), ("vanityurl", vanity_url)];
        let json = self.get_json::<Response>(VANITY_API, &query).await?;
        Ok(json
            .response
            .steam_id
            .ok_or_else(|| VanityUrlError::NotFound(vanity_url.to_string()))?
            .steam_id())
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::model::api::vanity_url::VanityUrl;
    use crate::model::SteamIdStr;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("vanity_url.json");
        let url: VanityUrl = json.into();
        assert_eq!(url.steam_id, Some(SteamIdStr(76561197960287930)))
    }
}
