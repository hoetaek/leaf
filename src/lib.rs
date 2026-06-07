mod cli;
mod git;
mod inventory;
mod lifecycle;
mod scaffold;
mod slug;
mod storage;

pub fn run() -> anyhow::Result<()> {
    cli::run()
}
