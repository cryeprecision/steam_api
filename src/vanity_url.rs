use crate::constants::{VANITY_API, VANITY_CONCURRENT_REQUESTS};
use crate::steam_id::SteamId;

#[derive(serde::Deserialize, Debug)]
struct ResponseInner {
    #[serde(rename = "steamid")]
    steam_id: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
struct Response {
    response: ResponseInner,
}

pub struct VanityUrl(pub Option<SteamId>);

impl From<Response> for VanityUrl {
    fn from(response: Response) -> Self {
        VanityUrl(response.response.steam_id.map(SteamId::from))
    }
}

async fn request(client: &reqwest::Client, api_key: &str, url: &str) -> reqwest::Result<Response> {
    use super::api_helper::{build_request, send_request};

    let req = build_request(client, api_key, VANITY_API, ("vanityurl", url));
    send_request(req, false, true).await
}

pub async fn resolve_vanity_urls(
    client: &reqwest::Client,
    api_key: &str,
    urls: &[&str],
) -> reqwest::Result<Vec<VanityUrl>> {
    use futures::StreamExt;

    let url_stream = futures::stream::iter(urls);

    let mut mapped = url_stream
        .map(|url| request(client, api_key, *url))
        .buffered(VANITY_CONCURRENT_REQUESTS);

    let mut buffer = Vec::<VanityUrl>::with_capacity(urls.len());

    let bar = styled_progress_bar(urls.len(), "Vanity");
    while let Some(response) = mapped.next().await {
        bar.inc(1);
        buffer.push(VanityUrl::from(response?));
    }
    abandon_progress_bar(bar);

    for (res, url) in buffer.iter().zip(urls.iter()) {
        match res.0 {
            Some(_) => continue,
            None => log::warn!("couldn't resolve vanity {}", url),
        }
    }

    return Ok(buffer);
}
