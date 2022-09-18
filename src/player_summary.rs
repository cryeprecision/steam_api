use crate::constants::{PLAYER_SUMMARIES_API, PLAYER_SUMMARIES_IDS_PER_REQUEST};
use crate::enums::{CommunityVisibilityState, PersonaState};
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::str::FromStr;

use chrono::TimeZone;
use chrono::{DateTime, Local, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerSummaryError {
    #[error("too many ids passed for request")]
    TooManyIds,
    #[error("ids must be unique")]
    NonUniqueIds(SteamId),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("invalid community visibility state: {0}")]
    InvalidCommunityVisibilityState(i32),
    #[error("invalid primary-clan-id: {0}")]
    InvalidPrimaryClanId(String),
    #[error("invalid persona-state: {0}")]
    InvalidPersonaState(i32),
    #[error("invalid steam-id: {0}")]
    InvalidSteamId(String),
}
type Result<T> = std::result::Result<T, PlayerSummaryError>;

#[derive(serde::Deserialize, Debug, Default, Clone)]
struct ResponseInnerElement {
    #[serde(rename = "steamid")]
    steam_id: String,
    #[serde(rename = "communityvisibilitystate")]
    community_visibility_state: i32,
    #[serde(rename = "profilestate")]
    profile_state: Option<i32>,
    #[serde(rename = "personaname")]
    persona_name: String,
    #[serde(rename = "profileurl")]
    profile_url: String,
    #[serde(rename = "avatar")]
    avatar: String,
    #[serde(rename = "avatarmedium")]
    avatar_medium: String,
    #[serde(rename = "avatarfull")]
    avatar_full: String,
    #[serde(rename = "avatarhash")]
    avatar_hash: String,
    #[serde(rename = "lastlogoff")]
    last_logoff: Option<u64>,
    #[serde(rename = "personastate")]
    persona_state: i32,
    #[serde(rename = "realname")]
    real_name: Option<String>,
    #[serde(rename = "primaryclanid")]
    primary_clan_id: Option<String>,
    #[serde(rename = "timecreated")]
    time_created: Option<u64>,
    #[serde(rename = "personastateflags")]
    persona_state_flags: Option<u32>,
    #[serde(rename = "loccountrycode")]
    local_country_code: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct ResponseInner {
    players: Vec<ResponseInnerElement>,
}

#[derive(serde::Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

#[derive(Debug)]
pub struct PlayerSummary {
    pub steam_id: SteamId,
    pub community_visibility_state: CommunityVisibilityState,
    pub profile_configured: bool,
    pub persona_name: String,
    pub profile_url: String,
    pub avatar: String,
    pub avatar_medium: String,
    pub avatar_full: String,
    pub avatar_hash: String,
    pub last_logoff: Option<DateTime<Local>>,
    pub persona_state: PersonaState,
    pub real_name: Option<String>,
    pub primary_clan_id: Option<u64>,
    pub time_created: Option<DateTime<Local>>,
    pub persona_state_flags: Option<u32>,
    pub local_country_code: Option<String>,
}
type SummaryMap = HashMap<SteamId, Option<PlayerSummary>>;

impl TryFrom<ResponseInnerElement> for PlayerSummary {
    type Error = PlayerSummaryError;
    fn try_from(value: ResponseInnerElement) -> Result<Self> {
        let last_logoff = value
            .last_logoff
            .map(|unix| Utc.timestamp(unix as i64, 0))
            .map(DateTime::<Local>::from);
        let time_created = value
            .time_created
            .map(|unix| Utc.timestamp(unix as i64, 0))
            .map(DateTime::<Local>::from);

        let vis_state = CommunityVisibilityState::new(value.community_visibility_state).ok_or(
            PlayerSummaryError::InvalidCommunityVisibilityState(value.community_visibility_state),
        )?;

        let persona_state = PersonaState::new(value.persona_state)
            .ok_or(PlayerSummaryError::InvalidPersonaState(value.persona_state))?;

        let clan_id = match value.primary_clan_id {
            Some(clan_id) => Some(
                u64::from_str(&clan_id)
                    .map_err(|_| PlayerSummaryError::InvalidPrimaryClanId(clan_id))?,
            ),
            None => None,
        };

        let steam_id = SteamId::from_str(&value.steam_id)
            .map_err(|_| PlayerSummaryError::InvalidSteamId(value.steam_id))?;

        Ok(Self {
            steam_id: steam_id,
            community_visibility_state: vis_state,
            profile_configured: value.profile_state.is_some(),
            persona_name: value.persona_name,
            profile_url: value.profile_url,
            avatar: value.avatar,
            avatar_medium: value.avatar_medium,
            avatar_full: value.avatar_full,
            avatar_hash: value.avatar_hash,
            last_logoff: last_logoff,
            persona_state: persona_state,
            real_name: value.real_name,
            primary_clan_id: clan_id,
            time_created: time_created,
            persona_state_flags: value.persona_state_flags,
            local_country_code: value.local_country_code,
        })
    }
}

#[derive(Debug)]
pub struct PlayerSummaries<'a> {
    pub query: &'a [SteamId],
    pub summaries: SummaryMap,
}

impl<'a> TryFrom<(Response, &'a [SteamId], SummaryMap)> for PlayerSummaries<'a> {
    type Error = PlayerSummaryError;
    fn try_from((response, query, mut map): (Response, &'a [SteamId], SummaryMap)) -> Result<Self> {
        for inner_elem in response.response.players.into_iter() {
            let summary = PlayerSummary::try_from(inner_elem)?;
            let _ = map.insert(summary.steam_id, Some(summary));
        }
        Ok(Self {
            query: query,
            summaries: map,
        })
    }
}

impl std::fmt::Display for PlayerSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.steam_id)?;
        if let Some(time) = self.time_created {
            write!(f, " ({})", time.format("%Y/%m/%d %H:%M:%S"))?
        }
        write!(f, ", {}", self.persona_name)?;
        if let Some(name) = &self.real_name {
            write!(f, " ({})", name)?;
        }
        if let Some(country_code) = &self.local_country_code {
            write!(f, ", {}", country_code)?;
        }
        write!(f, ", {:?}", self.persona_state)?;
        if let Some(t) = &self.last_logoff {
            write!(f, ", {}", t.format("%Y/%m/%d %H:%M:%S"))?
        }
        Ok(())
    }
}

pub async fn get_player_summaries<'a>(
    client: &'a reqwest::Client,
    api_key: &'a str,
    steam_id_chunk: &'a [SteamId],
) -> Result<PlayerSummaries<'a>> {
    if steam_id_chunk.len() > PLAYER_SUMMARIES_IDS_PER_REQUEST {
        return Err(PlayerSummaryError::TooManyIds);
    }

    let mut map = SummaryMap::with_capacity(steam_id_chunk.len());
    for &id in steam_id_chunk {
        if let Some(_) = map.insert(id, None) {
            return Err(PlayerSummaryError::NonUniqueIds(id));
        }
    }

    let ids = steam_id_chunk.iter().to_steam_id_string(",");
    let query = [("key", api_key), ("steamids", &ids)];
    let req = client.get(PLAYER_SUMMARIES_API).query(&query);
    let resp = crate::request_helper::send_request::<Response>(req, true, true).await?;

    PlayerSummaries::try_from((resp, steam_id_chunk, map))
}

#[cfg(test)]
mod tests {
    use super::get_player_summaries;
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

        let mut stream = futures::stream::iter(ids.iter())
            .map(|&chunk| get_player_summaries(&client, &api_key, &chunk))
            .buffer_unordered(2);

        while let Some(res) = stream.next().await {
            for (id, summary) in res.unwrap().summaries.iter() {
                if let Some(summary) = summary {
                    println!("{}", summary);
                } else {
                    println!("{} missing", id);
                }
            }
        }
    }
}
