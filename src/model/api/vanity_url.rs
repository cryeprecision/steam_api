use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::VANITY_API;
use crate::model::SteamId;

#[derive(Error, Debug)]
pub enum VanityUrlError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("invalid steam-id: {0}")]
    Json(#[from] serde_json::Error),
}
type Result<T> = std::result::Result<T, VanityUrlError>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VanityUrl {
    #[serde(rename = "steamid")]
    steam_id: Option<SteamId>,
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
    pub async fn resolve_vanity_url(&self, vanity_url: &str) -> Result<VanityUrl> {
        let query = [("key", self.api_key()), ("vanityurl", vanity_url)];
        let json = self.get_json::<Response>(VANITY_API, &query).await?;
        Ok(json.into())
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::model::api::vanity_url::VanityUrl;
    use crate::model::SteamId;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("vanity_url.json");
        let url: VanityUrl = json.into();
        assert_eq!(url.steam_id, Some(SteamId(76561197960287930)))
    }
}
