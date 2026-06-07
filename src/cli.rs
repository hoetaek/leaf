use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "leaf")]
#[command(version)]
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
    /// Move an active leaf into the fallen archive.
    Fall {
        /// Path-safe leaf slug.
        slug: String,
        /// Human-readable closure reason.
        #[arg(long)]
        reason: String,
    },
}

pub(crate) fn run() -> Result<()> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let changed = crate::storage::ensure_leaf_root(&paths)?;
            if changed {
                println!("initialized .leaf/");
            } else {
                println!(".leaf/ already initialized");
            }
            Ok(())
        }
        Commands::New { slug } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            crate::scaffold::create_seed(&paths.root, &slug)?;
            println!("created .leaf/seeds/{slug}/");
            Ok(())
        }
        Commands::Fall { slug, reason } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            crate::lifecycle::fall_leaf(&paths.root, &slug, &reason)?;
            println!("moved .leaf/leaves/{slug}/ to .leaf/fallen/{slug}/");
            Ok(())
        }
    }
}
