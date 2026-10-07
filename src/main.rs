use std::{fs, io::Read, process::ExitCode, time::Duration};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, Args};
use reqwest::{Method, blocking::Client, header::{HeaderName, HeaderValue}};

#[derive(Parser)]
#[command(name = "coati", version, about = "REST API client CLI")]
struct Cli {
    /// Base URL prepended to relative paths
    #[arg(short, long, env = "COATI_BASE_URL", global = true)]
    base_url: Option<String>,

    /// Bearer token for the Authorization header
    #[arg(short, long, env = "COATI_TOKEN", global = true, hide_env_values = true)]
    token: Option<String>,

    /// Extra header, e.g. -H "Accept: application/json" (repeatable)
    #[arg(short = 'H', long = "header", global = true)]
    headers: Vec<String>,

    /// Query parameter key=value (repeatable)
    #[arg(short, long = "query", global = true)]
    query: Vec<String>,

    /// Request timeout in seconds
    #[arg(long, default_value_t = 30, global = true)]
    timeout: u64,

    /// Print status line and response headers
    #[arg(short, long, global = true)]
    include: bool,

    /// Do not pretty-print JSON responses
    #[arg(long, global = true)]
    raw: bool,

    #[command(subcommand)]
    command: Cmd,
}

#[derive(Args)]
struct Target {
    /// URL or path (joined with --base-url when relative)
    url: String,
}

#[derive(Args)]
struct WithBody {
    /// URL or path (joined with --base-url when relative)
    url: String,
    /// Request body; use @file to read a file, or @- for stdin
    #[arg(short, long)]
    data: Option<String>,
    /// Content-Type of the body
    #[arg(long, default_value = "application/json")]
    content_type: String,
}

#[derive(Subcommand)]
enum Cmd {
    /// Send a GET request
    Get(Target),
    /// Send a HEAD request
    Head(Target),
    /// Send a DELETE request
    Delete(Target),
    /// Send a POST request
    Post(WithBody),
    /// Send a PUT request
    Put(WithBody),
    /// Send a PATCH request
    Patch(WithBody),
}

fn build_url(base: Option<&str>, url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        return Ok(url.to_string());
    }
    match base {
        Some(b) => Ok(format!("{}/{}", b.trim_end_matches('/'), url.trim_start_matches('/'))),
        None => bail!("relative path '{url}' requires --base-url or COATI_BASE_URL"),
    }
}

fn read_body(data: &str) -> Result<String> {
    match data.strip_prefix('@') {
        Some("-") => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
        Some(path) => fs::read_to_string(path).with_context(|| format!("cannot read {path}")),
        None => Ok(data.to_string()),
    }
}

fn run(cli: Cli) -> Result<bool> {
    let (method, url, body) = match &cli.command {
        Cmd::Get(t) => (Method::GET, &t.url, None),
        Cmd::Head(t) => (Method::HEAD, &t.url, None),
        Cmd::Delete(t) => (Method::DELETE, &t.url, None),
        Cmd::Post(b) => (Method::POST, &b.url, Some(b)),
        Cmd::Put(b) => (Method::PUT, &b.url, Some(b)),
        Cmd::Patch(b) => (Method::PATCH, &b.url, Some(b)),
    };
    let url = build_url(cli.base_url.as_deref(), url)?;

    let client = Client::builder()
        .timeout(Duration::from_secs(cli.timeout))
        .user_agent(concat!("coati/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut req = client.request(method, &url);

    for q in &cli.query {
        let (k, v) = q.split_once('=').with_context(|| format!("invalid query '{q}', expected key=value"))?;
        req = req.query(&[(k, v)]);
    }
    for h in &cli.headers {
        let (k, v) = h.split_once(':').with_context(|| format!("invalid header '{h}', expected 'Name: value'"))?;
        req = req.header(HeaderName::from_bytes(k.trim().as_bytes())?, HeaderValue::from_str(v.trim())?);
    }
    if let Some(t) = &cli.token {
        req = req.bearer_auth(t);
    }
    if let Some(b) = body {
        if let Some(d) = &b.data {
            req = req.header("Content-Type", &b.content_type).body(read_body(d)?);
        }
    }

    let resp = req.send().with_context(|| format!("request to {url} failed"))?;
    let status = resp.status();
    if cli.include {
        eprintln!("{:?} {}", resp.version(), status);
        for (k, v) in resp.headers() {
            eprintln!("{}: {}", k, v.to_str().unwrap_or("<binary>"));
        }
        eprintln!();
    }
    let text = resp.text()?;
    let out = if cli.raw {
        text
    } else {
        serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or(text)
    };
    if !out.is_empty() {
        println!("{out}");
    }
    Ok(status.is_success() || status.is_redirection())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}
