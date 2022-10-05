use crate::client::Client;
use crate::constants::PLAYER_STEAM_LEVEL_API;
use crate::steam_id::SteamId;

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

impl Client {
    /// Get the Steam level of the given [SteamId]
    ///
    /// Uses [`PLAYER_STEAM_LEVEL_API`]
    pub async fn get_player_steam_level(&self, id: SteamId) -> Result<Option<usize>> {
        let query = [("key", self.api_key()), ("steamid", &id.to_string())];
        let resp = self
            .get_json::<Response>(PLAYER_STEAM_LEVEL_API, &query)
            .await?;
        Ok(resp.response.player_level)
    }
}

#[cfg(test)]
mod tests {
    use crate::steam_id::SteamId;
    use crate::ClientOptions;

    use futures::{FutureExt, StreamExt};

    #[tokio::test]
    async fn it_works() {
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561198196615742
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561198089612262
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561197992321696
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561198350302388
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561198159967543
        // https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/?key=E84C8EF965448E02C469BB3228D46311&steamid=76561199063760869

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
            .retries(5)
            .retry_timeout_ms(1000)
            .build()
            .await;

        let mut stream = futures::stream::iter(ids.iter())
            .map(|&id| client.get_player_steam_level(id).map(move |r| (id, r)))
            .buffer_unordered(2);

        let mut buffer = Vec::new();
        while let Some((id, lvl)) = stream.next().await {
            buffer.push((id, lvl.unwrap()));
        }

        buffer.sort_by_key(|(_, lvl)| std::cmp::Reverse(*lvl));

        for (id, lvl) in buffer.iter() {
            println!("{}: {:?}", id, lvl);
        }
    }
}
