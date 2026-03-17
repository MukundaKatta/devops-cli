use assert_cmd::Command;
use predicates::prelude::*;

fn devtool() -> Command {
    Command::cargo_bin("devtool").unwrap()
}

#[test]
fn test_b64_encode() {
    devtool()
        .args(["encode", "b64encode", "Hello, World!"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SGVsbG8sIFdvcmxkIQ=="));
}

#[test]
fn test_b64_decode() {
    devtool()
        .args(["encode", "b64decode", "SGVsbG8sIFdvcmxkIQ=="])
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello, World!"));
}

#[test]
fn test_b64_decode_invalid() {
    devtool()
        .args(["encode", "b64decode", "!!!invalid!!!"])
        .assert()
        .failure();
}

#[test]
fn test_url_encode() {
    devtool()
        .args(["encode", "urlencode", "hello world&foo=bar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello%20world%26foo%3Dbar"));
}

#[test]
fn test_url_decode() {
    devtool()
        .args(["encode", "urldecode", "hello%20world%26foo%3Dbar"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello world&foo=bar"));
}

#[test]
fn test_hash_sha256() {
    devtool()
        .args(["encode", "hash", "--algo", "sha256", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        ));
}

#[test]
fn test_hash_md5() {
    devtool()
        .args(["encode", "hash", "--algo", "md5", "hello"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "5d41402abc4b2a76b9719d911017c592",
        ));
}

#[test]
fn test_hash_unknown_algo() {
    devtool()
        .args(["encode", "hash", "--algo", "bogus", "hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown algorithm"));
}

#[test]
fn test_jwt_invalid() {
    devtool()
        .args(["encode", "jwt", "not-a-jwt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("3 parts"));
}
