# devops-cli

A Rust CLI DevOps toolkit with subcommands for port scanning, HTTP requests, JSON processing, Docker operations, and more.

## What's actually here

This repo has two disconnected parts:

**Rust CLI (the real project)** - A Cargo-based Rust application using clap for argument parsing and tokio for async. The main.rs dispatches to 11 subcommands: port, http, encode, json, docker, git, env, serve, bench, cron, and system. There are also modules for TUI, config, errors, and utils. Whether the individual command implementations are fully functional or partially stubbed has not been verified, but the overall structure is real Rust code with proper module organization.

**src/core.py (unrelated stub)** - A Python file with a cookie-cutter `DevopsCli` class containing stub methods (detect, scan, monitor, alert, get_report, configure) that return `{"ok": True}`. This has nothing to do with the Rust CLI and was added separately.

The previous README incorrectly showed Python install/usage instructions (`pip install`, `from src.core import DevopsCli`) for what is actually a Rust project built with Cargo.

## Structure

- `Cargo.toml` - Rust project config
- `src/main.rs` - Entry point with clap CLI parsing
- `src/cli.rs` - CLI argument definitions
- `src/commands/` - Subcommand implementations
- `src/tui/` - Terminal UI components
- `src/utils/` - Utility modules
- `src/config.rs` - Configuration handling
- `src/errors.rs` - Error types
- `install.sh` - Install script

## Build

```
cargo build --release
```

## Status

The Rust CLI structure appears substantive. The Python core.py stub is unrelated filler.
