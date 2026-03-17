use crate::errors::Result;
use clap::Args;
use console::style;
use std::time::{Duration, Instant};

#[derive(Args)]
pub struct BenchArgs {
    /// URL to benchmark
    pub url: String,

    /// Total number of requests
    #[arg(short = 'n', long, default_value = "100")]
    pub requests: usize,

    /// Number of concurrent requests
    #[arg(short, long, default_value = "10")]
    pub concurrency: usize,

    /// HTTP method
    #[arg(short, long, default_value = "GET")]
    pub method: String,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// Request body
    #[arg(short = 'd', long)]
    pub data: Option<String>,

    /// Headers in key:value format
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,
}

pub async fn run(args: BenchArgs) -> Result<()> {
    use indicatif::{ProgressBar, ProgressStyle};

    println!(
        "\n{} Benchmarking {} {}\n",
        style(">>").blue().bold(),
        style(&args.method).cyan(),
        style(&args.url).underlined()
    );
    println!(
        "  Requests: {}, Concurrency: {}\n",
        style(args.requests).green(),
        style(args.concurrency).green()
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .build()?;

    let method: reqwest::Method = args.method.to_uppercase().parse().map_err(|_| {
        crate::errors::DevToolError::InvalidInput(format!("Invalid method: {}", args.method))
    })?;

    let pb = ProgressBar::new(args.requests as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("=>-"),
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel::<RequestResult>(args.requests);

    let sem = std::sync::Arc::new(tokio::sync::Semaphore::new(args.concurrency));
    let start = Instant::now();

    let mut handles = Vec::new();

    for _ in 0..args.requests {
        let client = client.clone();
        let method = method.clone();
        let url = args.url.clone();
        let data = args.data.clone();
        let headers = args.headers.clone();
        let tx = tx.clone();
        let sem = sem.clone();
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let req_start = Instant::now();

            let mut req = client.request(method, &url);
            for header in &headers {
                if let Some((k, v)) = header.split_once(':') {
                    req = req.header(k.trim(), v.trim());
                }
            }
            if let Some(body) = &data {
                req = req.body(body.clone());
            }

            let result = match req.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let size = resp.bytes().await.map(|b| b.len()).unwrap_or(0);
                    RequestResult {
                        latency: req_start.elapsed(),
                        status,
                        size,
                        error: None,
                    }
                }
                Err(e) => RequestResult {
                    latency: req_start.elapsed(),
                    status: 0,
                    size: 0,
                    error: Some(e.to_string()),
                },
            };

            pb.inc(1);
            let _ = tx.send(result).await;
        });

        handles.push(handle);
    }

    drop(tx);

    let mut results = Vec::new();
    while let Some(result) = rx.recv().await {
        results.push(result);
    }

    for handle in handles {
        let _ = handle.await;
    }

    let total_time = start.elapsed();
    pb.finish_and_clear();

    // Analyze results
    let mut latencies: Vec<Duration> = results.iter().map(|r| r.latency).collect();
    latencies.sort();

    let total = results.len();
    let errors = results.iter().filter(|r| r.error.is_some()).count();
    let successes = total - errors;

    let total_bytes: usize = results.iter().map(|r| r.size).sum();

    let status_counts = {
        let mut map = std::collections::HashMap::new();
        for r in &results {
            if r.status > 0 {
                *map.entry(r.status).or_insert(0) += 1;
            }
        }
        map
    };

    let avg_latency = if !latencies.is_empty() {
        latencies.iter().sum::<Duration>() / latencies.len() as u32
    } else {
        Duration::ZERO
    };

    let p50 = percentile(&latencies, 50);
    let p90 = percentile(&latencies, 90);
    let p95 = percentile(&latencies, 95);
    let p99 = percentile(&latencies, 99);
    let min_lat = latencies.first().copied().unwrap_or(Duration::ZERO);
    let max_lat = latencies.last().copied().unwrap_or(Duration::ZERO);

    let rps = if total_time.as_secs_f64() > 0.0 {
        successes as f64 / total_time.as_secs_f64()
    } else {
        0.0
    };

    // Print results
    println!("{}", style("  Results:").bold().underlined());
    println!(
        "    {} {:.2}s",
        style("Total time:").cyan(),
        total_time.as_secs_f64()
    );
    println!(
        "    {} {:.2} req/s",
        style("Requests/sec:").cyan(),
        rps
    );
    println!(
        "    {} {}",
        style("Successful:").cyan(),
        style(successes).green()
    );
    println!(
        "    {} {}",
        style("Failed:").cyan(),
        if errors > 0 {
            style(errors).red().to_string()
        } else {
            style(errors).green().to_string()
        }
    );
    println!(
        "    {} {}",
        style("Data transferred:").cyan(),
        format_bytes(total_bytes)
    );

    println!("\n{}", style("  Status codes:").bold().underlined());
    let mut sorted_status: Vec<_> = status_counts.into_iter().collect();
    sorted_status.sort_by_key(|(k, _)| *k);
    for (status, count) in &sorted_status {
        let s = if *status < 400 {
            style(format!("    {} : {}", status, count)).green()
        } else {
            style(format!("    {} : {}", status, count)).red()
        };
        println!("{}", s);
    }

    println!("\n{}", style("  Latency:").bold().underlined());
    println!(
        "    {} {:.2}ms",
        style("Min:").cyan(),
        min_lat.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("Avg:").cyan(),
        avg_latency.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("Max:").cyan(),
        max_lat.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("p50:").cyan(),
        p50.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("p90:").cyan(),
        p90.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("p95:").cyan(),
        p95.as_secs_f64() * 1000.0
    );
    println!(
        "    {} {:.2}ms",
        style("p99:").cyan(),
        p99.as_secs_f64() * 1000.0
    );

    // Latency histogram
    println!("\n{}", style("  Latency Distribution:").bold().underlined());
    print_histogram(&latencies);

    println!();
    Ok(())
}

struct RequestResult {
    latency: Duration,
    status: u16,
    size: usize,
    error: Option<String>,
}

fn percentile(sorted: &[Duration], p: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = (p * sorted.len() / 100).min(sorted.len() - 1);
    sorted[idx]
}

fn print_histogram(latencies: &[Duration]) {
    if latencies.is_empty() {
        return;
    }

    let min = latencies.first().unwrap().as_secs_f64() * 1000.0;
    let max = latencies.last().unwrap().as_secs_f64() * 1000.0;
    let bucket_count = 10;
    let step = (max - min) / bucket_count as f64;

    if step <= 0.0 {
        println!("    All requests: {:.2}ms", min);
        return;
    }

    let mut buckets = vec![0usize; bucket_count];
    for lat in latencies {
        let ms = lat.as_secs_f64() * 1000.0;
        let idx = ((ms - min) / step).floor() as usize;
        let idx = idx.min(bucket_count - 1);
        buckets[idx] += 1;
    }

    let max_count = *buckets.iter().max().unwrap_or(&1);

    for (i, count) in buckets.iter().enumerate() {
        let lower = min + (i as f64 * step);
        let upper = lower + step;
        let bar_width = if max_count > 0 {
            (*count as f64 / max_count as f64 * 30.0) as usize
        } else {
            0
        };
        let bar: String = "#".repeat(bar_width);

        println!(
            "    {:>8.2}-{:<8.2}ms | {:>4} | {}",
            lower,
            upper,
            count,
            style(&bar).green()
        );
    }
}

fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
