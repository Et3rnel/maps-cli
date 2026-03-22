mod commands;

use anyhow::Result;
use clap::Parser;

use commands::Command;

#[derive(Parser, Debug)]
#[command(name = "maps-cli", author, version, about = "CLI for Google Maps APIs")]
struct Cli {
    #[arg(
        long,
        env = "GOOGLE_MAPS_API_KEY",
        hide_env_values = true,
        global = true
    )]
    api_key: String,

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
    cli.command.execute(&cli.api_key)
}
