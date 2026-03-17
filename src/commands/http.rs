use crate::errors::Result;
use clap::Args;
use console::style;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Args)]
pub struct HttpArgs {
    /// HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD)
    #[arg(default_value = "GET")]
    pub method: String,

    /// URL to request
    pub url: String,

    /// Request body (JSON string or @filename)
    #[arg(short, long)]
    pub data: Option<String>,

    /// Headers in key:value format
    #[arg(short = 'H', long = "header", num_args = 1)]
    pub headers: Vec<String>,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// Show response headers
    #[arg(long)]
    pub show_headers: bool,

    /// Output only the response body (no decoration)
    #[arg(short, long)]
    pub raw: bool,
}

pub async fn run(args: HttpArgs) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(args.timeout))
        .build()?;

    let method: reqwest::Method = args
        .method
        .to_uppercase()
        .parse()
        .map_err(|_| crate::errors::DevToolError::InvalidInput(format!("Invalid HTTP method: {}", args.method)))?;

    let mut req = client.request(method.clone(), &args.url);

    // Parse and add headers
    let mut has_content_type = false;
    for header in &args.headers {
        if let Some((key, value)) = header.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if key.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            req = req.header(key, value);
        }
    }

    // Add body
    if let Some(ref data) = args.data {
        let body = if data.starts_with('@') {
            std::fs::read_to_string(&data[1..])?
        } else {
            data.clone()
        };
        if !has_content_type {
            req = req.header("Content-Type", "application/json");
        }
        req = req.body(body);
    }

    if !args.raw {
        println!(
            "\n{} {} {}",
            style(">>").blue().bold(),
            style(method.as_str()).cyan().bold(),
            style(&args.url).underlined()
        );
        println!();
    }

    let start = Instant::now();
    let response = req.send().await?;
    let elapsed = start.elapsed();

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.text().await?;

    if args.raw {
        println!("{}", body);
        return Ok(());
    }

    // Status line
    let status_style = if status.is_success() {
        style(format!("{}", status)).green().bold()
    } else if status.is_client_error() {
        style(format!("{}", status)).yellow().bold()
    } else if status.is_server_error() {
        style(format!("{}", status)).red().bold()
    } else {
        style(format!("{}", status)).cyan().bold()
    };

    println!("  {} {}", style("Status:").bold(), status_style);
    println!(
        "  {} {:.2}ms",
        style("Time:").bold(),
        elapsed.as_secs_f64() * 1000.0
    );
    println!(
        "  {} {} bytes",
        style("Size:").bold(),
        body.len()
    );

    // Headers
    if args.show_headers {
        println!("\n  {}", style("Headers:").bold().underlined());
        for (key, value) in headers.iter() {
            println!(
                "    {}: {}",
                style(key.as_str()).cyan(),
                value.to_str().unwrap_or("<binary>")
            );
        }
    }

    // Body
    println!("\n  {}", style("Body:").bold().underlined());

    // Try to pretty-print JSON
    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&body) {
        let pretty = serde_json::to_string_pretty(&json_val)?;
        match colored_json::to_colored_json_auto(&json_val) {
            Ok(colored) => {
                for line in colored.lines() {
                    println!("  {}", line);
                }
            }
            Err(_) => {
                for line in pretty.lines() {
                    println!("  {}", line);
                }
            }
        }
    } else {
        // Just print raw body with line limit
        let lines: Vec<&str> = body.lines().collect();
        let show = lines.len().min(100);
        for line in &lines[..show] {
            println!("  {}", line);
        }
        if lines.len() > 100 {
            println!(
                "  {} ({} more lines...)",
                style("...").dim(),
                lines.len() - 100
            );
        }
    }

    println!();
    Ok(())
}
