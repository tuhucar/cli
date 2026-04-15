use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::config::Config;
use crate::types::Notice;

const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60; // 24 hours
const FAILED_RETRY_SECS: u64 = 60 * 60; // 1 hour

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCheck {
    pub checked_at: String,
    pub current: String,
    pub latest: String,
    pub status: UpdateStatus,
    pub install_source: InstallSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus {
    UpToDate,
    UpdateAvailable,
    Downloaded,
    CheckFailed,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InstallSource {
    InstallSh,
    Npm,
    Homebrew,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNotified {
    pub latest: String,
    pub notified_at: String,
}

fn update_dir() -> PathBuf {
    Config::config_dir()
}

pub fn check_file_path() -> PathBuf {
    update_dir().join(".update_check")
}

pub fn notified_file_path() -> PathBuf {
    update_dir().join(".update_notified")
}

pub fn detect_install_source() -> InstallSource {
    let exe = std::env::current_exe().unwrap_or_default();
    classify_path(&exe.to_string_lossy())
}

/// Pure function for testability — extracts path-matching logic
pub fn classify_path(path: &str) -> InstallSource {
    if path.contains("/.tuhucar/bin/") {
        InstallSource::InstallSh
    } else if path.contains("node_modules/") {
        InstallSource::Npm
    } else if path.contains("/Cellar/") || path.contains("/homebrew/") {
        InstallSource::Homebrew
    } else {
        InstallSource::Unknown
    }
}

pub fn should_check(current_version: &str) -> bool {
    let check = match read_check_file() {
        Some(c) => c,
        None => return true,
    };

    // If version changed (upgraded), clean up and start fresh
    if check.current != current_version {
        let _ = std::fs::remove_file(check_file_path());
        let _ = std::fs::remove_file(notified_file_path());
        return true;
    }

    let interval = match check.status {
        UpdateStatus::CheckFailed => FAILED_RETRY_SECS,
        _ => CHECK_INTERVAL_SECS,
    };

    let checked_at = parse_epoch_or_zero(&check.checked_at);
    let elapsed = SystemTime::now()
        .duration_since(checked_at)
        .unwrap_or(Duration::from_secs(interval + 1));

    elapsed.as_secs() > interval
}

pub fn pending_notice() -> Option<Notice> {
    let check = read_check_file()?;

    match check.status {
        UpdateStatus::UpdateAvailable | UpdateStatus::Downloaded => {}
        _ => return None,
    }

    let notified = read_notified_file();
    if let Some(n) = &notified {
        if n.latest == check.latest {
            return None; // already notified for this version
        }
    }

    let message = match check.install_source {
        InstallSource::Npm => format!("新版本 {} 可用，请运行: npm update -g @tuhucar/cli", check.latest),
        InstallSource::Homebrew => {
            format!("新版本 {} 可用，请运行: brew upgrade tuhucar", check.latest)
        }
        InstallSource::InstallSh => format!("新版本 {} 可用，将在下次启动时自动更新", check.latest),
        InstallSource::Unknown => format!("新版本 {} 可用，请手动更新", check.latest),
    };

    Some(Notice::Update {
        current: check.current,
        latest: check.latest,
        message,
    })
}

pub fn mark_notified(latest: &str) {
    let notified = UpdateNotified {
        latest: latest.to_string(),
        notified_at: now_epoch_secs(),
    };
    let json = serde_json::to_string_pretty(&notified).unwrap();
    atomic_write(&notified_file_path(), &json);
}

fn read_check_file() -> Option<UpdateCheck> {
    let content = std::fs::read_to_string(check_file_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn read_notified_file() -> Option<UpdateNotified> {
    let content = std::fs::read_to_string(notified_file_path()).ok()?;
    serde_json::from_str(&content).ok()
}

fn atomic_write(path: &PathBuf, content: &str) {
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, content).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
}

fn now_epoch_secs() -> String {
    let d = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    format!("{}", d.as_secs())
}

fn parse_epoch_or_zero(s: &str) -> SystemTime {
    if let Ok(secs) = s.parse::<u64>() {
        return SystemTime::UNIX_EPOCH + Duration::from_secs(secs);
    }
    SystemTime::UNIX_EPOCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_path_identifies_sources() {
        assert_eq!(
            classify_path("/home/user/.tuhucar/bin/tuhucar"),
            InstallSource::InstallSh
        );
        assert_eq!(
            classify_path("/usr/lib/node_modules/@tuhucar/cli/bin/tuhucar"),
            InstallSource::Npm
        );
        assert_eq!(
            classify_path("/opt/homebrew/Cellar/tuhucar/0.1.0/bin/tuhucar"),
            InstallSource::Homebrew
        );
        assert_eq!(
            classify_path("/home/linuxbrew/.linuxbrew/homebrew/bin/tuhucar"),
            InstallSource::Homebrew
        );
        assert_eq!(classify_path("/usr/local/bin/tuhucar"), InstallSource::Unknown);
    }

    #[test]
    fn update_status_serializes_correctly() {
        let check = UpdateCheck {
            checked_at: "1234567890".into(),
            current: "0.1.0".into(),
            latest: "0.2.0".into(),
            status: UpdateStatus::UpdateAvailable,
            install_source: InstallSource::Npm,
            staging_path: None,
        };
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("update_available"));
        assert!(json.contains("npm"));
    }

    #[test]
    fn notified_file_roundtrips() {
        let notified = UpdateNotified {
            latest: "0.2.0".into(),
            notified_at: "1234567890".into(),
        };
        let json = serde_json::to_string(&notified).unwrap();
        let parsed: UpdateNotified = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.latest, "0.2.0");
    }

    // ── parse_epoch_or_zero tests ──────────────────────────────────────

    #[test]
    fn parse_epoch_or_zero_valid() {
        let t = parse_epoch_or_zero("1700000000");
        let expected = SystemTime::UNIX_EPOCH + Duration::from_secs(1700000000);
        assert_eq!(t, expected);
    }

    #[test]
    fn parse_epoch_or_zero_invalid_returns_epoch() {
        let t = parse_epoch_or_zero("not-a-number");
        assert_eq!(t, SystemTime::UNIX_EPOCH);
    }

    #[test]
    fn parse_epoch_or_zero_empty_returns_epoch() {
        let t = parse_epoch_or_zero("");
        assert_eq!(t, SystemTime::UNIX_EPOCH);
    }

    // ── HOME-dependent test helpers ────────────────────────────────────

    use std::env;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Set HOME to a tempdir and return it. Caller must hold the tempdir alive.
    fn setup_test_home() -> std::path::PathBuf {
        let dir = env::temp_dir().join(format!("tuhucar-test-{}", std::process::id()));
        let tuhucar_dir = dir.join(".tuhucar");
        std::fs::create_dir_all(&tuhucar_dir).unwrap();
        env::set_var("HOME", &dir);
        dir
    }

    fn cleanup_test_home(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    fn write_check_file(dir: &std::path::Path, check: &UpdateCheck) {
        let path = dir.join(".tuhucar").join(".update_check");
        let json = serde_json::to_string_pretty(check).unwrap();
        std::fs::write(path, json).unwrap();
    }

    fn write_notified_file(dir: &std::path::Path, notified: &UpdateNotified) {
        let path = dir.join(".tuhucar").join(".update_notified");
        let json = serde_json::to_string_pretty(notified).unwrap();
        std::fs::write(path, json).unwrap();
    }

    // ── should_check tests ─────────────────────────────────────────────

    #[test]
    fn should_check_returns_true_when_no_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        assert!(should_check("0.1.0"));
        cleanup_test_home(&dir);
    }

    #[test]
    fn should_check_returns_true_when_version_changed() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: format!(
                    "{}",
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                ),
                current: "0.1.0".into(),
                latest: "0.1.0".into(),
                status: UpdateStatus::UpToDate,
                install_source: InstallSource::Unknown,
                staging_path: None,
            },
        );
        // Version changed from 0.1.0 to 0.2.0 → should check
        assert!(should_check("0.2.0"));
        // Verify cleanup happened
        assert!(!dir.join(".tuhucar").join(".update_check").exists());
        cleanup_test_home(&dir);
    }

    #[test]
    fn should_check_returns_false_when_recently_checked() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: format!("{}", now),
                current: "0.1.0".into(),
                latest: "0.1.0".into(),
                status: UpdateStatus::UpToDate,
                install_source: InstallSource::Unknown,
                staging_path: None,
            },
        );
        assert!(!should_check("0.1.0"));
        cleanup_test_home(&dir);
    }

    #[test]
    fn should_check_uses_short_interval_for_failed() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        // Set checked_at to 2 hours ago — longer than FAILED_RETRY (1h) but shorter than CHECK_INTERVAL (24h)
        let two_hours_ago = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 7200;
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: format!("{}", two_hours_ago),
                current: "0.1.0".into(),
                latest: "0.1.0".into(),
                status: UpdateStatus::CheckFailed,
                install_source: InstallSource::Unknown,
                staging_path: None,
            },
        );
        // CheckFailed uses 1h interval, 2h ago > 1h → should check
        assert!(should_check("0.1.0"));
        cleanup_test_home(&dir);
    }

    #[test]
    fn should_check_uses_long_interval_for_up_to_date() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        // Set checked_at to 2 hours ago
        let two_hours_ago = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - 7200;
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: format!("{}", two_hours_ago),
                current: "0.1.0".into(),
                latest: "0.1.0".into(),
                status: UpdateStatus::UpToDate,
                install_source: InstallSource::Unknown,
                staging_path: None,
            },
        );
        // UpToDate uses 24h interval, 2h ago < 24h → should NOT check
        assert!(!should_check("0.1.0"));
        cleanup_test_home(&dir);
    }

    // ── pending_notice tests ───────────────────────────────────────────

    #[test]
    fn pending_notice_returns_none_when_no_file() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        assert!(pending_notice().is_none());
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_returns_none_for_up_to_date() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.1.0".into(),
                status: UpdateStatus::UpToDate,
                install_source: InstallSource::Npm,
                staging_path: None,
            },
        );
        assert!(pending_notice().is_none());
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_returns_notice_for_update_available() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                status: UpdateStatus::UpdateAvailable,
                install_source: InstallSource::Npm,
                staging_path: None,
            },
        );
        let notice = pending_notice().unwrap();
        match notice {
            Notice::Update {
                current,
                latest,
                message,
            } => {
                assert_eq!(current, "0.1.0");
                assert_eq!(latest, "0.2.0");
                assert!(message.contains("npm update"));
            }
        }
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_returns_homebrew_message() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                status: UpdateStatus::UpdateAvailable,
                install_source: InstallSource::Homebrew,
                staging_path: None,
            },
        );
        let notice = pending_notice().unwrap();
        let Notice::Update { message, .. } = notice;
        assert!(message.contains("brew upgrade"));
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_returns_install_sh_message() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                status: UpdateStatus::UpdateAvailable,
                install_source: InstallSource::InstallSh,
                staging_path: None,
            },
        );
        let notice = pending_notice().unwrap();
        let Notice::Update { message, .. } = notice;
        assert!(message.contains("自动更新"));
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_skips_already_notified_version() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                status: UpdateStatus::UpdateAvailable,
                install_source: InstallSource::Npm,
                staging_path: None,
            },
        );
        write_notified_file(
            &dir,
            &UpdateNotified {
                latest: "0.2.0".into(),
                notified_at: "456".into(),
            },
        );
        assert!(pending_notice().is_none());
        cleanup_test_home(&dir);
    }

    #[test]
    fn pending_notice_shows_for_newer_version() {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = setup_test_home();
        write_check_file(
            &dir,
            &UpdateCheck {
                checked_at: "123".into(),
                current: "0.1.0".into(),
                latest: "0.3.0".into(),
                status: UpdateStatus::Downloaded,
                install_source: InstallSource::Unknown,
                staging_path: None,
            },
        );
        write_notified_file(
            &dir,
            &UpdateNotified {
                latest: "0.2.0".into(),
                notified_at: "456".into(),
            },
        );
        let notice = pending_notice().unwrap();
        let Notice::Update { latest, message, .. } = notice;
        assert_eq!(latest, "0.3.0");
        assert!(message.contains("手动更新"));
        cleanup_test_home(&dir);
    }
}
