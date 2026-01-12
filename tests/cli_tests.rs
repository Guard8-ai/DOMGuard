//! CLI integration tests for DOMGuard
//!
//! Tests CLI argument parsing and basic command execution

use assert_cmd::Command;
use predicates::prelude::*;

fn domguard() -> Command {
    Command::cargo_bin("domguard").unwrap()
}

#[test]
fn test_version() {
    domguard()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("domguard"));
}

#[test]
fn test_help() {
    domguard()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Local-first Chrome DevTools CLI"))
        .stdout(predicate::str::contains("inspire"))
        .stdout(predicate::str::contains("debug"))
        .stdout(predicate::str::contains("interact"));
}

#[test]
fn test_no_args_shows_help() {
    domguard()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_invalid_subcommand() {
    domguard()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn test_debug_help() {
    domguard()
        .args(["debug", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("dom"))
        .stdout(predicate::str::contains("console"))
        .stdout(predicate::str::contains("network"));
}

#[test]
fn test_interact_help() {
    domguard()
        .args(["interact", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("click"))
        .stdout(predicate::str::contains("type"))
        .stdout(predicate::str::contains("navigate"));
}

#[test]
fn test_session_help() {
    domguard()
        .args(["session", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("list"));
}

#[test]
fn test_workflow_help() {
    domguard()
        .args(["workflow", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("run"));
}

#[test]
fn test_security_help() {
    domguard()
        .args(["security", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("block"));
}

#[test]
fn test_json_flag_global() {
    // --json should be accepted as global flag
    domguard()
        .args(["--json", "--help"])
        .assert()
        .success();
}

#[test]
fn test_host_flag() {
    // --host should be accepted
    domguard()
        .args(["--host", "localhost", "--help"])
        .assert()
        .success();
}

#[test]
fn test_port_flag() {
    // --port should be accepted
    domguard()
        .args(["--port", "9222", "--help"])
        .assert()
        .success();
}

#[test]
fn test_timeout_flag() {
    // --timeout should be accepted
    domguard()
        .args(["--timeout", "5000", "--help"])
        .assert()
        .success();
}

#[test]
fn test_interact_click_requires_target() {
    // click without selector, coords, or text should fail
    domguard()
        .args(["interact", "click"])
        .assert()
        .failure();
}

#[test]
fn test_interact_type_requires_text() {
    // type without text should fail
    domguard()
        .args(["interact", "type"])
        .assert()
        .failure();
}

#[test]
fn test_interact_navigate_requires_url() {
    domguard()
        .args(["interact", "navigate"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL"));
}

#[test]
fn test_debug_styles_requires_selector() {
    domguard()
        .args(["debug", "styles"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SELECTOR"));
}

#[test]
fn test_inspire_requires_url() {
    domguard()
        .arg("inspire")
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL"));
}
