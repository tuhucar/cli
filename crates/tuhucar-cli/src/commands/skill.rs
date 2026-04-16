use clap::Subcommand;
use std::path::{Path, PathBuf};
use tuhucar_core::{OutputFormat, TuhucarError};

// Embed skill assets at compile time so they're always available
const SHARED_SKILL: &str = include_str!("../../../../skills/tuhucar-shared/SKILL.md");
const CAR_ASSISTANT_SKILL: &str = include_str!("../../../../skills/tuhucar-car-assistant/SKILL.md");
const COMMAND_REFERENCE: &str =
    include_str!("../../../../skills/tuhucar-car-assistant/references/command-reference.md");
const GEMINI_MD: &str = include_str!("../../../../GEMINI.md");
const GEMINI_EXTENSION_JSON: &str = include_str!("../../../../gemini-extension.json");

#[derive(Subcommand)]
pub enum SkillAction {
    /// Install skills to detected AI platforms
    Install,
    /// Uninstall skills from AI platforms
    Uninstall,
}

#[derive(Debug)]
enum PlatformStatus {
    Installed,
    Skipped(String),
    Failed(String),
}

struct PlatformResult {
    name: &'static str,
    status: PlatformStatus,
}

pub async fn run(action: SkillAction, _format: OutputFormat) -> Result<(), TuhucarError> {
    match action {
        SkillAction::Install => install_skills(),
        SkillAction::Uninstall => uninstall_skills(),
    }
}

fn install_skills() -> Result<(), TuhucarError> {
    let home = dirs::home_dir().expect("Cannot determine home directory");

    let results = vec![
        install_claude_code(&home),
        install_cursor(&home),
        install_codex(&home),
        install_opencode(&home),
        install_gemini(&home),
    ];

    print_summary("installation", &results);
    Ok(())
}

fn uninstall_skills() -> Result<(), TuhucarError> {
    let home = dirs::home_dir().expect("Cannot determine home directory");

    let results = vec![
        uninstall_claude_code(&home),
        uninstall_cursor(&home),
        uninstall_codex(&home),
        uninstall_opencode(&home),
        uninstall_gemini(&home),
    ];

    print_summary("uninstallation", &results);
    Ok(())
}

fn print_summary(action: &str, results: &[PlatformResult]) {
    println!("\nSkill {} summary:", action);
    for r in results {
        let (icon, detail) = match &r.status {
            PlatformStatus::Installed => (
                "\u{2713}",
                if action == "uninstallation" {
                    "removed".to_string()
                } else {
                    "installed".to_string()
                },
            ),
            PlatformStatus::Skipped(reason) => ("-", format!("skipped ({})", reason)),
            PlatformStatus::Failed(err) => ("\u{2717}", format!("failed ({})", err)),
        };
        println!("  {} {:<14} \u{2014} {}", icon, r.name, detail);
    }

    let any_installed = results.iter().any(|r| matches!(r.status, PlatformStatus::Installed));
    let any_failed = results.iter().any(|r| matches!(r.status, PlatformStatus::Failed(_)));

    println!();
    if any_installed {
        println!("CLI is ready.");
    }
    if any_failed {
        println!("Run `tuhucar skill install` to retry failed platforms.");
    }
}

fn write_shared_skill(dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    std::fs::write(dest.join("SKILL.md"), SHARED_SKILL)?;
    Ok(())
}

fn write_knowledge_skill(dest: &Path) -> std::io::Result<()> {
    let ref_dir = dest.join("references");
    std::fs::create_dir_all(&ref_dir)?;
    std::fs::write(dest.join("SKILL.md"), CAR_ASSISTANT_SKILL)?;
    std::fs::write(ref_dir.join("command-reference.md"), COMMAND_REFERENCE)?;
    Ok(())
}

/// Write embedded skill files to a namespaced directory.
fn write_skill_files(dest: &Path) -> std::io::Result<()> {
    write_shared_skill(&dest.join("tuhucar-shared"))?;
    write_knowledge_skill(&dest.join("tuhucar-car-assistant"))?;
    Ok(())
}

/// Write skills into directories that assistants discover directly.
fn write_direct_skill_files(skills_root: &Path) -> std::io::Result<()> {
    write_shared_skill(&skills_root.join("tuhucar-shared"))?;
    write_knowledge_skill(&skills_root.join("tuhucar-knowledge-assistant"))?;
    Ok(())
}

fn prune_opencode_legacy_plugin_entries(config: &mut serde_json::Value) -> bool {
    let Some(config_object) = config.as_object_mut() else {
        return false;
    };

    let (changed, should_remove_key) = {
        let Some(plugins) = config_object.get_mut("plugins").and_then(|value| value.as_array_mut()) else {
            return false;
        };

        let original_len = plugins.len();
        plugins.retain(|entry| {
            entry.get("name").and_then(|value| value.as_str()) != Some("tuhucar")
                || entry.get("type").and_then(|value| value.as_str()) != Some("skill")
        });

        (plugins.len() != original_len, plugins.is_empty())
    };

    if should_remove_key {
        config_object.remove("plugins");
    }

    changed
}

fn cleanup_opencode_legacy_plugin_entries(opencode_dir: &Path) -> std::io::Result<()> {
    for config_path in [opencode_dir.join("opencode.jsonc"), opencode_dir.join("opencode.json")] {
        if !config_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&config_path)?;
        let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) else {
            continue;
        };

        if prune_opencode_legacy_plugin_entries(&mut config) {
            std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())?;
        }
    }

    Ok(())
}

// --- Claude Code ---

fn install_claude_code(home: &Path) -> PlatformResult {
    let claude_dir = home.join(".claude");
    if !claude_dir.exists() {
        return PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let result = (|| -> std::io::Result<()> {
        let skills_root = claude_dir.join("skills");
        write_direct_skill_files(&skills_root)?;

        // Clean up the legacy namespaced skill/plugin layout from earlier installs.
        let _ = std::fs::remove_dir_all(skills_root.join("tuhucar"));
        let _ = std::fs::remove_dir_all(claude_dir.join("plugins").join("tuhucar"));
        Ok(())
    })();
    match result {
        Ok(_) => PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn uninstall_claude_code(home: &Path) -> PlatformResult {
    let skills_root = home.join(".claude").join("skills");
    let legacy_skills_dir = skills_root.join("tuhucar");
    let shared_skill_dir = skills_root.join("tuhucar-shared");
    let assistant_skill_dir = skills_root.join("tuhucar-knowledge-assistant");
    let plugin_dir = home.join(".claude").join("plugins").join("tuhucar");
    if !legacy_skills_dir.exists()
        && !shared_skill_dir.exists()
        && !assistant_skill_dir.exists()
        && !plugin_dir.exists()
    {
        return PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&legacy_skills_dir);
    let _ = std::fs::remove_dir_all(&shared_skill_dir);
    let _ = std::fs::remove_dir_all(&assistant_skill_dir);
    let _ = std::fs::remove_dir_all(&plugin_dir);
    PlatformResult {
        name: "Claude Code",
        status: PlatformStatus::Installed,
    }
}

// --- Cursor ---

fn install_cursor(home: &Path) -> PlatformResult {
    let cursor_dir = home.join(".cursor");
    if !cursor_dir.exists() {
        return PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let result = (|| -> std::io::Result<()> {
        let skills_root = cursor_dir.join("skills");
        write_direct_skill_files(&skills_root)?;

        // Clean up the legacy namespaced skill/plugin layout from earlier installs.
        let _ = std::fs::remove_dir_all(skills_root.join("tuhucar"));
        let _ = std::fs::remove_dir_all(cursor_dir.join("plugins").join("tuhucar"));
        Ok(())
    })();
    match result {
        Ok(_) => PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn uninstall_cursor(home: &Path) -> PlatformResult {
    let skills_root = home.join(".cursor").join("skills");
    let legacy_skills_dir = skills_root.join("tuhucar");
    let shared_skill_dir = skills_root.join("tuhucar-shared");
    let assistant_skill_dir = skills_root.join("tuhucar-knowledge-assistant");
    let plugin_dir = home.join(".cursor").join("plugins").join("tuhucar");
    if !legacy_skills_dir.exists()
        && !shared_skill_dir.exists()
        && !assistant_skill_dir.exists()
        && !plugin_dir.exists()
    {
        return PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&legacy_skills_dir);
    let _ = std::fs::remove_dir_all(&shared_skill_dir);
    let _ = std::fs::remove_dir_all(&assistant_skill_dir);
    let _ = std::fs::remove_dir_all(&plugin_dir);
    PlatformResult {
        name: "Cursor",
        status: PlatformStatus::Installed,
    }
}

// --- Codex ---

fn resolve_codex_home(home: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CODEX_HOME") {
        let codex_home = PathBuf::from(path);
        if codex_home.exists() {
            return Some(codex_home);
        }
    }

    let default_home = home.join(".codex");
    if default_home.exists() {
        return Some(default_home);
    }

    None
}

fn install_codex(home: &Path) -> PlatformResult {
    let codex_home = match resolve_codex_home(home) {
        Some(path) => path,
        None => {
            return PlatformResult {
                name: "Codex",
                status: PlatformStatus::Skipped("not detected".into()),
            }
        }
    };
    let result = (|| -> std::io::Result<()> {
        let skills_dest = codex_home.join("skills").join("tuhucar");
        // Remove existing before writing fresh
        let _ = std::fs::remove_dir_all(&skills_dest);
        write_skill_files(&skills_dest)?;
        Ok(())
    })();
    match result {
        Ok(_) => PlatformResult {
            name: "Codex",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "Codex",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn uninstall_codex(home: &Path) -> PlatformResult {
    let codex_home = match resolve_codex_home(home) {
        Some(path) => path,
        None => {
            return PlatformResult {
                name: "Codex",
                status: PlatformStatus::Skipped("not detected".into()),
            }
        }
    };
    let skills_dir = codex_home.join("skills").join("tuhucar");
    if !skills_dir.exists() {
        return PlatformResult {
            name: "Codex",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&skills_dir);
    PlatformResult {
        name: "Codex",
        status: PlatformStatus::Installed,
    }
}

// --- OpenCode ---

fn resolve_opencode_dir(home: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("OPENCODE_CONFIG_DIR") {
        let opencode_dir = PathBuf::from(path);
        if opencode_dir.exists() {
            return Some(opencode_dir);
        }
    }

    let xdg_dir = home.join(".config").join("opencode");
    if xdg_dir.exists() {
        return Some(xdg_dir);
    }

    let legacy_dir = home.join(".opencode");
    if legacy_dir.exists() {
        return Some(legacy_dir);
    }

    None
}

fn install_opencode(home: &Path) -> PlatformResult {
    let opencode_dir = match resolve_opencode_dir(home) {
        Some(path) => path,
        None => {
            return PlatformResult {
                name: "OpenCode",
                status: PlatformStatus::Skipped("not detected".into()),
            }
        }
    };
    let result = (|| -> std::io::Result<()> {
        let skills_root = opencode_dir.join("skills");
        write_direct_skill_files(&skills_root)?;

        // Remove the legacy bundled directory from earlier installs.
        let legacy_bundle_dir = skills_root.join("tuhucar");
        let _ = std::fs::remove_dir_all(&legacy_bundle_dir);

        cleanup_opencode_legacy_plugin_entries(&opencode_dir)?;
        Ok(())
    })();
    match result {
        Ok(_) => PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn uninstall_opencode(home: &Path) -> PlatformResult {
    let opencode_dir = match resolve_opencode_dir(home) {
        Some(path) => path,
        None => {
            return PlatformResult {
                name: "OpenCode",
                status: PlatformStatus::Skipped("not detected".into()),
            }
        }
    };
    let skills_root = opencode_dir.join("skills");
    let legacy_bundle_dir = skills_root.join("tuhucar");
    let shared_skill_dir = skills_root.join("tuhucar-shared");
    let assistant_skill_dir = skills_root.join("tuhucar-knowledge-assistant");
    let has_config_file = opencode_dir.join("opencode.json").exists() || opencode_dir.join("opencode.jsonc").exists();

    if !legacy_bundle_dir.exists() && !shared_skill_dir.exists() && !assistant_skill_dir.exists() && !has_config_file {
        return PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }

    let result = (|| -> std::io::Result<()> {
        let _ = std::fs::remove_dir_all(&legacy_bundle_dir);
        let _ = std::fs::remove_dir_all(&shared_skill_dir);
        let _ = std::fs::remove_dir_all(&assistant_skill_dir);
        cleanup_opencode_legacy_plugin_entries(&opencode_dir)?;
        Ok(())
    })();

    match result {
        Ok(_) => PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

// --- Gemini CLI ---

fn install_gemini(home: &Path) -> PlatformResult {
    let gemini_dir = home.join(".gemini");
    if !gemini_dir.exists() {
        return PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let result = (|| -> std::io::Result<()> {
        let ext_dir = gemini_dir.join("extensions").join("tuhucar");
        std::fs::create_dir_all(&ext_dir)?;
        std::fs::write(ext_dir.join("GEMINI.md"), GEMINI_MD)?;
        std::fs::write(ext_dir.join("gemini-extension.json"), GEMINI_EXTENSION_JSON)?;
        Ok(())
    })();
    match result {
        Ok(_) => PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn uninstall_gemini(home: &Path) -> PlatformResult {
    let ext_dir = home.join(".gemini").join("extensions").join("tuhucar");
    if !ext_dir.exists() {
        return PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&ext_dir);
    PlatformResult {
        name: "Gemini CLI",
        status: PlatformStatus::Installed,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        prune_opencode_legacy_plugin_entries, resolve_codex_home, resolve_opencode_dir, write_direct_skill_files,
    };
    use serde_json::json;

    #[test]
    fn resolve_codex_home_falls_back_to_default_directory() {
        let temp_home = std::env::temp_dir().join(format!("tuhucar-skill-test-{}", std::process::id()));
        let codex_home = temp_home.join(".codex");
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&codex_home).unwrap();
        std::env::remove_var("CODEX_HOME");

        let resolved = resolve_codex_home(&temp_home).unwrap();
        assert_eq!(resolved, codex_home);

        let _ = std::fs::remove_dir_all(&temp_home);
    }

    #[test]
    fn resolve_opencode_dir_prefers_xdg_directory() {
        let temp_home = std::env::temp_dir().join(format!("tuhucar-opencode-test-{}", std::process::id()));
        let xdg_dir = temp_home.join(".config").join("opencode");
        let legacy_dir = temp_home.join(".opencode");
        let _ = std::fs::remove_dir_all(&temp_home);
        std::fs::create_dir_all(&xdg_dir).unwrap();
        std::fs::create_dir_all(&legacy_dir).unwrap();
        std::env::remove_var("OPENCODE_CONFIG_DIR");

        let resolved = resolve_opencode_dir(&temp_home).unwrap();
        assert_eq!(resolved, xdg_dir);

        let _ = std::fs::remove_dir_all(&temp_home);
    }

    #[test]
    fn write_direct_skill_files_uses_direct_skill_directories() {
        let temp_root = std::env::temp_dir().join(format!("tuhucar-opencode-layout-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_root);

        write_direct_skill_files(&temp_root).unwrap();

        assert!(temp_root.join("tuhucar-shared").join("SKILL.md").exists());
        assert!(temp_root.join("tuhucar-knowledge-assistant").join("SKILL.md").exists());
        assert!(temp_root
            .join("tuhucar-knowledge-assistant")
            .join("references")
            .join("command-reference.md")
            .exists());
        assert!(!temp_root.join("tuhucar").exists());

        let _ = std::fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn prune_opencode_legacy_plugin_entries_removes_only_legacy_tuhucar_entries() {
        let mut config = json!({
            "theme": "opencode",
            "plugins": [
                {
                    "name": "tuhucar",
                    "type": "skill",
                    "path": "/tmp/custom-opencode/skills/tuhucar"
                },
                {
                    "name": "keep-me",
                    "type": "plugin",
                    "path": "/tmp/custom-opencode/plugins/keep-me"
                }
            ]
        });

        let changed = prune_opencode_legacy_plugin_entries(&mut config);

        assert!(changed);
        assert_eq!(config["plugins"][0]["name"], "keep-me");
    }

    #[test]
    fn prune_opencode_legacy_plugin_entries_drops_empty_plugins_key() {
        let mut config = json!({
            "plugins": [
                {
                    "name": "tuhucar",
                    "type": "skill",
                    "path": "/tmp/custom-opencode/skills/tuhucar"
                }
            ]
        });

        let changed = prune_opencode_legacy_plugin_entries(&mut config);

        assert!(changed);
        assert!(config.get("plugins").is_none());
    }
}
