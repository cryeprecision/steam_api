pub const VANITY_API: &str = "https://api.steampowered.com/ISteamUser/ResolveVanityURL/v1/";
pub const VANITY_CONCURRENT_REQUESTS: usize = 100;

pub const PLAYER_SUMMARIES_API: &str =
    "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/";
pub const PLAYER_SUMMARIES_CONCURRENT_REQUESTS: usize = 100;
pub const PLAYER_SUMMARIES_IDS_PER_REQUEST: usize = 100;

pub const PLAYER_FRIENDS_API: &str = "https://api.steampowered.com/ISteamUser/GetFriendList/v1/";
pub const PLAYER_FRIENDS_CONCURRENT_REQUESTS: usize = 100;

pub const PLAYER_BANS_API: &str = "https://api.steampowered.com/ISteamUser/GetPlayerBans/v1/";
pub const PLAYER_BANS_CONCURRENT_REQUESTS: usize = 100;
pub const PLAYER_BANS_IDS_PER_REQUEST: usize = 100;

pub const USER_SEARCH_API: &str = "https://steamcommunity.com/search/SearchCommunityAjax/";
pub const USER_SEARCH_CONCURRENT_REQUESTS: usize = 100;
pub const USER_SEARCH_RESULTS_PER_PAGE: usize = 20;
pub const USER_SEARCH_MAX_PAGES: usize = 500;
pub const USER_SEARCH_MAX_RESULTS: usize = USER_SEARCH_MAX_PAGES * USER_SEARCH_RESULTS_PER_PAGE;

pub const GET_SESSION_ID_URL: &str = "https://steamcommunity.com/search/users/";
pub const PROFILE_URL_FIXED_PREFIX: &str = "https://steamcommunity.com/profiles/";
pub const PROFILE_URL_VANITY_PREFIX: &str = "https://steamcommunity.com/id/";
