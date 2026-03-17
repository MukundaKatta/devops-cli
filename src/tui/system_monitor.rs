use crate::errors::Result;
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
    widgets::{Block, Borders, Cell, Gauge, Paragraph, Row, Table, Sparkline},
    Terminal,
};
use sysinfo::System;

/// Run the system monitor TUI with CPU history sparkline.
pub fn run() -> Result<()> {
    let mut sys = System::new_all();

    // CPU history for sparkline (last 60 samples)
    let mut cpu_history: Vec<u64> = Vec::with_capacity(60);

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        sys.refresh_all();

        let cpus = sys.cpus();
        let avg_cpu: f64 =
            cpus.iter().map(|c| c.cpu_usage() as f64).sum::<f64>() / cpus.len().max(1) as f64;

        // Track history
        cpu_history.push(avg_cpu.min(100.0) as u64);
        if cpu_history.len() > 60 {
            cpu_history.remove(0);
        }

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
        procs.sort_by(|a, b| {
            b.cpu_usage()
                .partial_cmp(&a.cpu_usage())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let top_procs: Vec<_> = procs.into_iter().take(12).collect();

        terminal.draw(|f| {
            let area = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(5),  // CPU sparkline
                    Constraint::Length(3),  // CPU gauge
                    Constraint::Length(3),  // Memory gauge
                    Constraint::Length(3),  // Swap gauge
                    Constraint::Min(8),    // Process table
                ])
                .split(area);

            // Title
            let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
            let os = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", hostname),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("- {} ", os),
                    Style::default().fg(Color::DarkGray),
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
            f.render_widget(title, main_chunks[0]);

            // CPU Sparkline
            let sparkline = Sparkline::default()
                .block(
                    Block::default()
                        .title(format!(" CPU History (avg {:.1}%) ", avg_cpu))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .data(&cpu_history)
                .max(100)
                .style(Style::default().fg(Color::Cyan));
            f.render_widget(sparkline, main_chunks[1]);

            // CPU Gauge
            let cpu_color = if avg_cpu > 80.0 {
                Color::Red
            } else if avg_cpu > 50.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            let cpu_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" CPU ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .gauge_style(Style::default().fg(cpu_color))
                .percent(avg_cpu.min(100.0) as u16)
                .label(format!("{:.1}% ({} cores)", avg_cpu, cpus.len()));
            f.render_widget(cpu_gauge, main_chunks[2]);

            // Memory Gauge
            let mem_color = if mem_pct > 80.0 {
                Color::Red
            } else if mem_pct > 60.0 {
                Color::Yellow
            } else {
                Color::Green
            };
            let mem_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" Memory ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .gauge_style(Style::default().fg(mem_color))
                .percent(mem_pct.min(100.0) as u16)
                .label(format!(
                    "{} / {} ({:.1}%)",
                    format_bytes(used_mem),
                    format_bytes(total_mem),
                    mem_pct
                ));
            f.render_widget(mem_gauge, main_chunks[3]);

            // Swap Gauge
            let swap_gauge = Gauge::default()
                .block(
                    Block::default()
                        .title(" Swap ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .gauge_style(Style::default().fg(Color::Cyan))
                .percent(swap_pct.min(100.0) as u16)
                .label(format!(
                    "{} / {} ({:.1}%)",
                    format_bytes(used_swap),
                    format_bytes(total_swap),
                    swap_pct
                ));
            f.render_widget(swap_gauge, main_chunks[4]);

            // Process table
            let header = Row::new(vec![
                Cell::from(Span::styled(
                    "PID",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "NAME",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "CPU %",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
                Cell::from(Span::styled(
                    "MEMORY",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )),
            ]);

            let rows: Vec<Row> = top_procs
                .iter()
                .map(|p| {
                    let cpu_s = if p.cpu_usage() > 50.0 {
                        Style::default().fg(Color::Red)
                    } else if p.cpu_usage() > 20.0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Green)
                    };

                    Row::new(vec![
                        Cell::from(p.pid().to_string()),
                        Cell::from(p.name().to_string_lossy().to_string()),
                        Cell::from(format!("{:.1}", p.cpu_usage())).style(cpu_s),
                        Cell::from(format_bytes(p.memory())),
                    ])
                })
                .collect();

            let widths = [
                Constraint::Length(8),
                Constraint::Percentage(50),
                Constraint::Length(10),
                Constraint::Length(12),
            ];

            let table = Table::new(rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .title(format!(
                            " Top Processes ({} total) ",
                            sys.processes().len()
                        ))
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );
            f.render_widget(table, main_chunks[5]);
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
