use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use zedkata_core::{catalog::Catalog, default_session_path};
use zedkata_lsp::{feedback::FeedbackEngine, run_server_with_engine, LspServer};

#[derive(Parser, Debug)]
#[command(
    name = "zedkata-lsp",
    about = "Language server observer for zedkata drills in Zed",
    version
)]
struct Cli {
    /// Speak Language Server Protocol over stdin/stdout
    #[arg(long)]
    stdio: bool,

    /// Explicit path to curriculum lessons directory
    #[arg(long)]
    lessons_dir: Option<PathBuf>,

    /// Explicit path to session.json state file
    #[arg(long)]
    session_path: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stdio {
        let lessons_dir = cli
            .lessons_dir
            .or_else(|| std::env::var("ZEDKATA_LESSONS_DIR").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("lessons"));

        let session_path = cli
            .session_path
            .or_else(|| {
                std::env::var("ZEDKATA_SESSION_PATH")
                    .ok()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| {
                default_session_path().unwrap_or_else(|_| PathBuf::from("session.json"))
            });

        let server = match Catalog::load_from_dir(&lessons_dir) {
            Ok(catalog) => {
                eprintln!(
                    "[zedkata-lsp] Loaded {} lesson(s) from {:?}",
                    catalog.all_lessons().len(),
                    lessons_dir
                );
                let feedback = FeedbackEngine::new(session_path, catalog);
                LspServer::with_feedback(feedback)
            }
            Err(e) => {
                eprintln!(
                    "[zedkata-lsp] Warning: Could not load curriculum catalog from {:?}: {}. Running in standalone observer mode.",
                    lessons_dir, e
                );
                LspServer::new()
            }
        };

        let exit_code =
            run_server_with_engine(tokio::io::stdin(), tokio::io::stdout(), server).await?;
        std::process::exit(exit_code);
    } else {
        println!("zedkata-lsp {}", env!("CARGO_PKG_VERSION"));
        println!("Use --stdio to start the Language Server Protocol observer.");
    }

    Ok(())
}
