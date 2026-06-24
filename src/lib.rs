mod checkpoint;
mod cli;
mod codex_config;
mod doctor;
mod doctor_output;
mod fs_ext;
mod git;
mod graph;
mod inventory;
mod lifecycle;
mod list_columns;
mod list_output;
mod phase;
mod preview;
mod profile;
mod review;
mod scaffold;
mod serve;
mod slug;
mod storage;
mod syntax;
mod tree;
mod tui;
mod update;

pub fn run() -> anyhow::Result<std::process::ExitCode> {
    cli::run()
}
