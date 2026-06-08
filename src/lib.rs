mod cli;
#[allow(dead_code)] // Wired into the CLI in doctor task T5.
mod doctor;
mod git;
mod inventory;
mod lifecycle;
mod list_output;
mod preview;
mod scaffold;
mod slug;
mod storage;
mod tui;

pub fn run() -> anyhow::Result<()> {
    cli::run()
}
