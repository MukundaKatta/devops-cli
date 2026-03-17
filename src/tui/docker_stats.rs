use crate::errors::{DevToolError, Result};
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
    widgets::{Block, Borders, Cell, Gauge, Row, Table, Paragraph},
    Terminal,
};
use std::process::Command;

/// Container stats data parsed from `docker stats`.
struct ContainerStats {
    name: String,
    cpu_pct: String,
    mem_usage: String,
    mem_pct: String,
    net_io: String,
    block_io: String,
    pids: String,
}

fn fetch_stats() -> Result<Vec<ContainerStats>> {
    let output = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}\t{{.PIDs}}",
        ])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker stats: {}", e)))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let containers: Vec<ContainerStats> = text
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            ContainerStats {
                name: cols.first().unwrap_or(&"").to_string(),
                cpu_pct: cols.get(1).unwrap_or(&"0%").to_string(),
                mem_usage: cols.get(2).unwrap_or(&"").to_string(),
                mem_pct: cols.get(3).unwrap_or(&"0%").to_string(),
                net_io: cols.get(4).unwrap_or(&"").to_string(),
                block_io: cols.get(5).unwrap_or(&"").to_string(),
                pids: cols.get(6).unwrap_or(&"0").to_string(),
            }
        })
        .collect();

    Ok(containers)
}

fn parse_pct(s: &str) -> f64 {
    s.trim_end_matches('%').trim().parse::<f64>().unwrap_or(0.0)
}

/// Run the Docker stats TUI dashboard.
pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let containers = fetch_stats().unwrap_or_default();

        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(5),
                    Constraint::Min(10),
                ])
                .split(area);

            // Title
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    " Docker Dashboard ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" ({} containers) ", containers.len()),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    " (q: quit) ",
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
            f.render_widget(title, chunks[0]);

            // Summary gauges: total CPU and memory
            let total_cpu: f64 = containers.iter().map(|c| parse_pct(&c.cpu_pct)).sum();
            let avg_mem: f64 = if containers.is_empty() {
                0.0
            } else {
                containers.iter().map(|c| parse_pct(&c.mem_pct)).sum::<f64>()
                    / containers.len() as f64
            };

            let gauge_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let cpu_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" Total CPU ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .gauge_style(Style::default().fg(if total_cpu > 80.0 {
                    Color::Red
                } else if total_cpu > 50.0 {
                    Color::Yellow
                } else {
                    Color::Green
                }))
                .percent(total_cpu.min(100.0) as u16)
                .label(format!("{:.1}%", total_cpu));
            f.render_widget(cpu_gauge, gauge_chunks[0]);

            let mem_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" Avg Memory ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .gauge_style(Style::default().fg(if avg_mem > 80.0 {
                    Color::Red
                } else if avg_mem > 60.0 {
                    Color::Yellow
                } else {
                    Color::Green
                }))
                .percent(avg_mem.min(100.0) as u16)
                .label(format!("{:.1}%", avg_mem));
            f.render_widget(mem_gauge, gauge_chunks[1]);

            // Detailed table
            let header = Row::new(vec![
                Cell::from(Span::styled(
                    "CONTAINER",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled("CPU %", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("MEM USAGE", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("MEM %", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("NET I/O", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("BLOCK I/O", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Cell::from(Span::styled("PIDS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            ])
            .height(1);

            let rows: Vec<Row> = containers
                .iter()
                .map(|c| {
                    let cpu_val = parse_pct(&c.cpu_pct);
                    let cpu_style = if cpu_val > 80.0 {
                        Style::default().fg(Color::Red)
                    } else if cpu_val > 50.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    Row::new(vec![
                        Cell::from(c.name.as_str()),
                        Cell::from(c.cpu_pct.as_str()).style(cpu_style),
                        Cell::from(c.mem_usage.as_str()),
                        Cell::from(c.mem_pct.as_str()),
                        Cell::from(c.net_io.as_str()),
                        Cell::from(c.block_io.as_str()),
                        Cell::from(c.pids.as_str()),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Percentage(18),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(8),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .title(" Container Details ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .row_highlight_style(Style::default().bg(Color::DarkGray));
            f.render_widget(table, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_secs(2))? {
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
