use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::{PLAYER_SUMMARIES_API, PLAYER_SUMMARIES_IDS_PER_REQUEST};
use crate::model::{
    CommunityVisibilityState, PersonaState, ProfileState, SteamIdQueryExt, SteamIdStr, SteamTime,
};
use crate::SteamId;

#[derive(Error, Debug)]
pub enum PlayerSummaryError {
    /// This API can only handle up to [`PLAYER_SUMMARIES_IDS_PER_REQUEST`] ids per request
    #[error("too many ids passed for request")]
    TooManyIds,

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
type Result<T> = std::result::Result<T, PlayerSummaryError>;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerSummary {
    #[serde(rename(deserialize = "steamid"))]
    steam_id: SteamIdStr,
    #[serde(rename(deserialize = "communityvisibilitystate"))]
    community_visibility_state: CommunityVisibilityState,
    #[serde(rename(deserialize = "profilestate"))]
    profile_state: ProfileState,
    #[serde(rename(deserialize = "personaname"))]
    persona_name: String,
    #[serde(rename(deserialize = "profileurl"))]
    profile_url: String,
    #[serde(rename(deserialize = "avatar"))]
    avatar: String,
    #[serde(rename(deserialize = "avatarmedium"))]
    avatar_medium: String,
    #[serde(rename(deserialize = "avatarfull"))]
    avatar_full: String,
    #[serde(rename(deserialize = "avatarhash"))]
    avatar_hash: String,
    #[serde(rename(deserialize = "lastlogoff"))]
    last_logoff: Option<SteamTime>,
    #[serde(rename(deserialize = "personastate"))]
    persona_state: PersonaState,
    #[serde(rename(deserialize = "realname"))]
    real_name: Option<String>,
    #[serde(rename(deserialize = "primaryclanid"))]
    primary_clan_id: Option<String>,
    #[serde(rename(deserialize = "timecreated"))]
    time_created: Option<SteamTime>,
    #[serde(rename(deserialize = "personastateflags"))]
    persona_state_flags: Option<u64>,
    #[serde(rename(deserialize = "loccountrycode"))]
    local_country_code: Option<String>,
}

#[derive(Debug)]
pub struct PlayerSummaries {
    inner: HashMap<SteamId, PlayerSummary>,
}

impl PlayerSummaries {
    pub fn into_inner(self) -> HashMap<SteamId, PlayerSummary> {
        self.inner
    }
}

impl Deref for PlayerSummaries {
    type Target = HashMap<SteamId, PlayerSummary>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Deserialize, Debug)]
struct ResponseInner {
    players: Vec<PlayerSummary>,
}

#[derive(Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

impl From<Response> for PlayerSummaries {
    fn from(value: Response) -> Self {
        let summaries = value.response.players;

        let map = summaries
            .into_iter()
            .map(|summary| (summary.steam_id.into(), summary))
            .collect();

        PlayerSummaries { inner: map }
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

        Ok(resp.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{PlayerSummaries, Response};

    #[test]
    fn parses() {
        let json: Response = load_test_json!("player_summaries.json");
        let summaries: PlayerSummaries = json.into();
        println!("{:?}", summaries);
    }
}
