use clap::Subcommand;
use std::path::PathBuf;
use tuhucar_core::{OutputFormat, TuhucarError};

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
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    // Try to find skills directory relative to the executable or current dir
    let skills_dir = find_skills_dir(&exe_dir);

    let mut results = Vec::new();

    // Claude Code
    results.push(install_claude_code(&home, &skills_dir));

    // Cursor
    results.push(install_cursor(&home, &skills_dir));

    // Codex
    results.push(install_codex(&skills_dir));

    // OpenCode
    results.push(install_opencode(&home));

    // Gemini CLI
    results.push(install_gemini(&home, &skills_dir));

    // Print summary
    println!("\nSkill installation summary:");
    for r in &results {
        let (icon, detail) = match &r.status {
            PlatformStatus::Installed => ("\u{2713}", "installed".to_string()),
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

    Ok(())
}

fn uninstall_skills() -> Result<(), TuhucarError> {
    let home = dirs::home_dir().expect("Cannot determine home directory");
    let mut results = Vec::new();

    // Claude Code
    let claude_dir = home.join(".claude").join("skills").join("tuhucar");
    results.push(uninstall_dir("Claude Code", &claude_dir));

    // Cursor
    let cursor_dir = home.join(".cursor").join("skills").join("tuhucar");
    results.push(uninstall_dir("Cursor", &cursor_dir));

    // Codex
    if let Ok(codex_home) = std::env::var("CODEX_HOME") {
        let codex_link = PathBuf::from(&codex_home).join("skills").join("tuhucar");
        results.push(uninstall_dir("Codex", &codex_link));
    } else {
        results.push(PlatformResult {
            name: "Codex",
            status: PlatformStatus::Skipped("not detected".into()),
        });
    }

    // OpenCode -- just report, don't modify opencode.json automatically
    results.push(PlatformResult {
        name: "OpenCode",
        status: PlatformStatus::Skipped("manual removal needed from opencode.json".into()),
    });

    // Gemini CLI
    let gemini_dir = home.join(".gemini").join("extensions").join("tuhucar");
    results.push(uninstall_dir("Gemini CLI", &gemini_dir));

    println!("\nSkill uninstallation summary:");
    for r in &results {
        let (icon, detail) = match &r.status {
            PlatformStatus::Installed => ("\u{2713}", "removed".to_string()),
            PlatformStatus::Skipped(reason) => ("-", format!("skipped ({})", reason)),
            PlatformStatus::Failed(err) => ("\u{2717}", format!("failed ({})", err)),
        };
        println!("  {} {:<14} \u{2014} {}", icon, r.name, detail);
    }

    Ok(())
}

fn find_skills_dir(exe_dir: &Option<PathBuf>) -> Option<PathBuf> {
    // Try relative to executable: ../skills, ../../skills
    if let Some(dir) = exe_dir {
        for ancestor in [dir.as_path(), dir.parent().unwrap_or(dir)] {
            let candidate = ancestor.join("skills");
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }
    // Try current directory
    let cwd = std::env::current_dir().ok()?;
    let candidate = cwd.join("skills");
    if candidate.is_dir() {
        return Some(candidate);
    }
    None
}

fn install_claude_code(home: &PathBuf, skills_dir: &Option<PathBuf>) -> PlatformResult {
    let claude_dir = home.join(".claude");
    if !claude_dir.exists() {
        return PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let Some(src) = skills_dir else {
        return PlatformResult {
            name: "Claude Code",
            status: PlatformStatus::Failed("skills directory not found".into()),
        };
    };
    let dest = claude_dir.join("skills").join("tuhucar");
    match copy_dir_recursive(src, &dest) {
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

fn install_cursor(home: &PathBuf, skills_dir: &Option<PathBuf>) -> PlatformResult {
    let cursor_dir = home.join(".cursor");
    if !cursor_dir.exists() {
        return PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let Some(src) = skills_dir else {
        return PlatformResult {
            name: "Cursor",
            status: PlatformStatus::Failed("skills directory not found".into()),
        };
    };
    let dest = cursor_dir.join("skills").join("tuhucar");
    match copy_dir_recursive(src, &dest) {
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

fn install_codex(skills_dir: &Option<PathBuf>) -> PlatformResult {
    let codex_home = match std::env::var("CODEX_HOME") {
        Ok(h) => PathBuf::from(h),
        Err(_) => {
            return PlatformResult {
                name: "Codex",
                status: PlatformStatus::Skipped("not detected".into()),
            }
        }
    };
    let Some(src) = skills_dir else {
        return PlatformResult {
            name: "Codex",
            status: PlatformStatus::Failed("skills directory not found".into()),
        };
    };
    let dest = codex_home.join("skills").join("tuhucar");
    if let Err(e) = std::fs::create_dir_all(dest.parent().unwrap()) {
        return PlatformResult {
            name: "Codex",
            status: PlatformStatus::Failed(e.to_string()),
        };
    }
    // Remove existing symlink/dir before creating new one
    let _ = std::fs::remove_file(&dest);
    let _ = std::fs::remove_dir_all(&dest);
    #[cfg(unix)]
    {
        match std::os::unix::fs::symlink(src, &dest) {
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
    #[cfg(not(unix))]
    {
        match copy_dir_recursive(src, &dest) {
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
}

fn install_opencode(home: &PathBuf) -> PlatformResult {
    let opencode_dir = home.join(".opencode");
    if !opencode_dir.exists() {
        return PlatformResult {
            name: "OpenCode",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    // For now, just report as needing manual setup
    // Full JSON manipulation would be added in a future iteration
    PlatformResult {
        name: "OpenCode",
        status: PlatformStatus::Skipped("manual setup needed".into()),
    }
}

fn install_gemini(home: &PathBuf, skills_dir: &Option<PathBuf>) -> PlatformResult {
    let gemini_dir = home.join(".gemini");
    if !gemini_dir.exists() {
        return PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Skipped("not detected".into()),
        };
    }
    let Some(src) = skills_dir else {
        return PlatformResult {
            name: "Gemini CLI",
            status: PlatformStatus::Failed("skills directory not found".into()),
        };
    };
    let dest = gemini_dir.join("extensions").join("tuhucar");
    match copy_dir_recursive(src, &dest) {
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

fn uninstall_dir(name: &'static str, path: &PathBuf) -> PlatformResult {
    if !path.exists() {
        return PlatformResult {
            name,
            status: PlatformStatus::Skipped("not installed".into()),
        };
    }
    match std::fs::remove_dir_all(path) {
        Ok(_) => PlatformResult {
            name,
            status: PlatformStatus::Installed,
        },
        Err(e) => PlatformResult {
            name,
            status: PlatformStatus::Failed(e.to_string()),
        },
    }
}

fn copy_dir_recursive(src: &PathBuf, dest: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
