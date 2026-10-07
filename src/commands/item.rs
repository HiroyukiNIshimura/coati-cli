use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

use crate::client::{Api, Request};

#[derive(Subcommand)]
pub enum Args {
    /// List items in a workspace
    List {
        /// Workspace ID or code
        workspace: String,
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        is_active: Option<bool>,
        #[arg(long)]
        is_archived: Option<bool>,
        #[arg(long)]
        is_draft: Option<bool>,
    },
    /// Get an item
    Get {
        /// Workspace ID or code
        workspace: String,
        /// Item number
        number: String,
    },
}

pub fn run(args: Args, api: &Api) -> Result<bool> {
    let req = match args {
        Args::List { workspace, page, is_active, is_archived, is_draft } => {
            Request::new(Method::GET, api.endpoint(&["api", "external", "workspaces", &workspace, "items"])?)
                .query_opt("page", page)
                .query_opt("isActive", is_active)
                .query_opt("isArchived", is_archived)
                .query_opt("isDraft", is_draft)
        }
        Args::Get { workspace, number } => {
            Request::new(Method::GET, api.endpoint(&["api", "external", "workspaces", &workspace, "items", &number])?)
        }
    };
    api.call(req)
}
