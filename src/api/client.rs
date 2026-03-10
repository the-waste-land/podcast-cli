use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::Proxy;
use serde::de::DeserializeOwned;
use sha1::{Digest, Sha1};

use crate::error::{ApiContext, PodcastCliError, Result};

const DEFAULT_BASE_URL: &str = "https://api.podcastindex.org/api/1.0";
const USER_AGENT_VALUE: &str = "podcast-cli/0.1";

fn build_http_client() -> reqwest::Client {
    let mut builder = reqwest::Client::builder();

    // Read proxy from environment variables
    // Use ALL_PROXY for all schemes, or HTTPS_PROXY/HTTP_PROXY individually
    if let Some(all_proxy) = get_env_proxy("ALL_PROXY").or_else(|| get_env_proxy("all_proxy")) {
        match Proxy::all(&all_proxy) {
            Ok(proxy) => {
                builder = builder.proxy(proxy);
            }
            Err(e) => {
                eprintln!("Warning: failed to parse ALL_PROXY: {}", e);
            }
        }
    } else {
        // Try HTTPS_PROXY for HTTPS requests
        if let Some(https_proxy) =
            get_env_proxy("HTTPS_PROXY").or_else(|| get_env_proxy("https_proxy"))
        {
            match Proxy::https(&https_proxy) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    eprintln!("Warning: failed to parse HTTPS_PROXY: {}", e);
                }
            }
        }
        // Try HTTP_PROXY for HTTP requests
        if let Some(http_proxy) =
            get_env_proxy("HTTP_PROXY").or_else(|| get_env_proxy("http_proxy"))
        {
            match Proxy::http(&http_proxy) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                }
                Err(e) => {
                    eprintln!("Warning: failed to parse HTTP_PROXY: {}", e);
                }
            }
        }
    }

    match builder.build() {
        Ok(client) => client,
        Err(e) => {
            eprintln!(
                "Warning: failed to build HTTP client with proxy, falling back to default: {}",
                e
            );
            reqwest::Client::new()
        }
    }
}

/// Get proxy URL from environment variable, filtering empty values
fn get_env_proxy(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

#[derive(Debug, Clone)]
pub struct PodcastIndexClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
    api_secret: String,
}

impl PodcastIndexClient {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::with_base_url(api_key, api_secret, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        api_secret: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let http = build_http_client();
        Self {
            http,
            base_url: base_url.into(),
            api_key: api_key.into(),
            api_secret: api_secret.into(),
        }
    }

    pub async fn get_json<T>(&self, path: &str, query: &[(&str, String)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), path);
        let headers = self.auth_headers()?;
        let response = self
            .http
            .get(url)
            .query(query)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(PodcastCliError::Api(format!(
                "request failed with status {status}: {body}"
            )));
        }

        serde_json::from_str(&body).map_err(|err| {
            PodcastCliError::Api(format!("failed to parse API response: {err}; body={body}"))
        })
    }

    fn auth_headers(&self) -> Result<HeaderMap> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .api_context("invalid system time")?
            .as_secs()
            .to_string();

        let payload = format!("{}{}{}", self.api_key, self.api_secret, timestamp);
        let mut hasher = Sha1::new();
        hasher.update(payload.as_bytes());
        let digest = hex::encode(hasher.finalize());

        let mut headers = HeaderMap::new();
        headers.insert(
            "X-Auth-Date",
            HeaderValue::from_str(&timestamp).api_context("invalid auth date")?,
        );
        headers.insert(
            "X-Auth-Key",
            HeaderValue::from_str(&self.api_key).api_context("invalid api key")?,
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&digest).api_context("invalid auth token")?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static(USER_AGENT_VALUE));

        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::USER_AGENT;

    #[test]
    fn auth_headers_include_user_agent() {
        let client = PodcastIndexClient::new("key", "secret");
        let headers = client.auth_headers().expect("headers should build");

        assert!(headers.contains_key(USER_AGENT));
    }
}
