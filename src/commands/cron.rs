use crate::errors::{DevToolError, Result};
use chrono::Utc;
use clap::{Args, Subcommand};
use console::style;
use cron::Schedule;
use std::str::FromStr;

#[derive(Args)]
pub struct CronArgs {
    #[command(subcommand)]
    command: CronCommand,
}

#[derive(Subcommand)]
pub enum CronCommand {
    /// Explain a cron expression in human-readable form
    Explain {
        /// Cron expression (5 or 7 fields, quote it!)
        expression: String,
    },
    /// Show the next N execution times for a cron expression
    Next {
        /// Cron expression
        expression: String,
        /// Number of executions to show
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Interactive cron builder TUI
    Builder,
}

pub fn run(args: CronArgs) -> Result<()> {
    match args.command {
        CronCommand::Explain { expression } => explain(&expression),
        CronCommand::Next { expression, count } => next_runs(&expression, count),
        CronCommand::Builder => builder(),
    }
}

fn normalize_cron(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    match parts.len() {
        5 => {
            // Standard 5-field cron: prepend seconds (0) and append year (*)
            format!("0 {} *", expr)
        }
        6 => {
            // 6-field: append year
            format!("{} *", expr)
        }
        7 => expr.to_string(),
        _ => expr.to_string(),
    }
}

fn parse_cron(expr: &str) -> Result<Schedule> {
    let normalized = normalize_cron(expr);
    Schedule::from_str(&normalized)
        .map_err(|e| DevToolError::InvalidInput(format!("Invalid cron expression '{}': {}", expr, e)))
}

fn explain(expression: &str) -> Result<()> {
    let _ = parse_cron(expression)?; // Validate it parses

    let parts: Vec<&str> = expression.split_whitespace().collect();

    println!(
        "\n{} Cron expression: {}\n",
        style(">>").blue().bold(),
        style(expression).cyan().bold()
    );

    // Explain each field based on standard 5-field cron
    let (minute, hour, dom, month, dow) = if parts.len() == 5 {
        (parts[0], parts[1], parts[2], parts[3], parts[4])
    } else if parts.len() == 6 {
        // sec min hour dom month dow
        (parts[1], parts[2], parts[3], parts[4], parts[5])
    } else if parts.len() == 7 {
        (parts[1], parts[2], parts[3], parts[4], parts[5])
    } else {
        return Err(DevToolError::InvalidInput("Expected 5, 6, or 7 fields".to_string()));
    };

    println!("  {} {}", style("Minute:").cyan().bold(), explain_field(minute, "minute"));
    println!("  {} {}", style("Hour:").cyan().bold(), explain_field(hour, "hour"));
    println!(
        "  {} {}",
        style("Day of Month:").cyan().bold(),
        explain_field(dom, "day of month")
    );
    println!("  {} {}", style("Month:").cyan().bold(), explain_field(month, "month"));
    println!(
        "  {} {}",
        style("Day of Week:").cyan().bold(),
        explain_field(dow, "day of week")
    );

    println!(
        "\n  {} {}",
        style("Summary:").green().bold(),
        generate_summary(minute, hour, dom, month, dow)
    );

    println!();
    Ok(())
}

fn explain_field(field: &str, name: &str) -> String {
    if field == "*" {
        return format!("every {}", name);
    }
    if let Some(step) = field.strip_prefix("*/") {
        return format!("every {} {}s", step, name);
    }
    if field.contains(',') {
        return format!("at {} {}", field, name);
    }
    if field.contains('-') {
        let parts: Vec<&str> = field.split('-').collect();
        if parts.len() == 2 {
            return format!("{} {} through {}", name, parts[0], parts[1]);
        }
    }
    format!("at {} {}", field, name)
}

fn generate_summary(minute: &str, hour: &str, dom: &str, month: &str, dow: &str) -> String {
    let time_part = match (minute, hour) {
        ("*", "*") => "every minute".to_string(),
        (m, "*") if m.starts_with("*/") => {
            format!("every {} minutes", m.trim_start_matches("*/"))
        }
        ("0", "*") => "every hour".to_string(),
        ("0", h) if h.starts_with("*/") => {
            format!("every {} hours", h.trim_start_matches("*/"))
        }
        (m, h) => format!("at {}:{}", h.replace('*', "every hour"), if m.len() == 1 { format!("0{}", m) } else { m.to_string() }),
    };

    let day_part = match (dom, dow) {
        ("*", "*") => String::new(),
        (d, "*") => format!(", on day {} of the month", d),
        ("*", d) => {
            let day_name = match d {
                "0" | "7" => "Sunday",
                "1" => "Monday",
                "2" => "Tuesday",
                "3" => "Wednesday",
                "4" => "Thursday",
                "5" => "Friday",
                "6" => "Saturday",
                _ => d,
            };
            format!(", on {}", day_name)
        }
        (d, w) => format!(", on day {} and weekday {}", d, w),
    };

    let month_part = if month == "*" {
        String::new()
    } else {
        format!(", in month {}", month)
    };

    format!("{}{}{}", time_part, day_part, month_part)
}

fn next_runs(expression: &str, count: usize) -> Result<()> {
    let schedule = parse_cron(expression)?;

    println!(
        "\n{} Next {} executions for: {}\n",
        style(">>").blue().bold(),
        count,
        style(expression).cyan()
    );

    let now = Utc::now();
    for (i, datetime) in schedule.upcoming(Utc).take(count).enumerate() {
        let relative = datetime.signed_duration_since(now);
        let relative_str = format_duration(relative);

        println!(
            "  {}  {}  ({})",
            style(format!("{:>3}.", i + 1)).dim(),
            style(datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()).green(),
            style(relative_str).dim()
        );
    }

    println!();
    Ok(())
}

fn format_duration(dur: chrono::Duration) -> String {
    let total_secs = dur.num_seconds();
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;

    if days > 0 {
        format!("in {}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("in {}h {}m", hours, minutes)
    } else {
        format!("in {}m", minutes)
    }
}

fn builder() -> Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyModifiers},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
        Terminal,
    };

    let fields = ["Minute", "Hour", "Day of Month", "Month", "Day of Week"];
    let defaults = ["*", "*", "*", "*", "*"];
    let ranges = ["0-59", "0-23", "1-31", "1-12", "0-6 (Sun-Sat)"];

    let mut values: Vec<String> = defaults.iter().map(|s| s.to_string()).collect();
    let mut selected = 0usize;
    let mut editing = false;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let cron_expr = values.join(" ");
        let schedule_result = parse_cron(&cron_expr);

        terminal.draw(|f| {
            let area = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(fields.len() as u16 + 4),
                    Constraint::Length(8),
                    Constraint::Min(3),
                ])
                .split(area);

            // Title
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    " Cron Builder ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " (Tab: next field, Enter: edit, Esc/q: quit) ",
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(title, chunks[0]);

            // Fields
            let mut field_lines = vec![Line::from("")];
            for (i, field) in fields.iter().enumerate() {
                let is_selected = i == selected;
                let prefix = if is_selected {
                    if editing { "> " } else { "  " }
                } else {
                    "  "
                };

                let field_style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                field_lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Green)),
                    Span::styled(format!("{:<15}", field), field_style),
                    Span::styled(
                        format!(" {} ", values[i]),
                        if is_selected {
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Green)
                        },
                    ),
                    Span::styled(format!("  ({})", ranges[i]), Style::default().fg(Color::DarkGray)),
                ]));
            }
            field_lines.push(Line::from(""));

            let fields_widget = Paragraph::new(field_lines)
                .block(Block::default().title(" Fields ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(fields_widget, chunks[1]);

            // Expression preview
            let expr_style = if schedule_result.is_ok() {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            let mut preview_lines = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Expression: ", Style::default().fg(Color::Cyan)),
                    Span::styled(&cron_expr, expr_style),
                ]),
            ];

            if let Ok(ref schedule) = schedule_result {
                let next = schedule.upcoming(Utc).next();
                if let Some(dt) = next {
                    preview_lines.push(Line::from(vec![
                        Span::styled("  Next run:   ", Style::default().fg(Color::Cyan)),
                        Span::styled(
                            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
            } else {
                preview_lines.push(Line::from(Span::styled(
                    "  Invalid expression",
                    Style::default().fg(Color::Red),
                )));
            }

            let preview = Paragraph::new(preview_lines)
                .block(Block::default().title(" Preview ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
            f.render_widget(preview, chunks[2]);

            // Next 5 runs
            if let Ok(ref schedule) = schedule_result {
                let mut next_lines = vec![Line::from("")];
                for (i, dt) in schedule.upcoming(Utc).take(5).enumerate() {
                    next_lines.push(Line::from(vec![
                        Span::styled(format!("  {}. ", i + 1), Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }
                let next_widget = Paragraph::new(next_lines)
                    .block(Block::default().title(" Upcoming ").borders(Borders::ALL).border_style(Style::default().fg(Color::Blue)));
                f.render_widget(next_widget, chunks[3]);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if editing {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            editing = false;
                        }
                        KeyCode::Char(c) => {
                            if values[selected] == "*" || values[selected] == defaults[selected] {
                                values[selected] = String::new();
                            }
                            values[selected].push(c);
                        }
                        KeyCode::Backspace => {
                            values[selected].pop();
                            if values[selected].is_empty() {
                                values[selected] = "*".to_string();
                            }
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Tab | KeyCode::Down | KeyCode::Char('j') => {
                            selected = (selected + 1) % fields.len();
                        }
                        KeyCode::BackTab | KeyCode::Up | KeyCode::Char('k') => {
                            selected = if selected == 0 {
                                fields.len() - 1
                            } else {
                                selected - 1
                            };
                        }
                        KeyCode::Enter => {
                            editing = true;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Print final expression
    let final_expr = values.join(" ");
    println!(
        "\n{} Final cron expression: {}",
        style(">>").green().bold(),
        style(&final_expr).cyan().bold()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_5_field() {
        let result = normalize_cron("*/5 * * * *");
        assert_eq!(result, "0 */5 * * * * *");
    }

    #[test]
    fn test_normalize_7_field() {
        let result = normalize_cron("0 */5 * * * * *");
        assert_eq!(result, "0 */5 * * * * *");
    }

    #[test]
    fn test_parse_valid_cron() {
        let schedule = parse_cron("*/5 * * * *");
        assert!(schedule.is_ok());
    }

    #[test]
    fn test_explain_field_every() {
        assert_eq!(explain_field("*", "minute"), "every minute");
    }

    #[test]
    fn test_explain_field_step() {
        assert_eq!(explain_field("*/5", "minute"), "every 5 minutes");
    }

    #[test]
    fn test_explain_field_range() {
        assert_eq!(explain_field("1-5", "day of week"), "day of week 1 through 5");
    }

    #[test]
    fn test_next_runs_produces_results() {
        let schedule = parse_cron("* * * * *").unwrap();
        let runs: Vec<_> = schedule.upcoming(Utc).take(5).collect();
        assert_eq!(runs.len(), 5);
    }
}
