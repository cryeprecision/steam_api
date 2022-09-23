use std::time::Duration;

/// [`/ISteamUser/ResolveVanityURL/v1/`](https://partner.steamgames.com/doc/webapi/ISteamUser#:~:text=/ISteamUser/ResolveVanityURL/v1/)
pub const VANITY_API: &str = "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v1/";
pub const VANITY_CONCURRENT_REQUESTS: usize = 100;

/// [`/ISteamUser/GetPlayerSummaries/v2/`](https://partner.steamgames.com/doc/webapi/ISteamUser#:~:text=/ISteamUser/GetPlayerSummaries/v2/)
pub const PLAYER_SUMMARIES_API: &str =
    "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/";
pub const PLAYER_SUMMARIES_CONCURRENT_REQUESTS: usize = 100;
pub const PLAYER_SUMMARIES_IDS_PER_REQUEST: usize = 100;

/// [`/ISteamUser/GetFriendList/v1/`](https://partner.steamgames.com/doc/webapi/ISteamUser#:~:text=/ISteamUser/GetFriendList/v1/)
pub const PLAYER_FRIENDS_API: &str = "https://api.steampowered.com/ISteamUser/GetFriendList/v1/";
pub const PLAYER_FRIENDS_CONCURRENT_REQUESTS: usize = 100;

/// [`/ISteamUser/GetPlayerBans/v1/`](https://partner.steamgames.com/doc/webapi/ISteamUser#:~:text=/ISteamUser/GetPlayerBans/v1/)
pub const PLAYER_BANS_API: &str = "https://api.steampowered.com/ISteamUser/GetPlayerBans/v1/";
pub const PLAYER_BANS_CONCURRENT_REQUESTS: usize = 100;
pub const PLAYER_BANS_IDS_PER_REQUEST: usize = 100;

/// [`/IPlayerService/GetSteamLevel/v1/`](https://partner.steamgames.com/doc/webapi/IPlayerService#GetOwnedGames:~:text=/IPlayerService/GetSteamLevel/v1/)
pub const PLAYER_STEAM_LEVEL_API: &str =
    "https://api.steampowered.com/IPlayerService/GetSteamLevel/v1/";
pub const PLAYER_STEAM_LEVEL_CONCURRENT_REQUESTS: usize = 100;
pub const PLAYER_STEAM_LEVEL_IDS_PER_REQUEST: usize = 100;

/// Not documented
pub const USER_SEARCH_API: &str = "https://steamcommunity.com/search/SearchCommunityAjax/";
pub const USER_SEARCH_CONCURRENT_REQUESTS: usize = 100;
/// Each result will contain `20` results
pub const USER_SEARCH_RESULTS_PER_PAGE: usize = 20;
/// This endpoint will only return unique results for pages in the range `[1, 500]`
pub const USER_SEARCH_MAX_PAGES: usize = 500;
/// We can only request `500` pages with `20` results each
pub const USER_SEARCH_MAX_RESULTS: usize = USER_SEARCH_MAX_PAGES * USER_SEARCH_RESULTS_PER_PAGE;

pub const PROFILE_URL_FIXED_PREFIX: &str = "https://steamcommunity.com/profiles/";
pub const PROFILE_URL_VANITY_PREFIX: &str = "https://steamcommunity.com/id/";

/// How often we retry an endpoint in case of an error
pub const RETRIES: usize = 5;
/// How long we wait between each attempt while retrying
pub const WAIT_DURATION: Duration = Duration::from_millis(250);
