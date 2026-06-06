mod cli;
mod git;
mod scaffold;
mod slug;
mod storage;

pub fn run() -> anyhow::Result<()> {
    cli::run()
}
