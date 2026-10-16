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

    /// Explicit path to generated lessons directory
    #[arg(long)]
    generated_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stdio {
        let lessons_dir = zedkata_core::resolve_lessons_dir(cli.lessons_dir.as_deref())
            .unwrap_or_else(|_| PathBuf::from("lessons"));

        let generated_dir = cli.generated_dir.or_else(|| {
            std::env::var("ZEDKATA_GENERATED_DIR")
                .ok()
                .map(PathBuf::from)
        });

        let session_path = cli
            .session_path
            .or_else(|| {
                std::env::var("ZEDKATA_SESSION_PATH")
                    .or_else(|_| std::env::var("ZEDKATA_SESSION"))
                    .ok()
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| {
                default_session_path().unwrap_or_else(|_| PathBuf::from("session.json"))
            });

        let server = match Catalog::load_with_generated(&lessons_dir, generated_dir.as_deref()) {
            Ok((catalog, merge_summary)) => {
                let gen_count = merge_summary.map(|s| s.added.len()).unwrap_or(0);
                eprintln!(
                    "[zedkata-lsp] Loaded {} lesson(s) (including {} generated) from {:?}",
                    catalog.all_lessons().len(),
                    gen_count,
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
