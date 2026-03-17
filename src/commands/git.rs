use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;
use std::process::Command;

#[derive(Args)]
pub struct GitArgs {
    #[command(subcommand)]
    command: GitCommand,
}

#[derive(Subcommand)]
pub enum GitCommand {
    /// Show repository summary (commits, authors, files, etc.)
    Summary,
    /// Generate a changelog from git log
    Changelog {
        /// Number of commits to include
        #[arg(short, long, default_value = "50")]
        count: usize,
        /// Include commits since this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,
    },
    /// Clean merged branches (delete local branches merged into main/master)
    BranchClean {
        /// Actually delete branches (dry-run by default)
        #[arg(long)]
        execute: bool,
    },
    /// Undo the last commit (soft reset)
    Undo {
        /// Number of commits to undo
        #[arg(default_value = "1")]
        count: usize,
        /// Hard reset (discard changes)
        #[arg(long)]
        hard: bool,
    },
}

pub fn run(args: GitArgs) -> Result<()> {
    match args.command {
        GitCommand::Summary => summary(),
        GitCommand::Changelog { count, since } => changelog(count, since),
        GitCommand::BranchClean { execute } => branch_clean(execute),
        GitCommand::Undo { count, hard } => undo(count, hard),
    }
}

fn git_cmd(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| DevToolError::CommandFailed(format!("git: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DevToolError::CommandFailed(format!("git {}: {}", args[0], stderr.trim())));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn summary() -> Result<()> {
    println!(
        "\n{} Git Repository Summary\n",
        style(">>").blue().bold()
    );

    // Repo name
    let remote = git_cmd(&["remote", "get-url", "origin"]).unwrap_or_else(|_| "N/A".to_string());
    println!(
        "  {} {}",
        style("Remote:").cyan().bold(),
        remote.trim()
    );

    // Current branch
    let branch = git_cmd(&["branch", "--show-current"])?;
    println!(
        "  {} {}",
        style("Branch:").cyan().bold(),
        branch.trim()
    );

    // Total commits
    let commit_count = git_cmd(&["rev-list", "--count", "HEAD"])?;
    println!(
        "  {} {}",
        style("Commits:").cyan().bold(),
        commit_count.trim()
    );

    // Contributors
    let authors = git_cmd(&["shortlog", "-sn", "--no-merges", "HEAD"])?;
    let author_count = authors.lines().count();
    println!(
        "  {} {}",
        style("Authors:").cyan().bold(),
        author_count
    );

    // Files
    let files = git_cmd(&["ls-files"])?;
    let file_count = files.lines().count();
    println!(
        "  {} {}",
        style("Files:").cyan().bold(),
        file_count
    );

    // Last commit
    let last_commit = git_cmd(&["log", "-1", "--format=%h %s (%cr by %an)"])?;
    println!(
        "  {} {}",
        style("Last commit:").cyan().bold(),
        last_commit.trim()
    );

    // Top 5 contributors
    println!(
        "\n  {}",
        style("Top Contributors:").cyan().bold()
    );
    for line in authors.lines().take(5) {
        let line = line.trim();
        println!("    {}", line);
    }

    println!();
    Ok(())
}

fn changelog(count: usize, since: Option<String>) -> Result<()> {
    let mut args = vec![
        "log".to_string(),
        format!("-{}", count),
        "--pretty=format:%h|%s|%an|%ad".to_string(),
        "--date=short".to_string(),
    ];

    if let Some(since_date) = since {
        args.push(format!("--since={}", since_date));
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = git_cmd(&arg_refs)?;

    println!(
        "\n{} Changelog\n",
        style(">>").blue().bold()
    );

    let mut current_date = String::new();

    for line in output.lines() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }

        let hash = parts[0];
        let subject = parts[1];
        let _author = parts[2];
        let date = parts[3];

        if date != current_date {
            current_date = date.to_string();
            println!("  {}", style(&current_date).yellow().bold());
        }

        // Categorize by conventional commit prefix
        let (prefix, msg) = categorize_commit(subject);
        println!(
            "    {} {} {}",
            style(hash).dim(),
            style(prefix).green(),
            msg
        );
    }

    println!();
    Ok(())
}

fn categorize_commit(subject: &str) -> (&str, &str) {
    let lower = subject.to_lowercase();
    if lower.starts_with("feat") {
        ("[FEAT]", subject)
    } else if lower.starts_with("fix") {
        ("[FIX]", subject)
    } else if lower.starts_with("docs") {
        ("[DOCS]", subject)
    } else if lower.starts_with("refactor") {
        ("[REFACTOR]", subject)
    } else if lower.starts_with("test") {
        ("[TEST]", subject)
    } else if lower.starts_with("chore") {
        ("[CHORE]", subject)
    } else if lower.starts_with("ci") {
        ("[CI]", subject)
    } else {
        ("[OTHER]", subject)
    }
}

fn branch_clean(execute: bool) -> Result<()> {
    // Detect main branch
    let main_branch = if git_cmd(&["rev-parse", "--verify", "main"]).is_ok() {
        "main"
    } else {
        "master"
    };

    let output = git_cmd(&["branch", "--merged", main_branch])?;

    let branches: Vec<&str> = output
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('*') && *l != "main" && *l != "master")
        .collect();

    if branches.is_empty() {
        println!(
            "{} No merged branches to clean up",
            style("info:").green().bold()
        );
        return Ok(());
    }

    if execute {
        println!(
            "{} Deleting merged branches:\n",
            style(">>").blue().bold()
        );
        for branch in &branches {
            match git_cmd(&["branch", "-d", branch]) {
                Ok(_) => println!("  {} Deleted {}", style("done:").green().bold(), style(branch).cyan()),
                Err(e) => println!("  {} Failed to delete {}: {}", style("error:").red().bold(), branch, e),
            }
        }
    } else {
        println!(
            "{} Branches that would be deleted (use --execute to delete):\n",
            style("DRY RUN:").yellow().bold()
        );
        for branch in &branches {
            println!("  {} {}", style("-").red(), branch);
        }
    }

    Ok(())
}

fn undo(count: usize, hard: bool) -> Result<()> {
    let reset_type = if hard { "--hard" } else { "--soft" };
    let target = format!("HEAD~{}", count);

    println!(
        "{} Undoing {} commit(s) ({} reset)...",
        style(">>").blue().bold(),
        count,
        if hard { "hard" } else { "soft" }
    );

    git_cmd(&["reset", reset_type, &target])?;

    println!(
        "{} Successfully undid {} commit(s)",
        style("done:").green().bold(),
        count
    );

    if !hard {
        println!(
            "  {} Changes are staged and ready to be re-committed",
            style("info:").yellow()
        );
    }

    Ok(())
}
