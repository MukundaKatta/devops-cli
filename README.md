# DevOps CLI Toolkit

A fast, all-in-one command-line DevOps toolkit written in Rust.

## Features

- **Port Scanner** — Scan open ports on any host
- **HTTP Client** — Make requests with formatted, colorized output
- **Encode/Decode** — Base64, URL encoding, SHA-256, MD5, SHA-1, AES-GCM encryption
- **JSON Tools** — Pretty-print, query, diff, and validate JSON against schemas
- **Docker Utilities** — Container management with a live TUI stats dashboard
- **Git Helpers** — Repository insights and shortcuts
- **Env Manager** — View, diff, and validate environment variable files
- **File Server** — Spin up a local HTTP server with directory listing
- **Benchmarking** — HTTP endpoint benchmarking with statistics
- **Cron Builder** — Interactive TUI for building and validating cron expressions
- **System Monitor** — Live TUI dashboard for CPU, memory, disk, and network
- **QR Code** — Generate QR codes directly in the terminal

## Tech Stack

- **Language:** Rust (2021 edition)
- **CLI Framework:** clap (derive) with shell completions
- **Async Runtime:** Tokio
- **HTTP:** reqwest (rustls-tls)
- **TUI:** Ratatui + Crossterm
- **Serialization:** serde (JSON, YAML, TOML)
- **Crypto:** aes-gcm, sha2, md-5, base64
- **Testing:** assert_cmd + predicates

## Getting Started

### Prerequisites

- Rust 1.75+ and Cargo

### Installation

```bash
# From source
git clone <repo-url>
cd devops-cli
cargo build --release
cp target/release/devtool /usr/local/bin/

# Or use the install script
curl -sSf <repo-url>/install.sh | sh
```

### Usage

```bash
devtool port scan 192.168.1.1
devtool http get https://api.example.com
devtool encode base64 "hello world"
devtool json pretty input.json
devtool docker stats          # opens TUI dashboard
devtool env diff .env.dev .env.prod
devtool serve .               # serve current directory
devtool bench https://api.example.com -n 100
devtool cron build            # interactive cron builder
devtool system monitor        # live system stats TUI
```

## Project Structure

```
src/
├── main.rs              # Entry point
├── cli.rs               # Clap CLI definition and argument parsing
├── config.rs            # Configuration loading
├── errors.rs            # Custom error types
├── commands/
│   ├── port.rs          # Port scanning
│   ├── http.rs          # HTTP client
│   ├── encode.rs        # Encoding and hashing
│   ├── json.rs          # JSON tools
│   ├── docker.rs        # Docker management
│   ├── git.rs           # Git helpers
│   ├── env.rs           # Env file management
│   ├── serve.rs         # Local file server
│   ├── bench.rs         # HTTP benchmarking
│   ├── cron.rs          # Cron utilities
│   └── system.rs        # System info
├── tui/                 # Ratatui TUI dashboards
│   ├── docker_stats.rs
│   ├── system_monitor.rs
│   └── cron_builder.rs
└── utils/               # Colors, spinner, table, clipboard helpers
tests/
├── integration.rs       # Integration test harness
├── integration/         # Per-command integration tests
└── fixtures/            # Sample JSON, env, and schema files
```

## License

MIT
