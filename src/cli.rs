use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "leaf")]
#[command(about = "Domain-neutral human-agent collaboration CLI")]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize .leaf storage in the current git repository.
    Init,
    /// Create a new idea seed.
    New {
        /// Path-safe seed slug.
        slug: String,
    },
}

pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => {
            println!("leaf init is not implemented yet");
            Ok(())
        }
        Commands::New { slug } => {
            println!("leaf new {slug} is not implemented yet");
            Ok(())
        }
    }
}
