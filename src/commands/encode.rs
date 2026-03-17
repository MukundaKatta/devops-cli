use crate::errors::{DevToolError, Result};
use base64::{engine::general_purpose, Engine};
use clap::{Args, Subcommand};
use console::style;
use digest::Digest;

#[derive(Args)]
pub struct EncodeArgs {
    #[command(subcommand)]
    command: EncodeCommand,
}

#[derive(Subcommand)]
pub enum EncodeCommand {
    /// Base64 encode a string
    B64encode {
        /// Input string (or - for stdin)
        input: String,
    },
    /// Base64 decode a string
    B64decode {
        /// Base64 encoded string
        input: String,
    },
    /// URL encode a string
    Urlencode {
        /// Input string
        input: String,
    },
    /// URL decode a string
    Urldecode {
        /// URL encoded string
        input: String,
    },
    /// Decode a JWT token (no verification)
    Jwt {
        /// JWT token string
        token: String,
    },
    /// Hash a string with the specified algorithm
    Hash {
        /// Algorithm: sha256, sha1, sha512, md5
        #[arg(short, long, default_value = "sha256")]
        algo: String,
        /// Input string to hash
        input: String,
    },
}

pub fn run(args: EncodeArgs) -> Result<()> {
    match args.command {
        EncodeCommand::B64encode { input } => b64_encode(&input),
        EncodeCommand::B64decode { input } => b64_decode(&input),
        EncodeCommand::Urlencode { input } => url_encode(&input),
        EncodeCommand::Urldecode { input } => url_decode(&input),
        EncodeCommand::Jwt { token } => jwt_decode(&token),
        EncodeCommand::Hash { algo, input } => hash(&algo, &input),
    }
}

fn b64_encode(input: &str) -> Result<()> {
    let encoded = general_purpose::STANDARD.encode(input.as_bytes());
    println!("{}", encoded);
    Ok(())
}

fn b64_decode(input: &str) -> Result<()> {
    let decoded = general_purpose::STANDARD.decode(input)?;
    let text = String::from_utf8(decoded)
        .map_err(|e| DevToolError::InvalidInput(format!("Not valid UTF-8: {}", e)))?;
    println!("{}", text);
    Ok(())
}

fn url_encode(input: &str) -> Result<()> {
    let encoded = urlencoding::encode(input);
    println!("{}", encoded);
    Ok(())
}

fn url_decode(input: &str) -> Result<()> {
    let decoded = urlencoding::decode(input)
        .map_err(|e| DevToolError::InvalidInput(format!("Invalid URL encoding: {}", e)))?;
    println!("{}", decoded);
    Ok(())
}

fn jwt_decode(token: &str) -> Result<()> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(DevToolError::InvalidInput(
            "JWT must have 3 parts separated by dots".to_string(),
        ));
    }

    // Decode header
    let header_bytes = base64_url_decode(parts[0])?;
    let header: serde_json::Value = serde_json::from_slice(&header_bytes)
        .map_err(|e| DevToolError::InvalidInput(format!("Invalid JWT header: {}", e)))?;

    // Decode payload
    let payload_bytes = base64_url_decode(parts[1])?;
    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| DevToolError::InvalidInput(format!("Invalid JWT payload: {}", e)))?;

    println!("{}", style("Header:").cyan().bold());
    println!("{}", serde_json::to_string_pretty(&header)?);
    println!();
    println!("{}", style("Payload:").cyan().bold());
    println!("{}", serde_json::to_string_pretty(&payload)?);
    println!();
    println!(
        "{} {}",
        style("Signature:").cyan().bold(),
        style(parts[2]).dim()
    );

    // Check expiration
    if let Some(exp) = payload.get("exp").and_then(|v| v.as_i64()) {
        let now = chrono::Utc::now().timestamp();
        if exp < now {
            println!(
                "\n{} Token expired at {}",
                style("WARNING:").red().bold(),
                chrono::DateTime::from_timestamp(exp, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
        } else {
            println!(
                "\n{} Token expires at {}",
                style("INFO:").green().bold(),
                chrono::DateTime::from_timestamp(exp, 0)
                    .map(|dt| dt.to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            );
        }
    }

    Ok(())
}

fn base64_url_decode(input: &str) -> Result<Vec<u8>> {
    // JWT uses base64url encoding (no padding)
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };
    let normalized = padded.replace('-', "+").replace('_', "/");
    general_purpose::STANDARD
        .decode(&normalized)
        .map_err(|e| DevToolError::InvalidInput(format!("Base64 decode error: {}", e)))
}

fn hash(algo: &str, input: &str) -> Result<()> {
    let hash_hex = match algo.to_lowercase().as_str() {
        "sha256" => {
            let mut hasher = sha2::Sha256::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha512" => {
            let mut hasher = sha2::Sha512::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        "sha1" => {
            let mut hasher = sha1::Sha1::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        "md5" => {
            let mut hasher = md5::Md5::new();
            hasher.update(input.as_bytes());
            hex::encode(hasher.finalize())
        }
        _ => {
            return Err(DevToolError::InvalidInput(format!(
                "Unknown algorithm: {}. Use sha256, sha512, sha1, or md5",
                algo
            )));
        }
    };

    println!(
        "{} ({}): {}",
        style("Hash").cyan().bold(),
        algo,
        style(&hash_hex).green()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b64_roundtrip() {
        let input = "Hello, World!";
        let encoded = general_purpose::STANDARD.encode(input.as_bytes());
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        assert_eq!(input, String::from_utf8(decoded).unwrap());
    }

    #[test]
    fn test_url_encode_decode() {
        let input = "hello world&foo=bar";
        let encoded = urlencoding::encode(input);
        let decoded = urlencoding::decode(&encoded).unwrap();
        assert_eq!(input, decoded.as_ref());
    }

    #[test]
    fn test_sha256_hash() {
        let mut hasher = sha2::Sha256::new();
        hasher.update(b"hello");
        let result = hex::encode(hasher.finalize());
        assert_eq!(
            result,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_jwt_parts_validation() {
        let result = jwt_decode("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_url_decode() {
        // "eyJhbGciOiJIUzI1NiJ9" is base64url for {"alg":"HS256"}
        let decoded = base64_url_decode("eyJhbGciOiJIUzI1NiJ9").unwrap();
        let s = String::from_utf8(decoded).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["alg"], "HS256");
    }
}
