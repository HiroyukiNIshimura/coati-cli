use std::{fs, io::Read};

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use reqwest::Method;

use crate::client::{Api, Request};

#[derive(ClapArgs)]
pub struct Target {
    /// URL or path (joined with the base URL when relative)
    url: String,
    /// Query parameter key=value (repeatable)
    #[arg(short, long = "query")]
    query: Vec<String>,
}

#[derive(ClapArgs)]
pub struct WithBody {
    /// URL or path (joined with the base URL when relative)
    url: String,
    /// Query parameter key=value (repeatable)
    #[arg(short, long = "query")]
    query: Vec<String>,
    /// Request body; use @file to read a file, or @- for stdin
    #[arg(short, long)]
    data: Option<String>,
    /// Content-Type of the body
    #[arg(long, default_value = "application/json")]
    content_type: String,
}

#[derive(Subcommand)]
pub enum Args {
    Get(Target),
    Head(Target),
    Delete(Target),
    Post(WithBody),
    Put(WithBody),
    Patch(WithBody),
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

fn parse_query(items: &[String]) -> Result<Vec<(String, String)>> {
    items
        .iter()
        .map(|q| {
            let (k, v) = q.split_once('=').with_context(|| format!("invalid query '{q}', expected key=value"))?;
            Ok((k.to_string(), v.to_string()))
        })
        .collect()
}

pub fn run(args: Args, api: &Api) -> Result<bool> {
    let simple = |m: Method, t: Target| -> Result<Request> {
        let mut r = Request::new(m, api.resolve(&t.url));
        r.query = parse_query(&t.query)?;
        Ok(r)
    };
    let with_body = |m: Method, b: WithBody| -> Result<Request> {
        let mut r = Request::new(m, api.resolve(&b.url));
        r.query = parse_query(&b.query)?;
        if let Some(d) = &b.data {
            r.body = Some((b.content_type.clone(), read_body(d)?));
        }
        Ok(r)
    };
    let req = match args {
        Args::Get(t) => simple(Method::GET, t)?,
        Args::Head(t) => simple(Method::HEAD, t)?,
        Args::Delete(t) => simple(Method::DELETE, t)?,
        Args::Post(b) => with_body(Method::POST, b)?,
        Args::Put(b) => with_body(Method::PUT, b)?,
        Args::Patch(b) => with_body(Method::PATCH, b)?,
    };
    api.call(req)
}
