use indicatif::{ProgressBar, ProgressStyle};

/// Create a standard spinner with a message.
pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Create a progress bar with total count.
pub fn create_progress_bar(total: u64, template: Option<&str>) -> ProgressBar {
    let pb = ProgressBar::new(total);
    let tmpl = template.unwrap_or("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})");
    pb.set_style(
        ProgressStyle::with_template(tmpl)
            .unwrap()
            .progress_chars("=>-"),
    );
    pb
}

/// Create a bytes-based progress bar (for downloads, etc.).
pub fn create_bytes_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb
}

/// Finish a spinner with a success message.
pub fn finish_success(pb: &ProgressBar, msg: &str) {
    pb.set_style(ProgressStyle::with_template("{msg}").unwrap());
    pb.finish_with_message(format!("{} {}", console::style("done:").green().bold(), msg));
}

/// Finish a spinner with an error message.
pub fn finish_error(pb: &ProgressBar, msg: &str) {
    pb.set_style(ProgressStyle::with_template("{msg}").unwrap());
    pb.finish_with_message(format!("{} {}", console::style("error:").red().bold(), msg));
}
