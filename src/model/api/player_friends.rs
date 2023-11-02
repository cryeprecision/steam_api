use std::collections::HashMap;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::PLAYER_FRIENDS_API;
use crate::model::{SteamId, SteamTime};
use crate::SteamIdStr;

#[derive(Error, Debug)]
pub enum PlayerFriendsError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
type Result<T> = std::result::Result<T, PlayerFriendsError>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Friend {
    #[serde(rename(deserialize = "steamid"))]
    steam_id: SteamIdStr,
    #[serde(rename(deserialize = "relationship"))]
    relationship: String,
    #[serde(rename(deserialize = "friend_since"))]
    friends_since: SteamTime,
}

#[derive(Debug, Clone)]
pub struct FriendsList {
    /// - [`None`], if the user has set his friends to **private**
    /// - [`Some`], if the user has set his friends to **public**
    ///
    /// The [`HashMap`] is empty, if the user has **no friends**
    inner: Option<HashMap<SteamId, Friend>>,
}

#[derive(Deserialize)]
struct ResponseInner {
    friends: Vec<Friend>,
}

#[derive(Deserialize)]
struct Response {
    #[serde(rename(deserialize = "friendslist"))]
    friend_list: Option<ResponseInner>,
}

impl From<Response> for FriendsList {
    fn from(value: Response) -> Self {
        let friends = match value.friend_list {
            None => return FriendsList { inner: None },
            Some(friends) => friends,
        };

        let map = friends
            .friends
            .into_iter()
            .map(|friend| (friend.steam_id.into(), friend))
            .collect();

        FriendsList { inner: Some(map) }
    }
}

impl FriendsList {
    pub fn into_inner(self) -> Option<HashMap<SteamId, Friend>> {
        self.inner
    }
    pub const fn as_inner_ref(&self) -> Option<&HashMap<SteamId, Friend>> {
        self.inner.as_ref()
    }
}

impl Client {
    /// Get the friends of the profile with the given [`SteamId`]
    ///
    /// Uses [`PLAYER_FRIENDS_API`]
    pub async fn get_player_friends(&self, id: SteamId) -> Result<FriendsList> {
        let query = [
            ("key", self.api_key()),
            ("relationship", "friend"),
            ("steamid", &id.to_string()),
        ];

        let resp = match self.get_json::<Response>(PLAYER_FRIENDS_API, &query).await {
            Ok(resp) => resp,
            Err(err) => match err.status() {
                Some(StatusCode::UNAUTHORIZED) => todo!("get data if response code is non 2XX"),
                _ => return Err(err.into()),
            },
        };

        Ok(resp.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{FriendsList, Response};

    #[test]
    fn parses_private() {
        let resp: Response = load_test_json!("player_friends_private.json");
        let bans: FriendsList = resp.into();
        println!("{:#?}", bans);
    }

    #[test]
    fn parses_public() {
        let resp: Response = load_test_json!("player_friends_public.json");
        let bans: FriendsList = resp.into();
        println!("{:#?}", bans);
    }
}
