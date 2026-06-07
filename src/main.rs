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
    #[arg(
        long = "cluster",
        visible_alias = "c",
        conflicts_with = "print_cwd",
        help = "Open the read-only cluster health dashboard with route and selected host launch actions"
    )]
    cluster_status: bool,

    #[arg(
        long,
        value_name = "FILE",
        requires = "cluster_status",
        help = "Read multi-server status inventory from a JSON file"
    )]
    cluster_config: Option<PathBuf>,

    #[arg(
        long,
        help = "Print the final directory after exit for shell cd wrappers"
    )]
    print_cwd: bool,

    #[arg(conflicts_with = "cluster_status")]
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.cluster_status {
        return tersh::cluster::run_with_config_path(cli.cluster_config.as_deref());
    }
    tersh::app::run_with_options(
        cli.path.unwrap_or_else(|| PathBuf::from(".")),
        tersh::app::RunOptions {
            print_cwd: cli.print_cwd,
        },
    )
}
