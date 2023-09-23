use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use chrono::{DateTime, Local, TimeZone, Utc};
use serde::Deserialize;
use thiserror::Error;

use crate::client::Client;
use crate::constants::{PLAYER_SUMMARIES_API, PLAYER_SUMMARIES_IDS_PER_REQUEST};
use crate::enums::{CommunityVisibilityState, PersonaState};
use crate::parse_response::ParseJsonResponse;
use crate::steam_id::SteamId;
use crate::steam_id_ext::SteamIdExt;

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
    InvalidCommunityVisibilityState(i64),

    /// The primary-clan-id was not parseable as u64
    #[error("invalid primary-clan-id: `{0}`")]
    InvalidPrimaryClanId(String),

    /// The response contained an invalid [`PersonaState`]
    #[error("invalid persona-state: `{0}`")]
    InvalidPersonaState(i64),

    /// The response contained an invalid [SteamId]
    #[error("invalid steam-id: `{0}`")]
    InvalidSteamId(String),

    #[error("invalid timestamp: `{0}`")]
    InvalidTimestamp(i64),
}
type Result<T> = std::result::Result<T, PlayerSummaryError>;

#[derive(Deserialize, Debug, Default, Clone)]
struct ResponseInnerElement {
    #[serde(rename = "steamid")]
    steam_id: String,
    #[serde(rename = "communityvisibilitystate")]
    community_visibility_state: i64,
    #[serde(rename = "profilestate")]
    profile_state: Option<i64>,
    #[serde(rename = "personaname")]
    persona_name: String,
    #[serde(rename = "profileurl")]
    profile_url: String,
    avatar: String,
    #[serde(rename = "avatarmedium")]
    avatar_medium: String,
    #[serde(rename = "avatarfull")]
    avatar_full: String,
    #[serde(rename = "avatarhash")]
    avatar_hash: String,
    #[serde(rename = "lastlogoff")]
    last_logoff: Option<i64>,
    #[serde(rename = "personastate")]
    persona_state: i64,
    #[serde(rename = "realname")]
    real_name: Option<String>,
    #[serde(rename = "primaryclanid")]
    primary_clan_id: Option<String>,
    #[serde(rename = "timecreated")]
    time_created: Option<i64>,
    #[serde(rename = "personastateflags")]
    persona_state_flags: Option<u64>,
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

/// TODO: Make this `HashMap<SteamId, Option<PlayerSummary>>`
/// to distinguish between profiles that didn't yield a response
/// and profiles that weren't requested.
#[derive(Debug)]
pub struct PlayerSummaries {
    inner: HashMap<SteamId, PlayerSummary>,
}

impl PlayerSummaries {
    pub fn into_innter(self) -> HashMap<SteamId, PlayerSummary> {
        self.inner
    }
}

impl Deref for PlayerSummaries {
    type Target = HashMap<SteamId, PlayerSummary>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
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
    pub persona_state_flags: Option<u64>,
    pub local_country_code: Option<String>,
}

impl ParseJsonResponse for ResponseInnerElement {
    type Error = PlayerSummaryError;
    type Output = PlayerSummary;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        let last_logoff = match self.last_logoff {
            Some(unix) => Some(
                Utc.timestamp_opt(unix, 0)
                    .single()
                    .ok_or(PlayerSummaryError::InvalidTimestamp(unix))?
                    .with_timezone(&Local),
            ),
            None => None,
        };
        let time_created = match self.time_created {
            Some(unix) => Some(
                Utc.timestamp_opt(unix, 0)
                    .single()
                    .ok_or(PlayerSummaryError::InvalidTimestamp(unix))?
                    .with_timezone(&Local),
            ),
            None => None,
        };

        let vis_state: CommunityVisibilityState =
            self.community_visibility_state.try_into().map_err(|_| {
                PlayerSummaryError::InvalidCommunityVisibilityState(self.community_visibility_state)
            })?;

        let persona_state: PersonaState = self
            .persona_state
            .try_into()
            .map_err(|_| (PlayerSummaryError::InvalidPersonaState(self.persona_state)))?;

        let clan_id = match self.primary_clan_id {
            Some(clan_id) => Some(
                u64::from_str(&clan_id)
                    .map_err(|_| PlayerSummaryError::InvalidPrimaryClanId(clan_id))?,
            ),
            None => None,
        };

        let steam_id = SteamId::from_str(&self.steam_id)
            .map_err(|_| PlayerSummaryError::InvalidSteamId(self.steam_id))?;

        Ok(PlayerSummary {
            steam_id,
            community_visibility_state: vis_state,
            profile_configured: self.profile_state.is_some(),
            persona_name: self.persona_name,
            profile_url: self.profile_url,
            avatar: self.avatar,
            avatar_medium: self.avatar_medium,
            avatar_full: self.avatar_full,
            avatar_hash: self.avatar_hash,
            last_logoff,
            persona_state,
            real_name: self.real_name,
            primary_clan_id: clan_id,
            time_created,
            persona_state_flags: self.persona_state_flags,
            local_country_code: self.local_country_code,
        })
    }
}

impl ParseJsonResponse for Response {
    type Error = PlayerSummaryError;
    type Output = PlayerSummaries;

    fn parse_steam_json(self) -> std::result::Result<Self::Output, Self::Error> {
        let mut map = HashMap::with_capacity(PLAYER_SUMMARIES_IDS_PER_REQUEST);

        for elem in self.response.players {
            let sum = elem.parse_steam_json()?;
            map.insert(sum.steam_id, sum);
        }

        Ok(PlayerSummaries { inner: map })
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

impl Client {
    /// Get the summaries of the profiles with the given [`SteamId`]
    ///
    /// Uses [`PLAYER_SUMMARIES_API`]
    pub async fn get_player_summaries(
        &self,
        steam_id_chunk: Cow<'_, [SteamId]>,
    ) -> Result<PlayerSummaries> {
        let mut steam_ids = steam_id_chunk.into_owned();
        steam_ids.sort_unstable();
        steam_ids.dedup();

        if steam_ids.len() > PLAYER_SUMMARIES_IDS_PER_REQUEST {
            return Err(PlayerSummaryError::TooManyIds);
        }

        let ids = steam_ids.iter().to_steam_id_string(",");
        let query = [("key", self.api_key()), ("steamids", &ids)];
        let resp = self
            .get_json::<Response>(PLAYER_SUMMARIES_API, &query)
            .await?;
        resp.parse_steam_json()
    }
}

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::parse_response::ParseJsonResponse;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("player_summaries.json");
        let summaries = json.parse_steam_json().unwrap();
        println!("{:#?}", summaries);
    }
}
