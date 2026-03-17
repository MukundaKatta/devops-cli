use console::Style;

/// Centralized color scheme for consistent terminal output.
pub struct Theme {
    pub heading: Style,
    pub success: Style,
    pub error: Style,
    pub warning: Style,
    pub info: Style,
    pub dim: Style,
    pub key: Style,
    pub value: Style,
    pub url: Style,
    pub number: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

impl Theme {
    pub fn new() -> Self {
        Self {
            heading: Style::new().blue().bold(),
            success: Style::new().green().bold(),
            error: Style::new().red().bold(),
            warning: Style::new().yellow().bold(),
            info: Style::new().cyan().bold(),
            dim: Style::new().dim(),
            key: Style::new().cyan().bold(),
            value: Style::new().green(),
            url: Style::new().underlined(),
            number: Style::new().yellow(),
        }
    }

    /// Print a styled heading line.
    pub fn print_heading(&self, msg: &str) {
        println!("{} {}", self.heading.apply_to(">>"), self.heading.apply_to(msg));
    }

    /// Print a success message.
    pub fn print_success(&self, msg: &str) {
        println!("{} {}", self.success.apply_to("done:"), msg);
    }

    /// Print an error message.
    pub fn print_error(&self, msg: &str) {
        eprintln!("{} {}", self.error.apply_to("error:"), msg);
    }

    /// Print a warning message.
    pub fn print_warning(&self, msg: &str) {
        println!("{} {}", self.warning.apply_to("warn:"), msg);
    }

    /// Print a key-value pair.
    pub fn print_kv(&self, key: &str, value: &str) {
        println!(
            "  {} {}",
            self.key.apply_to(format!("{}:", key)),
            self.value.apply_to(value)
        );
    }
}

/// TUI color palette for ratatui widgets.
pub mod tui_colors {
    use ratatui::style::Color;

    pub const PRIMARY: Color = Color::Cyan;
    pub const SECONDARY: Color = Color::Blue;
    pub const SUCCESS: Color = Color::Green;
    pub const DANGER: Color = Color::Red;
    pub const WARNING: Color = Color::Yellow;
    pub const MUTED: Color = Color::DarkGray;
    pub const BORDER: Color = Color::Blue;
    pub const HIGHLIGHT_BG: Color = Color::DarkGray;
    pub const TEXT: Color = Color::White;
    pub const TEXT_DIM: Color = Color::Gray;
}
