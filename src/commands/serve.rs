use crate::errors::Result;
use clap::Args;
use console::style;
use std::path::PathBuf;

#[derive(Args)]
pub struct ServeArgs {
    /// Directory to serve
    #[arg(default_value = ".")]
    pub dir: String,

    /// Port to listen on
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Enable CORS headers
    #[arg(long)]
    pub cors: bool,

    /// Show QR code for the URL
    #[arg(long)]
    pub qr: bool,

    /// Bind address
    #[arg(short, long, default_value = "0.0.0.0")]
    pub bind: String,
}

pub async fn run(args: ServeArgs) -> Result<()> {
    use warp::Filter;

    let dir = PathBuf::from(&args.dir).canonicalize()?;

    println!(
        "\n{} Serving {} on {}:{}\n",
        style(">>").blue().bold(),
        style(dir.display()).cyan(),
        args.bind,
        style(args.port).green().bold()
    );

    let local_url = format!("http://127.0.0.1:{}", args.port);
    println!("  {} {}", style("Local:").bold(), style(&local_url).underlined());

    // Try to get LAN IP
    if let Ok(ip) = local_ip_address::local_ip() {
        let network_url = format!("http://{}:{}", ip, args.port);
        println!(
            "  {} {}",
            style("Network:").bold(),
            style(&network_url).underlined()
        );

        if args.qr {
            println!("\n{}", style("Scan to open on mobile:").dim());
            qr2term::print_qr(&network_url).ok();
        }
    }

    println!(
        "\n  {} Press Ctrl+C to stop\n",
        style("info:").yellow()
    );

    // Build directory listing + static file serving
    let dir_clone = dir.clone();
    let dir_listing = warp::path::tail()
        .and(warp::get())
        .and_then(move |tail: warp::path::Tail| {
            let dir = dir_clone.clone();
            async move {
                let req_path = dir.join(tail.as_str());
                if req_path.is_dir() {
                    // Check for index.html
                    let index = req_path.join("index.html");
                    if index.exists() {
                        let content = tokio::fs::read_to_string(&index).await.unwrap_or_default();
                        return Ok::<_, warp::Rejection>(warp::reply::html(content).into_response());
                    }
                    // Generate directory listing
                    let html = generate_dir_listing(&req_path, tail.as_str());
                    Ok(warp::reply::html(html).into_response())
                } else {
                    Err(warp::reject::not_found())
                }
            }
        });

    let static_files = warp::fs::dir(dir.clone());

    let routes = static_files.or(dir_listing);

    // Root directory listing
    let dir_root = dir.clone();
    let root_listing = warp::path::end().and(warp::get()).map(move || {
        let index = dir_root.join("index.html");
        if index.exists() {
            let content = std::fs::read_to_string(&index).unwrap_or_default();
            warp::reply::html(content)
        } else {
            let html = generate_dir_listing(&dir_root, "");
            warp::reply::html(html)
        }
    });

    let all_routes = root_listing.or(routes);

    // Log requests
    let logged = all_routes.with(warp::log::custom(|info| {
        println!(
            "  {} {} {} ({}ms)",
            style(info.method().to_string()).cyan(),
            info.path(),
            style(info.status().as_u16()).green(),
            info.elapsed().as_millis()
        );
    }));

    let addr: std::net::SocketAddr = format!("{}:{}", args.bind, args.port)
        .parse()
        .map_err(|e| crate::errors::DevToolError::InvalidInput(format!("Invalid address: {}", e)))?;

    if args.cors {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allow_headers(vec!["Content-Type", "Authorization"]);
        warp::serve(logged.with(cors)).run(addr).await;
    } else {
        warp::serve(logged).run(addr).await;
    }

    Ok(())
}

use warp::Reply;

fn generate_dir_listing(dir: &std::path::Path, prefix: &str) -> String {
    let mut entries = Vec::new();

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.path().is_dir();
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            let modified = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .and_then(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    Some(datetime.format("%Y-%m-%d %H:%M").to_string())
                })
                .unwrap_or_else(|| "-".to_string());

            entries.push((name, is_dir, size, modified));
        }
    }

    entries.sort_by(|a, b| {
        // Directories first, then alphabetical
        b.1.cmp(&a.1).then(a.0.to_lowercase().cmp(&b.0.to_lowercase()))
    });

    let title = if prefix.is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", prefix)
    };

    let mut html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Index of {title}</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, monospace; margin: 2rem; background: #1e1e2e; color: #cdd6f4; }}
h1 {{ color: #89b4fa; border-bottom: 1px solid #45475a; padding-bottom: 0.5rem; }}
table {{ border-collapse: collapse; width: 100%; }}
th {{ text-align: left; padding: 0.5rem; color: #a6adc8; border-bottom: 2px solid #45475a; }}
td {{ padding: 0.4rem 0.5rem; border-bottom: 1px solid #313244; }}
a {{ color: #89dceb; text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
.dir {{ color: #f9e2af; }}
.size {{ color: #a6adc8; text-align: right; }}
</style>
</head>
<body>
<h1>Index of {title}</h1>
<table>
<tr><th>Name</th><th>Size</th><th>Modified</th></tr>
"#
    );

    if !prefix.is_empty() {
        html.push_str(r#"<tr><td><a href="../">..</a></td><td>-</td><td>-</td></tr>"#);
    }

    for (name, is_dir, size, modified) in &entries {
        let display_name = if *is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };
        let class = if *is_dir { " class=\"dir\"" } else { "" };
        let href = if *is_dir {
            format!("{}/", name)
        } else {
            name.clone()
        };
        let size_str = if *is_dir {
            "-".to_string()
        } else {
            format_size(*size)
        };

        html.push_str(&format!(
            "<tr><td><a href=\"{}\"{}>{}</a></td><td class=\"size\">{}</td><td>{}</td></tr>\n",
            href, class, display_name, size_str, modified
        ));
    }

    html.push_str("</table></body></html>");
    html
}

fn format_size(bytes: u64) -> String {
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
