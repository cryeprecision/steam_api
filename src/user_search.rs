use crate::constants::USER_SEARCH_API;

use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use reqwest::cookie::{CookieStore, Jar};
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserSearchError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("api didn't return success")]
    NoSuccess,
    #[error("selector is invalid")]
    InvalidSelector,
    #[error("search_page is not a valid usize")]
    InvalidSearchPage,
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
    pub fn short_url(&self) -> &str {
        const ID: &str = "/profiles/";
        const URL: &str = "/id/";
        match self
            .profile_url
            .find(ID)
            .or_else(|| self.profile_url.find(URL))
        {
            Some(idx) => &self.profile_url[idx..],
            None => &self.profile_url,
        }
    }
}

impl fmt::Debug for UserSearchEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserSearchEntry")
            .field("name", &self.persona_name)
            .field("url", &self.short_url())
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

impl TryFrom<Response> for UserSearchPage {
    type Error = UserSearchError;
    fn try_from(value: Response) -> Result<Self> {
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
            results: results,
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
    #[error("no profile info")]
    NoProfileInfo,
    #[error("no profile avatar")]
    NoProfileAvatar,
}
pub type ParseResult<T> = std::result::Result<T, ParseError>;

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
                Some(avatar) => (&avatar[..avatar.len() - AVATAR_MEDIUM_SUFFIX.len()]).to_owned(),
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
            profile_url,
            persona_name,
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

pub async fn get_search_page(
    client: &Client,
    session_id: &str,
    query: &str,
    page: usize,
) -> Result<UserSearchPage> {
    let headers = [
        ("filter", "users"),
        ("text", query),
        ("sessionid", session_id),
        ("page", &page.to_string()),
    ];
    let req = client.get(USER_SEARCH_API).query(&headers);
    let resp = crate::request_helper::send_request::<Response>(req, true, true).await?;
    UserSearchPage::try_from(resp)
}

pub async fn client_with_session_id() -> Option<(Client, String)> {
    // SAFETY: I'm pretty sure this is a valid URL :^)
    let url = Url::from_str("https://steamcommunity.com/").ok()?;

    let jar = Arc::new(Jar::default());
    let builder = Client::builder().cookie_provider(Arc::clone(&jar));
    let client = builder.build().ok()?;

    client.get(USER_SEARCH_API).send().await.ok()?;

    let cookies = jar.cookies(&url)?;
    let cookie_str = cookies.to_str().ok()?;

    let session_id = cookie_str
        .split("; ")
        .find(|&str| str.starts_with("sessionid="))?;
    let session_id = session_id.split_once('=')?.1.to_owned();

    Some((client, session_id))
}

#[cfg(test)]
mod tests {
    use super::{client_with_session_id, get_search_page};
    use futures::StreamExt;

    #[tokio::test]
    async fn it_works() {
        let (client, session_id) = client_with_session_id().await.unwrap();
        let searches: [(&str, usize); _] = [
            ("soser", 2),
            ("soser", 1),
            ("masterlooser", 1),
            ("masterlooser", 2),
            ("masterlooser", 3),
            ("masterlooser", 4),
            ("masterlooser", 5),
            ("masterlooser", 6),
            ("masterlooser", 7),
            ("masterlooser", 8),
            ("masterlooser", 9),
            ("masterlooser", 10),
            ("masterlooser", 11),
            ("masterlooser", 12),
            ("masterlooser", 13),
            ("masterlooser", 14),
        ];

        let mut stream = futures::stream::iter(searches.iter())
            .map(|&(query, page)| get_search_page(&client, &session_id, query, page))
            .buffer_unordered(4);

        while let Some(res) = stream.next().await {
            let res = res.unwrap();
            let personas = res
                .results
                .iter()
                .map(|r| r.persona_name.as_str())
                .collect::<Vec<_>>();
            println!(
                "[p: {}, t: {}, c: {}]: {:?}",
                res.search_page,
                res.total_result_count,
                res.results.len(),
                personas,
            )
        }
    }
}
