use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::{PLAYER_BANS_API, PLAYER_BANS_IDS_PER_REQUEST};
use crate::model::{EconomyBan, SteamId, SteamIdQueryExt};

#[derive(Debug, Error)]
pub enum PlayerBanError {
    #[error("too many ids passed for request")]
    TooManyIds,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
type Result<T> = std::result::Result<T, PlayerBanError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlayerBan {
    #[serde(rename(deserialize = "SteamId"))]
    pub steam_id: SteamId,
    #[serde(rename(deserialize = "CommunityBanned"))]
    pub community_banned: bool,
    #[serde(rename(deserialize = "VACBanned"))]
    pub vac_banned: bool,
    #[serde(rename(deserialize = "NumberOfVACBans"))]
    pub number_of_vac_bans: i32,
    #[serde(rename(deserialize = "DaysSinceLastBan"))]
    pub days_since_last_ban: i32,
    #[serde(rename(deserialize = "NumberOfGameBans"))]
    pub number_of_game_bans: i32,
    #[serde(rename(deserialize = "EconomyBan"))]
    pub economy_ban: EconomyBan,
}

#[derive(Debug, Clone)]
pub struct PlayerBans {
    inner: HashMap<SteamId, PlayerBan>,
}

impl PlayerBans {
    pub fn into_inner(self) -> HashMap<SteamId, PlayerBan> {
        self.inner
    }
}

impl Deref for PlayerBans {
    type Target = HashMap<SteamId, PlayerBan>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    players: Vec<PlayerBan>,
}

impl From<Response> for PlayerBans {
    fn from(value: Response) -> Self {
        let bans = value.players;
        let map = bans.into_iter().map(|ban| (ban.steam_id, ban)).collect();
        PlayerBans { inner: map }
    }
}

impl Client {
    /// Get the bans of the profiles with the given [`SteamId`]
    ///
    /// Uses [`PLAYER_BANS_API`]
    pub async fn get_player_bans(&self, steam_id_chunk: Cow<'_, [SteamId]>) -> Result<PlayerBans> {
        // deduplicated ids
        let mut steam_ids = steam_id_chunk.into_owned();
        steam_ids.sort_unstable();
        steam_ids.dedup();

        // check length
        if steam_ids.len() > PLAYER_BANS_IDS_PER_REQUEST {
            return Err(PlayerBanError::TooManyIds);
        }

        // build query string
        let ids = steam_ids.iter().to_steam_id_string(",");
        let query = [("key", self.api_key()), ("steamids", &ids)];

        // make request
        let resp = self.get_json::<Response>(PLAYER_BANS_API, &query).await?;

        // conversion
        Ok(resp.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayerBans, Response};

    #[test]
    fn parses() {
        let resp: Response = load_test_json!("player_bans.json");
        let bans: PlayerBans = resp.into();
        println!("{:#?}", bans);
    }
}
