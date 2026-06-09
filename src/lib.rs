mod cli;
mod doctor;
mod doctor_output;
mod git;
mod inventory;
mod lifecycle;
mod list_output;
mod preview;
mod scaffold;
mod slug;
mod storage;
mod tui;

pub fn run() -> anyhow::Result<std::process::ExitCode> {
    cli::run()
}
