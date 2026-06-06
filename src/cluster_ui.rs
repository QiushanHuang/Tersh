use crate::cluster::{ClusterApp, ClusterMode, ConnectionState, HostKind, HostSnapshot};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

const ASCII_BORDER: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

pub fn draw(frame: &mut Frame, app: &ClusterApp) {
    let area = frame.area();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(area);

    draw_header(frame, rows[0], app);
    if app.mode() == ClusterMode::Detail {
        draw_dashboard(frame, rows[1], app);
    } else {
        draw_body(frame, rows[1], app);
    }
    draw_footer(frame, rows[2], app);

    if app.mode() == ClusterMode::Help {
        draw_help(frame, centered_rect(70, 60, area));
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let lines = vec![Line::from(vec![
        Span::styled(
            "Cluster Status",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!("OK {}", app.online_count()),
            Style::default().fg(Color::Green),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("OLD {}", app.stale_count()),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("FAIL {}", app.offline_count()),
            Style::default().fg(Color::Red),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("CHK {}/{}", app.checking_count(), app.hosts().len()),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | src "),
        Span::styled(
            ascii_safe(&app.inventory_label()),
            Style::default().fg(Color::Gray),
        ),
    ])];
    let paragraph = Paragraph::new(lines).block(base_block().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    if area.width >= 100 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_hosts(frame, columns[0], app);
        draw_dashboard(frame, columns[1], app);
    } else if area.width >= 72 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
            .split(area);
        draw_hosts(frame, rows[0], app);
        draw_dashboard(frame, rows[1], app);
    } else {
        draw_hosts(frame, area, app);
    }
}

fn draw_hosts(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let mut lines = vec![Line::from(Span::styled(
        "state  alias           role     lat    mem   disk  address",
        Style::default().fg(Color::Gray),
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
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            state_style(state)
        };
        lines.push(Line::from(Span::styled(row, style)));
    }

    let title = format!(
        "Hosts {}/{}",
        app.cursor().saturating_add(1),
        app.hosts().len()
    );
    let paragraph = Paragraph::new(lines).block(base_block().title(title).borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    if area.height < 14 {
        draw_compact_dashboard(frame, area, app);
        return;
    }

    let route_height = if area.height >= 18 { 7 } else { 6 };
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(route_height), Constraint::Min(5)])
        .split(area);
    draw_route(frame, rows[0], app);
    draw_detail_panel(frame, rows[1], app);
}

fn draw_compact_dashboard(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let mut lines = match app.selected_host() {
        Some(host) => compact_route_lines(host),
        None => vec![Line::from("No cluster hosts available"), Line::from("")],
    };
    lines.extend(compact_status_lines(app));

    let paragraph =
        Paragraph::new(lines).block(base_block().title("Route / Detail").borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn compact_route_lines(host: &crate::cluster::HostConfig) -> Vec<Line<'static>> {
    match host.kind() {
        HostKind::Local => vec![
            Line::from(Span::styled(
                "Route: LOCAL ONLY",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Host: {}", ascii_safe(host.alias()))),
        ],
        HostKind::Jump => vec![
            Line::from(vec![
                Span::styled("Route: LOCAL", Style::default().fg(Color::Cyan)),
                Span::raw(" => "),
                Span::styled("JUMP", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(format!("Path: {}", ascii_safe(host.ssh_target()))),
        ],
        HostKind::Server => {
            if let Some(jump_target) = host.proxy_jump_target().or_else(|| host.proxy_jump()) {
                vec![
                    Line::from(vec![
                        Span::styled("Route: LOCAL", Style::default().fg(Color::Cyan)),
                        Span::raw(" => "),
                        Span::styled("JUMP", Style::default().fg(Color::Yellow)),
                        Span::raw(" => "),
                        Span::styled("SERVER", Style::default().fg(Color::Green)),
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
                        Span::styled("Route: LOCAL", Style::default().fg(Color::Cyan)),
                        Span::raw(" => "),
                        Span::styled("SERVER", Style::default().fg(Color::Green)),
                    ]),
                    Line::from(format!("Path: {}", ascii_safe(host.ssh_target()))),
                ]
            }
        }
    }
}

fn compact_status_lines(app: &ClusterApp) -> Vec<Line<'static>> {
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
        resource_line("Memory", report.memory.as_deref(), MetricKind::Memory),
        resource_line("Storage", report.storage.as_deref(), MetricKind::Storage),
        resource_line("GPU", report.gpu.as_deref(), MetricKind::Gpu),
        Line::from(format!("Log: {}", ascii_safe(latest_log))),
    ]
}

fn draw_route(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let paragraph = Paragraph::new(match app.selected_host() {
        Some(host) => route_lines(host),
        None => vec![Line::from("No route available")],
    })
    .block(base_block().title("Route").borders(Borders::ALL))
    .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn route_lines(host: &crate::cluster::HostConfig) -> Vec<Line<'static>> {
    match host.kind() {
        HostKind::Local => vec![
            Line::from(Span::styled(
                "LOCAL ONLY",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("local: {}", ascii_safe(host.alias()))),
            Line::from(format!("address: {}", ascii_safe(host.address()))),
        ],
        HostKind::Jump => vec![
            Line::from(vec![
                Span::styled("LOCAL", Style::default().fg(Color::Cyan)),
                Span::raw(" => "),
                Span::styled("JUMP", Style::default().fg(Color::Yellow)),
            ]),
            Line::from(format!("jump: {}", ascii_safe(host.ssh_target()))),
            Line::from(format!("command: ssh {}", ascii_safe(host.ssh_target()))),
        ],
        HostKind::Server => {
            if let Some(jump_target) = host.proxy_jump_target().or_else(|| host.proxy_jump()) {
                vec![
                    Line::from(vec![
                        Span::styled("LOCAL", Style::default().fg(Color::Cyan)),
                        Span::raw(" => "),
                        Span::styled("JUMP", Style::default().fg(Color::Yellow)),
                        Span::raw(" => "),
                        Span::styled("SERVER", Style::default().fg(Color::Green)),
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
                        Span::styled("LOCAL", Style::default().fg(Color::Cyan)),
                        Span::raw(" => "),
                        Span::styled("SERVER", Style::default().fg(Color::Green)),
                    ]),
                    Line::from(format!("server: {}", ascii_safe(host.ssh_target()))),
                    Line::from(format!("command: ssh {}", ascii_safe(host.ssh_target()))),
                ]
            }
        }
    }
}

fn draw_detail_panel(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let lines = detail_lines(app, true);
    let paragraph = Paragraph::new(lines)
        .block(base_block().title("Detail").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn detail_lines(app: &ClusterApp, include_logs: bool) -> Vec<Line<'static>> {
    let Some(host) = app.selected_host() else {
        return vec![Line::from("No host selected")];
    };
    let snapshot = app.selected_snapshot();
    let mut lines = vec![Line::from(vec![
        Span::styled(
            ascii_safe(host.alias()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(format!("  {}", host.kind().label())),
    ])];
    lines.push(Line::from(""));

    if let Some(snapshot) = snapshot {
        push_snapshot_lines(&mut lines, snapshot);
    } else {
        lines.push(Line::from("Status: unknown"));
    }
    if include_logs {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Log",
            Style::default().fg(Color::Gray),
        )));
        for log in app.logs().iter().rev().take(4) {
            lines.push(Line::from(ascii_safe(log)));
        }
    }

    if let Some(workdir) = host.workdir() {
        lines.push(Line::from(format!("Tersh dir: {}", ascii_safe(workdir))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(format!("Role: {}", ascii_safe(host.role()))));
    lines.push(Line::from(format!(
        "Address: {}",
        ascii_safe(host.address())
    )));
    lines.push(Line::from(format!(
        "SSH target: {}",
        ascii_safe(host.ssh_target())
    )));
    if let Some(user) = host.user() {
        lines.push(Line::from(format!("User: {}", ascii_safe(user))));
    }
    if host.kind() == HostKind::Server {
        lines.push(Line::from(format!(
            "ProxyJump: {}",
            ascii_safe(host.proxy_jump().unwrap_or("none"))
        )));
        if let Some(target) = host.proxy_jump_target() {
            lines.push(Line::from(format!("Jump target: {}", ascii_safe(target))));
        }
    }
    let route = host
        .proxy_jump_target()
        .map(|jump| format!("via {}", ascii_safe(jump)))
        .unwrap_or_else(|| "direct/local".to_string());
    lines.push(Line::from(format!("Network route: {route}")));
    lines
}

fn push_snapshot_lines(lines: &mut Vec<Line<'static>>, snapshot: &HostSnapshot) {
    let latency = snapshot
        .latency_ms
        .map(|value| format!("{value} ms"))
        .unwrap_or_else(|| "-".to_string());
    lines.push(Line::from(format!(
        "Connection: {} ({latency})",
        snapshot.connection.label()
    )));
    if let Some(error) = &snapshot.error {
        lines.push(Line::from(Span::styled(
            format!("Error: {}", ascii_safe(error)),
            Style::default().fg(Color::Red),
        )));
        if snapshot.report.is_empty() {
            return;
        }
    }

    let report = &snapshot.report;
    lines.push(Line::from(Span::styled(
        "Health",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(format!(
        "CPU load: {}",
        ascii_safe(report.cpu_load.as_deref().unwrap_or("unknown"))
    )));
    lines.push(resource_line(
        "Memory",
        report.memory.as_deref(),
        MetricKind::Memory,
    ));
    lines.push(resource_line(
        "Storage",
        report.storage.as_deref(),
        MetricKind::Storage,
    ));
    lines.push(Line::from(format!(
        "Tasks: {}",
        ascii_safe(report.tasks.as_deref().unwrap_or("unknown"))
    )));
    lines.push(resource_line("GPU", report.gpu.as_deref(), MetricKind::Gpu));
    lines.push(Line::from(Span::styled(
        "System",
        Style::default().fg(Color::Gray),
    )));
    lines.push(Line::from(format!(
        "Hostname: {}",
        ascii_safe(report.hostname.as_deref().unwrap_or("unknown"))
    )));
    lines.push(Line::from(format!(
        "System: {}",
        ascii_safe(report.system.as_deref().unwrap_or("unknown"))
    )));
    lines.push(Line::from(format!(
        "Uptime: {}",
        ascii_safe(report.uptime.as_deref().unwrap_or("unknown"))
    )));
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricKind {
    Memory,
    Storage,
    Gpu,
}

fn resource_line(label: &'static str, value: Option<&str>, kind: MetricKind) -> Line<'static> {
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
    let bar = ascii_bar(bar_percent);
    let percent_text = metric
        .map(|(value, suffix)| {
            if suffix.is_empty() {
                format!("{value:>3}%")
            } else {
                format!("{value:>3}% {suffix}")
            }
        })
        .unwrap_or_else(|| " --".to_string());
    let style = bar_percent
        .map(metric_style)
        .unwrap_or_else(|| Style::default().fg(Color::Gray));

    Line::from(vec![
        Span::raw(format!("{label}: ")),
        Span::styled(bar, style),
        Span::raw(format!(" {percent_text}  {}", ascii_safe(raw))),
    ])
}

fn memory_metric(raw: &str) -> Option<(u16, &'static str)> {
    let percent = percent_from_token(raw)?;
    if raw.to_lowercase().contains("free") {
        Some((100_u16.saturating_sub(percent), "used"))
    } else {
        Some((percent, "used"))
    }
}

fn ascii_bar(percent: Option<u16>) -> String {
    const WIDTH: usize = 10;
    let filled = percent
        .map(|value| ((clamp_percent(value) as usize * WIDTH) + 50) / 100)
        .unwrap_or(0)
        .min(WIDTH);
    format!("[{}{}]", "#".repeat(filled), "-".repeat(WIDTH - filled))
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

fn metric_style(percent: u16) -> Style {
    match clamp_percent(percent) {
        0..=69 => Style::default().fg(Color::Green),
        70..=89 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Red),
    }
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let compact = area.width < 64;
    let text = match app.mode() {
        ClusterMode::Help if compact => "help | q/Esc | ^G | ^C",
        ClusterMode::Help => "help | q/?/Enter/Esc close | ^G close | ^C force",
        ClusterMode::Detail if compact => "detail | q/Esc | ^G | ^C",
        ClusterMode::Detail => {
            "detail | q/Esc back | ^G back | ^C force | r refresh | s shell/ssh | t tersh"
        }
        ClusterMode::Normal if compact => "q quit | ? help | ^G | ^C",
        ClusterMode::Normal => {
            "q quit | ? help | ^G back | ^C force | r refresh | Enter refresh host | s shell/ssh | t tersh | l detail | j/k move | Home/End"
        }
    };
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(base_block().borders(Borders::TOP));
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
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
        .block(base_block().title("Help").borders(Borders::ALL))
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

fn base_block() -> Block<'static> {
    Block::default().border_set(ASCII_BORDER)
}

fn state_style(state: ConnectionState) -> Style {
    match state {
        ConnectionState::Online => Style::default().fg(Color::Green),
        ConnectionState::Stale => Style::default().fg(Color::Yellow),
        ConnectionState::Timeout => Style::default().fg(Color::Red),
        ConnectionState::AuthFailed => Style::default().fg(Color::Red),
        ConnectionState::Offline => Style::default().fg(Color::Red),
        ConnectionState::Checking => Style::default().fg(Color::Yellow),
        ConnectionState::Unknown => Style::default().fg(Color::Gray),
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
    if value.chars().count() <= width {
        return value.to_string();
    }
    if width <= 3 {
        return ".".repeat(width);
    }
    let mut truncated = value.chars().take(width - 3).collect::<String>();
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
