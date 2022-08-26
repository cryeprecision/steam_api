//! SteamAPI abstraction, that tries to make bulk requests as fast as possible.
//!
//! # Current state
//!
//! Currently provides abstractions for the following endpoints:
//! - [ ] [`api.steampowered.com/ISteamUser/ResolveVanityURL/v1/`]
//! - [ ] [`api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/`]
//! - [ ] [`api.steampowered.com/ISteamUser/GetFriendList/v1/`]
//! - [ ] [`api.steampowered.com/ISteamUser/GetPlayerBans/v1/`]
//! - [ ] [`steamcommunity.com/search/SearchCommunityAjax/`]
//!
//! [`api.steampowered.com/ISteamUser/ResolveVanityURL/v1/`]: https://api.steampowered.com/ISteamUser/ResolveVanityURL/v1/
//! [`api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/`]: https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/
//! [`api.steampowered.com/ISteamUser/GetFriendList/v1/`]: https://api.steampowered.com/ISteamUser/GetFriendList/v1/
//! [`api.steampowered.com/ISteamUser/GetPlayerBans/v1/`]: https://api.steampowered.com/ISteamUser/GetPlayerBans/v1/
//! [`steamcommunity.com/search/SearchCommunityAjax/`]: https://steamcommunity.com/search/SearchCommunityAjax/
//!
//! # Other
//!
//! Also provides a class for handling [`SteamId`]s.

mod constants;

mod enums;
pub use enums::{CommunityVisibilityState, PersonaState};

mod steam_id;
pub use steam_id::{AccountType, SteamId, Universe};

mod steam_id_ext;
pub use steam_id_ext::SteamIdExt;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
