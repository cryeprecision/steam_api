use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::{PLAYER_BANS_API, PLAYER_BANS_IDS_PER_REQUEST};
use crate::enums::EconomyBan;
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

#[derive(Debug, Error)]
pub enum PlayerBanError {
    /// This API can only handle up to
    /// [`crate::constants::PLAYER_BANS_IDS_PER_REQUEST`] ids per request
    #[error("too many ids passed for request")]
    TooManyIds,

    /// For efficiency reasons the passed [SteamId]s must be unique
    #[error("ids must be unique")]
    NonUniqueIds(SteamId),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The response contained an invalid [SteamId]
    #[error("invalid steam-id: `{0}`")]
    InvalidSteamId(String),

    #[error("invalid economy ban value: `{0}`")]
    InvalidEconomyBan(String),
}
type Result<T> = std::result::Result<T, PlayerBanError>;

#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerBan {
    #[serde(rename(deserialize = "SteamId"))]
    steam_id: SteamId,
    #[serde(rename(deserialize = "CommunityBanned"))]
    community_banned: bool,
    #[serde(rename(deserialize = "VACBanned"))]
    vac_banned: bool,
    #[serde(rename(deserialize = "NumberOfVACBans"))]
    number_of_vac_bans: i32,
    #[serde(rename(deserialize = "DaysSinceLastBan"))]
    days_since_last_ban: i32,
    #[serde(rename(deserialize = "NumberOfGameBans"))]
    number_of_game_bans: i32,
    #[serde(rename(deserialize = "EconomyBan"))]
    economy_ban: EconomyBan,
}

#[derive(Deserialize, Debug)]
struct Response {
    players: Vec<PlayerBan>,
}

impl std::fmt::Display for PlayerBan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerBan")
            .field("SteamID", &self.steam_id)
            .field("VAC", &self.number_of_vac_bans)
            .field("GameBan", &self.number_of_game_bans)
            .field("CommunityBan", &self.community_banned)
            .field("LastBan", &self.days_since_last_ban)
            .field("Econ", &self.economy_ban)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct PlayerBans {
    inner: HashMap<SteamId, PlayerBan>,
}

impl From<Response> for PlayerBans {
    fn from(value: Response) -> Self {
        let bans = value.players;

        let map = bans.into_iter().map(|ban| (ban.steam_id, ban)).collect();

        PlayerBans { inner: map }
    }
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
