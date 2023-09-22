#![forbid(unsafe_code)]
#![warn(
    clippy::copy_iterator,
    clippy::default_trait_access,
    clippy::doc_link_with_quotes,
    clippy::enum_glob_use,
    clippy::expl_impl_clone_on_copy,
    clippy::implicit_clone,
    clippy::inconsistent_struct_constructor,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::redundant_else,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_wraps,
    clippy::unnested_or_patterns,
    clippy::unused_async,
    clippy::used_underscore_binding
)]
#![warn(clippy::wildcard_dependencies)]
#![warn(
    clippy::branches_sharing_code,
    clippy::clear_with_drain,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::debug_assert_with_mut_call,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_line_after_doc_comments,
    clippy::empty_line_after_outer_attr,
    clippy::equatable_if_let,
    clippy::fallible_impl_from,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_stack_frames,
    clippy::manual_clamp,
    clippy::missing_const_for_fn,
    clippy::mutex_integer,
    clippy::needless_collect,
    clippy::nonstandard_macro_braces,
    clippy::option_if_let_else,
    clippy::or_fun_call,
    clippy::path_buf_push_overwrite,
    clippy::redundant_clone,
    clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,
    clippy::suspicious_operation_groupings,
    clippy::trait_duplication_in_bounds,
    clippy::type_repetition_in_bounds,
    clippy::unnecessary_struct_initialization,
    clippy::unused_rounding,
    clippy::useless_let_if_seq
)]
//! `SteamAPI` abstraction, that tries to make bulk requests as fast as possible.
//!
//! # Current state
//!
//! Currently provides abstractions for the following endpoints:
//! - [X] [`api.steampowered.com/ISteamUser/ResolveVanityURL/v1/`][constants::VANITY_API]
//! - [X] [`api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/`][constants::PLAYER_SUMMARIES_API]
//! - [X] [`api.steampowered.com/ISteamUser/GetFriendList/v1/`][constants::PLAYER_FRIENDS_API]
//! - [X] [`api.steampowered.com/ISteamUser/GetPlayerBans/v1/`][constants::PLAYER_BANS_API]
//! - [X] [`api.steampowered.com/IPlayerService/GetSteamLevel/v1/`][constants::PLAYER_STEAM_LEVEL_API]
//! - [X] [`steamcommunity.com/search/SearchCommunityAjax/`][constants::USER_SEARCH_API]
//!
//! # Other
//!
//! Also provides a class for handling [`SteamId`][crate::steam_id::SteamId]s.

#![feature(int_roundings)]
#![feature(generic_arg_infer)]

#[cfg(test)]
#[macro_use]
mod test_util;

pub mod constants;

mod enums;

pub use enums::*;

mod client;
pub use client::*;

pub mod rate_limit;

mod steam_id;
pub use steam_id::*;

mod steam_id_ext;
pub use steam_id_ext::SteamIdExt;

#[cfg(feature = "friend_code")]
mod steam_id_friend_code;

#[cfg(feature = "friend_code")]
mod bit_chunks;

mod vanity_url;
pub use vanity_url::*;

mod player_summary;
pub use player_summary::*;

mod player_bans;
pub use player_bans::*;

mod player_friends;
pub use player_friends::*;

mod steam_level;
pub use steam_level::*;

mod parse_response;

#[cfg(feature = "user_search")]
mod user_search;
#[cfg(feature = "user_search")]
pub use user_search::*;
