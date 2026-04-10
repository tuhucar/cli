use clap::Subcommand;
use std::path::{Path, PathBuf};
use tuhucar_core::{OutputFormat, TuhucarError};

// Embed skill assets at compile time so they're always available
const SHARED_SKILL: &str = include_str!("../../../../skills/tuhucar-shared/SKILL.md");
const CAR_ASSISTANT_SKILL: &str = include_str!("../../../../skills/tuhucar-car-assistant/SKILL.md");
const COMMAND_REFERENCE: &str =
    include_str!("../../../../skills/tuhucar-car-assistant/references/command-reference.md");
const CLAUDE_PLUGIN_JSON: &str = include_str!("../../../../.claude-plugin/plugin.json");
const CURSOR_PLUGIN_JSON: &str = include_str!("../../../../.cursor-plugin/plugin.json");
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
        uninstall_codex(),
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

    let any_installed = results
        .iter()
        .any(|r| matches!(r.status, PlatformStatus::Installed));
    let any_failed = results
        .iter()
        .any(|r| matches!(r.status, PlatformStatus::Failed(_)));

    println!();
    if any_installed {
        println!("CLI is ready.");
    }
    if any_failed {
        println!("Run `tuhucar skill install` to retry failed platforms.");
    }
}

/// Write embedded skill files to a directory
fn write_skill_files(dest: &Path) -> std::io::Result<()> {
    // tuhucar-shared/SKILL.md
    let shared_dir = dest.join("tuhucar-shared");
    std::fs::create_dir_all(&shared_dir)?;
    std::fs::write(shared_dir.join("SKILL.md"), SHARED_SKILL)?;

    // tuhucar-car-assistant/SKILL.md + references/command-reference.md
    let car_dir = dest.join("tuhucar-car-assistant");
    let ref_dir = car_dir.join("references");
    std::fs::create_dir_all(&ref_dir)?;
    std::fs::write(car_dir.join("SKILL.md"), CAR_ASSISTANT_SKILL)?;
    std::fs::write(ref_dir.join("command-reference.md"), COMMAND_REFERENCE)?;

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
        // Write skill files
        let skills_dest = claude_dir.join("skills").join("tuhucar");
        write_skill_files(&skills_dest)?;
        // Register plugin manifest
        let plugin_dir = claude_dir.join("plugins").join("tuhucar");
        std::fs::create_dir_all(&plugin_dir)?;
        std::fs::write(plugin_dir.join("plugin.json"), CLAUDE_PLUGIN_JSON)?;
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
    let skills_dir = home.join(".claude").join("skills").join("tuhucar");
    let plugin_dir = home.join(".claude").join("plugins").join("tuhucar");
    if !skills_dir.exists() && !plugin_dir.exists() {
        return PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&skills_dir);
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
        let skills_dest = cursor_dir.join("skills").join("tuhucar");
        write_skill_files(&skills_dest)?;
        let plugin_dir = cursor_dir.join("plugins").join("tuhucar");
        std::fs::create_dir_all(&plugin_dir)?;
        std::fs::write(plugin_dir.join("plugin.json"), CURSOR_PLUGIN_JSON)?;
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
    let skills_dir = home.join(".cursor").join("skills").join("tuhucar");
    let plugin_dir = home.join(".cursor").join("plugins").join("tuhucar");
    if !skills_dir.exists() && !plugin_dir.exists() {
        return PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    let _ = std::fs::remove_dir_all(&skills_dir);
    let _ = std::fs::remove_dir_all(&plugin_dir);
    PlatformResult {
        name: "Cursor",
        status: PlatformStatus::Installed,
    }
}

// --- Codex ---

fn install_codex(_home: &Path) -> PlatformResult {
    let codex_home = match std::env::var("CODEX_HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
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

fn uninstall_codex() -> PlatformResult {
    let codex_home = match std::env::var("CODEX_HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
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

const OPENCODE_PLUGIN_ENTRY: &str =
    r#"{"name":"tuhucar","type":"skill","path":"~/.opencode/skills/tuhucar"}"#;

fn install_opencode(home: &Path) -> PlatformResult {
    let opencode_dir = home.join(".opencode");
    if !opencode_dir.exists() {
        return PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let result = (|| -> std::io::Result<()> {
        // Write skill files
        let skills_dest = opencode_dir.join("skills").join("tuhucar");
        write_skill_files(&skills_dest)?;

        // Edit opencode.json to add plugin entry
        let config_path = opencode_dir.join("opencode.json");
        let mut config: serde_json::Value = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        let entry: serde_json::Value = serde_json::from_str(OPENCODE_PLUGIN_ENTRY).unwrap();

        // Get or create plugins array
        let plugins = config
            .as_object_mut()
            .unwrap()
            .entry("plugins")
            .or_insert_with(|| serde_json::json!([]))
            .as_array_mut()
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, "plugins is not an array")
            })?;

        // Remove existing tuhucar entry if present
        plugins.retain(|p| p.get("name").and_then(|n| n.as_str()) != Some("tuhucar"));
        plugins.push(entry);

        std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap())?;
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
    let opencode_dir = home.join(".opencode");
    let skills_dir = opencode_dir.join("skills").join("tuhucar");
    let config_path = opencode_dir.join("opencode.json");

    if !skills_dir.exists() && !config_path.exists() {
        return PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }

    let _ = std::fs::remove_dir_all(&skills_dir);

    // Remove plugin entry from opencode.json
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(plugins) = config.get_mut("plugins").and_then(|p| p.as_array_mut()) {
                    plugins.retain(|p| p.get("name").and_then(|n| n.as_str()) != Some("tuhucar"));
                    let _ = std::fs::write(
                        &config_path,
                        serde_json::to_string_pretty(&config).unwrap(),
                    );
                }
            }
        }
    }

    PlatformResult {
        name: "OpenCode",
        status: PlatformStatus::Installed,
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
