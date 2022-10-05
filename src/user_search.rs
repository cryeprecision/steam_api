use crate::client::Client;
use crate::constants::USER_SEARCH_API;
use crate::parse_response::ParseResponse;
use crate::steam_id::SteamId;

use std::fmt;
use std::str::FromStr;

use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserSearchError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The `success` member in the response was not set to `1`
    #[error("api didn't return success")]
    NoSuccess,

    /// A selector for parsing the html-payload was invalid
    #[error("selector is invalid")]
    InvalidSelector,

    /// The `search_page` member in the response was not a valid [`usize`]
    #[error("search_page is not a valid usize")]
    InvalidSearchPage,

    /// There was an error while parsing the html-payload
    #[error("couldn't parse html payload ({0})")]
    InvalidHtmlPayload(#[from] ParseError),
}
type Result<T> = std::result::Result<T, UserSearchError>;

#[derive(Deserialize, Debug)]
struct Response {
    success: i32,
    search_text: String,
    search_result_count: usize,
    search_filter: String,
    search_page: serde_json::Value,
    html: String,
}

pub struct UserSearchEntry {
    pub persona_name: String,
    pub profile_url: String,
    pub avatar_full: String,
    pub aliases: Vec<String>,
}

impl UserSearchEntry {
    /// Abbreviate the profile-url from the html-payload.
    ///
    /// # Example
    ///
    /// `https://steamcommunity.com/id/GabeLoganNewell => id/GabeLoganNewell`
    /// `http://steamcommunity.com/profiles/76561197960287930 => profiles/76561197960287930`
    pub fn short_url(&self) -> Option<&str> {
        const ID: &str = "/profiles/";
        const URL: &str = "/id/";
        let find = self.profile_url.find(ID);
        let find = find.or_else(|| self.profile_url.find(URL));
        find.map(|idx| &self.profile_url[idx + 1..])
    }
    /// Get the [`SteamId`] from the URL if possible
    ///
    /// # Example
    ///
    /// `https://steamcommunity.com/id/GabeLoganNewell => GabeLoganNewell`
    pub fn steam_id(&self) -> Option<SteamId> {
        const ID: &str = "/profiles/";
        SteamId::from_str(self.profile_url.split_once(ID)?.1).ok()
    }
    /// Get the `VanityURL` from the URL if possible
    ///
    /// # Example
    ///
    /// `http://steamcommunity.com/profiles/76561197960287930 => 76561197960287930`
    pub fn vanity_url(&self) -> Option<&str> {
        const URL: &str = "/id/";
        Some(self.profile_url.split_once(URL)?.1)
    }
}

impl fmt::Debug for UserSearchEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSearchEntry")
            .field("name", &self.persona_name)
            .field("url", &self.short_url().unwrap_or_default())
            .field("aliases", &self.aliases.len())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct UserSearchPage {
    pub search_string: String,
    pub total_result_count: usize,
    pub search_filter: String,
    pub search_page: usize,
    pub results: Vec<UserSearchEntry>,
}

impl ParseResponse<Response> for UserSearchPage {
    type Error = UserSearchError;
    fn parse_response(value: Response) -> Result<Self> {
        if value.success != 1 {
            return Err(UserSearchError::NoSuccess);
        }
        let parser = Parser::new().ok_or(UserSearchError::InvalidSelector)?;
        let search_page = match value.search_page {
            serde_json::Value::Number(num) => num.as_u64(),
            serde_json::Value::String(str) => str.parse::<u64>().ok(),
            _ => None,
        }
        .ok_or(UserSearchError::InvalidSearchPage)?;
        let results = parser.parse(&value.html)?;

        Ok(Self {
            search_string: value.search_text,
            total_result_count: value.search_result_count,
            search_filter: value.search_filter,
            search_page: search_page as usize,
            results,
        })
    }
}

struct Parser {
    row: Selector,
    info: Selector,
    alias_outer: Selector,
    alias_inner: Selector,
    profile_pic: Selector,
}

#[derive(Debug, Error)]
pub enum ParseError {
    /// Couldn't parse the profile-info from a row in the html-payload
    #[error("no profile info")]
    NoProfileInfo,

    /// Couldn't parse the profile-avatar from a row in the html-payload
    #[error("no profile avatar")]
    NoProfileAvatar,
}
type ParseResult<T> = std::result::Result<T, ParseError>;

impl Parser {
    fn new() -> Option<Self> {
        Some(Self {
            row: Selector::parse("div.search_row").ok()?,
            info: Selector::parse("a.searchPersonaName").ok()?,
            alias_outer: Selector::parse("div.search_match_info>div").ok()?,
            alias_inner: Selector::parse("span").ok()?,
            profile_pic: Selector::parse("div.avatarMedium>a>img").ok()?,
        })
    }
    fn parse_row(&self, row: ElementRef) -> ParseResult<UserSearchEntry> {
        const AVATAR_MEDIUM_SUFFIX: &str = "_medium.jpg";
        const AVATAR_FULL_SUFFIX: &str = "_full.jpg";

        let (profile_url, persona_name) = {
            let info = match row.select(&self.info).next() {
                Some(info) => info,
                None => return Err(ParseError::NoProfileInfo),
            };
            let profile_url = match info.value().attr("href") {
                Some(href) => href.to_owned(),
                None => return Err(ParseError::NoProfileInfo),
            };
            (profile_url, info.inner_html())
        };

        let avatar_full = {
            let image = match row.select(&self.profile_pic).next() {
                Some(image) => image,
                None => return Err(ParseError::NoProfileAvatar),
            };
            let mut avatar_medium = match image.value().attr("src") {
                Some(avatar) => (avatar[..avatar.len() - AVATAR_MEDIUM_SUFFIX.len()]).to_owned(),
                None => return Err(ParseError::NoProfileAvatar),
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

    fn parse(&self, html: &str) -> ParseResult<Vec<UserSearchEntry>> {
        let html = Html::parse_fragment(html);
        html.select(&self.row)
            .map(|row| self.parse_row(row))
            .collect()
    }
}

impl Client {
    /// Query [`USER_SEARCH_API`] for the name `query` and the page `page`
    pub async fn get_search_page(&self, query: &str, page: usize) -> Result<UserSearchPage> {
        let query = [
            ("filter", "users"),
            ("text", query),
            ("sessionid", self.session_id()),
            ("page", &page.to_string()),
        ];
        let resp = self.get_json::<Response>(USER_SEARCH_API, &query).await?;
        UserSearchPage::parse_response(resp)
    }
}

#[cfg(test)]
mod tests {
    use crate::ClientOptions;

    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        dotenv::dotenv().unwrap();
        let api_key = dotenv::var("STEAM_API_KEY").unwrap();
        let client = ClientOptions::new().api_key(api_key).build().await;

        let searches = std::iter::repeat("prog").zip(1..=148);

        let mut stream = futures::stream::iter(searches)
            .map(|(query, page)| client.get_search_page(query, page))
            .buffer_unordered(20);

        while let Some(res) = stream.next().await {
            let res = res.unwrap();
            println!(
                "Page: {:03}, Results: {}/{}",
                res.search_page,
                res.results.len(),
                res.total_result_count
            );
        }
    }
}
