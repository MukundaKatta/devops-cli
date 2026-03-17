use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;
use std::process::Command;

#[derive(Args)]
pub struct DockerArgs {
    #[command(subcommand)]
    command: DockerCommand,
}

#[derive(Subcommand)]
pub enum DockerCommand {
    /// Clean up unused Docker resources (images, containers, volumes, networks)
    Clean {
        /// Also remove all stopped containers
        #[arg(long)]
        all: bool,
    },
    /// Show live Docker stats in a TUI dashboard
    Stats,
    /// Tail Docker container logs
    Logs {
        /// Container name or ID
        container: String,
        /// Number of lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: usize,
        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
    /// Show Docker disk usage / image sizes
    Size,
}

pub async fn run(args: DockerArgs) -> Result<()> {
    match args.command {
        DockerCommand::Clean { all } => clean(all),
        DockerCommand::Stats => stats().await,
        DockerCommand::Logs {
            container,
            lines,
            follow,
        } => logs(&container, lines, follow),
        DockerCommand::Size => size(),
    }
}

fn clean(all: bool) -> Result<()> {
    println!(
        "{} Cleaning Docker resources...\n",
        style(">>").blue().bold()
    );

    if all {
        println!("  {} Removing stopped containers...", style(">>").cyan());
        let output = Command::new("docker")
            .args(["container", "prune", "-f"])
            .output()
            .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;
        println!("  {}", String::from_utf8_lossy(&output.stdout).trim());
    }

    println!("  {} Removing dangling images...", style(">>").cyan());
    let output = Command::new("docker")
        .args(["image", "prune", "-f"])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;
    println!("  {}", String::from_utf8_lossy(&output.stdout).trim());

    println!("  {} Removing unused volumes...", style(">>").cyan());
    let output = Command::new("docker")
        .args(["volume", "prune", "-f"])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;
    println!("  {}", String::from_utf8_lossy(&output.stdout).trim());

    println!("  {} Removing unused networks...", style(">>").cyan());
    let output = Command::new("docker")
        .args(["network", "prune", "-f"])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;
    println!("  {}", String::from_utf8_lossy(&output.stdout).trim());

    println!(
        "\n{} Docker cleanup complete!",
        style("done:").green().bold()
    );
    Ok(())
}

async fn stats() -> Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{
        backend::CrosstermBackend,
        layout::{Constraint, Layout},
        style::{Color, Modifier, Style},
        text::Span,
        widgets::{Block, Borders, Cell, Row, Table},
        Terminal,
    };

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        // Get docker stats
        let output = Command::new("docker")
            .args(["stats", "--no-stream", "--format", "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}\t{{.PIDs}}"])
            .output()
            .map_err(|e| DevToolError::CommandFailed(format!("docker stats: {}", e)))?;

        let stats_text = String::from_utf8_lossy(&output.stdout);
        let rows: Vec<Vec<String>> = stats_text
            .lines()
            .filter(|l| !l.is_empty())
            .map(|line| line.split('\t').map(|s| s.to_string()).collect())
            .collect();

        terminal.draw(|f| {
            let area = f.area();

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

            let data_rows: Vec<Row> = rows
                .iter()
                .map(|cols| {
                    let cells: Vec<Cell> = cols.iter().map(|c| Cell::from(c.as_str())).collect();
                    Row::new(cells)
                })
                .collect();

            let widths = [
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
            ];

            let table = Table::new(data_rows, widths)
                .header(header)
                .block(
                    Block::default()
                        .title(" Docker Stats (press 'q' to quit) ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .row_highlight_style(Style::default().bg(Color::DarkGray));

            f.render_widget(table, area);
        })?;

        // Check for quit
        if event::poll(std::time::Duration::from_secs(2))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn logs(container: &str, lines: usize, follow: bool) -> Result<()> {
    let mut args = vec!["logs", "--tail", &lines.to_string()];
    if follow {
        args.push("-f");
    }
    args.push(container);

    let status = Command::new("docker")
        .args(&args)
        .status()
        .map_err(|e| DevToolError::CommandFailed(format!("docker logs: {}", e)))?;

    if !status.success() {
        return Err(DevToolError::CommandFailed(format!(
            "docker logs exited with status {}",
            status
        )));
    }

    Ok(())
}

fn size() -> Result<()> {
    println!(
        "{} Docker disk usage:\n",
        style(">>").blue().bold()
    );

    let output = Command::new("docker")
        .args(["system", "df"])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    println!("\n{} Image sizes:\n", style(">>").blue().bold());

    let output = Command::new("docker")
        .args([
            "images",
            "--format",
            "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedSince}}",
        ])
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("docker: {}", e)))?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
