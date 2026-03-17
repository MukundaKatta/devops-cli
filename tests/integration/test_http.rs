use assert_cmd::Command;
use predicates::prelude::*;

fn devtool() -> Command {
    Command::cargo_bin("devtool").unwrap()
}

#[test]
fn test_http_get() {
    devtool()
        .args(["http", "GET", "https://httpbin.org/get"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Status:"))
        .stdout(predicate::str::contains("200"));
}

#[test]
fn test_http_get_raw() {
    devtool()
        .args(["http", "GET", "https://httpbin.org/get", "--raw"])
        .assert()
        .success()
        .stdout(predicate::str::contains("origin"));
}

#[test]
fn test_http_get_with_headers() {
    devtool()
        .args([
            "http",
            "GET",
            "https://httpbin.org/headers",
            "-H",
            "X-Custom: test-value",
            "--raw",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("test-value"));
}

#[test]
fn test_http_post() {
    devtool()
        .args([
            "http",
            "POST",
            "https://httpbin.org/post",
            "-d",
            r#"{"key":"value"}"#,
            "--raw",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("key"));
}

#[test]
fn test_http_invalid_url() {
    devtool()
        .args(["http", "GET", "http://invalid.test.nonexistent"])
        .assert()
        .failure();
}

#[test]
fn test_http_show_headers() {
    devtool()
        .args([
            "http",
            "GET",
            "https://httpbin.org/get",
            "--show-headers",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Headers:"));
}

#[test]
fn test_http_invalid_method() {
    devtool()
        .args(["http", "BOGUS", "https://httpbin.org/get"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid HTTP method"));
}
