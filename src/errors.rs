use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevToolError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, DevToolError>;
