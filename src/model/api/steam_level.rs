use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::PLAYER_STEAM_LEVEL_API;
use crate::model::SteamId;

#[derive(Error, Debug)]
pub enum SteamLevelError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
type Result<T> = std::result::Result<T, SteamLevelError>;

#[derive(Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SteamLevel(Option<u64>);

impl SteamLevel {
    pub const fn lvl(self) -> Option<u64> {
        self.0
    }
    pub fn lvl_or(self, default: u64) -> u64 {
        self.0.unwrap_or(default)
    }
    pub const fn into_inner(self) -> Option<u64> {
        self.0
    }
}

#[derive(Deserialize, Debug)]
struct ResponseInner {
    player_level: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

impl From<Response> for SteamLevel {
    fn from(value: Response) -> Self {
        SteamLevel(value.response.player_level)
    }
}

impl Client {
    /// Get the Steam level of the given [`SteamId`]
    ///
    /// Uses [`PLAYER_STEAM_LEVEL_API`]
    pub async fn get_player_steam_level(&self, id: SteamId) -> Result<SteamLevel> {
        let query = [("key", self.api_key()), ("steamid", &id.to_string())];

        let json = self
            .get_json::<Response>(PLAYER_STEAM_LEVEL_API, &query)
            .await?;

        Ok(json.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, SteamLevel};

    #[test]
    fn parses() {
        let json: Response = load_test_json!("steam_level.json");
        let lvl: SteamLevel = json.into();
        assert_eq!(lvl, SteamLevel(Some(135)));
    }

    #[test]
    fn parses_deleted() {
        let json: Response = load_test_json!("steam_level_deleted.json");
        let lvl: SteamLevel = json.into();
        assert_eq!(lvl, SteamLevel(None));
    }
}
