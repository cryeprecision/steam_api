use std::convert::Infallible;

use serde::Deserialize;
use thiserror::Error;

use crate::client::Client;
use crate::constants::PLAYER_STEAM_LEVEL_API;
use crate::parse_response::ParseJsonResponse;
use crate::steam_id::SteamId;

#[derive(Error, Debug)]
pub enum SteamLevelError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
type Result<T> = std::result::Result<T, SteamLevelError>;

#[derive(Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

#[derive(Deserialize, Debug)]
struct ResponseInner {
    player_level: Option<u64>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SteamLevel(u64);

impl SteamLevel {
    pub fn lvl(self) -> u64 {
        self.0
    }
}

impl Client {
    /// Get the Steam level of the given [`SteamId`]
    ///
    /// Uses [`PLAYER_STEAM_LEVEL_API`]
    pub async fn get_player_steam_level(&self, id: SteamId) -> Result<Option<SteamLevel>> {
        let query = [("key", self.api_key()), ("steamid", &id.to_string())];
        let json = self
            .get_json::<Response>(PLAYER_STEAM_LEVEL_API, &query)
            .await?;
        Ok(json.parse_steam_json().unwrap())
    }
}

impl ParseJsonResponse for Response {
    type Error = Infallible;
    type Output = Option<SteamLevel>;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        Ok(self.response.player_level.map(SteamLevel))
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, SteamLevel};
    use crate::parse_response::ParseJsonResponse;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("steam_level.json");
        let lvl = json.parse_steam_json().unwrap();
        assert_eq!(lvl, Some(SteamLevel(135)));
    }

    #[test]
    fn parses_deleted() {
        let json: Response = load_test_json!("steam_level_deleted.json");
        let lvl = json.parse_steam_json().unwrap();
        assert_eq!(lvl, None);
    }
}
