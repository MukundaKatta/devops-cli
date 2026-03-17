use crate::errors::Result;
use chrono::Utc;
use cron::Schedule;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::str::FromStr;

const FIELD_NAMES: [&str; 5] = ["Minute", "Hour", "Day of Month", "Month", "Day of Week"];
const FIELD_RANGES: [&str; 5] = ["0-59", "0-23", "1-31", "1-12", "0-6 (Sun-Sat)"];
const FIELD_DEFAULTS: [&str; 5] = ["*", "*", "*", "*", "*"];

/// Common cron presets.
const PRESETS: [(&str, &str); 8] = [
    ("Every minute", "* * * * *"),
    ("Every 5 minutes", "*/5 * * * *"),
    ("Every hour", "0 * * * *"),
    ("Every day at midnight", "0 0 * * *"),
    ("Every day at 9 AM", "0 9 * * *"),
    ("Every Monday at 9 AM", "0 9 * * 1"),
    ("Every 1st of month", "0 0 1 * *"),
    ("Weekdays at 8 AM", "0 8 * * 1-5"),
];

fn normalize_and_parse(expr: &str) -> std::result::Result<Schedule, String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    let normalized = match parts.len() {
        5 => format!("0 {} *", expr),
        6 => format!("{} *", expr),
        7 => expr.to_string(),
        _ => expr.to_string(),
    };
    Schedule::from_str(&normalized).map_err(|e| e.to_string())
}

/// Run the interactive cron builder TUI.
pub fn run() -> Result<()> {
    let mut values: Vec<String> = FIELD_DEFAULTS.iter().map(|s| s.to_string()).collect();
    let mut selected: usize = 0;
    let mut editing = false;
    let mut show_presets = false;
    let mut preset_idx: usize = 0;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let cron_expr = values.join(" ");
        let schedule_result = normalize_and_parse(&cron_expr);

        terminal.draw(|f| {
            let area = f.area();

            let main_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),  // Title
                    Constraint::Length(FIELD_NAMES.len() as u16 + 4), // Fields
                    Constraint::Length(6),  // Preview
                    Constraint::Min(8),    // Next runs or presets
                ])
                .split(area);

            // Title
            let help_text = if show_presets {
                "(up/down: select, Enter: apply, Esc: back)"
            } else if editing {
                "(type value, Enter: confirm, Esc: cancel)"
            } else {
                "(up/down: nav, Enter: edit, p: presets, q: quit)"
            };
            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    " Cron Builder ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", help_text),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
            f.render_widget(title, main_chunks[0]);

            // Fields
            let mut field_lines = vec![Line::from("")];
            for (i, field) in FIELD_NAMES.iter().enumerate() {
                let is_selected = i == selected && !show_presets;
                let prefix = if is_selected && editing {
                    "> "
                } else if is_selected {
                    "  "
                } else {
                    "  "
                };

                let marker = if is_selected { ">> " } else { "   " };

                let field_style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let val_style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };

                field_lines.push(Line::from(vec![
                    Span::styled(marker, Style::default().fg(Color::Green)),
                    Span::styled(format!("{:<15}", field), field_style),
                    Span::styled(format!(" {} ", values[i]), val_style),
                    Span::styled(
                        format!("  ({})", FIELD_RANGES[i]),
                        Style::default().fg(Color::DarkGray),
                    ),
                    if is_selected && editing {
                        Span::styled(" [editing]", Style::default().fg(Color::Yellow))
                    } else {
                        Span::raw("")
                    },
                ]));
            }
            field_lines.push(Line::from(""));

            let fields_widget = Paragraph::new(field_lines).block(
                Block::default()
                    .title(" Fields ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
            f.render_widget(fields_widget, main_chunks[1]);

            // Preview
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
                if let Some(dt) = schedule.upcoming(Utc).next() {
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

            let preview = Paragraph::new(preview_lines).block(
                Block::default()
                    .title(" Preview ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue)),
            );
            f.render_widget(preview, main_chunks[2]);

            // Bottom panel: presets or upcoming runs
            if show_presets {
                let items: Vec<ListItem> = PRESETS
                    .iter()
                    .enumerate()
                    .map(|(i, (label, expr))| {
                        let style = if i == preset_idx {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        };
                        let marker = if i == preset_idx { ">> " } else { "   " };
                        ListItem::new(Line::from(vec![
                            Span::styled(marker, Style::default().fg(Color::Green)),
                            Span::styled(format!("{:<30}", label), style),
                            Span::styled(
                                format!("  {}", expr),
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]))
                    })
                    .collect();

                let presets_list = List::new(items).block(
                    Block::default()
                        .title(" Presets (Enter: apply, Esc: back) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Yellow)),
                );
                f.render_widget(presets_list, main_chunks[3]);
            } else if let Ok(ref schedule) = schedule_result {
                let mut next_lines = vec![Line::from("")];
                for (i, dt) in schedule.upcoming(Utc).take(8).enumerate() {
                    next_lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {:>2}. ", i + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(
                            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }
                let next_widget = Paragraph::new(next_lines).block(
                    Block::default()
                        .title(" Upcoming Runs ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                );
                f.render_widget(next_widget, main_chunks[3]);
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if show_presets {
                    match key.code {
                        KeyCode::Esc => {
                            show_presets = false;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            preset_idx = if preset_idx == 0 {
                                PRESETS.len() - 1
                            } else {
                                preset_idx - 1
                            };
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            preset_idx = (preset_idx + 1) % PRESETS.len();
                        }
                        KeyCode::Enter => {
                            let (_, expr) = PRESETS[preset_idx];
                            let parts: Vec<&str> = expr.split_whitespace().collect();
                            for (i, part) in parts.iter().enumerate() {
                                if i < values.len() {
                                    values[i] = part.to_string();
                                }
                            }
                            show_presets = false;
                        }
                        _ => {}
                    }
                } else if editing {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => {
                            editing = false;
                        }
                        KeyCode::Char(c) => {
                            if values[selected] == "*" {
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
                        KeyCode::Char('c')
                            if key.modifiers.contains(KeyModifiers::CONTROL) =>
                        {
                            break;
                        }
                        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                            selected = (selected + 1) % FIELD_NAMES.len();
                        }
                        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                            selected = if selected == 0 {
                                FIELD_NAMES.len() - 1
                            } else {
                                selected - 1
                            };
                        }
                        KeyCode::Enter => {
                            editing = true;
                        }
                        KeyCode::Char('p') => {
                            show_presets = true;
                            preset_idx = 0;
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
        console::style(">>").green().bold(),
        console::style(&final_expr).cyan().bold()
    );

    Ok(())
}
