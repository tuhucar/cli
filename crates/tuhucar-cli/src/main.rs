mod commands;

use clap::Parser;
use tuhucar_core::{ApiError, OutputFormat, Response, TuhucarError};

/// tuhucar - 途虎养车 CLI 工具
#[derive(Parser)]
#[command(name = "tuhucar", version, about)]
struct Cli {
    /// Output format: json or markdown
    #[arg(long, global = true, default_value = "markdown")]
    format: String,

    /// Preview request without sending
    #[arg(long, global = true)]
    dry_run: bool,

    /// Verbose output
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: commands::Commands,
}

fn pre_scan_format() -> bool {
    std::env::args().any(|a| a == "--format=json")
        || std::env::args()
            .collect::<Vec<_>>()
            .windows(2)
            .any(|w| w[0] == "--format" && w[1] == "json")
}

#[tokio::main]
async fn main() {
    let wants_json = pre_scan_format();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if wants_json {
                let api_err = ApiError::from(TuhucarError::InvalidArgs {
                    message: e.to_string(),
                    suggestion: "Run: tuhucar --help".to_string(),
                });
                let resp: Response<()> = Response::error(api_err, None);
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
                std::process::exit(2);
            } else {
                e.exit();
            }
        }
    };

    let format = OutputFormat::from_str_opt(&cli.format).unwrap_or(OutputFormat::Markdown);

    if let Err(e) = commands::run(cli.command, format, cli.dry_run, cli.verbose).await {
        let api_err: ApiError = e.into();
        let resp: Response<()> = Response::error(api_err, None);
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            }
            OutputFormat::Markdown => {
                eprintln!("Error [{}]: {}", resp.error.as_ref().unwrap().code, resp.error.as_ref().unwrap().message);
                if let Some(s) = &resp.error.as_ref().unwrap().suggestion {
                    eprintln!("\n  {}", s);
                }
            }
        }
        std::process::exit(1);
    }
}
