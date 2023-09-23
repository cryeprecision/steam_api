//! The `sessionid` has to be set both as a cookie and as a query parameter!
//! Otherwise the request is rejected as UNAUTHORIZED.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::client::Client;
use crate::constants::USER_SEARCH_API;
use crate::model::html::user_search;

#[derive(Debug, Error)]
pub enum UserSearchError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The `success` member in the response was not set to `1`
    #[error("api didn't return success")]
    NoSuccess,

    /// The `search_page` member in the response was not a valid [`usize`]
    #[error("search_page is not a valid usize")]
    InvalidSearchPage,

    /// There was an error while parsing the html-payload
    #[error("couldn't parse html payload ({0})")]
    ParseError(#[from] user_search::Error),
}
type Result<T> = std::result::Result<T, UserSearchError>;

#[derive(Serialize, Debug, Clone)]
pub struct UserSearchPage {
    pub search_string: String,
    pub total_result_count: usize,
    pub search_filter: String,
    pub search_page: usize,
    pub results: Vec<user_search::UserSearchEntry>,
}

#[derive(Deserialize)]
struct Response {
    success: i32,
    search_text: String,
    search_result_count: usize,
    search_filter: String,
    search_page: serde_json::Value,
    html: String,
}

impl TryFrom<Response> for UserSearchPage {
    type Error = UserSearchError;
    fn try_from(value: Response) -> Result<Self> {
        if value.success != 1 {
            return Err(UserSearchError::NoSuccess);
        }

        let parser = user_search::Parser::new()?;
        let results = parser.parse(&value.html)?;

        // Steam sometimes returns this as a number
        // and sometimes as a string 🤡
        let search_page = match value.search_page {
            serde_json::Value::Number(num) => num.as_u64(),
            serde_json::Value::String(str) => str.parse::<u64>().ok(),
            _ => None,
        }
        .ok_or(UserSearchError::InvalidSearchPage)?;

        Ok(Self {
            search_string: value.search_text,
            total_result_count: value.search_result_count,
            search_filter: value.search_filter,
            search_page: search_page as usize,
            results,
        })
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
        resp.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, UserSearchPage};
    use crate::model::SteamId;

    #[test]
    fn parses() {
        let json: Response = load_test_json!("user_search.json");
        let search: UserSearchPage = json.try_into().unwrap();

        assert_eq!(search.search_string, "sauce");
        assert_eq!(search.total_result_count, 47813);
        assert_eq!(search.search_page, 1);

        let results = search.results;
        assert_eq!(results.len(), 20);

        let snd = results.iter().nth(1).unwrap();
        assert_eq!(snd.persona_name, "The Sauce");
        assert_eq!(snd.aliases.len(), 0);
        assert_eq!(snd.steam_id(), Some(SteamId(76561197971683832)));
    }
}
