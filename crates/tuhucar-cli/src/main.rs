mod commands;

use clap::Parser;
use tuhucar_core::update;
use tuhucar_core::{ApiError, Notice, OutputFormat, Response, ResponseMeta, TuhucarError};

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

/// Build ResponseMeta with version and any pending update notices.
/// Does NOT mark as notified — caller must call mark_notices_notified() after output.
fn build_meta() -> ResponseMeta {
    let mut notices = Vec::new();
    if let Some(notice) = update::pending_notice() {
        notices.push(notice);
    }
    ResponseMeta {
        version: VERSION.to_string(),
        notices,
    }
}

/// Mark all notices in meta as notified so they won't repeat.
fn mark_notices_notified(meta: &ResponseMeta) {
    for notice in &meta.notices {
        let Notice::Update { ref latest, .. } = notice;
        update::mark_notified(latest);
    }
}

#[tokio::main]
async fn main() {
    let wants_json = pre_scan_format();

    // Trigger background update check if needed (best-effort, non-blocking)
    let _ = update::should_check(VERSION);

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if wants_json {
                let api_err = ApiError::from(TuhucarError::InvalidArgs {
                    message: e.to_string(),
                    suggestion: "Run: tuhucar --help".to_string(),
                });
                let meta = build_meta();
                let resp: Response<()> = Response::error(api_err, Some(meta));
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
                std::process::exit(2);
            } else {
                e.exit();
            }
        }
    };

    let format = match OutputFormat::from_str_opt(&cli.format) {
        Some(f) => f,
        None => {
            let api_err = ApiError::from(TuhucarError::InvalidArgs {
                message: format!(
                    "Invalid format '{}'. Must be 'json' or 'markdown'.",
                    cli.format
                ),
                suggestion: "Use --format json or --format markdown".to_string(),
            });
            let resp: Response<()> = Response::error(api_err, None);
            if wants_json {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            } else {
                eprintln!(
                    "Error: Invalid format '{}'. Must be 'json' or 'markdown'.",
                    cli.format
                );
            }
            std::process::exit(2);
        }
    };

    let meta = build_meta();

    if let Err(e) = commands::run(cli.command, format, cli.dry_run, cli.verbose, meta.clone()).await
    {
        let api_err: ApiError = e.into();
        let resp: Response<()> = Response::error(api_err, Some(meta.clone()));
        match format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&resp).unwrap());
            }
            OutputFormat::Markdown => {
                eprintln!(
                    "Error [{}]: {}",
                    resp.error.as_ref().unwrap().code,
                    resp.error.as_ref().unwrap().message
                );
                if let Some(s) = &resp.error.as_ref().unwrap().suggestion {
                    eprintln!("\n  {}", s);
                }
            }
        }
        mark_notices_notified(&meta);
        std::process::exit(1);
    }

    // For markdown mode, print update notices to stderr after command output
    if format == OutputFormat::Markdown {
        for notice in &meta.notices {
            let Notice::Update { ref message, .. } = notice;
            eprintln!("\n{}", message);
        }
    }

    // Mark notified after all output is done
    mark_notices_notified(&meta);
}
