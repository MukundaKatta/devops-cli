use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Args)]
pub struct EnvArgs {
    #[command(subcommand)]
    command: EnvCommand,
}

#[derive(Subcommand)]
pub enum EnvCommand {
    /// Diff two .env files
    Diff {
        /// First .env file
        file1: String,
        /// Second .env file
        file2: String,
    },
    /// Validate a .env file (check for empty values, duplicates, syntax)
    Validate {
        /// Path to .env file
        file: String,
    },
    /// Generate .env from .env.example (prompts for values or uses defaults)
    Generate {
        /// Path to .env.example
        #[arg(default_value = ".env.example")]
        example: String,
        /// Output file path
        #[arg(short, long, default_value = ".env")]
        output: String,
    },
    /// Encrypt a .env file using AES-256-GCM
    Encrypt {
        /// Path to .env file
        file: String,
        /// Encryption key (or set DEVTOOL_ENV_KEY)
        #[arg(short, long, env = "DEVTOOL_ENV_KEY")]
        key: String,
    },
    /// Decrypt an encrypted .env file
    Decrypt {
        /// Path to encrypted .env file
        file: String,
        /// Decryption key (or set DEVTOOL_ENV_KEY)
        #[arg(short, long, env = "DEVTOOL_ENV_KEY")]
        key: String,
    },
}

pub fn run(args: EnvArgs) -> Result<()> {
    match args.command {
        EnvCommand::Diff { file1, file2 } => diff_env(&file1, &file2),
        EnvCommand::Validate { file } => validate_env(&file),
        EnvCommand::Generate { example, output } => generate_env(&example, &output),
        EnvCommand::Encrypt { file, key } => encrypt_env(&file, &key),
        EnvCommand::Decrypt { file, key } => decrypt_env(&file, &key),
    }
}

fn parse_env_file(path: &str) -> Result<BTreeMap<String, String>> {
    let content = std::fs::read_to_string(path)?;
    let mut map = BTreeMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
            map.insert(key, value);
        }
    }

    Ok(map)
}

fn diff_env(file1: &str, file2: &str) -> Result<()> {
    let env1 = parse_env_file(file1)?;
    let env2 = parse_env_file(file2)?;

    println!(
        "\n{} {} vs {}\n",
        style(">>").blue().bold(),
        style(file1).cyan(),
        style(file2).cyan()
    );

    let mut has_diff = false;

    // Keys only in file1
    for key in env1.keys() {
        if !env2.contains_key(key) {
            println!(
                "  {} {} (only in {})",
                style("-").red().bold(),
                style(key).red(),
                file1
            );
            has_diff = true;
        }
    }

    // Keys only in file2
    for key in env2.keys() {
        if !env1.contains_key(key) {
            println!(
                "  {} {} (only in {})",
                style("+").green().bold(),
                style(key).green(),
                file2
            );
            has_diff = true;
        }
    }

    // Changed values
    for (key, val1) in &env1 {
        if let Some(val2) = env2.get(key) {
            if val1 != val2 {
                println!(
                    "  {} {} : {} -> {}",
                    style("~").yellow().bold(),
                    style(key).yellow(),
                    style(val1).red(),
                    style(val2).green()
                );
                has_diff = true;
            }
        }
    }

    if !has_diff {
        println!("  {}", style("Files are identical").green());
    }

    println!();
    Ok(())
}

fn validate_env(file: &str) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut seen_keys: BTreeMap<String, usize> = BTreeMap::new();

    println!(
        "\n{} Validating {}\n",
        style(">>").blue().bold(),
        style(file).cyan()
    );

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !trimmed.contains('=') {
            errors.push(format!("Line {}: Missing '=' separator: {}", line_num, trimmed));
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let key = key.trim();

            if key.is_empty() {
                errors.push(format!("Line {}: Empty key", line_num));
                continue;
            }

            if key.contains(' ') {
                errors.push(format!("Line {}: Key contains spaces: '{}'", line_num, key));
            }

            if let Some(prev_line) = seen_keys.get(key) {
                warnings.push(format!(
                    "Line {}: Duplicate key '{}' (first seen at line {})",
                    line_num, key, prev_line
                ));
            }
            seen_keys.insert(key.to_string(), line_num);

            let value = value.trim();
            if value.is_empty() {
                warnings.push(format!("Line {}: Empty value for '{}'", line_num, key));
            }
        }
    }

    if errors.is_empty() && warnings.is_empty() {
        println!(
            "  {} No issues found ({} variables)",
            style("PASS").green().bold(),
            seen_keys.len()
        );
    } else {
        for error in &errors {
            println!("  {} {}", style("ERROR").red().bold(), error);
        }
        for warning in &warnings {
            println!("  {} {}", style("WARN").yellow().bold(), warning);
        }
        println!(
            "\n  {} {} error(s), {} warning(s)",
            style(">>").blue().bold(),
            errors.len(),
            warnings.len()
        );
    }

    println!();
    Ok(())
}

fn generate_env(example: &str, output: &str) -> Result<()> {
    if !Path::new(example).exists() {
        return Err(DevToolError::NotFound(format!("File not found: {}", example)));
    }

    if Path::new(output).exists() {
        return Err(DevToolError::InvalidInput(format!(
            "Output file already exists: {}. Remove it first.",
            output
        )));
    }

    let content = std::fs::read_to_string(example)?;
    let mut output_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            output_lines.push(line.to_string());
            continue;
        }

        if let Some((key, value)) = trimmed.split_once('=') {
            let value = value.trim();
            if value.is_empty() {
                // Keep empty as placeholder
                output_lines.push(format!("{}=", key.trim()));
            } else {
                output_lines.push(format!("{}={}", key.trim(), value));
            }
        } else {
            output_lines.push(line.to_string());
        }
    }

    let result = output_lines.join("\n");
    std::fs::write(output, &result)?;

    println!(
        "{} Generated {} from {}",
        style("done:").green().bold(),
        style(output).cyan(),
        style(example).cyan()
    );

    Ok(())
}

fn derive_key(password: &str) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

fn encrypt_env(file: &str, key: &str) -> Result<()> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use rand::RngCore;

    let plaintext = std::fs::read_to_string(file)?;
    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| DevToolError::Encryption(format!("Key init error: {}", e)))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| DevToolError::Encryption(format!("Encrypt error: {}", e)))?;

    // Prepend nonce to ciphertext, then base64 encode
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);

    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &combined);

    let out_path = format!("{}.enc", file);
    std::fs::write(&out_path, &encoded)?;

    println!(
        "{} Encrypted {} -> {}",
        style("done:").green().bold(),
        style(file).cyan(),
        style(&out_path).cyan()
    );

    Ok(())
}

fn decrypt_env(file: &str, key: &str) -> Result<()> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };

    let encoded = std::fs::read_to_string(file)?;
    let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded.trim())
        .map_err(|e| DevToolError::Encryption(format!("Base64 decode error: {}", e)))?;

    if combined.len() < 12 {
        return Err(DevToolError::Encryption("Invalid encrypted data".to_string()));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let key_bytes = derive_key(key);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| DevToolError::Encryption(format!("Key init error: {}", e)))?;

    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| DevToolError::Encryption(format!("Decrypt error: {}", e)))?;

    let text = String::from_utf8(plaintext)
        .map_err(|e| DevToolError::Encryption(format!("UTF-8 error: {}", e)))?;

    // Output to file without .enc extension, or stdout
    let out_path = if file.ends_with(".enc") {
        file.trim_end_matches(".enc").to_string()
    } else {
        format!("{}.dec", file)
    };

    std::fs::write(&out_path, &text)?;

    println!(
        "{} Decrypted {} -> {}",
        style("done:").green().bold(),
        style(file).cyan(),
        style(&out_path).cyan()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_env_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Comment").unwrap();
        writeln!(file, "KEY1=value1").unwrap();
        writeln!(file, "KEY2=\"value2\"").unwrap();
        writeln!(file, "KEY3=").unwrap();
        writeln!(file, "").unwrap();

        let env = parse_env_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(env.get("KEY1").unwrap(), "value1");
        assert_eq!(env.get("KEY2").unwrap(), "value2");
        assert_eq!(env.get("KEY3").unwrap(), "");
        assert_eq!(env.len(), 3);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let k1 = derive_key("my-secret");
        let k2 = derive_key("my-secret");
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };
        use rand::RngCore;

        let password = "test-password-123";
        let plaintext = "DB_HOST=localhost\nDB_PORT=5432\n";

        let key_bytes = derive_key(password);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).unwrap();

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();

        let decrypted = cipher.decrypt(nonce, ciphertext.as_ref()).unwrap();
        assert_eq!(plaintext.as_bytes(), decrypted.as_slice());
    }
}
