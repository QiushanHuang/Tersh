use ratatui::{Terminal, backend::TestBackend, style::Color};
use std::{ffi::OsString, sync::Mutex};
use tersh::{
    app::App,
    cluster::{ClusterApp, ClusterInventory, HostSnapshot, ProbeReport},
    cluster_ui,
    theme::{ChipTone, Theme, Tone, footer_line, panel_title, resource_bar},
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

fn render_cluster_bar_styles(app: &ClusterApp, width: u16, height: u16) -> (Color, Color) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| cluster_ui::draw(frame, app)).unwrap();
    let content = terminal.backend().buffer().content();
    let line_width = width as usize;
    let hash_index = content
        .iter()
        .position(|cell| cell.symbol() == "#")
        .expect("resource bar should contain filled cells");
    let row = hash_index / line_width;
    let dash_index = content
        .iter()
        .enumerate()
        .skip(hash_index + 1)
        .take(line_width - (hash_index % line_width) - 1)
        .find(|(_, cell)| cell.symbol() == "-")
        .map(|(index, _)| index)
        .expect("resource bar should contain empty cells");
    assert_eq!(row, dash_index / line_width);
    (content[hash_index].fg, content[dash_index].fg)
}

fn render_app_title_style(app: &App, title: &str, width: u16, height: u16) -> Color {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    let content = terminal.backend().buffer().content();
    let rendered = content.iter().map(|cell| cell.symbol()).collect::<String>();
    let index = rendered
        .find(title)
        .unwrap_or_else(|| panic!("{title} should render"));
    content[index].fg
}

fn render_app_cell_colors(app: &App, text: &str, width: u16, height: u16) -> (Color, Color) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| draw(frame, app)).unwrap();
    let content = terminal.backend().buffer().content();
    let rendered = content.iter().map(|cell| cell.symbol()).collect::<String>();
    let index = rendered
        .find(text)
        .unwrap_or_else(|| panic!("{text} should render"));
    (content[index].fg, content[index].bg)
}

fn with_env_var(name: &str, value: &str, run: impl FnOnce()) {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
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
fn rounded_border_can_be_enabled_for_workbench_and_cluster() {
    with_env_var("TERSH_BORDER", "rounded", || {
        let app = App::for_test();
        let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
        let cluster = ClusterApp::new(inventory.hosts().to_vec());

        let workbench = render_app(&app, 120, 30);
        let cluster = render_cluster(&cluster, 120, 30);

        assert!(workbench.contains("╭"));
        assert!(workbench.contains("╯"));
        assert!(cluster.contains("╭"));
        assert!(cluster.contains("╯"));
    });
}

#[test]
fn thick_border_can_be_enabled_for_workbench_and_cluster() {
    with_env_var("TERSH_BORDER", "thick", || {
        let app = App::for_test();
        let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
        let cluster = ClusterApp::new(inventory.hosts().to_vec());

        let workbench = render_app(&app, 120, 30);
        let cluster = render_cluster(&cluster, 120, 30);

        assert!(workbench.contains("┏"));
        assert!(workbench.contains("┛"));
        assert!(cluster.contains("┏"));
        assert!(cluster.contains("┛"));
    });
}

#[test]
fn default_border_stays_ascii_for_remote_compatibility() {
    with_env_var("TERSH_BORDER", "ascii", || {
        let app = App::for_test();

        let buffer = render_app(&app, 120, 30);

        assert!(buffer.contains("+"));
        assert!(!buffer.contains("╭"));
    });
}

#[test]
fn aurora_theme_exposes_distinct_semantic_palette() {
    let theme = Theme::from_name("aurora");
    let palette = theme.palette();

    assert_eq!(theme, Theme::Aurora);
    assert_ne!(palette.accent, palette.panel_title);
    assert_ne!(palette.copy, palette.cut);
    assert_ne!(palette.search_match, palette.selected_bg);
}

#[test]
fn panel_title_uses_semantic_palette_color() {
    let theme = Theme::Aurora;
    let title = panel_title(theme, "Preview");

    assert_eq!(title.spans[0].style.fg, Some(theme.palette().panel_title));
}

#[test]
fn semantic_tones_and_chips_route_through_palette() {
    let theme = Theme::Aurora;
    let palette = theme.palette();

    assert_eq!(theme.style(Tone::Title).fg, Some(palette.panel_title));
    assert_eq!(theme.style(Tone::Copy).fg, Some(palette.copy));
    assert_eq!(theme.bold(Tone::Danger).fg, Some(palette.danger));
    assert!(
        theme
            .bold(Tone::Danger)
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    );

    let chip = theme.chip_tone(ChipTone::Copy);
    assert_eq!(chip.bg, Some(palette.copy));
    assert_eq!(chip.fg, Some(palette.text));

    let mono_chip = Theme::Mono.chip_tone(ChipTone::Danger);
    assert_eq!(mono_chip.fg, None);
    assert_eq!(mono_chip.bg, None);
    assert!(
        mono_chip
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD)
    );
}

#[test]
fn resource_bar_component_clamps_and_preserves_ascii_shape() {
    let danger_bar = resource_bar(Theme::Btop, Some(95), 10);
    let rendered = danger_bar
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "[##########]");
    assert_eq!(danger_bar[1].style.fg, Some(Theme::Btop.palette().danger));

    let half_bar = resource_bar(Theme::Btop, Some(50), 10);
    let rendered = half_bar
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();

    assert_eq!(rendered, "[#####-----]");
    assert_ne!(half_bar[1].style.fg, half_bar[2].style.fg);

    let mono_bar = resource_bar(Theme::Mono, Some(50), 10);
    assert!(
        mono_bar
            .iter()
            .all(|span| span.style.fg.is_none() && span.style.bg.is_none())
    );
}

#[test]
fn workbench_files_panel_is_active_while_preview_is_inactive() {
    with_env_var("TERSH_THEME", "btop", || {
        let app = App::for_test();
        let files_fg = render_app_title_style(&app, "Files", 120, 30);
        let preview_fg = render_app_title_style(&app, "Preview", 120, 30);

        assert_eq!(files_fg, Theme::Btop.palette().active);
        assert_eq!(preview_fg, Theme::Btop.palette().inactive);
    });
}

#[test]
fn workbench_header_hidden_state_uses_plain_key_value_style_without_background() {
    with_env_var("TERSH_THEME", "btop", || {
        let app = App::for_test();
        let palette = Theme::Btop.palette();

        let (label_fg, label_bg) = render_app_cell_colors(&app, "hidden", 120, 30);
        let (value_fg, value_bg) = render_app_cell_colors(&app, "OFF", 120, 30);

        assert_eq!(label_bg, Color::Reset);
        assert_eq!(value_bg, Color::Reset);
        assert_eq!(label_fg, palette.key);
        assert_eq!(value_fg, palette.inactive);
        assert_ne!(label_fg, value_fg);
    });
}

#[test]
fn footer_line_styles_next_cancel_and_force_segments() {
    let line = footer_line(
        Theme::Btop,
        "normal | next: Enter open dir | ^G cancel | ^C force",
    );

    let next = line
        .spans
        .iter()
        .find(|span| span.content.as_ref() == "next: Enter open dir")
        .unwrap();
    let cancel = line
        .spans
        .iter()
        .find(|span| span.content.as_ref() == "^G cancel")
        .unwrap();
    let force = line
        .spans
        .iter()
        .find(|span| span.content.as_ref() == "^C force")
        .unwrap();

    assert_ne!(next.style.fg, cancel.style.fg);
    assert_ne!(cancel.style.fg, force.style.fg);
}

#[test]
fn cluster_resource_bar_filled_and_empty_segments_use_distinct_styles() {
    with_env_var("TERSH_THEME", "btop", || {
        let inventory = ClusterInventory::from_json(CAMPUS_JSON).unwrap();
        let mut app = ClusterApp::new(inventory.hosts().to_vec());
        app.apply_snapshot(HostSnapshot::online(
            "qiushan-mbp",
            ProbeReport::parse(
                "hostname=local\nload=0.12 0.20 0.30\nmemory=9/10 GB (90%)\nstorage=5G/10G 50% used\ntasks=72 processes\ngpu=none\n",
            ),
            12,
        ));

        let (filled, empty) = render_cluster_bar_styles(&app, 120, 30);

        assert_ne!(filled, empty);
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
