mod cli;
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
