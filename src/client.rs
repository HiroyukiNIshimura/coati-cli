use std::time::Duration;

use anyhow::{Context, Result, bail};
use reqwest::{
    Method, Url,
    blocking::Client,
    header::{HeaderName, HeaderValue},
};

pub struct Options {
    pub base_url: String,
    pub api_key: String,
    pub headers: Vec<String>,
    pub timeout: u64,
    pub insecure: bool,
    pub include: bool,
    pub raw: bool,
}

pub struct Request {
    pub method: Method,
    pub url: String,
    pub query: Vec<(String, String)>,
    pub body: Option<(String, String)>,
}

impl Request {
    pub fn new(method: Method, url: impl Into<String>) -> Self {
        Self { method, url: url.into(), query: Vec::new(), body: None }
    }

    /// Adds a query parameter only when a value is present.
    pub fn query_opt(mut self, key: &str, value: Option<impl ToString>) -> Self {
        if let Some(v) = value {
            self.query.push((key.to_string(), v.to_string()));
        }
        self
    }
}

pub struct Reply {
    pub status: reqwest::StatusCode,
    pub text: String,
}

impl Reply {
    pub fn ok(&self) -> bool {
        self.status.is_success() || self.status.is_redirection()
    }
}

pub struct Api {
    http: Client,
    opts: Options,
}

impl Api {
    pub fn new(opts: Options) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(opts.timeout))
            .danger_accept_invalid_certs(opts.insecure)
            .user_agent(concat!("coati/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self { http, opts })
    }

    /// Builds an absolute URL from path segments; each segment is percent-encoded.
    pub fn endpoint(&self, segments: &[&str]) -> Result<String> {
        let mut url = Url::parse(&self.opts.base_url).context("invalid base URL")?;
        url.path_segments_mut()
            .map_err(|_| anyhow::anyhow!("base URL cannot be a base"))?
            .pop_if_empty()
            .extend(segments);
        Ok(url.into())
    }

    /// Resolves a user-supplied URL or path against the base URL.
    pub fn resolve(&self, url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("{}/{}", self.opts.base_url.trim_end_matches('/'), url.trim_start_matches('/'))
        }
    }

    pub fn send(&self, r: Request) -> Result<Reply> {
        let mut req = self.http.request(r.method, &r.url);
        if !r.query.is_empty() {
            req = req.query(&r.query);
        }
        for h in &self.opts.headers {
            let (k, v) = h
                .split_once(':')
                .with_context(|| format!("invalid header '{h}', expected 'Name: value'"))?;
            req = req.header(HeaderName::from_bytes(k.trim().as_bytes())?, HeaderValue::from_str(v.trim())?);
        }
        if !self.opts.api_key.is_empty() {
            req = req.header("X-API-KEY", HeaderValue::from_str(&self.opts.api_key)?);
        }
        if let Some((content_type, body)) = r.body {
            req = req.header("Content-Type", content_type).body(body);
        }

        let resp = req.send().with_context(|| format!("request to {} failed", r.url))?;
        let status = resp.status();
        if self.opts.include {
            eprintln!("{:?} {}", resp.version(), status);
            for (k, v) in resp.headers() {
                eprintln!("{}: {}", k, v.to_str().unwrap_or("<binary>"));
            }
            eprintln!();
        }
        Ok(Reply { status, text: resp.text()? })
    }

    /// Prints the response body (pretty-printed JSON unless --raw) and returns whether it succeeded.
    pub fn emit(&self, reply: Reply) -> bool {
        let ok = reply.ok();
        let out = if self.opts.raw {
            reply.text
        } else {
            serde_json::from_str::<serde_json::Value>(&reply.text)
                .ok()
                .and_then(|v| serde_json::to_string_pretty(&v).ok())
                .unwrap_or(reply.text)
        };
        if !out.is_empty() {
            println!("{out}");
        }
        ok
    }

    /// Sends the request and prints the response.
    pub fn call(&self, r: Request) -> Result<bool> {
        Ok(self.emit(self.send(r)?))
    }
}

pub fn ensure_success(reply: &Reply, what: &str) -> Result<()> {
    if !reply.status.is_success() {
        bail!("{what}: HTTP {}", reply.status);
    }
    Ok(())
}
