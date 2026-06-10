use crate::{
    cluster::{ClusterApp, ClusterMode, ConnectionState, HostKind, HostSnapshot},
    theme::{
        Theme, Tone, base_block, chip, footer_compact, footer_paragraph, kv_line, panel_block,
        resource_bar, section_line,
    },
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Borders, Clear, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw(frame: &mut Frame, app: &ClusterApp) {
    let area = frame.area();
    let theme = Theme::current();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    draw_header(frame, rows[0], app, theme);
    if app.mode() == ClusterMode::Detail {
        draw_dashboard(frame, rows[1], app, theme, true);
    } else {
        draw_body(frame, rows[1], app, theme);
    }
    draw_footer(frame, rows[2], app, theme);

    if app.mode() == ClusterMode::Help {
        draw_help(frame, centered_rect(70, 60, area), theme);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let palette = theme.palette();
    let lines = vec![Line::from(vec![
        Span::styled("Cluster Status", theme.fg_bold(palette.panel_title)),
        Span::raw("  "),
        chip(
            "OK",
            app.online_count(),
            theme.chip(palette.text, palette.ok),
        ),
        Span::raw(" "),
        chip(
            "OLD",
            app.stale_count(),
            theme.chip(palette.text, palette.warn),
        ),
        Span::raw(" "),
        chip(
            "FAIL",
            app.offline_count(),
            theme.chip(palette.text, palette.danger),
        ),
        Span::raw(" "),
        chip(
            "CHK",
            format!("{}/{}", app.checking_count(), app.hosts().len()),
            theme.chip(palette.text, palette.accent),
        ),
        Span::styled(" | ", theme.fg(palette.separator)),
        Span::styled("src ", theme.fg(palette.key)),
        Span::styled(ascii_safe(&app.inventory_label()), theme.fg(palette.muted)),
    ])];
    let paragraph = Paragraph::new(lines).block(base_block().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    if area.width >= 100 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_hosts(frame, columns[0], app, theme);
        draw_dashboard(frame, columns[1], app, theme, false);
    } else if area.width >= 72 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
            .split(area);
        draw_hosts(frame, rows[0], app, theme);
        draw_dashboard(frame, rows[1], app, theme, false);
    } else {
        draw_hosts(frame, area, app, theme);
    }
}

fn draw_hosts(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let mut lines = vec![Line::from(Span::styled(
        "state  alias           role     lat    mem   disk  address",
        theme.fg_bold(theme.palette().key),
    ))];
    let capacity = area.height.saturating_sub(3) as usize;
    let start = visible_start(app.cursor(), app.hosts().len(), capacity);

    for (index, host) in app.hosts().iter().enumerate().skip(start).take(capacity) {
        let snapshot = app.snapshot_for(host.alias());
        let state = snapshot
            .map(|snapshot| snapshot.connection)
            .unwrap_or(ConnectionState::Unknown);
        let latency = snapshot
            .and_then(|snapshot| snapshot.latency_ms)
            .map(|value| format!("{value}ms"))
            .unwrap_or_else(|| "--".to_string());
        let memory = snapshot
            .and_then(|snapshot| snapshot.report.memory.as_deref())
            .and_then(percent_from_token)
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let storage = snapshot
            .and_then(|snapshot| snapshot.report.storage.as_deref())
            .and_then(percent_from_token)
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let cursor = if index == app.cursor() { ">" } else { " " };
        let row = format!(
            "{cursor} {:<5} {:<15} {:<8} {:<6} {:<5} {:<5} {}",
            state_short(state),
            ascii_safe(host.alias()),
            host.kind().label(),
            latency,
            memory,
            storage,
            ascii_safe(host.address())
        );
        let row = truncate_to_width(&row, area.width.saturating_sub(2) as usize);
        let style = if index == app.cursor() {
            theme.selected()
        } else {
            state_style(theme, state)
        };
        lines.push(Line::from(Span::styled(row, style)));
    }

    let title = format!(
        "Hosts {}/{}",
        app.cursor().saturating_add(1),
        app.hosts().len()
    );
    let paragraph = Paragraph::new(lines).block(panel_block(theme, title, Tone::Active));
    frame.render_widget(paragraph, area);
}

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme, active: bool) {
    if area.height < 14 {
        draw_compact_dashboard(frame, area, app, theme, active);
        return;
    }

    let route_height = if area.height >= 18 { 7 } else { 6 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(route_height), Constraint::Min(5)])
        .split(area);
    draw_route(frame, rows[0], app, theme, active);
    draw_detail_panel(frame, rows[1], app, theme, active);
}

fn draw_compact_dashboard(
    frame: &mut Frame,
    area: Rect,
    app: &ClusterApp,
    theme: Theme,
    active: bool,
) {
    let mut lines = match app.selected_host() {
        Some(host) => compact_route_lines(host, theme),
        None => vec![Line::from("No cluster hosts available"), Line::from("")],
    };
    lines.extend(compact_status_lines(app, theme));

    let tone = if active { Tone::Active } else { Tone::Inactive };
    let paragraph = Paragraph::new(lines).block(panel_block(theme, "Route / Detail", tone));
    frame.render_widget(paragraph, area);
}

fn compact_route_lines(host: &crate::cluster::HostConfig, theme: Theme) -> Vec<Line<'static>> {
    let palette = theme.palette();
    match host.kind() {
        HostKind::Local => vec![
            Line::from(Span::styled(
                "Route: LOCAL ONLY",
                theme.fg_bold(palette.accent),
            )),
            Line::from(format!("Host: {}", ascii_safe(host.alias()))),
        ],
        HostKind::Jump => vec![
            Line::from(vec![
                Span::styled("Route: LOCAL", theme.fg(palette.accent)),
                route_arrow(theme),
                Span::styled("JUMP", theme.fg(palette.warn)),
            ]),
            Line::from(format!("Path: {}", ascii_safe(host.ssh_target()))),
        ],
        HostKind::Server => {
            if let Some(jump_target) = host.proxy_jump_target().or_else(|| host.proxy_jump()) {
                vec![
                    Line::from(vec![
                        Span::styled("Route: LOCAL", theme.fg(palette.accent)),
                        route_arrow(theme),
                        Span::styled("JUMP", theme.fg(palette.warn)),
                        route_arrow(theme),
                        Span::styled("SERVER", theme.fg(palette.ok)),
                    ]),
                    Line::from(format!(
                        "Path: {} => {}",
                        ascii_safe(jump_target),
                        ascii_safe(host.ssh_target())
                    )),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::styled("Route: LOCAL", theme.fg(palette.accent)),
                        route_arrow(theme),
                        Span::styled("SERVER", theme.fg(palette.ok)),
                    ]),
                    Line::from(format!("Path: {}", ascii_safe(host.ssh_target()))),
                ]
            }
        }
    }
}

fn compact_status_lines(app: &ClusterApp, theme: Theme) -> Vec<Line<'static>> {
    let Some(host) = app.selected_host() else {
        return vec![Line::from("Host: n/a")];
    };
    let Some(snapshot) = app.selected_snapshot() else {
        return vec![
            Line::from(format!(
                "Host: {} {}",
                ascii_safe(host.alias()),
                host.kind().label()
            )),
            Line::from("Connection: unknown"),
        ];
    };
    let latency = snapshot
        .latency_ms
        .map(|value| format!("{value} ms"))
        .unwrap_or_else(|| "-".to_string());
    let report = &snapshot.report;
    let latest_log = app
        .logs()
        .last()
        .map(String::as_str)
        .unwrap_or("no recent log");
    vec![
        Line::from(format!(
            "Host: {} {} | {} ({latency})",
            ascii_safe(host.alias()),
            host.kind().label(),
            snapshot.connection.label()
        )),
        Line::from(format!(
            "CPU load: {}",
            ascii_safe(report.cpu_load.as_deref().unwrap_or("unknown"))
        )),
        resource_line(
            theme,
            "Memory",
            report.memory.as_deref(),
            MetricKind::Memory,
        ),
        resource_line(
            theme,
            "Storage",
            report.storage.as_deref(),
            MetricKind::Storage,
        ),
        resource_line(theme, "GPU", report.gpu.as_deref(), MetricKind::Gpu),
        Line::from(format!("Log: {}", ascii_safe(latest_log))),
    ]
}

fn draw_route(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme, active: bool) {
    let tone = if active { Tone::Active } else { Tone::Inactive };
    let paragraph = Paragraph::new(match app.selected_host() {
        Some(host) => route_lines(host, theme),
        None => vec![Line::from("No route available")],
    })
    .block(panel_block(theme, "Route", tone))
    .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn route_lines(host: &crate::cluster::HostConfig, theme: Theme) -> Vec<Line<'static>> {
    let palette = theme.palette();
    match host.kind() {
        HostKind::Local => vec![
            Line::from(Span::styled("LOCAL ONLY", theme.fg_bold(palette.accent))),
            Line::from(format!("local: {}", ascii_safe(host.alias()))),
            Line::from(format!("address: {}", ascii_safe(host.address()))),
        ],
        HostKind::Jump => vec![
            Line::from(vec![
                Span::styled("LOCAL", theme.fg(palette.accent)),
                route_arrow(theme),
                Span::styled("JUMP", theme.fg(palette.warn)),
            ]),
            Line::from(format!("jump: {}", ascii_safe(host.ssh_target()))),
            Line::from(format!("command: ssh {}", ascii_safe(host.ssh_target()))),
        ],
        HostKind::Server => {
            if let Some(jump_target) = host.proxy_jump_target().or_else(|| host.proxy_jump()) {
                vec![
                    Line::from(vec![
                        Span::styled("LOCAL", theme.fg(palette.accent)),
                        route_arrow(theme),
                        Span::styled("JUMP", theme.fg(palette.warn)),
                        route_arrow(theme),
                        Span::styled("SERVER", theme.fg(palette.ok)),
                    ]),
                    Line::from(format!("jump: {}", ascii_safe(jump_target))),
                    Line::from(format!("server: {}", ascii_safe(host.ssh_target()))),
                    Line::from(format!(
                        "command: ssh -J {} {}",
                        ascii_safe(jump_target),
                        ascii_safe(host.ssh_target())
                    )),
                ]
            } else {
                vec![
                    Line::from(vec![
                        Span::styled("LOCAL", theme.fg(palette.accent)),
                        route_arrow(theme),
                        Span::styled("SERVER", theme.fg(palette.ok)),
                    ]),
                    Line::from(format!("server: {}", ascii_safe(host.ssh_target()))),
                    Line::from(format!("command: ssh {}", ascii_safe(host.ssh_target()))),
                ]
            }
        }
    }
}

fn draw_detail_panel(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme, active: bool) {
    let lines = detail_lines(app, true, theme);
    let tone = if active { Tone::Active } else { Tone::Inactive };
    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, "Detail", tone))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn detail_lines(app: &ClusterApp, include_logs: bool, theme: Theme) -> Vec<Line<'static>> {
    let palette = theme.palette();
    let Some(host) = app.selected_host() else {
        return vec![Line::from("No host selected")];
    };
    let snapshot = app.selected_snapshot();
    let mut lines = vec![Line::from(vec![
        Span::styled(ascii_safe(host.alias()), theme.fg_bold(palette.path)),
        Span::styled(format!("  {}", host.kind().label()), theme.fg(palette.key)),
    ])];
    lines.push(Line::from(""));

    if let Some(snapshot) = snapshot {
        push_snapshot_lines(&mut lines, snapshot, theme);
        if let Some(hint) = action_hint(snapshot) {
            lines.push(Line::from(Span::styled(
                format!("Hint: {hint}"),
                theme
                    .fg(hint_color(theme, snapshot.connection))
                    .add_modifier(Modifier::BOLD),
            )));
        }
    } else {
        lines.push(kv_line(theme, "Status", "unknown"));
        lines.push(Line::from(Span::styled(
            "Hint: press Enter to refresh this host",
            theme.fg(palette.muted),
        )));
    }
    if include_logs {
        lines.push(Line::from(""));
        lines.push(section_line(theme, "Log"));
        for log in app.logs().iter().rev().take(4) {
            lines.push(Line::from(Span::styled(
                ascii_safe(log),
                cluster_log_style(theme, log),
            )));
        }
    }

    if let Some(workdir) = host.workdir() {
        lines.push(kv_line(theme, "Tersh dir", ascii_safe(workdir)));
    }

    lines.push(Line::from(""));
    lines.push(kv_line(theme, "Role", ascii_safe(host.role())));
    lines.push(kv_line(theme, "Address", ascii_safe(host.address())));
    lines.push(kv_line(theme, "SSH target", ascii_safe(host.ssh_target())));
    if let Some(user) = host.user() {
        lines.push(kv_line(theme, "User", ascii_safe(user)));
    }
    if host.kind() == HostKind::Server {
        lines.push(kv_line(
            theme,
            "ProxyJump",
            ascii_safe(host.proxy_jump().unwrap_or("none")),
        ));
        if let Some(target) = host.proxy_jump_target() {
            lines.push(kv_line(theme, "Jump target", ascii_safe(target)));
        }
    }
    let route = host
        .proxy_jump_target()
        .map(|jump| format!("via {}", ascii_safe(jump)))
        .unwrap_or_else(|| "direct/local".to_string());
    lines.push(kv_line(theme, "Network route", route));
    lines
}

fn push_snapshot_lines(lines: &mut Vec<Line<'static>>, snapshot: &HostSnapshot, theme: Theme) {
    let palette = theme.palette();
    let latency = snapshot
        .latency_ms
        .map(|value| format!("{value} ms"))
        .unwrap_or_else(|| "-".to_string());
    lines.push(Line::from(vec![
        Span::styled("Connection: ", theme.fg(palette.key)),
        Span::styled(
            snapshot.connection.label(),
            state_style(theme, snapshot.connection),
        ),
        Span::styled(format!(" ({latency})"), theme.fg(palette.muted)),
    ]));
    if let Some(error) = &snapshot.error {
        lines.push(Line::from(Span::styled(
            format!("Error: {}", ascii_safe(error)),
            theme.fg(palette.danger),
        )));
        if snapshot.report.is_empty() {
            return;
        }
    }

    let report = &snapshot.report;
    lines.push(section_line(theme, "Health"));
    lines.push(kv_line(
        theme,
        "CPU load",
        ascii_safe(report.cpu_load.as_deref().unwrap_or("unknown")),
    ));
    lines.push(resource_line(
        theme,
        "Memory",
        report.memory.as_deref(),
        MetricKind::Memory,
    ));
    lines.push(resource_line(
        theme,
        "Storage",
        report.storage.as_deref(),
        MetricKind::Storage,
    ));
    lines.push(kv_line(
        theme,
        "Tasks",
        ascii_safe(report.tasks.as_deref().unwrap_or("unknown")),
    ));
    lines.push(resource_line(
        theme,
        "GPU",
        report.gpu.as_deref(),
        MetricKind::Gpu,
    ));
    lines.push(section_line(theme, "System"));
    lines.push(kv_line(
        theme,
        "Hostname",
        ascii_safe(report.hostname.as_deref().unwrap_or("unknown")),
    ));
    lines.push(kv_line(
        theme,
        "System",
        ascii_safe(report.system.as_deref().unwrap_or("unknown")),
    ));
    lines.push(kv_line(
        theme,
        "Uptime",
        ascii_safe(report.uptime.as_deref().unwrap_or("unknown")),
    ));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricKind {
    Memory,
    Storage,
    Gpu,
}

fn resource_line(
    theme: Theme,
    label: &'static str,
    value: Option<&str>,
    kind: MetricKind,
) -> Line<'static> {
    let raw = value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let metric = match kind {
        MetricKind::Memory => memory_metric(raw),
        MetricKind::Storage => percent_from_token(raw).map(|percent| (percent, "used")),
        MetricKind::Gpu if raw.eq_ignore_ascii_case("none") => None,
        MetricKind::Gpu => percent_from_token(raw).map(|percent| (percent, "")),
    };
    let bar_percent = metric.map(|(percent, _)| percent);
    let percent_text = metric
        .map(|(value, suffix)| {
            if suffix.is_empty() {
                format!("{value:>3}%")
            } else {
                format!("{value:>3}% {suffix}")
            }
        })
        .unwrap_or_else(|| " --".to_string());
    let palette = theme.palette();

    let mut spans = vec![Span::styled(format!("{label}: "), theme.fg(palette.key))];
    spans.extend(resource_bar(theme, bar_percent, 10));
    spans.extend([
        Span::styled(format!(" {percent_text}"), theme.fg(palette.value)),
        Span::styled("  ", theme.fg(palette.separator)),
        Span::styled(ascii_safe(raw), theme.fg(palette.muted)),
    ]);
    Line::from(spans)
}

fn memory_metric(raw: &str) -> Option<(u16, &'static str)> {
    let percent = percent_from_token(raw)?;
    if raw.to_lowercase().contains("free") {
        Some((100_u16.saturating_sub(percent), "used"))
    } else {
        Some((percent, "used"))
    }
}

fn percent_from_token(input: &str) -> Option<u16> {
    for (index, ch) in input.char_indices() {
        if ch != '%' && ch != '％' {
            continue;
        }
        let before = input[..index].trim_end();
        let number = before
            .chars()
            .rev()
            .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
            .collect::<String>()
            .chars()
            .rev()
            .collect::<String>();
        if number.is_empty() {
            continue;
        }
        if let Ok(value) = number.parse::<f64>() {
            return Some(clamp_percent(value.round().max(0.0) as u16));
        }
    }
    None
}

fn clamp_percent(value: u16) -> u16 {
    value.min(100)
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let compact = footer_compact(area.width, 64);
    let text = match app.mode() {
        ClusterMode::Help if compact => "help | q/Esc | ^G | ^C",
        ClusterMode::Help => "help | q/?/Enter/Esc close | ^G close | ^C force",
        ClusterMode::Detail if compact => "detail | q/Esc | ^G | ^C",
        ClusterMode::Detail => {
            "detail | q/Esc back | ^G back | ^C force | r refresh | s shell/ssh | t tersh"
        }
        ClusterMode::Normal if compact && area.width < 50 => "q quit | ? help | l detail | ^G | ^C",
        ClusterMode::Normal if compact => {
            return render_footer(
                frame,
                area,
                theme,
                format!(
                    "next: {} | q quit | ? help | l detail | ^G | ^C",
                    cluster_next_action(app)
                ),
            );
        }
        ClusterMode::Normal => {
            return render_footer(
                frame,
                area,
                theme,
                format!(
                    "next: {} | q quit | ? help | ^G cancel | ^C force | r refresh | Enter refresh host | s shell/ssh | t tersh | l detail | j/k move | Home/End",
                    cluster_next_action(app)
                ),
            );
        }
    };
    render_footer(frame, area, theme, text.to_string());
}

fn render_footer(frame: &mut Frame, area: Rect, theme: Theme, text: String) {
    let paragraph = footer_paragraph(theme, &text);
    frame.render_widget(paragraph, area);
}

fn cluster_next_action(app: &ClusterApp) -> &'static str {
    match app.selected_snapshot().map(|snapshot| snapshot.connection) {
        Some(ConnectionState::Online) => "t tersh",
        Some(ConnectionState::Checking) => "wait",
        Some(ConnectionState::Stale) => "Enter refresh",
        Some(ConnectionState::AuthFailed) => "l detail",
        Some(ConnectionState::Timeout | ConnectionState::Offline) => "l detail",
        Some(ConnectionState::Unknown) | None => "Enter refresh",
    }
}

fn action_hint(snapshot: &HostSnapshot) -> Option<&'static str> {
    match snapshot.connection {
        ConnectionState::AuthFailed => Some("check SSH auth and trusted host key"),
        ConnectionState::Timeout => Some("check VPN, jump host, and network route"),
        ConnectionState::Offline => Some("inspect the probe error and host availability"),
        ConnectionState::Stale => Some("refresh failed; showing last good metrics"),
        ConnectionState::Unknown => Some("press Enter to refresh this host"),
        ConnectionState::Checking => Some("probe is still running"),
        ConnectionState::Online => None,
    }
}

fn hint_color(theme: Theme, state: ConnectionState) -> Color {
    let palette = theme.palette();
    match state {
        ConnectionState::AuthFailed | ConnectionState::Timeout | ConnectionState::Offline => {
            palette.danger
        }
        ConnectionState::Stale | ConnectionState::Checking => palette.warn,
        ConnectionState::Unknown => palette.muted,
        ConnectionState::Online => palette.ok,
    }
}

fn route_arrow(theme: Theme) -> Span<'static> {
    Span::styled(" => ", theme.fg(theme.palette().separator))
}

fn draw_help(frame: &mut Frame, area: Rect, theme: Theme) {
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from("Cluster Status"),
        Line::from("  r: refresh every configured host"),
        Line::from("  Enter: refresh selected host"),
        Line::from("  s: open a shell/ssh session for the selected host"),
        Line::from("  t: open Tersh on the selected host"),
        Line::from("  l: show selected host detail on narrow terminals"),
        Line::from("  j/k or arrows: move selection"),
        Line::from("  Home/End: jump to first/last host"),
        Line::from("  q: close help or quit"),
        Line::from("  Esc/Ctrl+G: close help"),
        Line::from("  Ctrl+C: force quit"),
        Line::from(""),
        Line::from("Inventory"),
        Line::from("  TERSH_SERVERS_JSON overrides the default servers.json path."),
    ];
    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, "Help", Tone::Active))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn state_short(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Unknown => "--",
        ConnectionState::Checking => "..",
        ConnectionState::Online => "ok",
        ConnectionState::Stale => "old",
        ConnectionState::Timeout => "timeo",
        ConnectionState::AuthFailed => "auth",
        ConnectionState::Offline => "down",
    }
}

fn cluster_log_style(theme: Theme, log: &str) -> Style {
    let palette = theme.palette();
    let lower = log.to_ascii_lowercase();
    if lower.contains("auth") || lower.contains("error") || lower.contains("timeout") {
        theme.fg(palette.danger)
    } else if lower.contains("stale") || lower.contains("refresh") {
        theme.fg(palette.warn)
    } else {
        theme.fg(palette.muted)
    }
}

fn state_style(theme: Theme, state: ConnectionState) -> Style {
    let palette = theme.palette();
    match state {
        ConnectionState::Online => theme.fg(palette.ok),
        ConnectionState::Stale => theme.fg(palette.warn),
        ConnectionState::Timeout => theme.fg(palette.danger),
        ConnectionState::AuthFailed => theme.fg(palette.danger),
        ConnectionState::Offline => theme.fg(palette.danger),
        ConnectionState::Checking => theme.fg(palette.warn),
        ConnectionState::Unknown => theme.fg(palette.muted),
    }
}

fn visible_start(cursor: usize, total: usize, capacity: usize) -> usize {
    if capacity == 0 || total <= capacity {
        return 0;
    }
    cursor
        .saturating_add(1)
        .saturating_sub(capacity)
        .min(total.saturating_sub(capacity))
}

fn truncate_to_width(value: &str, width: usize) -> String {
    if UnicodeWidthStr::width(value) <= width {
        return value.to_string();
    }
    if width <= 3 {
        return ".".repeat(width);
    }
    let mut used = 0;
    let mut truncated = String::new();
    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > width - 3 {
            break;
        }
        truncated.push(ch);
        used += ch_width;
    }
    truncated.push_str("...");
    truncated
}

fn ascii_safe(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii() && !ch.is_control() {
                ch
            } else {
                '?'
            }
        })
        .collect()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_parser_handles_ascii_and_fullwidth_percent_tokens() {
        assert_eq!(percent_from_token("memory=42% free"), Some(42));
        assert_eq!(percent_from_token("memory=42％ free"), Some(42));
        assert_eq!(percent_from_token("gpu util 99.6%"), Some(100));
    }
}
