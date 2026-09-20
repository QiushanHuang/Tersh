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
        long,
        value_name = "FILE",
        help = "Load custom JSON keybindings (otherwise TERSH_KEYMAP or config directory)"
    )]
    keymap: Option<PathBuf>,

    #[arg(
        long,
        help = "Print all effective keybindings as JSON and exit without starting the TUI"
    )]
    dump_keymap: bool,
    #[arg(long, value_parser = ["desktop", "mobile", "ssh"],
        help = "UI preset: desktop (rounded/graphs), mobile (compact), ssh (ASCII/no motion)")]
    ui_profile: Option<String>,

    #[arg(long, value_parser = ["btop", "aurora", "contrast", "mono"], help = "Override the UI theme")]
    theme: Option<String>,

    #[arg(long, help = "Disable probe animation")]
    no_motion: bool,
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
    let keymap = load_keymap(cli.keymap.as_deref())?;
    if cli.dump_keymap {
        println!("{}", keymap.to_json()?);
        return Ok(());
    }
    // Startup is single-threaded here. Set process-local UI defaults before any
    // probe threads start; explicit flags override inherited UI environment.
    unsafe {
        if let Some(profile) = cli.ui_profile.as_deref() {
            for (key, value) in tersh::theme::profile_settings(profile) {
                std::env::set_var(key, value);
            }
        }
        if let Some(theme) = &cli.theme {
            std::env::set_var("TERSH_THEME", theme);
        }
        if cli.no_motion {
            std::env::set_var("TERSH_MOTION", "off");
        }
    }
    if cli.cluster_status {
        return tersh::cluster::run_with_keymap(cli.cluster_config.as_deref(), keymap);
    }
    tersh::app::run_with_keymap(
        cli.path.unwrap_or_else(|| PathBuf::from(".")),
        tersh::app::RunOptions {
            print_cwd: cli.print_cwd,
        },
        keymap,
    )
}

fn load_keymap(explicit: Option<&std::path::Path>) -> Result<tersh::keymap::Keymap> {
    if let Some(path) = explicit {
        return tersh::keymap::Keymap::load(path);
    }
    if let Some(path) = std::env::var_os("TERSH_KEYMAP") {
        return tersh::keymap::Keymap::load(&PathBuf::from(path));
    }
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
        .map(|path| path.join("tersh/keymap.json"));
    if let Some(path) = config {
        match std::fs::symlink_metadata(&path) {
            Ok(_) => return tersh::keymap::Keymap::load(&path),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(tersh::keymap::Keymap::default())
}
