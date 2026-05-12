use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "tersh",
    version,
    about = "Tersh is a lightweight terminal file workbench for local and SSH shell sessions."
)]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tersh::app::run(cli.path)
}
