use crate::errors::{DevToolError, Result};
use std::process::Command;

/// Copy text to the system clipboard.
/// Uses pbcopy on macOS, xclip/xsel on Linux, clip on Windows.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let result = copy_impl(text);
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(DevToolError::CommandFailed(format!(
            "Failed to copy to clipboard: {}",
            e
        ))),
    }
}

/// Read text from the system clipboard.
pub fn read_from_clipboard() -> Result<String> {
    let result = read_impl();
    match result {
        Ok(text) => Ok(text),
        Err(e) => Err(DevToolError::CommandFailed(format!(
            "Failed to read from clipboard: {}",
            e
        ))),
    }
}

#[cfg(target_os = "macos")]
fn copy_impl(text: &str) -> std::result::Result<(), String> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("pbcopy: {}", e))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("write: {}", e))?;
    }

    child.wait().map_err(|e| format!("wait: {}", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn read_impl() -> std::result::Result<String, String> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| format!("pbpaste: {}", e))?;
    String::from_utf8(output.stdout).map_err(|e| format!("utf8: {}", e))
}

#[cfg(target_os = "linux")]
fn copy_impl(text: &str) -> std::result::Result<(), String> {
    use std::io::Write;
    // Try xclip first, then xsel
    let cmd = if which_exists("xclip") {
        "xclip"
    } else if which_exists("xsel") {
        "xsel"
    } else {
        return Err("Neither xclip nor xsel found. Install one of them.".to_string());
    };

    let args: &[&str] = if cmd == "xclip" {
        &["-selection", "clipboard"]
    } else {
        &["--clipboard", "--input"]
    };

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {}", cmd, e))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("write: {}", e))?;
    }

    child.wait().map_err(|e| format!("wait: {}", e))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn read_impl() -> std::result::Result<String, String> {
    let (cmd, args): (&str, &[&str]) = if which_exists("xclip") {
        ("xclip", &["-selection", "clipboard", "-o"])
    } else if which_exists("xsel") {
        ("xsel", &["--clipboard", "--output"])
    } else {
        return Err("Neither xclip nor xsel found.".to_string());
    };

    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{}: {}", cmd, e))?;
    String::from_utf8(output.stdout).map_err(|e| format!("utf8: {}", e))
}

#[cfg(target_os = "windows")]
fn copy_impl(text: &str) -> std::result::Result<(), String> {
    use std::io::Write;
    let mut child = Command::new("clip")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("clip: {}", e))?;

    if let Some(ref mut stdin) = child.stdin {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("write: {}", e))?;
    }

    child.wait().map_err(|e| format!("wait: {}", e))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn read_impl() -> std::result::Result<String, String> {
    let output = Command::new("powershell")
        .args(["-command", "Get-Clipboard"])
        .output()
        .map_err(|e| format!("powershell: {}", e))?;
    String::from_utf8(output.stdout).map_err(|e| format!("utf8: {}", e))
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn copy_impl(_text: &str) -> std::result::Result<(), String> {
    Err("Clipboard not supported on this platform".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn read_impl() -> std::result::Result<String, String> {
    Err("Clipboard not supported on this platform".to_string())
}

#[cfg(target_os = "linux")]
fn which_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
