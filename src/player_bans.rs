use crate::constants::PLAYER_BANS_API;
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

use std::convert::TryFrom;
use std::str::FromStr;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerBanError {
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
    pub bans: Vec<Result<PlayerBan>>,
}

impl<'a> From<(Response, &'a [SteamId])> for PlayerBans<'a> {
    fn from((response, query): (Response, &'a [SteamId])) -> Self {
        let iter = response.players.into_iter();
        Self {
            query: query,
            bans: iter.map(PlayerBan::try_from).collect(),
        }
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
) -> reqwest::Result<PlayerBans<'a>> {
    let ids = steam_id_chunk.iter().to_steam_id_string(",");
    let query = [("key", api_key), ("steamids", &ids)];
    let req = client.get(PLAYER_BANS_API).query(&query);
    let resp = crate::request_helper::send_request::<Response>(req, true, true).await?;
    Ok(PlayerBans::from((resp, steam_id_chunk)))
}

#[cfg(test)]
mod tests {
    use super::get_player_bans;
    use crate::steam_id::SteamId;
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let ids: [&[SteamId]; _] = [
            &[76561199123543583.into(), 76561198196615742.into()],
            &[76561199159691884.into(), 76561198230177976.into()],
            &[76561198414415313.into(), 76561197992321696.into()],
            &[76561197992321696.into(), 76561198350302388.into()],
            &[76561198159967543.into(), 76561197981967565.into()],
        ];

        dotenv::dotenv().unwrap();
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(ids.iter().map(|&ids| ids))
            .map(|ids| get_player_bans(&client, &api_key, ids))
            .buffered(10);

        while let Some(res) = stream.next().await {
            let inner = match res {
                Err(err) => {
                    println!("Request failed: {}", err);
                    continue;
                }
                Ok(val) => val,
            };
            for bans in inner.bans {
                match bans {
                    Ok(ban) => println!("{}", ban),
                    Err(err) => println!("{}", err),
                };
            }
        }
    }
}
