use anyhow::Result;
use clap::Subcommand;
use reqwest::Method;

use crate::client::{Api, Request};

#[derive(Subcommand)]
pub enum Args {
    /// List tasks of an item
    List {
        /// Workspace ID or code
        workspace: String,
        /// Item number
        item: String,
        #[arg(long)]
        is_completed: Option<bool>,
        #[arg(long)]
        is_discarded: Option<bool>,
    },
    /// Get a task
    Get {
        workspace: String,
        item: String,
        /// Task sequence number
        sequence: String,
    },
    /// List comments of a task
    Comments {
        workspace: String,
        item: String,
        sequence: String,
    },
}

pub fn run(args: Args, api: &Api) -> Result<bool> {
    let base = |ws: &str, item: &str| -> Vec<String> {
        ["api", "external", "workspaces", ws, "items", item, "tasks"].map(String::from).to_vec()
    };
    let endpoint = |segs: Vec<String>| api.endpoint(&segs.iter().map(String::as_str).collect::<Vec<_>>());
    let req = match args {
        Args::List { workspace, item, is_completed, is_discarded } => {
            Request::new(Method::GET, endpoint(base(&workspace, &item))?)
                .query_opt("isCompleted", is_completed)
                .query_opt("isDiscarded", is_discarded)
        }
        Args::Get { workspace, item, sequence } => {
            let mut s = base(&workspace, &item);
            s.push(sequence);
            Request::new(Method::GET, endpoint(s)?)
        }
        Args::Comments { workspace, item, sequence } => {
            let mut s = base(&workspace, &item);
            s.extend([sequence, "comments".into()]);
            Request::new(Method::GET, endpoint(s)?)
        }
    };
    api.call(req)
}
