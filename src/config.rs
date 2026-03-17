use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub serve: ServeConfig,
    #[serde(default)]
    pub bench: BenchConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default)]
    pub default_headers: std::collections::HashMap<String, String>,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            default_headers: std::collections::HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServeConfig {
    #[serde(default = "default_serve_port")]
    pub port: u16,
    #[serde(default)]
    pub cors: bool,
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            cors: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BenchConfig {
    #[serde(default = "default_bench_requests")]
    pub requests: usize,
    #[serde(default = "default_bench_concurrency")]
    pub concurrency: usize,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            requests: 100,
            concurrency: 10,
        }
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_serve_port() -> u16 {
    8080
}

fn default_bench_requests() -> usize {
    100
}

fn default_bench_concurrency() -> usize {
    10
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("devtool")
        .join("config.toml")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}
