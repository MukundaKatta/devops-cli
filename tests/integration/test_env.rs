use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn devtool() -> Command {
    Command::cargo_bin("devtool").unwrap()
}

#[test]
fn test_env_validate_good() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.env");
    devtool()
        .args(["env", "validate", fixture])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS").or(predicate::str::contains("variables")));
}

#[test]
fn test_env_validate_with_issues() {
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "GOOD_KEY=value").unwrap();
    writeln!(file, "EMPTY_KEY=").unwrap();
    writeln!(file, "GOOD_KEY=duplicate").unwrap();

    devtool()
        .args(["env", "validate", file.path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("WARN"));
}

#[test]
fn test_env_diff_same_file() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.env");
    devtool()
        .args(["env", "diff", fixture, fixture])
        .assert()
        .success()
        .stdout(predicate::str::contains("identical"));
}

#[test]
fn test_env_diff_different() {
    let mut f1 = NamedTempFile::new().unwrap();
    let mut f2 = NamedTempFile::new().unwrap();
    writeln!(f1, "KEY1=val1\nKEY2=val2").unwrap();
    writeln!(f2, "KEY1=val1\nKEY3=val3").unwrap();

    devtool()
        .args([
            "env",
            "diff",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn test_env_generate() {
    let example = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample.env.example"
    );
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join(".env");

    devtool()
        .args([
            "env",
            "generate",
            example,
            "-o",
            output.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    assert!(output.exists());
    let content = std::fs::read_to_string(&output).unwrap();
    assert!(content.contains("DATABASE_URL"));
}

#[test]
fn test_env_generate_output_exists() {
    let example = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/sample.env.example"
    );
    // Use the example file itself as the output -- it already exists
    devtool()
        .args(["env", "generate", example, "-o", example])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_env_validate_missing_file() {
    devtool()
        .args(["env", "validate", "/nonexistent/path/.env"])
        .assert()
        .failure();
}
