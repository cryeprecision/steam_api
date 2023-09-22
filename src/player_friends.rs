use std::str::FromStr;

use chrono::{DateTime, Local, TimeZone, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

use crate::client::Client;
use crate::constants::PLAYER_FRIENDS_API;
use crate::parse_response::ParseResponse;
use crate::steam_id::SteamId;

#[derive(Error, Debug)]
pub enum PlayerFriendsError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The result contained an invalid [SteamId]
    #[error("invalid steam-id: `{0}`")]
    InvalidSteamId(String),

    #[error("invalid timestamp: `{0}`")]
    InvalidTimestamp(i64),
}
type Result<T> = std::result::Result<T, PlayerFriendsError>;

#[derive(Deserialize, Debug)]
struct ResponseInnerElement {
    #[serde(rename = "steamid")]
    steam_id: String,
    #[serde(rename = "relationship")]
    _relationship: String,
    #[serde(rename = "friend_since")]
    friends_since: i64,
}

#[derive(Deserialize, Debug, Default)]
struct ResponseInner {
    friends: Vec<ResponseInnerElement>,
}

#[derive(Deserialize, Debug)]
struct Response {
    #[serde(rename = "friendslist")]
    friend_list: ResponseInner,
}

#[derive(Debug)]
pub struct Friend {
    pub steam_id: SteamId,
    pub friends_since: DateTime<Local>,
}

/// List of friends of a Steam-profile
type FriendList = Vec<Friend>;

impl ParseResponse<ResponseInnerElement> for Friend {
    type Error = PlayerFriendsError;
    fn parse_response(value: ResponseInnerElement) -> Result<Self> {
        let id = SteamId::from_str(&value.steam_id)
            .map_err(|_| PlayerFriendsError::InvalidSteamId(value.steam_id))?;

        let time = DateTime::<Local>::from(
            Utc.timestamp_opt(value.friends_since, 0)
                .single()
                .ok_or(PlayerFriendsError::InvalidTimestamp(value.friends_since))?,
        );

        Ok(Self {
            steam_id: id,
            friends_since: time,
        })
    }
}

impl Client {
    /// Get the friends of the profile with the given [`SteamId`]
    ///
    /// Uses [`PLAYER_FRIENDS_API`]
    pub async fn get_player_friends(&self, id: SteamId) -> Result<Option<FriendList>> {
        let query = [
            ("key", self.api_key()),
            ("relationship", "friend"),
            ("steamid", &id.to_string()),
        ];

        let resp = match self.get_json::<Response>(PLAYER_FRIENDS_API, &query).await {
            Ok(resp) => resp,
            Err(err) => match err.status() {
                Some(StatusCode::UNAUTHORIZED) => return Ok(None),
                _ => return Err(err.into()),
            },
        };

        let mut friends = Vec::with_capacity(resp.friend_list.friends.len());
        for friend in resp.friend_list.friends {
            friends.push(Friend::parse_response(friend)?);
        }

        Ok(Some(friends))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn parses_public() {}

    #[test]
    fn parses_private() {}
}
