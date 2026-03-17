use clap::{Parser, Subcommand};
use clap_complete::Shell;

use crate::commands;

#[derive(Parser)]
#[command(
    name = "devtool",
    version,
    about = "A complete CLI DevOps toolkit",
    long_about = "DevTool - A Swiss Army knife for developers.\nPort scanning, HTTP client, encoding, JSON processing, Docker management, Git utilities, and more."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Port scanning, killing processes on ports, listing listeners
    Port(commands::port::PortArgs),
    /// HTTP client with pretty-printed responses and timing
    Http(commands::http::HttpArgs),
    /// Encode/decode: base64, URL, JWT, hashes
    Encode(commands::encode::EncodeArgs),
    /// JSON formatting, minifying, querying, diffing, validation
    Json(commands::json::JsonArgs),
    /// Docker utilities: clean, stats, logs, size
    Docker(commands::docker::DockerArgs),
    /// Git utilities: summary, changelog, branch-clean, undo
    Git(commands::git::GitArgs),
    /// Environment file management: diff, validate, generate, encrypt
    Env(commands::env::EnvArgs),
    /// Static file server with directory listing and CORS
    Serve(commands::serve::ServeArgs),
    /// HTTP benchmarking with latency percentiles
    Bench(commands::bench::BenchArgs),
    /// Cron expression utilities: explain, next runs, builder
    Cron(commands::cron::CronArgs),
    /// System information, live monitor, IP addresses
    System(commands::system::SystemArgs),
    /// Generate shell completions
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: Shell,
    },
}
