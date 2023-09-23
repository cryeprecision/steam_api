//! Parse the HTML payload for user search requests

use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use thiserror::Error;

use crate::constants::PROFILE_URL_ID64_PREFIX;
use crate::model::SteamId;

#[derive(Debug, Error)]
pub enum Error {
    /// Couldn't parse the profile-info from a row in the html-payload
    #[error("no profile info")]
    NoProfileInfo,

    /// Couldn't parse the profile-avatar from a row in the html-payload
    #[error("no profile avatar")]
    NoProfileAvatar,

    #[error("couldn't construct the html parser")]
    InvalidSelector(#[from] scraper::error::SelectorErrorKind<'static>),
}
type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Debug, Clone)]
pub struct UserSearchEntry {
    pub persona_name: String,
    pub profile_url: String,
    pub avatar_full: String,
    pub aliases: Vec<String>,
}

impl UserSearchEntry {
    /// Get the [`SteamId`] from the URL if possible
    ///
    /// # Example
    ///
    /// `https://steamcommunity.com/profiles/76561197960287930 => 76561197960287930`
    pub fn steam_id(&self) -> Option<SteamId> {
        let suffix = self.profile_url.strip_prefix(PROFILE_URL_ID64_PREFIX)?;
        suffix.parse().ok()
    }
}

pub struct Parser {
    row: Selector,
    info: Selector,
    alias_outer: Selector,
    alias_inner: Selector,
    profile_pic: Selector,
}

impl Parser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            row: Selector::parse("div.search_row")?,
            info: Selector::parse("a.searchPersonaName")?,
            alias_outer: Selector::parse("div.search_match_info>div")?,
            alias_inner: Selector::parse("span")?,
            profile_pic: Selector::parse("div.avatarMedium>a>img")?,
        })
    }

    fn parse_row(&self, row: ElementRef) -> Result<UserSearchEntry> {
        const AVATAR_MEDIUM_SUFFIX: &str = "_medium.jpg";
        const AVATAR_FULL_SUFFIX: &str = "_full.jpg";

        let (profile_url, persona_name) = {
            let Some(info) = row.select(&self.info).next() else {
                return Err(Error::NoProfileInfo);
            };
            let profile_url = match info.value().attr("href") {
                Some(href) => href.to_owned(),
                None => return Err(Error::NoProfileInfo),
            };
            (profile_url, info.inner_html())
        };

        let avatar_full = {
            let Some(image) = row.select(&self.profile_pic).next() else {
                return Err(Error::NoProfileAvatar);
            };
            let mut avatar_medium = match image.value().attr("src") {
                Some(avatar) => (avatar[..avatar.len() - AVATAR_MEDIUM_SUFFIX.len()]).to_owned(),
                None => return Err(Error::NoProfileAvatar),
            };
            avatar_medium.push_str(AVATAR_FULL_SUFFIX);
            avatar_medium
        };

        let mut aliases = Vec::new();
        for inner_div in row.select(&self.alias_outer) {
            let div_inner = inner_div.inner_html();
            if !div_inner.trim_start().starts_with("Also known as") {
                continue;
            }
            for inner_span in inner_div.select(&self.alias_inner) {
                aliases.push(inner_span.inner_html());
            }
        }

        Ok(UserSearchEntry {
            persona_name,
            profile_url,
            avatar_full,
            aliases,
        })
    }

    pub fn parse(&self, html: &str) -> Result<Vec<UserSearchEntry>> {
        let html = Html::parse_fragment(html);
        html.select(&self.row)
            .map(|row| self.parse_row(row))
            .collect()
    }
}
