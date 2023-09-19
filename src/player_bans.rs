use crate::client::Client;
use crate::constants::{PLAYER_BANS_API, PLAYER_BANS_IDS_PER_REQUEST};
use crate::enums::EconomyBan;
use crate::parse_response::ParseResponse;
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

use std::collections::HashMap;
use std::str::FromStr;

use serde::Deserialize;
use thiserror::Error;

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

/// If a given [`SteamId`] does not exist anymore,
/// its corresponding entry will be `None`
pub type BanMap = HashMap<SteamId, Option<PlayerBan>>;

impl ParseResponse<ResponseElement> for PlayerBan {
    type Error = PlayerBanError;
    fn parse_response(value: ResponseElement) -> Result<Self> {
        let steam_id = SteamId::from_str(&value.steam_id)
            .map_err(|_| PlayerBanError::InvalidSteamId(value.steam_id))?;

        Ok(Self {
            steam_id,
            community_banned: value.community_banned,
            vac_banned: value.vac_banned,
            number_of_vac_bans: value.number_of_vac_bans,
            days_since_last_ban: value.days_since_last_ban,
            number_of_game_bans: value.number_of_game_bans,
            economy_ban: EconomyBan::from(value.economy_ban),
        })
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
    pub async fn get_player_bans(&self, steam_id_chunk: &[SteamId]) -> Result<BanMap> {
        if steam_id_chunk.len() > PLAYER_BANS_IDS_PER_REQUEST {
            return Err(PlayerBanError::TooManyIds);
        }

        let mut map = BanMap::with_capacity(steam_id_chunk.len());
        for &id in steam_id_chunk {
            if map.insert(id, None).is_some() {
                return Err(PlayerBanError::NonUniqueIds(id));
            }
        }

        let ids = steam_id_chunk.iter().to_steam_id_string(",");
        let query = [("key", self.api_key()), ("steamids", &ids)];

        let resp = self.get_json::<Response>(PLAYER_BANS_API, &query).await?;

        for elem in resp.players {
            let ban = PlayerBan::parse_response(elem)?;
            map.insert(ban.steam_id, Some(ban));
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use crate::{steam_id::SteamId, ClientOptions};
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let ids: [&[SteamId]; _] = [
            &[76561199123543583.into()],
            &[76561198196615742.into()],
            &[76561199159691884.into()],
            &[76561198230177976.into()],
            &[76561198414415313.into()],
            &[76561197992321696.into()],
            &[76561198350302388.into()],
            &[76561198159967543.into()],
            &[76561197981967565.into()],
            &[76561199049236696.into()],
            &[76561199063760869.into()],
            &[76561197961074129.into()],
            &[76561198292293761.into()],
            &[76561198145832850.into()],
            &[76561198151659207.into()],
            &[76561198405122517.into()],
        ];

        dotenv::dotenv().unwrap();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let client = ClientOptions::new().api_key(api_key).build().await;

        let mut stream = futures::stream::iter(ids.iter().cloned())
            .map(|ids| client.get_player_bans(ids))
            .buffer_unordered(2);

        while let Some(res) = stream.next().await {
            for (id, ban) in res.unwrap().iter() {
                if let Some(ban) = ban {
                    println!("{}", ban);
                } else {
                    println!("{} missing", id);
                }
            }
        }
    }
}
