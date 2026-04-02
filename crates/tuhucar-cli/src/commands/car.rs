use clap::Subcommand;
use tuhucar_core::config::Config;
use tuhucar_core::mcp::McpClient;
use tuhucar_core::output::format_response;
use tuhucar_core::{Command as TuhucarCommand, OutputFormat, Response, ResponseMeta, TuhucarError};
use tuhucar_car::CarCommand;
use tuhucar_car::models::CarMatchInput;

#[derive(Subcommand)]
pub enum CarAction {
    /// Match a car model by description
    Match {
        /// Car description, e.g. "2024款朗逸1.5L"
        query: String,
    },
    /// Show car match command schema (for LLM introspection)
    Schema,
}

pub async fn run(
    action: CarAction,
    format: OutputFormat,
    dry_run: bool,
    _verbose: bool,
    meta: ResponseMeta,
) -> Result<(), TuhucarError> {
    match action {
        CarAction::Schema => {
            let schema = CarCommand::schema_static();
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
            Ok(())
        }
        CarAction::Match { query } => {
            if dry_run {
                println!("MCP tools/call car_match {{\"query\":\"{}\"}}", query);
                return Ok(());
            }
            let config = Config::load()?;
            let client = McpClient::connect(&config).await?;
            let cmd = CarCommand::new(client);
            let input = CarMatchInput { query };
            let result = cmd.execute(input).await?;
            let resp = Response::success(result, Some(meta));
            println!("{}", format_response(&resp, format));
            Ok(())
        }
    }
}
