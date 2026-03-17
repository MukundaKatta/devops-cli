use crate::errors::{DevToolError, Result};
use clap::{Args, Subcommand};
use console::style;

#[derive(Args)]
pub struct JsonArgs {
    #[command(subcommand)]
    command: JsonCommand,
}

#[derive(Subcommand)]
pub enum JsonCommand {
    /// Pretty-print JSON (from file or stdin)
    Fmt {
        /// JSON file path, or JSON string, or - for stdin
        input: String,
        /// Indentation (number of spaces)
        #[arg(short, long, default_value = "2")]
        indent: usize,
    },
    /// Minify JSON
    Minify {
        /// JSON file path or string
        input: String,
    },
    /// Query JSON with a dot-notation path (e.g. "data.items[0].name")
    Query {
        /// JSON file path or string
        input: String,
        /// Query path (dot notation: data.users[0].name)
        path: String,
    },
    /// Diff two JSON files
    Diff {
        /// First JSON file
        file1: String,
        /// Second JSON file
        file2: String,
    },
    /// Validate JSON against a JSON Schema
    Validate {
        /// JSON file path
        input: String,
        /// JSON Schema file path
        schema: String,
    },
}

pub fn run(args: JsonArgs) -> Result<()> {
    match args.command {
        JsonCommand::Fmt { input, indent } => fmt_json(&input, indent),
        JsonCommand::Minify { input } => minify_json(&input),
        JsonCommand::Query { input, path } => query_json(&input, &path),
        JsonCommand::Diff { file1, file2 } => diff_json(&file1, &file2),
        JsonCommand::Validate { input, schema } => validate_json(&input, &schema),
    }
}

fn read_json_input(input: &str) -> Result<String> {
    if input == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf)
    } else if std::path::Path::new(input).exists() {
        Ok(std::fs::read_to_string(input)?)
    } else {
        // Assume it's a JSON string
        Ok(input.to_string())
    }
}

fn fmt_json(input: &str, indent: usize) -> Result<()> {
    let raw = read_json_input(input)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(&" ".repeat(indent).into_bytes());
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    serde::Serialize::serialize(&value, &mut ser)?;

    let pretty = String::from_utf8(buf)
        .map_err(|e| DevToolError::Other(format!("UTF-8 error: {}", e)))?;

    match colored_json::to_colored_json_auto(&value) {
        Ok(colored) => println!("{}", colored),
        Err(_) => println!("{}", pretty),
    }

    Ok(())
}

fn minify_json(input: &str) -> Result<()> {
    let raw = read_json_input(input)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let minified = serde_json::to_string(&value)?;
    println!("{}", minified);
    Ok(())
}

fn query_json(input: &str, path: &str) -> Result<()> {
    let raw = read_json_input(input)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;

    let result = json_path_query(&value, path)?;

    match colored_json::to_colored_json_auto(&result) {
        Ok(colored) => println!("{}", colored),
        Err(_) => println!("{}", serde_json::to_string_pretty(&result)?),
    }

    Ok(())
}

fn json_path_query<'a>(value: &'a serde_json::Value, path: &str) -> Result<serde_json::Value> {
    let mut current = value.clone();

    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }

        // Check for array indexing: "items[0]"
        if let Some(bracket_pos) = segment.find('[') {
            let key = &segment[..bracket_pos];
            let idx_str = &segment[bracket_pos + 1..segment.len() - 1];
            let idx: usize = idx_str
                .parse()
                .map_err(|_| DevToolError::InvalidInput(format!("Invalid array index: {}", idx_str)))?;

            if !key.is_empty() {
                current = current
                    .get(key)
                    .cloned()
                    .ok_or_else(|| DevToolError::NotFound(format!("Key '{}' not found", key)))?;
            }

            current = current
                .get(idx)
                .cloned()
                .ok_or_else(|| DevToolError::NotFound(format!("Index {} out of bounds", idx)))?;
        } else {
            current = current
                .get(segment)
                .cloned()
                .ok_or_else(|| DevToolError::NotFound(format!("Key '{}' not found", segment)))?;
        }
    }

    Ok(current)
}

fn diff_json(file1: &str, file2: &str) -> Result<()> {
    let raw1 = std::fs::read_to_string(file1)?;
    let raw2 = std::fs::read_to_string(file2)?;

    let val1: serde_json::Value = serde_json::from_str(&raw1)?;
    let val2: serde_json::Value = serde_json::from_str(&raw2)?;

    let pretty1 = serde_json::to_string_pretty(&val1)?;
    let pretty2 = serde_json::to_string_pretty(&val2)?;

    let diff = similar::TextDiff::from_lines(&pretty1, &pretty2);

    println!(
        "{} {} vs {}",
        style(">>").blue().bold(),
        style(file1).cyan(),
        style(file2).cyan()
    );
    println!();

    let mut has_diff = false;
    for change in diff.iter_all_changes() {
        match change.tag() {
            similar::ChangeTag::Equal => {
                print!("  {}", change);
            }
            similar::ChangeTag::Insert => {
                print!("{} {}", style("+").green().bold(), style(change).green());
                has_diff = true;
            }
            similar::ChangeTag::Delete => {
                print!("{} {}", style("-").red().bold(), style(change).red());
                has_diff = true;
            }
        }
    }

    if !has_diff {
        println!("{} Files are identical", style("info:").green().bold());
    }

    Ok(())
}

fn validate_json(input: &str, schema_path: &str) -> Result<()> {
    let raw = std::fs::read_to_string(input)?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;

    let schema_raw = std::fs::read_to_string(schema_path)?;
    let schema: serde_json::Value = serde_json::from_str(&schema_raw)?;

    let validator = jsonschema::validator_for(&schema)
        .map_err(|e| DevToolError::InvalidInput(format!("Invalid schema: {}", e)))?;

    let result = validator.validate(&value);

    match result {
        Ok(()) => {
            println!(
                "{} JSON is valid against the schema",
                style("PASS").green().bold()
            );
        }
        Err(errors) => {
            println!(
                "{} JSON validation failed:\n",
                style("FAIL").red().bold()
            );
            for error in errors {
                println!("  {} {}", style("-").red(), error);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_path_simple() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"name": "test", "nested": {"key": "value"}}"#).unwrap();
        let result = json_path_query(&json, "name").unwrap();
        assert_eq!(result, serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_json_path_nested() {
        let json: serde_json::Value =
            serde_json::from_str(r#"{"data": {"items": [{"id": 1}, {"id": 2}]}}"#).unwrap();
        let result = json_path_query(&json, "data.items[1].id").unwrap();
        assert_eq!(result, serde_json::json!(2));
    }

    #[test]
    fn test_json_path_not_found() {
        let json: serde_json::Value = serde_json::from_str(r#"{"a": 1}"#).unwrap();
        let result = json_path_query(&json, "b");
        assert!(result.is_err());
    }

    #[test]
    fn test_minify() {
        let input = r#"{
            "key": "value",
            "num": 42
        }"#;
        let value: serde_json::Value = serde_json::from_str(input).unwrap();
        let minified = serde_json::to_string(&value).unwrap();
        assert_eq!(minified, r#"{"key":"value","num":42}"#);
    }
}
