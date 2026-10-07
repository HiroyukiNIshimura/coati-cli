use std::{fs, io::{IsTerminal, Read, Write}, path::PathBuf, process::ExitCode, time::Duration};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, Args};
use serde::{Deserialize, Serialize};
use reqwest::{Method, blocking::Client, header::{HeaderName, HeaderValue}};

#[derive(Parser)]
#[command(name = "coati", version, about = "REST API client CLI")]
struct Cli {
    /// Initialize (or re-initialize) the stored URL and API key
    #[arg(long)]
    init: bool,

    /// API base URL (overrides the stored config)
    #[arg(short, long, env = "COATI_BASE_URL", global = true)]
    base_url: Option<String>,

    /// API key sent as a Bearer token (overrides the stored config)
    #[arg(short = 'k', long, visible_alias = "token", env = "COATI_API_KEY", global = true, hide_env_values = true)]
    api_key: Option<String>,

    /// Extra header, e.g. -H "Accept: application/json" (repeatable)
    #[arg(short = 'H', long = "header", global = true)]
    headers: Vec<String>,

    /// Query parameter key=value (repeatable)
    #[arg(short, long = "query", global = true)]
    query: Vec<String>,

    /// Request timeout in seconds
    #[arg(long, default_value_t = 30, global = true)]
    timeout: u64,

    /// Skip TLS certificate verification (e.g. localhost dev certificates)
    #[arg(long, global = true)]
    insecure: bool,

    /// Print status line and response headers
    #[arg(short, long, global = true)]
    include: bool,

    /// Do not pretty-print JSON responses
    #[arg(long, global = true)]
    raw: bool,

    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Serialize, Deserialize)]
struct Config {
    url: String,
    api_key: String,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine the user home directory")?;
    Ok(home.join(".coati").join("config.toml"))
}

fn load_config() -> Result<Option<Config>> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
    let cfg = toml::from_str(&text).with_context(|| format!("invalid config {} (run `coati --init`)", path.display()))?;
    Ok(Some(cfg))
}

fn prompt(label: &str) -> Result<String> {
    eprint!("{label}");
    std::io::stderr().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

fn init_config() -> Result<Config> {
    if !std::io::stdin().is_terminal() {
        bail!("not initialized; run `coati --init` in an interactive terminal");
    }
    let url = prompt("API URL: ")?;
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        bail!("URL must start with http:// or https://");
    }
    let api_key = match rpassword::prompt_password("API key: ") {
        Ok(k) => k,
        Err(_) => {
            eprintln!("(no controlling terminal: the API key will be echoed)");
            prompt("API key: ")?
        }
    };
    let cfg = Config { url, api_key: api_key.trim().to_string() };

    let path = config_path()?;
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    fs::write(&path, toml::to_string_pretty(&cfg)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    eprintln!("Saved to {}", path.display());
    Ok(cfg)
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

#[derive(Args)]
struct SpecArgs {
    /// Output file
    #[arg(short, long, default_value = "docs/openapi.json")]
    output: PathBuf,
}

const SPEC_PATH: &str = "/api/external/openapi.json";

#[derive(Subcommand)]
enum Cmd {
    /// Download the API's OpenAPI spec (used as agent knowledge)
    Spec(SpecArgs),
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

fn run(mut cli: Cli) -> Result<bool> {
    let mut stored = None;
    if cli.init {
        stored = Some(init_config()?);
    }
    let Some(command) = cli.command.take() else {
        if cli.init {
            return Ok(true);
        }
        bail!("no command given; see `coati --help`");
    };
    if stored.is_none() && !(cli.base_url.is_some() && cli.api_key.is_some()) {
        stored = match load_config()? {
            Some(c) => Some(c),
            None => Some(init_config()?),
        };
    }
    if let Some(c) = stored {
        cli.base_url.get_or_insert(c.url);
        cli.api_key.get_or_insert(c.api_key);
    }

    let spec_out = match &command {
        Cmd::Spec(a) => Some(a.output.clone()),
        _ => None,
    };
    let (method, url, body) = match &command {
        Cmd::Spec(_) => (Method::GET, &SPEC_PATH.to_string(), None),
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
        .danger_accept_invalid_certs(cli.insecure)
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
    if let Some(t) = cli.api_key.as_deref().filter(|k| !k.is_empty()) {
        req = req.header("X-API-KEY", HeaderValue::from_str(t)?);
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
    if let Some(path) = spec_out {
        if !status.is_success() {
            bail!("failed to fetch spec: HTTP {status}");
        }
        let json: serde_json::Value = serde_json::from_str(&text).context("spec is not valid JSON")?;
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            fs::create_dir_all(dir)?;
        }
        fs::write(&path, serde_json::to_string_pretty(&json)? + "\n")?;
        eprintln!("Saved spec to {}", path.display());
        return Ok(true);
    }
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
