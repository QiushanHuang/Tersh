use crate::{
    cluster::{ClusterApp, ClusterMode, ConnectionState, HostKind, HostSnapshot},
    theme::{
        Theme, Tone, base_block, chip, footer_compact, footer_height, footer_rows, kv_line,
        panel_block, panel_title, resource_bar, section_line,
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
            Constraint::Length(footer_height(area.width, area.height)),
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
        draw_help(frame, centered_rect(85, 80, area), app, theme);
    }
    if app.mode() == ClusterMode::Filter {
        let rect = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(7) / 2,
            area.width,
            area.height.min(5),
        );
        frame.render_widget(Clear, rect);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(format!(
                    "Alias, address or role; {} apply, {} cancel",
                    cluster_key(app, "cluster_filter", "submit"),
                    cluster_key(app, "cluster_filter", "cancel")
                )),
                Line::from(crate::fs_core::escape_display(app.filter())),
            ])
            .block(
                base_block()
                    .borders(Borders::ALL)
                    .title(panel_title(theme, "Filter hosts")),
            ),
            rect,
        );
    }
    if let Some(menu) = app.actions() {
        crate::actions::draw(frame, area, menu, theme);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let palette = theme.palette();
    if area.width < 60 {
        let stats = Line::from(vec![
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
                app.checking_count(),
                theme.chip(palette.text, palette.accent),
            ),
        ]);
        frame.render_widget(
            Paragraph::new(stats).block(
                base_block()
                    .borders(Borders::ALL)
                    .title(panel_title(theme, "Tersh --c")),
            ),
            area,
        );
        return;
    }
    let lines = vec![Line::from(vec![
        Span::styled(
            if area.width < 60 {
                "Tersh --c"
            } else {
                "Cluster Status"
            },
            theme.fg_bold(palette.panel_title),
        ),
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
            if app.checking_count() > 0 {
                format!("{} {}", app.activity_symbol(), app.checking_count())
            } else {
                format!("0/{}", app.hosts().len())
            },
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
        if area.width < 56 {
            "state  alias         probe  mem% disk%"
        } else {
            "state  alias           role     probe  mem   disk  address"
        },
        theme.fg_bold(theme.palette().key),
    ))];
    let capacity = area.height.saturating_sub(3) as usize;
    let start = visible_start(app.cursor(), app.visible_count(), capacity);

    for (index, host) in app.visible_hosts().enumerate().skip(start).take(capacity) {
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
            .and_then(crate::metrics::memory_used)
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let storage = snapshot
            .and_then(|snapshot| snapshot.report.storage.as_deref())
            .and_then(percent_from_token)
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let cursor = if index == app.cursor() { ">" } else { " " };
        let row = if area.width < 56 {
            let alias_width = area.width.saturating_sub(29) as usize;
            format!(
                "{cursor} {:<5} {:alias_width$} {:>6} {:>4} {:>4}",
                state_short(state),
                truncate_to_width(&ascii_safe(host.alias()), alias_width),
                truncate_to_width(&latency, 6),
                memory,
                storage
            )
        } else {
            format!(
                "{cursor} {:<5} {:<15} {:<8} {:<6} {:<5} {:<5} {}",
                state_short(state),
                ascii_safe(host.alias()),
                host.kind().label(),
                latency,
                memory,
                storage,
                ascii_safe(host.address())
            )
        };
        let row = truncate_to_width(&row, area.width.saturating_sub(2) as usize);
        let style = if index == app.cursor() {
            theme.selected()
        } else {
            state_style(theme, state)
        };
        lines.push(Line::from(Span::styled(row, style)));
    }

    if app.visible_count() == 0 {
        lines.push(Line::from(format!(
            "No matching hosts; {} clears filter",
            cluster_key(app, "cluster", "clear_filter")
        )));
    }
    let title = format!(
        "Hosts {}/{} | {} actions | {}{}",
        if app.visible_count() == 0 {
            0
        } else {
            app.cursor() + 1
        },
        app.visible_count(),
        cluster_key(app, app.key_context(), "open_actions"),
        app.sort_label(),
        if app.filter().is_empty() {
            String::new()
        } else {
            format!(" | /{}", crate::fs_core::escape_display(app.filter()))
        }
    );
    let paragraph = Paragraph::new(lines).block(panel_block(theme, title, Tone::Active));
    frame.render_widget(paragraph, area);
}

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme, active: bool) {
    if app.mode() == ClusterMode::Detail && area.height < 14 {
        draw_detail_panel(frame, area, app, theme, active);
        return;
    }
    if area.height < 14 {
        draw_compact_dashboard(frame, area, app, theme, active);
        return;
    }

    let route_height = app
        .selected_host()
        .map(|host| route_lines(host, theme).len() as u16 + 2)
        .unwrap_or(3)
        .min(if area.height >= 18 { 7 } else { 6 });
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
        Line::from(format!("Log: {}", ascii_safe(latest_log))),
        resource_line(theme, "GPU", report.gpu.as_deref(), MetricKind::Gpu),
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
    let mut lines = detail_lines(app, true, theme);
    let trends = if app.mode() == ClusterMode::Detail
        && app
            .selected_host()
            .and_then(|host| app.history_for(host.alias()))
            .is_some_and(|history| history.len() >= 2)
    {
        trend_lines(app, area.width.saturating_sub(2), theme)
    } else {
        vec![]
    };
    // Put observations before system/log metadata without hiding current health.
    let insert_at = lines
        .iter()
        .position(|line| line.to_string() == "System")
        .unwrap_or(lines.len());
    lines.splice(insert_at..insert_at, trends);
    let max_offset = lines
        .len()
        .saturating_sub(area.height.saturating_sub(2) as usize)
        .min(u16::MAX as usize) as u16;
    app.set_detail_limit(max_offset);
    let title = if app.mode() == ClusterMode::Detail {
        format!(
            "Detail | {}/{} scroll",
            cluster_key(app, "cluster_detail", "detail_up"),
            cluster_key(app, "cluster_detail", "detail_down")
        )
    } else {
        format!(
            "Detail | {} expand",
            cluster_key(app, "cluster", "open_detail")
        )
    };
    let tone = if active { Tone::Active } else { Tone::Inactive };
    let paragraph = Paragraph::new(lines)
        .scroll((app.detail_offset().min(max_offset), 0))
        .block(panel_block(theme, title, tone));
    frame.render_widget(paragraph, area);
}

fn trend_lines(app: &ClusterApp, width: u16, theme: Theme) -> Vec<Line<'static>> {
    let Some(host) = app.selected_host() else {
        return vec![];
    };
    let Some(history) = app.history_for(host.alias()).filter(|h| !h.is_empty()) else {
        return vec![
            section_line(theme, "Trends"),
            Line::from("Waiting for completed probes"),
        ];
    };
    let graph_width = (width.saturating_sub(28).clamp(4, 60) as usize).min(history.len());
    let samples = history
        .iter()
        .skip(history.len().saturating_sub(graph_width))
        .collect::<Vec<_>>();
    let duration = samples
        .last()
        .unwrap()
        .at
        .duration_since(samples[0].at)
        .unwrap_or_default()
        .as_secs();
    let mut lines = vec![
        section_line(theme, "Trends"),
        Line::from(Span::styled(
            format!("{} samples / {}s; gaps = missing", samples.len(), duration),
            theme.fg(theme.palette().muted),
        )),
    ];
    let palette = theme.palette();
    for (metric, label, unit, color) in [
        (0, "Load 1m", "", palette.accent),
        (1, "Mem used", "%", palette.accent_alt),
        (2, "Disk used", "%", palette.warn),
        (3, "Probe", "ms", palette.ok),
    ] {
        let values = samples
            .iter()
            .map(|sample| sample.values[metric])
            .collect::<Vec<_>>();
        let ceiling = if metric == 1 || metric == 2 {
            100.0
        } else {
            values
                .iter()
                .flatten()
                .copied()
                .fold(0.0_f64, f64::max)
                .max(1.0)
        };
        let latest = values
            .last()
            .copied()
            .flatten()
            .map(|v| {
                if metric == 0 {
                    format!("{v:.2}")
                } else {
                    format!("{v:.0}{unit}")
                }
            })
            .unwrap_or_else(|| "--".into());
        let graph = crate::metrics::sparkline(&values, ceiling, crate::theme::unicode_graphs());
        let scale = if metric == 0 {
            format!("{ceiling:.2}")
        } else {
            format!("{ceiling:.0}{unit}")
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{label:<9}"), theme.fg(palette.key)),
            Span::styled(format!("{graph:graph_width$}"), theme.fg(color)),
            Span::styled(format!(" {latest} /{scale}"), theme.fg(palette.value)),
        ]));
    }
    if let Some(good) = history
        .iter()
        .rev()
        .find(|sample| sample.values.iter().any(Option::is_some))
    {
        let seconds = good
            .at
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        lines.push(Line::from(Span::styled(
            format!(
                "Last data {:02}:{:02}:{:02} UTC | load != CPU %",
                seconds / 3600 % 24,
                seconds / 60 % 60,
                seconds % 60
            ),
            theme.fg(palette.muted),
        )));
    }
    lines
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
            "Hint: refresh selected host",
            theme.fg(palette.muted),
        )));
    }
    if include_logs {
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
        MetricKind::Gpu => gpu_metric(raw),
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
    crate::metrics::memory_used(raw).map(|value| (value, "used"))
}

fn gpu_metric(raw: &str) -> Option<(u16, &'static str)> {
    if let Some(percent) = percent_from_token(raw) {
        return Some((percent, ""));
    }
    raw.split(';').find_map(|gpu| {
        let mut fields = gpu.split(',').map(str::trim);
        fields.next()?;
        fields
            .next()
            .and_then(parse_percent_number)
            .map(|percent| (percent, ""))
    })
}

fn percent_from_token(input: &str) -> Option<u16> {
    crate::metrics::percent(input)
}

fn parse_percent_number(value: &str) -> Option<u16> {
    let value = value.trim().parse::<f64>().ok()?;
    Some(clamp_percent(value.round().max(0.0) as u16))
}

fn clamp_percent(value: u16) -> u16 {
    value.min(100)
}

fn cluster_key(app: &ClusterApp, context: &str, action: &str) -> String {
    crate::bindings::label(app.keymap(), context, action).unwrap_or_else(|| "unbound".into())
}
fn draw_footer(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let context = app.key_context();
    let compact = footer_compact(area.width, 64);
    let mut pieces = if area.width < 50 && context == "cluster" {
        Vec::new()
    } else {
        vec![
            match context {
                "cluster" => "cluster",
                "cluster_detail" => "detail",
                "cluster_filter" => "filter",
                other => other,
            }
            .into(),
        ]
    };
    let mut add = |id: &str, description: &str| {
        if id == "cancel" {
            pieces.push(format!(
                "{}{}",
                crate::bindings::cancel_label(app.keymap(), context, compact),
                if compact {
                    String::new()
                } else {
                    format!(" {description}")
                }
            ));
        } else if let Some(hint) = crate::bindings::hint(app.keymap(), context, id, description) {
            pieces.push(hint);
        }
    };
    match context {
        "actions" => {
            add("cancel", "cancel");
            add("submit", "run");
            add("down", "next");
        }
        "help" => {
            add("cancel", "close");
            add("down", "scroll");
            add("up", "up");
        }
        "cluster_filter" => {
            add("cancel", "cancel");
            add("submit", "apply");
            add("backspace", "erase");
        }
        _ => {
            if context == "cluster" {
                add("quit", "quit");
                add("open_help", "help");
            } else {
                add("cancel", "back");
            }
            if area.width >= 50 || context != "cluster" {
                add("open_actions", "actions");
            }
            if context == "cluster" {
                add("cancel", "cancel");
                add("clear_filter", "clear filter");
            }
            if app.selected_host().is_some() {
                add("open_detail", "detail");
            }
            add("refresh_all", "refresh");
            add("open_filter", "filter");
            add("cycle_sort", "sort");
            if app.selected_host().is_some() {
                add("open_workbench", "tersh");
                add("open_session", "shell/ssh");
            }
            if context == "cluster_detail" {
                add("detail_down", "scroll");
            }
            if !compact {
                add("down", "down");
                add("up", "up");
            }
        }
    }
    pieces.insert(
        if pieces.is_empty() { 0 } else { 1 },
        if compact {
            "^C".into()
        } else {
            "^C force".into()
        },
    );
    if context == "cluster" && area.width >= 50 {
        let (id, label) = if app.selected_host().is_none() {
            ("clear_filter", "clear filter")
        } else {
            match app.selected_snapshot().map(|s| s.connection) {
                Some(ConnectionState::Online) => ("open_workbench", "tersh"),
                Some(
                    ConnectionState::AuthFailed
                    | ConnectionState::Timeout
                    | ConnectionState::Offline,
                ) => ("open_detail", "detail"),
                _ => ("refresh_selected", "refresh"),
            }
        };
        if let Some(hint) = crate::bindings::hint(app.keymap(), context, id, label) {
            pieces.push(format!("next: {hint}"));
        }
    }
    frame.render_widget(
        Paragraph::new(footer_rows(
            theme,
            &pieces.join(" | "),
            area.width,
            area.height.saturating_sub(1) as usize,
        ))
        .block(base_block().borders(Borders::TOP)),
        area,
    );
}

fn action_hint(snapshot: &HostSnapshot) -> Option<&'static str> {
    match snapshot.connection {
        ConnectionState::AuthFailed => Some("check SSH auth and trusted host key"),
        ConnectionState::Timeout => Some("probe timed out; SSH may still work"),
        ConnectionState::Offline => Some("inspect the probe error and host availability"),
        ConnectionState::Stale => Some("refresh failed; showing last good metrics"),
        ConnectionState::Unknown => Some("refresh selected host"),
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

fn draw_help(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    frame.render_widget(Clear, area);
    let lines = crate::bindings::help_lines(app.keymap(), app.help_context())
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((app.help_offset().min(u16::MAX as usize) as u16, 0))
            .block(panel_block(
                theme,
                format!(
                    "Help: {} | {} close",
                    app.help_context(),
                    cluster_key(app, "help", "cancel")
                ),
                Tone::Active,
            )),
        area,
    );
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
