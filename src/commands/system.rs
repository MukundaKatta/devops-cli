use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;
use sysinfo::System;

#[derive(Args)]
pub struct SystemArgs {
    #[command(subcommand)]
    command: SystemCommand,
}

#[derive(Subcommand)]
pub enum SystemCommand {
    /// Show system information (OS, CPU, memory, disk)
    Info,
    /// Live system monitor TUI (CPU, memory, processes)
    Monitor,
    /// Show all IP addresses (local, public, interfaces)
    Ip,
}

pub async fn run(args: SystemArgs) -> Result<()> {
    match args.command {
        SystemCommand::Info => info(),
        SystemCommand::Monitor => monitor(),
        SystemCommand::Ip => ip_addresses().await,
    }
}

fn info() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    println!(
        "\n{} System Information\n",
        style(">>").blue().bold()
    );

    // OS info
    println!(
        "  {} {}",
        style("OS:").cyan().bold(),
        System::long_os_version().unwrap_or_else(|| "Unknown".to_string())
    );
    println!(
        "  {} {}",
        style("Kernel:").cyan().bold(),
        System::kernel_version().unwrap_or_else(|| "Unknown".to_string())
    );
    println!(
        "  {} {}",
        style("Hostname:").cyan().bold(),
        System::host_name().unwrap_or_else(|| "Unknown".to_string())
    );

    // CPU info
    let cpus = sys.cpus();
    if let Some(cpu) = cpus.first() {
        println!(
            "  {} {} ({} cores)",
            style("CPU:").cyan().bold(),
            cpu.brand(),
            cpus.len()
        );
    }

    let cpu_usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len().max(1) as f32;
    println!(
        "  {} {:.1}%",
        style("CPU Usage:").cyan().bold(),
        cpu_usage
    );

    // Memory
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let mem_pct = if total_mem > 0 {
        (used_mem as f64 / total_mem as f64) * 100.0
    } else {
        0.0
    };
    println!(
        "  {} {} / {} ({:.1}%)",
        style("Memory:").cyan().bold(),
        format_bytes(used_mem),
        format_bytes(total_mem),
        mem_pct
    );

    // Swap
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();
    println!(
        "  {} {} / {}",
        style("Swap:").cyan().bold(),
        format_bytes(used_swap),
        format_bytes(total_swap)
    );

    // Uptime
    let uptime = System::uptime();
    let days = uptime / 86400;
    let hours = (uptime % 86400) / 3600;
    let minutes = (uptime % 3600) / 60;
    println!(
        "  {} {}d {}h {}m",
        style("Uptime:").cyan().bold(),
        days,
        hours,
        minutes
    );

    // Disks
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    if !disks.list().is_empty() {
        println!(
            "\n  {}",
            style("Disks:").cyan().bold()
        );
        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total.saturating_sub(available);
            let pct = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            println!(
                "    {} {} / {} ({:.1}%) [{}]",
                style(disk.mount_point().display()).yellow(),
                format_bytes(used),
                format_bytes(total),
                pct,
                disk.file_system().to_string_lossy()
            );
        }
    }

    // Process count
    println!(
        "\n  {} {}",
        style("Processes:").cyan().bold(),
        sys.processes().len()
    );

    println!();
    Ok(())
}

fn monitor() -> Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Gauge, Paragraph, Row, Table, Cell},
        Terminal,
    };

    let mut sys = System::new_all();

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        sys.refresh_all();

        let cpus = sys.cpus();
        let avg_cpu: f64 = cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cpus.len().max(1) as f64;

        let total_mem = sys.total_memory();
        let used_mem = sys.used_memory();
        let mem_pct = if total_mem > 0 {
            (used_mem as f64 / total_mem as f64) * 100.0
        } else {
            0.0
        };

        let total_swap = sys.total_swap();
        let used_swap = sys.used_swap();
        let swap_pct = if total_swap > 0 {
            (used_swap as f64 / total_swap as f64) * 100.0
        } else {
            0.0
        };

        // Top processes by CPU
        let mut procs: Vec<_> = sys.processes().values().collect();
        procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));
        let top_procs: Vec<_> = procs.iter().take(15).collect();

        terminal.draw(|f| {
            let area = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(3),  // CPU gauge
                    Constraint::Length(3),  // Memory gauge
                    Constraint::Length(3),  // Swap gauge
                    Constraint::Min(10),   // Process table
                ])
                .split(area);

            // Title
            let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" System Monitor - {} ", hostname),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " (q: quit) ",
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(title, main_chunks[0]);

            // CPU Gauge
            let cpu_color = if avg_cpu > 80.0 {
                Color::Red
            } else if avg_cpu > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            let cpu_gauge = Gauge::default()
                .block(Block::default().title(" CPU ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
                .gauge_style(Style::default().fg(cpu_color))
                .percent(avg_cpu.min(100.0) as u16)
                .label(format!("{:.1}% ({} cores)", avg_cpu, cpus.len()));
            f.render_widget(cpu_gauge, main_chunks[1]);

            // Memory Gauge
            let mem_color = if mem_pct > 80.0 {
                Color::Red
            } else if mem_pct > 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            let mem_gauge = Gauge::default()
                .block(Block::default().title(" Memory ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
                .gauge_style(Style::default().fg(mem_color))
                .percent(mem_pct.min(100.0) as u16)
                .label(format!(
                    "{} / {} ({:.1}%)",
                    format_bytes(used_mem),
                    format_bytes(total_mem),
                    mem_pct
                ));
            f.render_widget(mem_gauge, main_chunks[2]);

            // Swap Gauge
            let swap_gauge = Gauge::default()
                .block(Block::default().title(" Swap ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)))
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(swap_pct.min(100.0) as u16)
                .label(format!(
                    "{} / {} ({:.1}%)",
                    format_bytes(used_swap),
                    format_bytes(total_swap),
                    swap_pct
                ));
            f.render_widget(swap_gauge, main_chunks[3]);

            // Process Table
            let header = Row::new(vec![
                Cell::from(Span::styled("PID", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("NAME", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("CPU %", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("MEMORY", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("STATUS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            ]);

            let rows: Vec<Row> = top_procs
                .iter()
                .map(|p| {
                    let cpu_style = if p.cpu_usage() > 50.0 {
                        Style::default().fg(Color::Red)
                    } else if p.cpu_usage() > 20.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    Row::new(vec![
                        Cell::from(p.pid().to_string()),
                        Cell::from(p.name().to_string_lossy().to_string()),
                        Cell::from(format!("{:.1}", p.cpu_usage())).style(cpu_style),
                        Cell::from(format_bytes(p.memory())),
                        Cell::from(format!("{:?}", p.status())),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Length(8),
                Constraint::Percentage(35),
                Constraint::Length(10),
                Constraint::Length(12),
                Constraint::Length(12),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .title(format!(" Top Processes ({} total) ", sys.processes().len()))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );
            f.render_widget(table, main_chunks[4]);
        })?;

        if event::poll(std::time::Duration::from_secs(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn ip_addresses() -> Result<()> {
    println!(
        "\n{} IP Addresses\n",
        style(">>").blue().bold()
    );

    // Local IP
    match local_ip_address::local_ip() {
        Ok(ip) => {
            println!(
                "  {} {}",
                style("Local IP:").cyan().bold(),
                style(ip).green()
            );
        }
        Err(e) => {
            println!(
                "  {} Could not determine local IP: {}",
                style("Local IP:").cyan().bold(),
                style(e).red()
            );
        }
    }

    // All network interfaces
    match local_ip_address::list_afinet_netifas() {
        Ok(interfaces) => {
            println!(
                "\n  {}",
                style("Network Interfaces:").cyan().bold()
            );
            let mut sorted = interfaces;
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, ip) in &sorted {
                let ip_style = if ip.is_loopback() {
                    style(ip.to_string()).dim()
                } else {
                    style(ip.to_string()).green()
                };
                println!("    {:<20} {}", style(name).yellow(), ip_style);
            }
        }
        Err(e) => {
            println!(
                "  {} Could not list interfaces: {}",
                style("warn:").yellow().bold(),
                e
            );
        }
    }

    // Public IP (via external service)
    println!(
        "\n  {} Fetching...",
        style("Public IP:").cyan().bold()
    );

    match reqwest::get("https://api.ipify.org").await {
        Ok(resp) => {
            if let Ok(ip) = resp.text().await {
                // Move cursor up one line and overwrite
                print!("\x1b[1A\x1b[2K");
                println!(
                    "  {} {}",
                    style("Public IP:").cyan().bold(),
                    style(ip.trim()).green().bold()
                );
            }
        }
        Err(_) => {
            print!("\x1b[1A\x1b[2K");
            println!(
                "  {} {} (could not reach api.ipify.org)",
                style("Public IP:").cyan().bold(),
                style("N/A").dim()
            );
        }
    }

    println!();
    Ok(())
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }
}
