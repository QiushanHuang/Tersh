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
    #[arg(long, group = "update_mode", conflicts_with_all = ["install_patch_mode", "export_observation", "cluster_status", "path", "print_cwd", "dump_keymap", "ui_profile", "theme", "keymap", "no_motion"], help = "Check the latest official stable release without installing it")]
    check_update: bool,
    #[arg(long, group = "update_mode", conflicts_with_all = ["install_patch_mode", "export_observation", "cluster_status", "path", "print_cwd", "dump_keymap", "ui_profile", "theme", "keymap", "no_motion"], help = "Offer to install a verified newer release of the running executable")]
    update: bool,
    #[arg(
        long,
        requires = "update",
        help = "Confirm an explicitly requested --update without a terminal prompt"
    )]
    yes: bool,
    #[arg(long, requires_all = ["observation_host", "observation_resource", "observation_output"], conflicts_with_all = ["cluster_status", "install_patches", "check_install_patches", "path", "dump_keymap", "print_cwd"], help = "Collect exactly one selected host through the existing 30-second collector; no background service")]
    export_observation: Option<PathBuf>,
    #[arg(long, requires = "export_observation")]
    observation_host: Option<String>,
    #[arg(long, requires = "export_observation")]
    observation_resource: Option<String>,
    #[arg(long, requires = "export_observation")]
    observation_output: Option<PathBuf>,
    #[arg(long, group = "install_patch_mode",
        conflicts_with_all = ["cluster_status", "path", "print_cwd", "dump_keymap", "keymap", "ui_profile", "theme", "no_motion"],
        help = "Apply local WSL connection lifecycle patches with backups; no remote access")]
    install_patches: bool,

    #[arg(long, group = "install_patch_mode",
        conflicts_with_all = ["cluster_status", "path", "print_cwd", "dump_keymap", "keymap", "ui_profile", "theme", "no_motion"],
        help = "Check local WSL installation patches without writing; exits nonzero if needed")]
    check_install_patches: bool,

    #[arg(
        long,
        value_name = "FILE",
        requires = "install_patch_mode",
        help = "SSH configuration to patch/check (default: ~/.ssh/config; select included files explicitly)"
    )]
    patch_ssh_config: Option<PathBuf>,

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
        requires = "cluster_status",
        help = "Print the effective inventory and observer as JSON without starting probes"
    )]
    list_hosts: bool,

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
    if cli.check_update || cli.update {
        return tersh::update_command(cli.update, cli.yes);
    }
    if let Some(inventory) = cli.export_observation.as_deref() {
        return tersh::observation_export::run(
            inventory,
            cli.observation_host.as_deref().unwrap(),
            cli.observation_resource.as_deref().unwrap(),
            cli.observation_output.as_deref().unwrap(),
        );
    }
    if cli.install_patches || cli.check_install_patches {
        return tersh::install_patches::run(cli.install_patches, cli.patch_ssh_config.as_deref());
    }
    if cli.list_hosts {
        use tersh::cluster::{ClusterInventory, HostKind};
        let inventory = match cli.cluster_config.as_deref() {
            Some(path) => ClusterInventory::from_path(path)?,
            None => ClusterInventory::load_default()?,
        };
        let local = inventory
            .hosts()
            .iter()
            .find(|host| host.kind() == HostKind::Local);
        let hosts: Vec<_> = inventory
            .hosts()
            .iter()
            .map(|host| {
                serde_json::json!({
                    "alias": host.alias(), "kind": host.kind().label(), "via": host.proxy_jump(),
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "source": inventory.source(), "local_alias": local.map(|h| h.alias()), "hosts": hosts,
            }))?
        );
        return Ok(());
    }
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
        } else if std::env::var("TERM_PROGRAM").as_deref() == Ok("ghostty")
            && std::env::var_os("SSH_CONNECTION").is_none()
            && std::env::var_os("SSH_TTY").is_none()
        {
            for (key, value) in [("TERSH_BORDER", "rounded"), ("TERSH_GLYPHS", "unicode")] {
                if std::env::var_os(key).is_none() {
                    std::env::set_var(key, value);
                }
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
