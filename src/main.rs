mod client;
mod commands;
mod config;

use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::Parser;

use client::{Api, Options};
use commands::Cmd;

#[derive(Parser)]
#[command(name = "coati", version, about = "Coati External API client")]
struct Cli {
    /// Initialize (or re-initialize) the stored URL and API key
    #[arg(long)]
    init: bool,

    /// API base URL (overrides the stored config)
    #[arg(short, long, env = "COATI_BASE_URL", global = true)]
    base_url: Option<String>,

    /// API key sent as the X-API-KEY header (overrides the stored config)
    #[arg(short = 'k', long, visible_alias = "token", env = "COATI_API_KEY", global = true, hide_env_values = true)]
    api_key: Option<String>,

    /// Extra header, e.g. -H "Accept: application/json" (repeatable)
    #[arg(short = 'H', long = "header", global = true)]
    headers: Vec<String>,

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

fn run(cli: Cli) -> Result<bool> {
    let mut stored = if cli.init { Some(config::init()?) } else { None };
    let Some(command) = cli.command else {
        if cli.init {
            return Ok(true);
        }
        bail!("no command given; see `coati --help`");
    };
    if stored.is_none() && !(cli.base_url.is_some() && cli.api_key.is_some()) {
        stored = Some(match config::load()? {
            Some(c) => c,
            None => config::init()?,
        });
    }
    let (stored_url, stored_key) = stored.map(|c| (Some(c.url), Some(c.api_key))).unwrap_or_default();

    let api = Api::new(Options {
        base_url: cli.base_url.or(stored_url).unwrap_or_default(),
        api_key: cli.api_key.or(stored_key).unwrap_or_default(),
        headers: cli.headers,
        timeout: cli.timeout,
        insecure: cli.insecure,
        include: cli.include,
        raw: cli.raw,
    })?;
    commands::run(command, &api)
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
