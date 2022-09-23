use crate::constants::PLAYER_FRIENDS_API;
use crate::constants::{RETRIES, WAIT_DURATION};
use crate::steam_id::SteamId;

use std::convert::TryFrom;
use std::str::FromStr;

use chrono::{DateTime, Local, TimeZone, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use thiserror::Error;
use tokio::time::sleep;

#[derive(Error, Debug)]
pub enum PlayerFriendsError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The result contained an invalid [SteamId]
    #[error("invalid steam-id: `{0}`")]
    InvalidSteamId(String),

    /// The profile is either private or has been deleted
    #[error("cannot view friends")]
    Unauthorized,
}
pub type Result<T> = std::result::Result<T, PlayerFriendsError>;

#[derive(Deserialize, Debug)]
struct ResponseInnerElement {
    #[serde(rename = "steamid")]
    steam_id: String,
    #[serde(rename = "relationship")]
    _relationship: String,
    #[serde(rename = "friend_since")]
    friends_since: u64,
}

#[derive(Deserialize, Debug, Default)]
struct ResponseInner {
    friends: Vec<ResponseInnerElement>,
}

#[derive(Deserialize, Debug)]
struct Response {
    #[serde(rename = "friendslist")]
    friend_list: Option<ResponseInner>,
}

#[derive(Debug)]
pub struct Friend {
    pub steam_id: SteamId,
    pub friends_since: DateTime<Local>,
}

#[derive(Debug)]
pub struct FriendList {
    pub steam_id: SteamId,
    pub friends: Vec<Friend>,
}

impl TryFrom<ResponseInnerElement> for Friend {
    type Error = PlayerFriendsError;
    fn try_from(value: ResponseInnerElement) -> Result<Self> {
        let id = SteamId::from_str(&value.steam_id)
            .map_err(|_| PlayerFriendsError::InvalidSteamId(value.steam_id))?;
        let time = DateTime::<Local>::from(Utc.timestamp(value.friends_since as i64, 0));

        Ok(Self {
            steam_id: id,
            friends_since: time,
        })
    }
}

impl TryFrom<(Response, SteamId)> for FriendList {
    type Error = PlayerFriendsError;
    fn try_from((response, id): (Response, SteamId)) -> Result<Self> {
        let friends = response.friend_list.unwrap_or_default().friends.into_iter();
        let friends = friends.map(Friend::try_from).collect::<Result<Vec<_>>>()?;

        Ok(Self {
            steam_id: id,
            friends: friends,
        })
    }
}

/// Get the friends of the profile with the given [`SteamId`]
///
/// Uses [`PLAYER_FRIENDS_API`]
pub async fn get_player_friends(client: &Client, api_key: &str, id: SteamId) -> Result<FriendList> {
    let query = [
        ("key", api_key),
        ("relationship", "friend"),
        ("steamid", &id.to_string()),
    ];

    let mut retries = 0_usize;
    let resp = loop {
        let req = client.get(PLAYER_FRIENDS_API).query(&query);
        let resp = req.send().await?;

        // This means the profile has been deleted or is private
        if resp.status() == StatusCode::UNAUTHORIZED {
            return Err(PlayerFriendsError::Unauthorized);
        }
        let err = match resp.error_for_status() {
            Ok(resp) => break resp.json::<Response>().await?,
            Err(err) => err,
        };
        if retries >= RETRIES {
            return Err(err.into());
        }
        retries += 1;
        sleep(WAIT_DURATION).await;
    };

    FriendList::try_from((resp, id))
}

#[cfg(test)]
mod tests {
    use super::get_player_friends;
    use crate::steam_id::SteamId;
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let ids: [SteamId; _] = [
            76561199123543583.into(),
            76561198196615742.into(),
            76561199159691884.into(),
            76561198230177976.into(),
            76561198414415313.into(),
            76561197992321696.into(),
            76561198350302388.into(),
            76561198159967543.into(),
            76561197981967565.into(),
            76561199049236696.into(),
            76561199063760869.into(),
            76561197961074129.into(),
            76561198292293761.into(),
            76561198145832850.into(),
            76561198151659207.into(),
            76561198405122517.into(),
        ];

        dotenv::dotenv().unwrap();
        let client = reqwest::Client::new();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();

        let mut stream = futures::stream::iter(ids.iter())
            .map(|&id| get_player_friends(&client, &api_key, id))
            .buffer_unordered(2);

        while let Some(res) = stream.next().await {
            let res = match res {
                Ok(res) => res,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
            for friend in res.friends.iter() {
                println!(
                    "{} + {} = {}",
                    res.steam_id, friend.steam_id, friend.friends_since
                );
            }
        }
    }
}
