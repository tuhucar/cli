use std::process::Command;

fn tuhucar() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tuhucar"))
}

#[test]
fn help_flag_shows_usage() {
    let output = tuhucar().arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tuhucar"));
}

#[test]
fn version_flag_shows_version() {
    let output = tuhucar().arg("--version").output().unwrap();
    assert!(output.status.success());
}

#[test]
fn missing_subcommand_in_json_returns_envelope() {
    let output = tuhucar()
        .args(["--format", "json", "car"])
        .output()
        .unwrap();
    // clap will error because 'car' needs a subcommand
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["error"]["code"], "INVALID_ARGS");
}

#[test]
fn missing_subcommand_without_json_shows_help() {
    let output = tuhucar().arg("car").output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    // clap prints help to stderr
    assert!(stderr.contains("Usage") || stderr.contains("usage"));
}

#[test]
fn car_schema_returns_valid_json() {
    let output = tuhucar().args(["car", "schema"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["name"], "car.match");
    assert!(json["input"].is_object());
    assert!(json["wire_output"].is_object());
}

#[test]
fn knowledge_schema_returns_valid_json() {
    let output = tuhucar().args(["knowledge", "schema"]).output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["name"], "knowledge.query");
}

#[test]
fn dry_run_does_not_make_request() {
    let output = tuhucar()
        .args(["--dry-run", "car", "match", "朗逸"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GET /api/v1/car/match"));
}

#[test]
fn config_missing_returns_error_json() {
    // Use a temp HOME to avoid touching real config
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", "/tmp/tuhucar-test-nonexistent")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["error"]["code"], "CONFIG_MISSING");
}
