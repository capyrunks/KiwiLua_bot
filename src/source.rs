use std::env;
use std::fmt;
use std::time::Duration;

use reqwest::Client;

const DEFAULT_MAX_DOWNLOAD_BYTES: usize = 50 * 1024 * 1024;
const DEFAULT_SOURCE_TIMEOUT_SECS: u64 = 20;

const DEFAULT_SOURCE_TEMPLATES: &[&str] =
    &["https://pub-5b6d3b7c03fd4ac1afb5bd3017850e20.r2.dev/{app_id}.zip"];

#[derive(Debug, Clone)]
pub struct LuaSourceConfig {
    templates: Vec<String>,
    max_download_bytes: usize,
    request_timeout: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchedKind {
    Lua,
    Zip,
}

#[derive(Debug)]
pub struct FetchedConfig {
    pub source_url: String,
    pub kind: FetchedKind,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum FetchError {
    NotFound { attempts: Vec<String> },
    Unavailable { attempts: Vec<String> },
}

#[derive(Debug)]
enum AttemptError {
    NotFound,
    BadStatus(u16),
    Network(String),
    TooLarge { limit: usize },
    InvalidContent(String),
}

impl LuaSourceConfig {
    pub fn from_env() -> Self {
        let templates = read_env("LUA_SOURCE_URL_TEMPLATES")
            .or_else(|| read_env("LUA_SOURCE_URL_TEMPLATE"))
            .or_else(|| read_env("LUA_DB_URL").map(normalize_legacy_base_url))
            .map(|value| parse_templates(&value))
            .filter(|templates| !templates.is_empty())
            .unwrap_or_else(|| {
                DEFAULT_SOURCE_TEMPLATES
                    .iter()
                    .map(|template| (*template).to_owned())
                    .collect()
            });

        let max_download_bytes = env::var("LUA_SOURCE_MAX_BYTES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_DOWNLOAD_BYTES);

        let request_timeout = env::var("LUA_SOURCE_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(DEFAULT_SOURCE_TIMEOUT_SECS));

        log::info!(
            "Configured {} Lua source template(s), max download size {} bytes, source timeout {:?}",
            templates.len(),
            max_download_bytes,
            request_timeout
        );

        Self {
            templates,
            max_download_bytes,
            request_timeout,
        }
    }

    pub fn render_urls(&self, app_id: &str) -> Vec<String> {
        self.templates
            .iter()
            .map(|template| {
                template
                    .replace("{app_id}", app_id)
                    .replace("{AppID}", app_id)
                    .replace("{APPID}", app_id)
                    .replace("{appid}", app_id)
            })
            .collect()
    }
}

pub async fn fetch_config(
    client: &Client,
    config: &LuaSourceConfig,
    app_id: &str,
) -> Result<FetchedConfig, FetchError> {
    let mut attempts = Vec::new();
    let mut saw_non_404 = false;

    for url in config.render_urls(app_id) {
        match fetch_one(
            client,
            config.max_download_bytes,
            config.request_timeout,
            &url,
        )
        .await
        {
            Ok(config) => return Ok(config),
            Err(err) => {
                if !matches!(err, AttemptError::NotFound) {
                    saw_non_404 = true;
                }

                let summary = format!("{url}: {err}");
                log::warn!("Lua source attempt failed: {summary}");
                attempts.push(summary);
            }
        }
    }

    if saw_non_404 {
        Err(FetchError::Unavailable { attempts })
    } else {
        Err(FetchError::NotFound { attempts })
    }
}

async fn fetch_one(
    client: &Client,
    max_download_bytes: usize,
    request_timeout: Duration,
    url: &str,
) -> Result<FetchedConfig, AttemptError> {
    let mut response = client
        .get(url)
        .timeout(request_timeout)
        .send()
        .await
        .map_err(|err| AttemptError::Network(err.to_string()))?;

    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(AttemptError::NotFound);
    }
    if !status.is_success() {
        return Err(AttemptError::BadStatus(status.as_u16()));
    }

    if response
        .content_length()
        .is_some_and(|length| length > max_download_bytes as u64)
    {
        return Err(AttemptError::TooLarge {
            limit: max_download_bytes,
        });
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|err| AttemptError::Network(err.to_string()))?
    {
        if bytes.len().saturating_add(chunk.len()) > max_download_bytes {
            return Err(AttemptError::TooLarge {
                limit: max_download_bytes,
            });
        }
        bytes.extend_from_slice(&chunk);
    }

    let kind = classify_response(url, &content_type, &bytes)?;

    Ok(FetchedConfig {
        source_url: url.to_owned(),
        kind,
        bytes,
    })
}

fn parse_templates(value: &str) -> Vec<String> {
    value
        .split(['\n', ',', ';'])
        .map(str::trim)
        .filter(|template| !template.is_empty())
        .map(str::to_owned)
        .collect()
}

fn read_env(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_legacy_base_url(value: String) -> String {
    let value = value.trim();
    if value.contains("{app_id}")
        || value.contains("{AppID}")
        || value.contains("{APPID}")
        || value.contains("{appid}")
    {
        value.to_owned()
    } else {
        format!("{}/{{app_id}}.lua", value.trim_end_matches('/'))
    }
}

fn classify_response(
    url: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<FetchedKind, AttemptError> {
    if bytes.is_empty() {
        return Err(AttemptError::InvalidContent(
            "empty response body".to_owned(),
        ));
    }

    if is_zip(bytes) {
        return Ok(FetchedKind::Zip);
    }

    let lower_url = url.to_ascii_lowercase();
    if lower_url.ends_with(".zip") {
        return Err(AttemptError::InvalidContent(
            "URL looks like ZIP, but response is not a ZIP file".to_owned(),
        ));
    }

    let text = std::str::from_utf8(bytes)
        .map_err(|_| AttemptError::InvalidContent("response is not UTF-8 text".to_owned()))?;
    let text_start = text.trim_start().to_ascii_lowercase();

    if text_start.starts_with("<!doctype html") || text_start.starts_with("<html") {
        return Err(AttemptError::InvalidContent(
            "response is HTML, not Lua".to_owned(),
        ));
    }

    let looks_like_lua_content =
        text.contains("addappid") || text.contains("setManifestid") || lower_url.ends_with(".lua");
    let looks_like_text_type = content_type.starts_with("text/")
        || content_type.contains("lua")
        || content_type.contains("octet-stream")
        || content_type.is_empty();

    if looks_like_lua_content && looks_like_text_type {
        Ok(FetchedKind::Lua)
    } else {
        Err(AttemptError::InvalidContent(format!(
            "unsupported content type {content_type:?}"
        )))
    }
}

fn is_zip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04")
        || bytes.starts_with(b"PK\x05\x06")
        || bytes.starts_with(b"PK\x07\x08")
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { attempts } => write!(formatter, "config not found: {attempts:?}"),
            Self::Unavailable { attempts } => {
                write!(formatter, "sources unavailable: {attempts:?}")
            }
        }
    }
}

impl std::error::Error for FetchError {}

impl fmt::Display for AttemptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(formatter, "404 Not Found"),
            Self::BadStatus(status) => write!(formatter, "HTTP {status}"),
            Self::Network(err) => write!(formatter, "network error: {err}"),
            Self::TooLarge { limit } => write!(formatter, "download is larger than {limit} bytes"),
            Self::InvalidContent(reason) => write!(formatter, "invalid content: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_base_url_becomes_lua_template() {
        assert_eq!(
            normalize_legacy_base_url("https://example.test/database/".to_owned()),
            "https://example.test/database/{app_id}.lua"
        );
    }

    #[test]
    fn classifies_raw_lua() {
        let kind = classify_response(
            "https://example.test/730.lua",
            "text/plain",
            b"addappid(730)",
        )
        .expect("Lua response should be accepted");

        assert_eq!(kind, FetchedKind::Lua);
    }

    #[test]
    fn classifies_zip_by_magic_bytes() {
        let kind = classify_response(
            "https://example.test/730",
            "application/octet-stream",
            b"PK\x03\x04",
        )
        .expect("ZIP response should be accepted");

        assert_eq!(kind, FetchedKind::Zip);
    }

    #[test]
    fn rejects_html_as_lua() {
        let err = classify_response(
            "https://example.test/730.lua",
            "text/html",
            b"<!doctype html>",
        )
        .expect_err("HTML should not be accepted as Lua");

        assert!(matches!(err, AttemptError::InvalidContent(_)));
    }
}
