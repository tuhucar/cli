use std::process::Command;

fn tuhucar() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tuhucar"))
}

/// Create a unique temp dir for test isolation, return its path.
fn make_temp_home(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tuhucar-test-{}-{}", name, std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = std::fs::remove_dir_all(dir);
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

// P2 fix: use isolated temp dir instead of hardcoded /tmp path
#[test]
fn config_missing_returns_error_json() {
    let tmp = make_temp_home("config-missing");
    // Don't create .tuhucar/ — config should be missing
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["error"]["code"], "CONFIG_MISSING");
    cleanup(&tmp);
}

#[test]
fn invalid_format_returns_error_exit_code() {
    let output = tuhucar()
        .args(["--format", "xml", "car", "schema"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid format"));
}

// P1 fix: test that pre_scan_format() triggers JSON envelope on clap errors
#[test]
fn prescan_json_format_produces_json_error_envelope() {
    // pre_scan_format() detects --format=json, so even when clap fails
    // (e.g. missing subcommand), we get a JSON error envelope on stdout
    let output = tuhucar()
        .args(["--format=json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["error"]["code"], "INVALID_ARGS");
    // Also verify meta.version is present
    assert!(json["meta"]["version"].is_string());
}

#[test]
fn config_init_json_returns_envelope() {
    let tmp = make_temp_home("config-init");
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
    cleanup(&tmp);
}

#[test]
fn config_show_json_returns_envelope() {
    let tmp = make_temp_home("config-show");
    let tuhucar_dir = tmp.join(".tuhucar");
    std::fs::create_dir_all(&tuhucar_dir).unwrap();
    std::fs::write(
        tuhucar_dir.join("config.toml"),
        "[api]\nendpoint = \"https://test.example.com\"\n",
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
    assert_eq!(json["data"]["api"]["endpoint"], "https://test.example.com");
    cleanup(&tmp);
}

// P2 fix: use isolated temp dir
#[test]
fn json_error_response_includes_meta_version() {
    let tmp = make_temp_home("meta-version");
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(json["meta"]["version"].is_string());
    cleanup(&tmp);
}

// Verify old base_url configs still load (backwards compatibility)
#[test]
fn config_show_accepts_legacy_base_url() {
    let tmp = make_temp_home("legacy-config");
    let tuhucar_dir = tmp.join(".tuhucar");
    std::fs::create_dir_all(&tuhucar_dir).unwrap();
    std::fs::write(
        tuhucar_dir.join("config.toml"),
        "[api]\nbase_url = \"https://legacy.example.com\"\n",
    ).unwrap();
    let output = tuhucar()
        .args(["--format", "json", "config", "show"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(json["data"]["api"]["endpoint"], "https://legacy.example.com");
    cleanup(&tmp);
}

// P2 (update notice regression): verify markdown mode surfaces update notice
#[test]
fn markdown_mode_shows_update_notice_on_stderr() {
    let tmp = make_temp_home("update-notice");
    let tuhucar_dir = tmp.join(".tuhucar");
    std::fs::create_dir_all(&tuhucar_dir).unwrap();
    // Write update_check indicating an available update
    std::fs::write(
        tuhucar_dir.join(".update_check"),
        r#"{"checked_at":"1700000000","current":"0.1.0","latest":"9.9.9","status":"update_available","install_source":"unknown"}"#,
    ).unwrap();
    // Run a command that succeeds without config (schema doesn't need config)
    let output = tuhucar()
        .args(["car", "schema"])
        .env("HOME", &tmp)
        .output()
        .unwrap();
    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("9.9.9"), "stderr should contain update notice, got: {}", stderr);
    // Verify .update_notified was written
    assert!(tuhucar_dir.join(".update_notified").exists());
    cleanup(&tmp);
}
