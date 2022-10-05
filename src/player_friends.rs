use crate::constants::PLAYER_FRIENDS_API;
use crate::constants::{RETRIES, WAIT_DURATION};
use crate::parse_response::ParseResponse;
use crate::steam_id::SteamId;

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
        let time = DateTime::<Local>::from(Utc.timestamp(value.friends_since as i64, 0));

        Ok(Self {
            steam_id: id,
            friends_since: time,
        })
    }
}

/// Get the friends of the profile with the given [`SteamId`]
///
/// Uses [`PLAYER_FRIENDS_API`]
pub async fn get_player_friends(
    client: &Client,
    api_key: &str,
    id: SteamId,
) -> Result<Option<FriendList>> {
    let query = [
        ("key", api_key),
        ("relationship", "friend"),
        ("steamid", &id.to_string()),
    ];

    let mut retries = 0_usize;
    let resp = loop {
        let req = client.get(PLAYER_FRIENDS_API).query(&query);
        let resp = req.send().await?;

        // This means the profile has been deleted or friends are private
        if resp.status() == StatusCode::UNAUTHORIZED {
            return Ok(None);
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

    let mut friends = Vec::with_capacity(resp.friend_list.friends.len());
    for friend in resp.friend_list.friends.into_iter() {
        friends.push(Friend::parse_response(friend)?);
    }

    Ok(Some(friends))
}

impl crate::client::Client {
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
        for friend in resp.friend_list.friends.into_iter() {
            friends.push(Friend::parse_response(friend)?);
        }

        Ok(Some(friends))
    }
}

#[cfg(test)]
mod tests {
    use crate::client::ClientOptions;
    use crate::steam_id::SteamId;
    use futures::{FutureExt, StreamExt};
    use reqwest::StatusCode;

    #[tokio::test]
    async fn it_works() {
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561198196615742
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561198089612262
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561197992321696
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561198350302388
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561198159967543
        // https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key=E84C8EF965448E02C469BB3228D46311&relationship=friend&steamid=76561199063760869

        let ids: [SteamId; _] = [
            76561198196615742.into(), // normal, private friends
            76561198089612262.into(), // normal, public friends
            76561197992321696.into(), // deleted
            76561198350302388.into(), // community banned
            76561198159967543.into(), // private
            76561199063760869.into(), // private
        ];

        dotenv::dotenv().unwrap();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let client = ClientOptions::new()
            .api_key(api_key)
            .dont_retry(StatusCode::UNAUTHORIZED)
            .build()
            .await;

        let mut stream = futures::stream::iter(ids.iter())
            .map(|&id| client.get_player_friends(id).map(move |r| (id, r)))
            .buffer_unordered(100);

        while let Some((id, res)) = stream.next().await {
            let res = match res {
                Ok(res) => match res {
                    None => {
                        println!("[{}] private friends", id);
                        continue;
                    }
                    Some(res) => res,
                },
                Err(err) => {
                    println!("[{}] err: {}", id, err);
                    continue;
                }
            };
            println!("[{}] {} friends", id, res.len());
        }
        println!("Retries: {}", client.retries());
    }
}
