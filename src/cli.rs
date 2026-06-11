use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

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
    /// Create a new sprout.
    New {
        /// Path-safe sprout slug.
        slug: String,
    },
    /// Move a sprout or leaf into fallen.
    Fall {
        /// Path-safe sprout or leaf slug.
        slug: String,
        /// Human-readable closure reason.
        #[arg(long)]
        reason: String,
    },
    /// List .leaf workspace items.
    List {
        /// Write machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Render the .leaf workspace as a terminal tree.
    Tree {
        /// Suppress ANSI color.
        #[arg(long)]
        plain: bool,
        /// Show a top-to-bottom growth demo instead of the current workspace.
        #[arg(long)]
        demo: bool,
    },
    /// Open the review reader for one leaf-work slug.
    Review {
        /// Leaf-work slug to review.
        slug: String,
    },
    /// Diagnose .leaf readiness for leaf list.
    Doctor {
        /// Write machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
}

pub(crate) fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    execute(cli)
}

fn execute(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Commands::Init => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let changed = crate::storage::ensure_leaf_root(&paths)?;
            if changed {
                println!("initialized .leaf/");
            } else {
                println!(".leaf/ already initialized");
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::New { slug } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            crate::scaffold::create_sprout(&paths.root, &slug)?;
            println!("created .leaf/01-sprouts/{slug}/");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fall { slug, reason } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            let result = crate::lifecycle::fall_leaf(&paths.root, &slug, &reason)?;
            println!(
                "moved {}/ to {}/",
                repo_relative(&paths.root, &result.source),
                repo_relative(&paths.root, &result.destination)
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::List { json } => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let inventory = crate::inventory::load(&paths.root)?;
            if json {
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                crate::list_output::write_json(&mut stdout, &inventory)?;
            } else if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                crate::tui::run(&inventory)?;
            } else {
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                crate::list_output::write_text(&mut stdout, &inventory)?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Tree { plain, demo } => {
            let width = tree_output_width();
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let options = crate::tree::TreeRenderOptions {
                color: !plain,
                width,
            };
            if demo {
                crate::tree::write_demo_text(&mut stdout, options)?;
            } else {
                let paths = crate::git::repo_paths(std::env::current_dir()?)?;
                let inventory = crate::inventory::load(&paths.root)?;
                let model = crate::tree::TreeModel::from_inventory(&inventory);
                crate::tree::write_text(&mut stdout, &model, options)?;
            }
            stdout.flush().context("flush leaf tree text")?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Review { slug } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let inventory = crate::inventory::load(&paths.root)?;
            let source = review_source_for_slug(&inventory, &slug)?;
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                crate::tui::run_review(&inventory, source)?;
            } else {
                let document = crate::review::build(&source)?;
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                crate::review::write_text(&mut stdout, &document)?;
                stdout.flush().context("flush leaf review text")?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Doctor { json } => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let report = crate::doctor::check(&paths.root)?;
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            if json {
                crate::doctor_output::write_json(&mut stdout, &report)?;
            } else {
                crate::doctor_output::write_text(&mut stdout, &report)?;
            }
            if report.has_errors() {
                Ok(ExitCode::FAILURE)
            } else {
                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn tree_output_width() -> usize {
    if std::io::stdout().is_terminal()
        && let Ok((columns, _)) = crossterm::terminal::size()
    {
        return usize::from(columns.max(1));
    }
    112
}

fn review_source_for_slug(
    inventory: &crate::inventory::Inventory,
    slug: &str,
) -> Result<crate::review::ReviewSource> {
    let matches = inventory
        .stages
        .iter()
        .flat_map(|stage| stage.items.iter())
        .filter(|item| {
            item.kind == crate::inventory::ItemKind::LeafWork && item.slug.as_str() == slug
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => bail!("leaf work does not exist: {slug}"),
        [item] => item
            .review
            .clone()
            .context("review is only available for leaf work rows"),
        items => {
            let repo_root = inventory
                .leaf_root
                .parent()
                .context("inventory leaf root has no parent")?;
            let locations = items
                .iter()
                .map(|item| repo_relative(repo_root, &item.path))
                .collect::<Vec<_>>()
                .join(", ");
            bail!("leaf work slug is ambiguous: {slug} ({locations})");
        }
    }
}

fn repo_relative(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
