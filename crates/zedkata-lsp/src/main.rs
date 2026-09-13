use anyhow::Result;
use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stdio {
        // Stdio LSP server loop to be implemented in ZK-013
        eprintln!("zedkata-lsp initialized in stdio mode");
    } else {
        println!("zedkata-lsp {}", env!("CARGO_PKG_VERSION"));
        println!("Use --stdio to start the Language Server Protocol observer.");
    }

    Ok(())
}
