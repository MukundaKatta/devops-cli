use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn devtool() -> Command {
    Command::cargo_bin("devtool").unwrap()
}

#[test]
fn test_json_fmt_string() {
    devtool()
        .args(["json", "fmt", r#"{"a":1,"b":2}"#])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"a\""))
        .stdout(predicate::str::contains("\"b\""));
}

#[test]
fn test_json_fmt_file() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.json");
    devtool()
        .args(["json", "fmt", fixture])
        .assert()
        .success()
        .stdout(predicate::str::contains("name"));
}

#[test]
fn test_json_minify() {
    devtool()
        .args(["json", "minify", r#"{ "a" : 1,  "b" : 2 }"#])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"{"a":1,"b":2}"#));
}

#[test]
fn test_json_query() {
    devtool()
        .args([
            "json",
            "query",
            r#"{"data":{"name":"devtool","version":"1.0"}}"#,
            "data.name",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("devtool"));
}

#[test]
fn test_json_query_array() {
    devtool()
        .args([
            "json",
            "query",
            r#"{"items":[{"id":1},{"id":2}]}"#,
            "items[1].id",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_json_query_not_found() {
    devtool()
        .args(["json", "query", r#"{"a":1}"#, "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_json_diff_identical() {
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.json");
    devtool()
        .args(["json", "diff", fixture, fixture])
        .assert()
        .success()
        .stdout(predicate::str::contains("identical"));
}

#[test]
fn test_json_diff_different() {
    let mut f1 = NamedTempFile::new().unwrap();
    let mut f2 = NamedTempFile::new().unwrap();
    writeln!(f1, r#"{{"a": 1}}"#).unwrap();
    writeln!(f2, r#"{{"a": 2}}"#).unwrap();

    devtool()
        .args([
            "json",
            "diff",
            f1.path().to_str().unwrap(),
            f2.path().to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn test_json_validate_valid() {
    let sample = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/sample.json");
    let schema = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/schema.json");
    devtool()
        .args(["json", "validate", sample, schema])
        .assert()
        .success()
        .stdout(predicate::str::contains("PASS"));
}

#[test]
fn test_json_invalid_input() {
    devtool()
        .args(["json", "fmt", "not-valid-json{{{"])
        .assert()
        .failure();
}
