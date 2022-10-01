use crate::constants::{PLAYER_SUMMARIES_API, PLAYER_SUMMARIES_IDS_PER_REQUEST};
use crate::constants::{RETRIES, WAIT_DURATION};
use crate::enums::{CommunityVisibilityState, PersonaState};
use crate::parse_response::ParseResponse;
use crate::request_helper::send_request_with_reties;
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

use std::collections::HashMap;
use std::str::FromStr;

use chrono::TimeZone;
use chrono::{DateTime, Local, Utc};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlayerSummaryError {
    /// This API can only handle up to [`PLAYER_SUMMARIES_IDS_PER_REQUEST`] ids per request
    #[error("too many ids passed for request")]
    TooManyIds,

    /// For efficiency reasons the passed [SteamId] must be unique
    #[error("ids must be unique")]
    NonUniqueIds(SteamId),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The response contained an invalid [`CommunityVisibilityState`]
    #[error("invalid community visibility state: `{0}`")]
    InvalidCommunityVisibilityState(i32),

    /// The primary-clan-id was not parseable as u64
    #[error("invalid primary-clan-id: `{0}`")]
    InvalidPrimaryClanId(String),

    /// The response contained an invalid [`PersonaState`]
    #[error("invalid persona-state: `{0}`")]
    InvalidPersonaState(i32),

    /// The response contained an invalid [SteamId]
    #[error("invalid steam-id: `{0}`")]
    InvalidSteamId(String),
}
pub type Result<T> = std::result::Result<T, PlayerSummaryError>;

#[derive(Deserialize, Debug, Default, Clone)]
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

#[derive(Deserialize, Debug)]
struct ResponseInner {
    players: Vec<ResponseInnerElement>,
}

#[derive(Deserialize, Debug)]
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

/// If a given [SteamId] does not exist anymore,
/// its corresponding entry will be `None`
pub type SummaryMap = HashMap<SteamId, Option<PlayerSummary>>;

impl ParseResponse<ResponseInnerElement> for PlayerSummary {
    type Error = PlayerSummaryError;
    fn parse_response(value: ResponseInnerElement) -> Result<Self> {
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

impl std::fmt::Display for PlayerSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("PlayerSummary");
        dbg.field("SteamID", &self.steam_id);
        if let Some(time) = self.time_created {
            dbg.field("Created", &time.format("%Y/%m/%d %H:%M:%S").to_string());
        }
        dbg.field("Name", &self.persona_name);
        dbg.field("Vis", &self.community_visibility_state);
        if let Some(country_code) = &self.local_country_code {
            dbg.field("CC", &country_code);
        }
        dbg.field("PersState", &self.persona_state);
        if let Some(t) = &self.last_logoff {
            dbg.field("LastLogoff", &t.format("%Y/%m/%d %H:%M:%S").to_string());
        }
        dbg.finish()
    }
}

/// Get the summaries of the profiles with the given [SteamId]
///
/// Uses [`PLAYER_SUMMARIES_API`]
pub async fn get_player_summaries(
    client: &reqwest::Client,
    api_key: &str,
    steam_id_chunk: &[SteamId],
) -> Result<SummaryMap> {
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
    let resp = send_request_with_reties::<Response>(
        client,
        PLAYER_SUMMARIES_API,
        &query,
        true,
        true,
        RETRIES,
        WAIT_DURATION,
    )
    .await?;

    for elem in resp.response.players.into_iter() {
        let sum = PlayerSummary::parse_response(elem)?;
        let _ = map.insert(sum.steam_id, Some(sum));
    }

    Ok(map)
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
            for (id, summary) in res.unwrap().iter() {
                if let Some(summary) = summary {
                    println!("{}", summary);
                } else {
                    println!("{} missing", id);
                }
            }
        }
    }
}
