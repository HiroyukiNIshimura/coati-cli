use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Args as ClapArgs;
use reqwest::Method;

use crate::client::{Api, Request, ensure_success};

#[derive(ClapArgs)]
pub struct Args {
    /// Output file
    #[arg(short, long, default_value = "docs/openapi.json")]
    output: PathBuf,
}

pub fn run(args: Args, api: &Api) -> Result<bool> {
    let reply = api.send(Request::new(Method::GET, api.endpoint(&["api", "external", "openapi.json"])?))?;
    ensure_success(&reply, "failed to fetch spec")?;
    let json: serde_json::Value = serde_json::from_str(&reply.text).context("spec is not valid JSON")?;
    if let Some(dir) = args.output.parent().filter(|d| !d.as_os_str().is_empty()) {
        fs::create_dir_all(dir)?;
    }
    fs::write(&args.output, serde_json::to_string_pretty(&json)? + "\n")?;
    eprintln!("Saved spec to {}", args.output.display());
    Ok(true)
}
