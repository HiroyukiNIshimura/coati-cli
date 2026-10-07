use std::{fs, io::{IsTerminal, Write}, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub url: String,
    pub api_key: String,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot determine the user home directory")?;
    Ok(home.join(".coati").join("config.toml"))
}

pub fn load() -> Result<Option<Config>> {
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

pub fn init() -> Result<Config> {
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
