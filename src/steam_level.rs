use crate::constants::PLAYER_STEAM_LEVEL_API;
use crate::constants::{RETRIES, WAIT_DURATION};
use crate::request_helper::send_request_with_reties;
use crate::steam_id::SteamId;

use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SteamLevelError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}
pub type Result<T> = std::result::Result<T, SteamLevelError>;

#[derive(Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

#[derive(Deserialize, Debug)]
struct ResponseInner {
    player_level: Option<usize>,
}

/// Get the Steam-level of the profile with the given [`SteamId`]
///
/// Uses [`PLAYER_STEAM_LEVEL_API`]
pub async fn get_player_steam_level(
    client: &Client,
    api_key: &str,
    id: SteamId,
) -> Result<Option<usize>> {
    let query = [("key", api_key), ("steamid", &id.to_string())];

    let resp = send_request_with_reties::<Response>(
        client,
        PLAYER_STEAM_LEVEL_API,
        &query,
        true,
        true,
        RETRIES,
        WAIT_DURATION,
    )
    .await?;

    Ok(resp.response.player_level)
}

#[cfg(test)]
mod tests {
    use super::get_player_steam_level;
    use crate::steam_id::SteamId;
    use futures::{FutureExt, StreamExt};

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
            .map(|&id| get_player_steam_level(&client, &api_key, id).map(move |r| (id, r)))
            .buffer_unordered(2);

        while let Some((id, res)) = stream.next().await {
            let res = match res {
                Ok(res) => res,
                Err(err) => {
                    println!("{}", err);
                    continue;
                }
            };
            println!("{} = {:?}", id, res);
        }
    }
}
