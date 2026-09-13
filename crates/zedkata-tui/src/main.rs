use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "zedkata-tui",
    about = "Interactive terminal tutor for Zed vim mode and editor drills",
    version
)]
struct Cli {
    /// Path to a specific session.json file (defaults to standard platform state path)
    #[arg(short, long)]
    session: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(session_path) = cli.session {
        println!("zedkata-tui configured with session: {}", session_path);
    } else {
        println!("zedkata-tui {}", env!("CARGO_PKG_VERSION"));
        println!("Interactive terminal interface will launch in Phase 2.");
    }

    Ok(())
}
