use clap::Subcommand;
use tuhucar_core::config::Config;
use tuhucar_core::http::HttpClient;
use tuhucar_core::output::format_response;
use tuhucar_core::{Command as TuhucarCommand, OutputFormat, Response, TuhucarError};
use tuhucar_knowledge::KnowledgeCommand;
use tuhucar_knowledge::models::KnowledgeQueryInput;

#[derive(Subcommand)]
pub enum KnowledgeAction {
    /// Query car maintenance knowledge
    Query {
        /// Five-level car model ID
        #[arg(long)]
        car_id: String,
        /// Question to ask
        question: String,
    },
    /// Show knowledge query command schema (for LLM introspection)
    Schema,
}

pub async fn run(
    action: KnowledgeAction,
    format: OutputFormat,
    dry_run: bool,
    _verbose: bool,
) -> Result<(), TuhucarError> {
    match action {
        KnowledgeAction::Schema => {
            let config = Config::default_config();
            let client = HttpClient::new(&config);
            let cmd = KnowledgeCommand::new(client);
            let schema = cmd.schema();
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
            Ok(())
        }
        KnowledgeAction::Query { car_id, question } => {
            if dry_run {
                println!("GET /api/v1/knowledge/query?car_id={}&q={}", car_id, question);
                return Ok(());
            }
            let config = Config::load()?;
            let client = HttpClient::new(&config);
            let cmd = KnowledgeCommand::new(client);
            let input = KnowledgeQueryInput { car_id, question };
            let result = cmd.execute(input).await?;
            let resp = Response::success(result, None);
            println!("{}", format_response(&resp, format));
            Ok(())
        }
    }
}
