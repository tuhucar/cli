use clap::Subcommand;
use tuhucar_core::{OutputFormat, TuhucarError};

#[derive(Subcommand)]
pub enum SkillAction {
    /// Install skills to detected AI platforms
    Install,
    /// Uninstall skills from AI platforms
    Uninstall,
}

pub async fn run(action: SkillAction, _format: OutputFormat) -> Result<(), TuhucarError> {
    match action {
        SkillAction::Install => {
            println!("Skill installation — not yet implemented");
            Ok(())
        }
        SkillAction::Uninstall => {
            println!("Skill uninstallation — not yet implemented");
            Ok(())
        }
    }
}
