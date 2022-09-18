use crate::constants::{PLAYER_BANS_API, PLAYER_BANS_IDS_PER_REQUEST};
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

use std::collections::HashMap;
use std::convert::TryFrom;
use std::str::FromStr;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerBanError {
    #[error("too many ids passed for request")]
    TooManyIds,
    #[error("ids must be unique")]
    NonUniqueIds(SteamId),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("invalid steam-id: {0}")]
    InvalidSteamId(String),
}
pub type Result<T> = std::result::Result<T, PlayerBanError>;

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
    pub economy_ban: String,
}
type BanMap = HashMap<SteamId, Option<PlayerBan>>;

impl TryFrom<ResponseElement> for PlayerBan {
    type Error = PlayerBanError;
    fn try_from(value: ResponseElement) -> Result<Self> {
        let steam_id = SteamId::from_str(&value.steam_id)
            .map_err(|_| PlayerBanError::InvalidSteamId(value.steam_id))?;

        Ok(Self {
            steam_id: steam_id,
            community_banned: value.community_banned,
            vac_banned: value.vac_banned,
            number_of_vac_bans: value.number_of_vac_bans,
            days_since_last_ban: value.days_since_last_ban,
            number_of_game_bans: value.number_of_game_bans,
            economy_ban: value.economy_ban,
        })
    }
}

pub struct PlayerBans<'a> {
    pub query: &'a [SteamId],
    pub bans: BanMap,
}

impl<'a> TryFrom<(Response, &'a [SteamId], BanMap)> for PlayerBans<'a> {
    type Error = PlayerBanError;
    fn try_from((response, query, mut map): (Response, &'a [SteamId], BanMap)) -> Result<Self> {
        for elem in response.players.into_iter() {
            let ban = PlayerBan::try_from(elem)?;
            let _ = map.insert(ban.steam_id, Some(ban));
        }
        Ok(Self {
            query: query,
            bans: map,
        })
    }
}

impl std::fmt::Display for PlayerBan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: ", self.steam_id)?;
        let mut any_ban = false;
        if self.number_of_game_bans > 0 {
            any_ban = true;
            "Vacced".fmt(f)?;
        }
        if self.number_of_game_bans > 0 {
            if any_ban {
                ", ".fmt(f)?;
            }
            any_ban = true;
            "Gamed".fmt(f)?;
        }
        if self.community_banned {
            if any_ban {
                ", ".fmt(f)?;
            }
            any_ban = true;
            "Comunityd".fmt(f)?;
        }
        if !any_ban {
            "Clean".fmt(f)?;
        }
        Ok(())
    }
}

pub async fn get_player_bans<'a>(
    client: &'a Client,
    api_key: &'a str,
    steam_id_chunk: &'a [SteamId],
) -> Result<PlayerBans<'a>> {
    if steam_id_chunk.len() > PLAYER_BANS_IDS_PER_REQUEST {
        return Err(PlayerBanError::TooManyIds);
    }

    let mut map = BanMap::with_capacity(steam_id_chunk.len());
    for &id in steam_id_chunk {
        if let Some(_) = map.insert(id, None) {
            return Err(PlayerBanError::NonUniqueIds(id));
        }
    }

    let ids = steam_id_chunk.iter().to_steam_id_string(",");
    let query = [("key", api_key), ("steamids", &ids)];
    let req = client.get(PLAYER_BANS_API).query(&query);

    let resp = crate::request_helper::send_request::<Response>(req, true, true).await?;

    PlayerBans::try_from((resp, steam_id_chunk, map))
}

#[cfg(test)]
mod tests {
    use super::get_player_bans;
    use crate::steam_id::SteamId;
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
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(ids.iter().map(|&ids| ids))
            .map(|ids| get_player_bans(&client, &api_key, ids))
            .buffer_unordered(2);

        while let Some(res) = stream.next().await {
            for (id, ban) in res.unwrap().bans.iter() {
                if let Some(ban) = ban {
                    println!("{}", ban);
                } else {
                    println!("{} missing", id);
                }
            }
        }
    }
}
