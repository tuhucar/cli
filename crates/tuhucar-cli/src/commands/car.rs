use clap::Subcommand;
use tuhucar_core::config::Config;
use tuhucar_core::http::HttpClient;
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
            let config = Config::default_config();
            let client = HttpClient::new(&config);
            let cmd = CarCommand::new(client);
            let schema = cmd.schema();
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
            Ok(())
        }
        CarAction::Match { query } => {
            if dry_run {
                println!("GET /api/v1/car/match?q={}", query);
                return Ok(());
            }
            let config = Config::load()?;
            let client = HttpClient::new(&config);
            let cmd = CarCommand::new(client);
            let input = CarMatchInput { query };
            let result = cmd.execute(input).await?;
            let resp = Response::success(result, Some(meta));
            println!("{}", format_response(&resp, format));
            Ok(())
        }
    }
}
