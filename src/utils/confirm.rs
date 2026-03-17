use console::style;
use std::io::{self, Write};

/// Prompt the user with a yes/no question. Returns true if yes.
/// `default` controls what happens when the user presses Enter with no input.
pub fn confirm(prompt: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };

    print!(
        "{} {} {} ",
        style("?").green().bold(),
        prompt,
        style(suffix).dim()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    if input.is_empty() {
        return Ok(default);
    }

    match input.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            println!("  {} Please answer y or n", style("warn:").yellow());
            confirm(prompt, default)
        }
    }
}

/// Prompt the user for text input with an optional default value.
pub fn prompt_input(prompt: &str, default: Option<&str>) -> io::Result<String> {
    if let Some(def) = default {
        print!(
            "{} {} {}: ",
            style("?").green().bold(),
            prompt,
            style(format!("[{}]", def)).dim()
        );
    } else {
        print!("{} {}: ", style("?").green().bold(), prompt);
    }
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_string();

    if input.is_empty() {
        if let Some(def) = default {
            return Ok(def.to_string());
        }
    }

    Ok(input)
}
