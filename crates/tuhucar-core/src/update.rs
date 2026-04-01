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
        InstallSource::Homebrew => format!("新版本 {} 可用，请运行: brew upgrade tuhucar", check.latest),
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
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
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
    fn detect_install_source_unknown_for_random_path() {
        let source = detect_install_source();
        assert_eq!(source, InstallSource::Unknown);
    }

    #[test]
    fn classify_path_identifies_sources() {
        assert_eq!(classify_path("/home/user/.tuhucar/bin/tuhucar"), InstallSource::InstallSh);
        assert_eq!(classify_path("/usr/lib/node_modules/@tuhucar/cli/bin/tuhucar"), InstallSource::Npm);
        assert_eq!(classify_path("/opt/homebrew/Cellar/tuhucar/0.1.0/bin/tuhucar"), InstallSource::Homebrew);
        assert_eq!(classify_path("/home/linuxbrew/.linuxbrew/homebrew/bin/tuhucar"), InstallSource::Homebrew);
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
}
