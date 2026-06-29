use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
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
    /// Promote a completed sprout into a leaf (keep it on the tree).
    Keep {
        /// Path-safe sprout slug to keep as a leaf.
        slug: String,
    },
    /// Author and maintain SRP sidecar contracts (validation stays in `leaf doctor`).
    Sidecar {
        #[command(subcommand)]
        command: crate::sidecar::SidecarCommand,
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
    /// Export the pressed leaf knowledge graph.
    Graph {
        /// Write machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Open the review reader for one leaf-work slug.
    Review {
        /// Leaf-work slug to review.
        slug: String,
        /// Write machine-readable JSON (the 11 gate sources with raw markdown).
        #[arg(long)]
        json: bool,
    },
    /// Print the effective profile (global profile layered with repo-local PROFILE.md).
    Profile,
    /// Advance to the next LEAF phase, pausing if the current phase is unpolished.
    Next {
        /// Leaf-work slug to advance.
        slug: String,
    },
    /// Preserve a timestamped copy of one canonical gate document.
    Checkpoint(CheckpointArgs),
    /// Diagnose .leaf readiness for leaf list.
    Doctor {
        /// Write machine-readable JSON.
        #[arg(long)]
        json: bool,
    },
    /// Serve a read-only local web UI over the .leaf workspace.
    Serve {
        /// Preferred port to bind on 127.0.0.1.
        #[arg(long, default_value_t = 4173)]
        port: u16,
        /// Fail instead of trying the next port when the preferred port is busy.
        #[arg(long)]
        strict_port: bool,
    },
    /// Update leaf to the latest stable release.
    Update,
}

#[derive(Debug, Args)]
struct CheckpointArgs {
    /// Leaf-work slug to checkpoint.
    slug: String,
    /// Gate name or number, for example criteria, 3, or g3.
    #[arg(long, value_name = "GATE")]
    gate: Option<String>,
    /// Checkpoint gate ① Intent.
    #[arg(long)]
    intent: bool,
    /// Checkpoint gate ② Unknowns.
    #[arg(long)]
    unknowns: bool,
    /// Checkpoint gate ③ Criteria.
    #[arg(long)]
    criteria: bool,
    /// Checkpoint gate ④ Wireframe.
    #[arg(long)]
    wireframe: bool,
    /// Checkpoint gate ⑤ Design.
    #[arg(long)]
    design: bool,
    /// Checkpoint gate ⑥ Critic.
    #[arg(long)]
    critic: bool,
    /// Checkpoint gate ⑦ Tasks.
    #[arg(long)]
    tasks: bool,
    /// Checkpoint gate ⑧ Execution.
    #[arg(long)]
    execution: bool,
    /// Checkpoint gate ⑨ Review.
    #[arg(long)]
    review: bool,
    /// Checkpoint gate ⑩ Retrospect.
    #[arg(long)]
    retrospect: bool,
    /// Checkpoint gate ① Intent.
    #[arg(long = "1")]
    gate_1: bool,
    /// Checkpoint gate ② Unknowns.
    #[arg(long = "2")]
    gate_2: bool,
    /// Checkpoint gate ③ Criteria.
    #[arg(long = "3")]
    gate_3: bool,
    /// Checkpoint gate ④ Wireframe.
    #[arg(long = "4")]
    gate_4: bool,
    /// Checkpoint gate ⑤ Design.
    #[arg(long = "5")]
    gate_5: bool,
    /// Checkpoint gate ⑥ Critic.
    #[arg(long = "6")]
    gate_6: bool,
    /// Checkpoint gate ⑦ Tasks.
    #[arg(long = "7")]
    gate_7: bool,
    /// Checkpoint gate ⑧ Execution.
    #[arg(long = "8")]
    gate_8: bool,
    /// Checkpoint gate ⑨ Review.
    #[arg(long = "9")]
    gate_9: bool,
    /// Checkpoint gate ⑩ Retrospect.
    #[arg(long = "10")]
    gate_10: bool,
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
            if let Some(config_dir) = crate::profile::global_config_dir()
                && crate::profile::ensure_global_profile(&config_dir)?
            {
                println!(
                    "initialized {}",
                    crate::profile::global_profile_path(&config_dir).display()
                );
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
        Commands::Keep { slug } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            let result = crate::lifecycle::keep_leaf(&paths.root, &slug)?;
            if let Some(phase) = &result.premature {
                eprintln!(
                    "⚠ {slug}는 아직 Feedback에 도달하지 않았습니다 (current phase: {phase}). \
                     완료 전 keep일 수 있습니다 — 그래도 진행합니다."
                );
            }
            println!(
                "✓ {slug}: sprout → leaf\n  {}/ → {}/\n  stage: sprout → leaf",
                repo_relative(&paths.root, &result.source),
                repo_relative(&paths.root, &result.destination)
            );
            Ok(ExitCode::SUCCESS)
        }
        Commands::Sidecar { command } => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::sidecar::run(command, &paths, std::time::SystemTime::now())?;
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
        Commands::Graph { json } => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let graph = crate::graph::load(&paths.root)?;
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            if json {
                crate::graph::write_json(&mut stdout, &graph)?;
            } else {
                crate::graph::write_text(&mut stdout, &graph)?;
            }
            stdout.flush().context("flush leaf graph output")?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Review { slug, json } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let inventory = crate::inventory::load(&paths.root)?;
            let source = crate::review::source_for_slug(&inventory, &slug)?;
            if json {
                let document = crate::review::build_json(&source)?;
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                crate::review::write_json(&mut stdout, &document)?;
                stdout.flush().context("flush leaf review json")?;
            } else if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                crate::tui::run_review(&inventory, source)?;
            } else {
                let document = crate::review::build(&source)?;
                let references = crate::review::reference_files(&source)?;
                let stdout = std::io::stdout();
                let mut stdout = stdout.lock();
                crate::review::write_text(&mut stdout, &document)?;
                crate::review::write_references_text(&mut stdout, &references)?;
                stdout.flush().context("flush leaf review text")?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Profile => {
            let text = crate::profile::effective_profile(std::env::current_dir()?)?;
            print!("{text}");
            Ok(ExitCode::SUCCESS)
        }
        Commands::Next { slug } => {
            let slug = crate::slug::validate(&slug)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            crate::storage::ensure_leaf_root(&paths)?;
            let inventory = crate::inventory::load(&paths.root)?;
            let root_path = leaf_work_path_for_slug(&inventory, &slug)?;
            crate::phase::run_next(&root_path)
        }
        Commands::Checkpoint(args) => {
            let slug = crate::slug::validate(&args.slug)?;
            let gate = checkpoint_gate(&args)?;
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let inventory = crate::inventory::load(&paths.root)?;
            let root_path = leaf_work_path_for_slug(&inventory, &slug)?;
            let results = crate::checkpoint::create(&root_path, gate)?;
            for result in &results {
                println!(
                    "checkpointed {} to {}",
                    repo_relative(&paths.root, &result.source),
                    repo_relative(&paths.root, &result.checkpoint)
                );
            }
            Ok(ExitCode::SUCCESS)
        }
        Commands::Doctor { json } => {
            let paths = crate::git::repo_paths(std::env::current_dir()?)?;
            let report = crate::doctor::check_with_git_exclude(&paths.root, Some(&paths.exclude))?;
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
        Commands::Serve { port, strict_port } => {
            let fallback = if strict_port {
                crate::serve::PortFallback::Strict
            } else {
                crate::serve::PortFallback::Auto
            };
            crate::serve::run(port, fallback)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Update => crate::update::run(),
    }
}

fn checkpoint_gate(args: &CheckpointArgs) -> Result<crate::checkpoint::GateSpec> {
    let mut selected = Vec::new();
    if let Some(gate) = &args.gate {
        selected.push(gate.as_str());
    }
    for (is_set, gate) in [
        (args.intent, "intent"),
        (args.unknowns, "unknowns"),
        (args.criteria, "criteria"),
        (args.wireframe, "wireframe"),
        (args.design, "design"),
        (args.critic, "critic"),
        (args.tasks, "tasks"),
        (args.execution, "execution"),
        (args.review, "review"),
        (args.retrospect, "retrospect"),
        (args.gate_1, "1"),
        (args.gate_2, "2"),
        (args.gate_3, "3"),
        (args.gate_4, "4"),
        (args.gate_5, "5"),
        (args.gate_6, "6"),
        (args.gate_7, "7"),
        (args.gate_8, "8"),
        (args.gate_9, "9"),
        (args.gate_10, "10"),
    ] {
        if is_set {
            selected.push(gate);
        }
    }

    match selected.as_slice() {
        [] => bail!("missing gate flag; use --criteria, --3, or --gate <gate>"),
        [gate] => crate::checkpoint::gate_spec(gate),
        _ => bail!("choose exactly one gate flag"),
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

fn leaf_work_path_for_slug(
    inventory: &crate::inventory::Inventory,
    slug: &str,
) -> Result<std::path::PathBuf> {
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
        [item] => Ok(item.path.clone()),
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
