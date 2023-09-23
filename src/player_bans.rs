use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

use crate::client::Client;
use crate::constants::{PLAYER_BANS_API, PLAYER_BANS_IDS_PER_REQUEST};
use crate::enums::EconomyBan;
use crate::parse_response::ParseJsonResponse;
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

#[derive(Deserialize, Debug)]
struct ResponseElement {
    #[serde(rename = "SteamId")]
    steam_id: String,
    #[serde(rename = "CommunityBanned")]
    community_banned: bool,
    #[serde(rename = "VACBanned")]
    vac_banned: bool,
    #[serde(rename = "NumberOfVACBans")]
    number_of_vac_bans: i32,
    #[serde(rename = "DaysSinceLastBan")]
    days_since_last_ban: i32,
    #[serde(rename = "NumberOfGameBans")]
    number_of_game_bans: i32,
    #[serde(rename = "EconomyBan")]
    economy_ban: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    players: Vec<ResponseElement>,
}

#[derive(Debug)]
pub struct PlayerBan {
    pub steam_id: SteamId,
    pub community_banned: bool,
    pub vac_banned: bool,
    pub number_of_vac_bans: i32,
    pub days_since_last_ban: i32,
    pub number_of_game_bans: i32,
    pub economy_ban: EconomyBan,
}

impl ParseJsonResponse for ResponseElement {
    type Output = PlayerBan;
    type Error = PlayerBanError;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        let steam_id = SteamId::from_str(&self.steam_id)
            .map_err(|_| PlayerBanError::InvalidSteamId(self.steam_id))?;

        todo!("implement serde deserialize from steam id first");

        // let economy_ban: EconomyBan = match self.economy_ban.as_str().try_into() {
        //     Ok(v) => v,
        //     Err(_) => return Err(PlayerBanError::InvalidEconomyBan(self.economy_ban)),
        // };

        // Ok(PlayerBan {
        //     steam_id,
        //     community_banned: self.community_banned,
        //     vac_banned: self.vac_banned,
        //     number_of_vac_bans: self.number_of_vac_bans,
        //     days_since_last_ban: self.days_since_last_ban,
        //     number_of_game_bans: self.number_of_game_bans,
        //     economy_ban: economy_ban,
        // })
    }
}

impl ParseJsonResponse for Response {
    type Output = PlayerBans;
    type Error = PlayerBanError;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        let mut map = HashMap::with_capacity(PLAYER_BANS_IDS_PER_REQUEST);

        for elem in self.players {
            let ban = elem.parse_steam_json()?;
            map.insert(ban.steam_id, ban);
        }

        Ok(PlayerBans { inner: map })
    }
}

#[derive(Debug)]
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

impl Client {
    /// Get the bans of the profiles with the given [`SteamId`]
    ///
    /// Uses [`PLAYER_BANS_API`]
    pub async fn get_player_bans(&self, steam_id_chunk: Cow<'_, [SteamId]>) -> Result<PlayerBans> {
        let mut steam_ids = steam_id_chunk.into_owned();
        steam_ids.sort_unstable();
        steam_ids.dedup();

        if steam_ids.len() > PLAYER_BANS_IDS_PER_REQUEST {
            return Err(PlayerBanError::TooManyIds);
        }

        let ids = steam_ids.iter().to_steam_id_string(",");
        let query = [("key", self.api_key()), ("steamids", &ids)];

        let resp = self.get_json::<Response>(PLAYER_BANS_API, &query).await?;
        resp.parse_steam_json()
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::parse_response::ParseJsonResponse;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("player_bans.json");
        let bans = json.parse_steam_json().unwrap();
        println!("{:#?}", bans);
    }
}
