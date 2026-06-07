mod cli;
mod git;
mod lifecycle;
mod scaffold;
mod slug;
mod storage;

pub fn run() -> anyhow::Result<()> {
    cli::run()
}
