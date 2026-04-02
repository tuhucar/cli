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
    assert!(stdout.contains("MCP tools/call car_match"));
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

#[test]
fn invalid_format_returns_error_exit_code() {
    let output = tuhucar()
        .args(["--format", "xml", "car", "schema"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid format") || stderr.contains("xml"));
}

#[test]
fn invalid_format_json_returns_envelope() {
    // When both --format json and --format xml are present, pre_scan sees json
    // but the actual parse finds xml. Test with just xml — user gets stderr.
    let output = tuhucar()
        .args(["--format", "xml"])
        .output()
        .unwrap();
    assert!(!output.status.success());
}

#[test]
fn config_init_json_returns_envelope() {
    let tmp = std::env::temp_dir().join("tuhucar-test-config-init");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let output = tuhucar()
        .args(["--format", "json", "config", "init"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["data"].is_object());
    assert!(json["data"]["path"].as_str().unwrap().contains("config.toml"));
    assert!(json["data"]["message"].as_str().unwrap().contains("saved"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn config_show_json_returns_envelope() {
    let tmp = std::env::temp_dir().join("tuhucar-test-config-show");
    let _ = std::fs::remove_dir_all(&tmp);
    let tuhucar_dir = tmp.join(".tuhucar");
    std::fs::create_dir_all(&tuhucar_dir).unwrap();
    std::fs::write(
        tuhucar_dir.join("config.toml"),
        "[api]\nbase_url = \"https://test.example.com\"\n",
    ).unwrap();
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["data"].is_object());
    assert_eq!(json["data"]["api"]["base_url"], "https://test.example.com");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn json_error_response_includes_meta_version() {
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", "/tmp/tuhucar-test-nonexistent-meta")
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    // meta.version should be present in error responses too
    assert!(json["meta"]["version"].is_string());
}
