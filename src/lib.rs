mod cli;
mod codex_config;
mod doctor;
mod doctor_output;
mod fs_ext;
mod git;
mod inventory;
mod lifecycle;
mod list_columns;
mod list_output;
mod preview;
mod review;
mod scaffold;
mod slug;
mod storage;
mod syntax;
mod tui;

pub fn run() -> anyhow::Result<std::process::ExitCode> {
    cli::run()
}
