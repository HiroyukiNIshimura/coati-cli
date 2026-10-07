//! One module per command category. To add a category:
//! 1. create `src/commands/<name>.rs` with an `Args` (clap Subcommand) enum and `run`,
//! 2. register it in `Cmd` and `run` below.

mod api;
mod item;
mod spec;
mod task;

use anyhow::Result;
use clap::Subcommand;

use crate::client::Api;

#[derive(Subcommand)]
pub enum Cmd {
    /// Raw HTTP requests (get/post/put/patch/delete/head)
    Api {
        #[command(subcommand)]
        cmd: api::Args,
    },
    /// Items in a workspace
    Item {
        #[command(subcommand)]
        cmd: item::Args,
    },
    /// Tasks of an item
    Task {
        #[command(subcommand)]
        cmd: task::Args,
    },
    /// Download the OpenAPI spec (agent knowledge)
    Spec(spec::Args),
}

/// Returns whether the command succeeded.
pub fn run(cmd: Cmd, api: &Api) -> Result<bool> {
    match cmd {
        Cmd::Api { cmd } => api::run(cmd, api),
        Cmd::Item { cmd } => item::run(cmd, api),
        Cmd::Task { cmd } => task::run(cmd, api),
        Cmd::Spec(a) => spec::run(a, api),
    }
}
