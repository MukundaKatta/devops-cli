use assert_cmd::Command;
use predicates::prelude::*;

fn devtool() -> Command {
    Command::cargo_bin("devtool").unwrap()
}

#[test]
fn test_cron_explain_every_minute() {
    devtool()
        .args(["cron", "explain", "* * * * *"])
        .assert()
        .success()
        .stdout(predicate::str::contains("every minute"));
}

#[test]
fn test_cron_explain_every_5_min() {
    devtool()
        .args(["cron", "explain", "*/5 * * * *"])
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_cron_explain_specific_time() {
    devtool()
        .args(["cron", "explain", "30 9 * * *"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Minute"))
        .stdout(predicate::str::contains("Hour"));
}

#[test]
fn test_cron_next_runs() {
    devtool()
        .args(["cron", "next", "* * * * *", "-n", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1."))
        .stdout(predicate::str::contains("5."));
}

#[test]
fn test_cron_next_default_count() {
    devtool()
        .args(["cron", "next", "0 * * * *"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1."));
}

#[test]
fn test_cron_invalid_expression() {
    devtool()
        .args(["cron", "explain", "invalid cron"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid cron"));
}

#[test]
fn test_cron_explain_weekday() {
    devtool()
        .args(["cron", "explain", "0 9 * * 1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Day of Week"));
}
