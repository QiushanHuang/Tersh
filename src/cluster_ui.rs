use crate::cluster::{ClusterApp, ClusterMode, ConnectionState, HostKind, HostSnapshot};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
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
        draw_detail(frame, rows[1], app);
    } else {
        draw_body(frame, rows[1], app);
    }
    draw_footer(frame, rows[2]);

    if app.mode() == ClusterMode::Help {
        draw_help(frame, centered_rect(70, 60, area));
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let summary = format!(
        "online {} | stale {} | failed {} | checking {} | source {}",
        app.online_count(),
        app.stale_count(),
        app.offline_count(),
        app.checking_count(),
        app.inventory_label()
    );
    let lines = vec![Line::from(vec![
        Span::styled(
            "Cluster Status",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(summary, Style::default().fg(Color::Gray)),
    ])];
    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    if area.width >= 100 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_hosts(frame, columns[0], app);
        draw_detail(frame, columns[1], app);
    } else if area.width >= 72 {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
            .split(area);
        draw_hosts(frame, rows[0], app);
        draw_detail(frame, rows[1], app);
    } else {
        draw_hosts(frame, area, app);
    }
}

fn draw_hosts(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let mut lines = vec![Line::from(Span::styled(
        "state  alias           role     address",
        Style::default().fg(Color::Gray),
    ))];
    let capacity = area.height.saturating_sub(3) as usize;
    let start = visible_start(app.cursor(), app.hosts().len(), capacity);

    for (index, host) in app.hosts().iter().enumerate().skip(start).take(capacity) {
        let snapshot = app.snapshot_for(host.alias());
        let state = snapshot
            .map(|snapshot| snapshot.connection)
            .unwrap_or(ConnectionState::Unknown);
        let cursor = if index == app.cursor() { ">" } else { " " };
        let row = format!(
            "{cursor} {:<5} {:<15} {:<8} {}",
            state_short(state),
            host.alias(),
            host.kind().label(),
            host.address()
        );
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
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title(title).borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_detail(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let host = app.selected_host();
    let snapshot = app.selected_snapshot();
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                host.alias(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("  {}", host.kind().label())),
        ]),
        Line::from(format!("Role: {}", host.role())),
        Line::from(format!("Address: {}", host.address())),
        Line::from(format!("SSH target: {}", host.ssh_target())),
    ];

    if let Some(user) = host.user() {
        lines.push(Line::from(format!("User: {user}")));
    }
    if host.kind() == HostKind::Server {
        lines.push(Line::from(format!(
            "ProxyJump: {}",
            host.proxy_jump().unwrap_or("none")
        )));
        if let Some(target) = host.proxy_jump_target() {
            lines.push(Line::from(format!("Jump target: {target}")));
        }
    }
    let route = host
        .proxy_jump_target()
        .map(|jump| format!("via {jump}"))
        .unwrap_or_else(|| "direct/local".to_string());
    lines.push(Line::from(format!("Network route: {route}")));
    lines.push(Line::from(""));

    if let Some(snapshot) = snapshot {
        push_snapshot_lines(&mut lines, snapshot);
    } else {
        lines.push(Line::from("Status: unknown"));
    }
    if let Some(workdir) = host.workdir() {
        lines.push(Line::from(format!("Tersh dir: {workdir}")));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Log",
        Style::default().fg(Color::Gray),
    )));
    for log in app.logs().iter().rev().take(4) {
        lines.push(Line::from(log.clone()));
    }

    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Detail").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn push_snapshot_lines(lines: &mut Vec<Line<'_>>, snapshot: &HostSnapshot) {
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
            format!("Error: {error}"),
            Style::default().fg(Color::Red),
        )));
        if snapshot.report.is_empty() {
            return;
        }
    }

    let report = &snapshot.report;
    lines.push(Line::from(format!(
        "Hostname: {}",
        report.hostname.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "System: {}",
        report.system.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "Uptime: {}",
        report.uptime.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "CPU load: {}",
        report.cpu_load.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "Memory: {}",
        report.memory.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "Storage: {}",
        report.storage.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "Tasks: {}",
        report.tasks.as_deref().unwrap_or("unknown")
    )));
    lines.push(Line::from(format!(
        "GPU: {}",
        report.gpu.as_deref().unwrap_or("unknown")
    )));
}

fn draw_footer(frame: &mut Frame, area: Rect) {
    let paragraph = Paragraph::new(
        "r refresh | Enter refresh host | s shell/ssh | t tersh | l detail | j/k move | Home/End | ? help | q quit | Ctrl+C force",
    )
    .style(Style::default().fg(Color::Gray))
    .block(Block::default().borders(Borders::TOP));
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
        Line::from("  Ctrl+G: close help"),
        Line::from("  Ctrl+C: force quit"),
        Line::from(""),
        Line::from("Inventory"),
        Line::from("  TERSH_SERVERS_JSON overrides the default servers.json path."),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
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
