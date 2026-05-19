use serde_json::{Value, json};
use url::Url;

use crate::payload::{parse_acquire, parse_read, parse_search};
use crate::refs::{RegistryResolveError, resolve_remote_registry_ref};
use crate::types::{
    AcquiredRegistrySkill, RegistrySearchResult, RegistrySkillDetail, ResolvedRegistryRef,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub body: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub trait Transport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, RegistryClientError>;
}

#[derive(Clone, Debug)]
pub struct ReqwestTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestTransport {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Default for ReqwestTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for ReqwestTransport {
    fn send(&self, request: HttpRequest) -> Result<HttpResponse, RegistryClientError> {
        let method = match request.method.as_str() {
            "GET" => reqwest::Method::GET,
            "POST" => reqwest::Method::POST,
            method => return Err(RegistryClientError::UnsupportedMethod(method.to_owned())),
        };
        let mut builder = self.client.request(method, &request.url);
        if let Some(body) = request.body {
            builder = builder
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body);
        }
        let response = builder.send()?;
        let status = response.status().as_u16();
        let body = response.text()?;
        Ok(HttpResponse { status, body })
    }
}

#[derive(Clone, Debug)]
pub struct RegistryClient<T = ReqwestTransport> {
    base_url: String,
    transport: T,
}

impl RegistryClient<ReqwestTransport> {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, RegistryClientError> {
        Self::with_transport(base_url, ReqwestTransport::new())
    }
}

impl<T: Transport> RegistryClient<T> {
    pub fn with_transport(
        base_url: impl AsRef<str>,
        transport: T,
    ) -> Result<Self, RegistryClientError> {
        let base_url = strip_one_trailing_slash(base_url.as_ref());
        Url::parse(&base_url)?;
        Ok(Self {
            base_url,
            transport,
        })
    }

    pub fn search(&self, query: &str) -> Result<Vec<RegistrySearchResult>, RegistryClientError> {
        self.search_with_limit(query, 20)
    }

    pub fn search_with_limit(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RegistrySearchResult>, RegistryClientError> {
        let mut url = Url::parse(&format!("{}/v1/skills", self.base_url))?;
        {
            let mut pairs = url.query_pairs_mut();
            let trimmed = query.trim();
            if !trimmed.is_empty() {
                pairs.append_pair("q", trimmed);
            }
            pairs.append_pair("limit", &limit.to_string());
        }
        let route = route_path(url.path(), url.query());
        let response = self.transport.send(HttpRequest {
            method: "GET".to_owned(),
            url: url.to_string(),
            body: None,
        })?;
        ensure_success(&route, response.status)?;
        let payload = json_body(&route, &response.body)?;
        parse_search(&route, &payload)
    }

    pub fn read(
        &self,
        skill_id: &str,
        version: Option<&str>,
    ) -> Result<Option<RegistrySkillDetail>, RegistryClientError> {
        let (owner, name) = split_skill_id(skill_id)?;
        let suffix = version
            .map(|version| format!("{name}@{version}"))
            .unwrap_or_else(|| name.to_owned());
        let route = format!(
            "/v1/skills/{}/{}",
            encode_segment(owner),
            encode_segment(&suffix)
        );
        let response = self.transport.send(HttpRequest {
            method: "GET".to_owned(),
            url: format!("{}{}", self.base_url, route),
            body: None,
        })?;
        if response.status == 404 {
            return Ok(None);
        }
        ensure_success(&route, response.status)?;
        let payload = json_body(&route, &response.body)?;
        parse_read(&route, &payload).map(Some)
    }

    pub fn acquire(
        &self,
        skill_id: &str,
        options: AcquireOptions<'_>,
    ) -> Result<AcquiredRegistrySkill, RegistryClientError> {
        if options.installation_id.trim().is_empty() {
            return Err(RegistryClientError::MissingInstallationId);
        }
        let (owner, name) = split_skill_id(skill_id)?;
        let route = format!(
            "/v1/skills/{}/{}/acquire",
            encode_segment(owner),
            encode_segment(name)
        );
        let channel = options.channel.unwrap_or("cli");
        let body = json!({
            "installation_id": options.installation_id,
            "version": options.version,
            "channel": channel,
        })
        .to_string();
        let response = self.transport.send(HttpRequest {
            method: "POST".to_owned(),
            url: format!("{}{}", self.base_url, route),
            body: Some(body),
        })?;
        ensure_success(&route, response.status)?;
        let payload = json_body(&route, &response.body)?;
        parse_acquire(&route, &payload)
    }

    pub fn resolve_ref(
        &self,
        registry_ref: &str,
        version_override: Option<&str>,
    ) -> Result<Option<ResolvedRegistryRef>, RegistryResolveError> {
        resolve_remote_registry_ref(self, registry_ref, version_override)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AcquireOptions<'a> {
    pub installation_id: &'a str,
    pub version: Option<&'a str>,
    pub channel: Option<&'a str>,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryClientError {
    #[error("invalid registry url: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("registry transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("unsupported HTTP method: {0}")]
    UnsupportedMethod(String),
    #[error("invalid registry skill id '{0}'. Expected '<owner>/<name>'.")]
    InvalidSkillId(String),
    #[error("registry route {route} failed with HTTP {status}")]
    HttpStatus { route: String, status: u16 },
    #[error("registry route {route} returned invalid JSON: {message}")]
    InvalidJson { route: String, message: String },
    #[error("registry route {route} contract error at {field_path}: {message}")]
    Contract {
        route: String,
        field_path: String,
        message: String,
    },
    #[error("remote registry installs require an installation id")]
    MissingInstallationId,
}

fn strip_one_trailing_slash(value: &str) -> String {
    value.strip_suffix('/').unwrap_or(value).to_owned()
}

fn ensure_success(route: &str, status: u16) -> Result<(), RegistryClientError> {
    if (200..=299).contains(&status) {
        Ok(())
    } else {
        Err(RegistryClientError::HttpStatus {
            route: route.to_owned(),
            status,
        })
    }
}

fn json_body(route: &str, body: &str) -> Result<Value, RegistryClientError> {
    serde_json::from_str(body).map_err(|error| RegistryClientError::InvalidJson {
        route: route.to_owned(),
        message: error.to_string(),
    })
}

fn split_skill_id(skill_id: &str) -> Result<(&str, &str), RegistryClientError> {
    let mut parts = skill_id.split('/');
    let owner = parts.next().unwrap_or_default();
    let name = parts.next().unwrap_or_default();
    if owner.is_empty()
        || name.is_empty()
        || is_dot_segment(owner)
        || is_dot_segment(name)
        || parts.next().is_some()
    {
        return Err(RegistryClientError::InvalidSkillId(skill_id.to_owned()));
    }
    Ok((owner, name))
}

fn is_dot_segment(value: &str) -> bool {
    matches!(value, "." | "..")
}

fn encode_segment(value: &str) -> String {
    let mut output = String::new();
    for byte in value.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'~');
        if keep {
            output.push(char::from(byte));
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

fn route_path(path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) if !query.is_empty() => format!("{path}?{query}"),
        _ => path.to_owned(),
    }
}
