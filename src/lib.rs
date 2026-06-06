mod cli;
mod git;
mod storage;

pub fn run() -> anyhow::Result<()> {
    cli::run()
}
