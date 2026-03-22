mod commands;

use anyhow::{Result, bail};
use clap::Parser;

use commands::Command;

#[derive(Parser, Debug)]
#[command(name = "maps-cli", author, version, about = "CLI for Google Maps APIs")]
struct Cli {
    #[arg(long, env = "GOOGLE_MAPS_API_KEY", hide_env_values = true)]
    api_key: Option<String>,

    #[command(subcommand)]
    command: Command,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let api_key = cli.api_key.as_deref().filter(|k| !k.is_empty());

    let Some(api_key) = api_key else {
        bail!("API key required: set GOOGLE_MAPS_API_KEY or pass --api-key");
    };

    cli.command.execute(api_key)
}
