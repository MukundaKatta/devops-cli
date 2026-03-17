mod cli;
mod commands;
mod config;
mod errors;
mod tui;
mod utils;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands};
use std::io;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Port(args) => commands::port::run(args).await,
        Commands::Http(args) => commands::http::run(args).await,
        Commands::Encode(args) => commands::encode::run(args),
        Commands::Json(args) => commands::json::run(args),
        Commands::Docker(args) => commands::docker::run(args).await,
        Commands::Git(args) => commands::git::run(args),
        Commands::Env(args) => commands::env::run(args),
        Commands::Serve(args) => commands::serve::run(args).await,
        Commands::Bench(args) => commands::bench::run(args).await,
        Commands::Cron(args) => commands::cron::run(args),
        Commands::System(args) => commands::system::run(args).await,
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "devtool", &mut io::stdout());
            Ok(())
        }
    };

    if let Err(e) = result {
        use console::style;
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
