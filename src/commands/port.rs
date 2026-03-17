use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;
use std::process::Command;

#[derive(Args)]
pub struct PortArgs {
    #[command(subcommand)]
    command: PortCommand,
}

#[derive(Subcommand)]
pub enum PortCommand {
    /// Scan ports on a host
    Scan {
        /// Host to scan
        #[arg(default_value = "127.0.0.1")]
        host: String,
        /// Start port
        #[arg(short, long, default_value = "1")]
        start: u16,
        /// End port
        #[arg(short, long, default_value = "1024")]
        end: u16,
    },
    /// Kill process running on a specific port
    Kill {
        /// Port number
        port: u16,
    },
    /// List all listening ports
    List,
}

pub async fn run(args: PortArgs) -> Result<()> {
    match args.command {
        PortCommand::Scan { host, start, end } => scan_ports(&host, start, end).await,
        PortCommand::Kill { port } => kill_port(port),
        PortCommand::List => list_ports(),
    }
}

async fn scan_ports(host: &str, start: u16, end: u16) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    println!(
        "{} Scanning {}:{}-{}",
        style(">>").blue().bold(),
        host,
        start,
        end
    );

    let total = (end - start + 1) as u64;
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ports")
            .unwrap()
            .progress_chars("=>-"),
    );

    let mut open_ports = Vec::new();
    let host = host.to_string();

    for port in start..=end {
        let addr = format!("{}:{}", host, port);
        match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => {
                open_ports.push(port);
            }
            _ => {}
        }
        pb.inc(1);
    }

    pb.finish_and_clear();

    if open_ports.is_empty() {
        println!("{} No open ports found", style("info:").yellow().bold());
    } else {
        println!(
            "{} Found {} open port(s):\n",
            style(">>").green().bold(),
            open_ports.len()
        );
        for port in &open_ports {
            println!("  {} port {}", style("OPEN").green().bold(), style(port).cyan());
        }
    }

    Ok(())
}

fn kill_port(port: u16) -> Result<()> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-ti", &format!(":{}", port)])
            .output()?;

        let pids = String::from_utf8_lossy(&output.stdout);
        let pids: Vec<&str> = pids.trim().split('\n').filter(|s| !s.is_empty()).collect();

        if pids.is_empty() {
            println!(
                "{} No process found on port {}",
                style("info:").yellow().bold(),
                port
            );
            return Ok(());
        }

        for pid in &pids {
            let kill = Command::new("kill").args(["-9", pid]).output()?;

            if kill.status.success() {
                println!(
                    "{} Killed process {} on port {}",
                    style("done:").green().bold(),
                    style(pid).cyan(),
                    port
                );
            } else {
                eprintln!(
                    "{} Failed to kill process {}",
                    style("error:").red().bold(),
                    pid
                );
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(["-ano"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let port_str = format!(":{}", port);

        for line in stdout.lines() {
            if line.contains(&port_str) && line.contains("LISTENING") {
                if let Some(pid) = line.split_whitespace().last() {
                    let _ = Command::new("taskkill")
                        .args(["/PID", pid, "/F"])
                        .output()?;
                    println!(
                        "{} Killed process {} on port {}",
                        style("done:").green().bold(),
                        style(pid).cyan(),
                        port
                    );
                }
            }
        }
    }

    Ok(())
}

fn list_ports() -> Result<()> {
    #[cfg(unix)]
    {
        let output = Command::new("lsof")
            .args(["-i", "-P", "-n"])
            .output()
            .map_err(|e| DevToolError::CommandFailed(format!("lsof: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        println!(
            "{} Listening ports:\n",
            style(">>").blue().bold()
        );

        let mut found = false;
        for line in stdout.lines() {
            if line.contains("LISTEN") || line.starts_with("COMMAND") {
                println!("  {}", line);
                found = true;
            }
        }

        if !found {
            println!("  {}", style("No listening ports found").yellow());
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("netstat")
            .args(["-an"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!(
            "{} Listening ports:\n",
            style(">>").blue().bold()
        );
        for line in stdout.lines() {
            if line.contains("LISTENING") {
                println!("  {}", line);
            }
        }
    }

    Ok(())
}
