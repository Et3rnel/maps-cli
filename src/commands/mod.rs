pub mod text_search;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search for places using a text query
    TextSearch(text_search::Args),
}

impl Command {
    pub fn execute(&self, api_key: &str) -> Result<()> {
        match self {
            Command::TextSearch(args) => text_search::run(api_key, args),
        }
    }
}
