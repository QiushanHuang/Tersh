use ratatui::{Terminal, backend::TestBackend, style::Color};
use std::{ffi::OsString, sync::Mutex};
use tersh::{
    app::App,
    cluster::{ClusterApp, ClusterInventory},
    cluster_ui,
    ui::draw,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const CAMPUS_JSON: &str = r#"
{
  "main_machine": {
    "alias": "qiushan-mbp",
    "tailscale_ip": "100.116.123.45",
    "role": "Codex runs here"
  },
  "jump_host": {
    "alias": "campus-mac",
    "device_name": "joshuamacbook-pro",
    "tailscale_ip": "100.90.116.54",
    "ssh_user": "joshua",
    "role": "Campus network jump host"
  },
  "servers": [
    {
      "alias": "school-star",
      "ssh_user": "star",
      "campus_ip": "10.13.7.138",
      "proxy_jump": "campus-mac",
      "workdir": "/srv/star"
    }
  ]
}
"#;

fn render_app(app: &App, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

fn render_app_uses_color(app: &App, width: u16, height: u16) -> bool {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .any(|cell| cell.fg != Color::Reset || cell.bg != Color::Reset)
}

fn render_cluster(app: &ClusterApp, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| cluster_ui::draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>()
}

fn render_cluster_uses_color(app: &ClusterApp, width: u16, height: u16) -> bool {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| cluster_ui::draw(frame, app)).unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .any(|cell| cell.fg != Color::Reset || cell.bg != Color::Reset)
}

fn with_env_var(name: &str, value: &str, run: impl FnOnce()) {
    let _guard = ENV_LOCK.lock().unwrap();
    let _restore = EnvRestore {
        name: name.to_string(),
        old: std::env::var_os(name),
    };
    unsafe {
        std::env::set_var(name, value);
    }
    run();
}

struct EnvRestore {
    name: String,
    old: Option<OsString>,
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = &self.old {
                std::env::set_var(&self.name, value);
            } else {
                std::env::remove_var(&self.name);
            }
        }
    }
}

#[test]
fn mono_theme_disables_workbench_cell_colors() {
    with_env_var("TERSH_THEME", "mono", || {
        let app = App::for_test();

        assert!(!render_app_uses_color(&app, 120, 30));
    });
}

#[test]
fn compact_footer_can_be_forced_for_workbench() {
    with_env_var("TERSH_FOOTER", "compact", || {
        let app = App::for_test();

        let buffer = render_app(&app, 180, 30);

        assert!(buffer.contains("next: Enter open dir"));
        assert!(!buffer.contains("j/k move"));
    });
}

#[test]
fn mono_theme_disables_cluster_cell_colors() {
    with_env_var("TERSH_THEME", "mono", || {
        let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
        let app = ClusterApp::new(inventory.hosts().to_vec());

        assert!(!render_cluster_uses_color(&app, 120, 30));
    });
}

#[test]
fn compact_footer_can_be_forced_for_cluster() {
    with_env_var("TERSH_FOOTER", "compact", || {
        let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
        let app = ClusterApp::new(inventory.hosts().to_vec());

        let buffer = render_cluster(&app, 180, 24);

        assert!(buffer.contains("next: Enter refresh"));
        assert!(!buffer.contains("j/k move"));
    });
}
