use crate::{
    cluster::{ClusterApp, ClusterMode, ConnectionState, HostKind, HostSnapshot, ProbeReport},
    theme::{
        Theme, Tone, base_block, chip, draw_query, footer_compact, footer_height, footer_rows,
        kv_line, panel_block, resource_bar, section_line,
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
    draw_in(frame, frame.area(), app);
}

pub(crate) fn draw_in(frame: &mut Frame, area: Rect, app: &ClusterApp) {
    let theme = Theme::current();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(footer_height(area.width, area.height)),
        ])
        .split(area);

    if app.mode() == ClusterMode::Filter {
        draw_filter(frame, rows[0], app, theme);
    } else {
        draw_header(frame, rows[0], app, theme);
    }
    if app.mode() == ClusterMode::Hardware {
        draw_hardware(frame, rows[1], app, theme);
    } else if app.mode() == ClusterMode::Detail {
        draw_dashboard(frame, rows[1], app, theme, true);
    } else if app.topology_active() {
        draw_topology_page(frame, rows[1], app, theme);
    } else {
        draw_body(frame, rows[1], app, theme);
    }
    draw_footer(frame, rows[2], app, theme);

    if app.mode() == ClusterMode::Help {
        draw_help(frame, centered_rect(85, 80, area), app, theme);
    }
    if app.mode() == ClusterMode::Events {
        let rect = centered_rect(96, 90, area);
        let mut lines = app
            .events()
            .iter()
            .rev()
            .flat_map(|event| {
                let age = event.at.elapsed().unwrap_or_default().as_secs();
                wrap_display(
                    &format!(
                        "{age}s ago {}: {} -> {} | {}",
                        ascii_safe(&event.alias),
                        event.previous.label(),
                        event.current.label(),
                        ascii_safe(&event.message)
                    ),
                    rect.width.saturating_sub(2) as usize,
                )
            })
            .collect::<Vec<_>>();
        if lines.is_empty() {
            lines.push(Line::from("No completed state changes yet"));
        }
        let limit = lines
            .len()
            .saturating_sub(rect.height.saturating_sub(2) as usize);
        app.set_event_limit(limit);
        frame.render_widget(Clear, rect);
        frame.render_widget(
            Paragraph::new(lines)
                .scroll((
                    app.help_offset().min(limit).min(u16::MAX as usize) as u16,
                    0,
                ))
                .block(panel_block(
                    theme,
                    format!(
                        "Events {} / 100 | {} close",
                        app.events().len(),
                        cluster_key(app, "help", "cancel")
                    ),
                    Tone::Active,
                )),
            rect,
        );
    }
    if app.mode() == ClusterMode::Places {
        draw_places(frame, area, app, theme);
    }
    if let Some(menu) = app.actions() {
        crate::actions::draw(frame, area, menu, theme);
    }
}

fn draw_places(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let rect = centered_rect(96, 88, area);
    let mut lines = vec![Line::from(format!(
        "Find host/path: {}",
        crate::fs_core::escape_display(app.place_query())
    ))];
    lines.push(Line::from(if app.places_enabled() {
        format!(
            "{} opens the shown host and directory",
            cluster_key(app, "places", "submit")
        )
    } else {
        "History disabled; configured directories remain available".into()
    }));
    let capacity = rect.height.saturating_sub(6) as usize;
    let start = visible_start(app.place_cursor(), app.place_choices().len(), capacity);
    for (index, place) in app
        .place_choices()
        .iter()
        .enumerate()
        .skip(start)
        .take(capacity)
    {
        let line = format!(
            "{} {} {} {} {}",
            if index == app.place_cursor() {
                ">"
            } else {
                " "
            },
            if place.pinned { "*" } else { " " },
            if !place.valid {
                "[invalid]"
            } else if place.configured {
                "[config]"
            } else {
                "[recent]"
            },
            place.host,
            place.path.display()
        );
        lines.push(Line::from(Span::styled(
            truncate_to_width(&ascii_safe(&line), rect.width.saturating_sub(2) as usize),
            if index == app.place_cursor() {
                theme.selected()
            } else if !place.valid {
                theme.danger()
            } else {
                theme.fg(theme.palette().text)
            },
        )));
    }
    if app.place_choices().is_empty() {
        lines.push(Line::from(
            "Set an absolute host workdir, or save a local workbench place.",
        ));
    }
    if let Some(message) = app.place_message() {
        lines.push(Line::from(Span::styled(
            ascii_safe(message),
            theme.fg(theme.palette().warn),
        )));
    }
    lines.push(Line::from(format!(
        "{} pin | {} remove | {} clear recent",
        cluster_key(app, "places", "pin_place"),
        cluster_key(app, "places", "remove_place"),
        cluster_key(app, "places", "clear_recent")
    )));
    frame.render_widget(Clear, rect);
    frame.render_widget(
        Paragraph::new(lines).block(panel_block(
            theme,
            format!(
                "Places {} | {} back",
                app.place_choices().len(),
                cluster_key(app, "places", "cancel")
            ),
            Tone::Active,
        )),
        rect,
    );
}

fn draw_filter(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let mut title = format!(
        "Filter hosts | {}/{}",
        app.visible_count(),
        app.hosts().len()
    );
    if app.filter().is_empty() && area.width >= 48 {
        title.push_str(" | alias, address or role");
    }
    let block = panel_block(theme, title, Tone::Active);
    let field = block.inner(area);
    frame.render_widget(block, area);
    draw_query(
        frame,
        field,
        theme,
        "Find: ",
        &crate::fs_core::escape_display(app.filter()),
    );
}

fn draw_header(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let palette = theme.palette();
    let compact = area.width < 60;
    let mut spans = Vec::new();
    for (label, count, color) in [
        ("OK", app.online_count(), palette.ok),
        ("OLD", app.stale_count(), palette.warn),
        ("FAIL", app.failed_count(), palette.danger),
        ("CHK", app.checking_count(), palette.accent),
    ] {
        let value = match (label, count, compact) {
            ("CHK", 0, false) => format!("0/{}", app.hosts().len()),
            ("CHK", count, false) => format!("{} {count}", app.activity_symbol()),
            _ => count.to_string(),
        };
        let style = if count == 0 {
            theme.style(Tone::Muted)
        } else {
            theme.chip(palette.text, color)
        };
        spans.push(chip(label, value, style));
        spans.push(Span::raw(" "));
    }
    if !compact {
        spans.push(Span::styled(
            ascii_safe(&app.inventory_label()),
            theme.style(Tone::Muted),
        ));
    }
    let name = if compact {
        "Tersh --c"
    } else {
        "Cluster Status"
    };
    let title = if app.refresh_paused() {
        format!("{name} | PAUSED")
    } else {
        name.into()
    };
    let mut block = panel_block(theme, title, Tone::Title);
    let context = app.key_context();
    if matches!(context, "cluster" | "cluster_topology" | "cluster_detail") {
        let mut tabs = Vec::new();
        for (action, label, selected) in [
            (
                "show_overview",
                if area.width >= 80 {
                    "Overview"
                } else {
                    "Hosts"
                },
                app.mode() == ClusterMode::Normal,
            ),
            (
                "show_topology_trends",
                if area.width >= 80 {
                    "Map + trends"
                } else {
                    "Trends"
                },
                app.mode() == ClusterMode::Topology && app.topology_shows_trends(),
            ),
            (
                "show_topology_meters",
                if area.width >= 80 {
                    "Full meters"
                } else {
                    "Meters"
                },
                app.mode() == ClusterMode::Topology && !app.topology_shows_trends(),
            ),
        ] {
            if let Some(hint) = crate::bindings::hint(app.keymap(), context, action, label) {
                let style = if selected {
                    theme.selected()
                } else {
                    theme.bold(Tone::Key)
                };
                tabs.push(Span::styled(format!(" {hint} "), style));
            }
        }
        let switcher = Line::from(tabs);
        if switcher.width() + name.len() + 13 <= usize::from(area.width) {
            block = block.title_top(switcher.right_aligned());
        } else if !app.refresh_paused()
            && switcher.width() + 2 <= usize::from(area.width)
            && switcher.width() > 0
        {
            block = base_block()
                .borders(Borders::ALL)
                .border_style(theme.style(Tone::Title))
                .title(switcher);
        }
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);
}

fn draw_body(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    if area.width >= 100 {
        let columns = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);
        let list = columns[0];
        // Keep the host table first, then spend the remaining rows on routes and history.
        let host_rows = app.visible_count().saturating_add(3).clamp(5, 14) as u16;
        let history = app
            .selected_host()
            .and_then(|host| app.history_for(host.alias()))
            .is_some_and(|samples| samples.len() >= 2);
        let show_map = app.hosts().len() > 1 && list.height >= 13;
        let table_height = if show_map {
            host_rows.min(list.height.saturating_sub(7))
        } else {
            host_rows
        };
        let show_trends = history && list.height >= table_height + 9 + if show_map { 7 } else { 0 };
        // Tall windows have room to read both the routes and the selected
        // machine's telemetry. Keep the table fitted to the inventory.
        let tall = list.height >= 40 && app.selected_host().is_some() && (show_map || show_trends);
        let telemetry_height = if tall {
            (u32::from(list.height - table_height) * 9 / 20).clamp(12, 18) as u16
        } else if show_trends {
            9
        } else {
            0
        };
        let map_height = if show_map {
            list.height.saturating_sub(table_height + telemetry_height)
        } else {
            0
        };
        let rows = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(map_height),
            Constraint::Length(telemetry_height),
        ])
        .split(list);
        draw_hosts(frame, rows[0], app, theme);
        if show_map {
            crate::topology::draw(frame, rows[1], app, false);
        }
        if show_trends {
            draw_host_trends(frame, rows[2], app, theme);
        } else if tall {
            draw_selected_usage(frame, rows[2], app, theme);
        }
        draw_dashboard(frame, columns[1], app, theme, false);
    } else if area.width >= 72 {
        let rows =
            Layout::vertical([Constraint::Percentage(48), Constraint::Percentage(52)]).split(area);
        if rows[0].height >= 8 && app.hosts().iter().any(|host| host.proxy_jump().is_some()) {
            let columns =
                Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
                    .split(rows[0]);
            draw_hosts(frame, columns[0], app, theme);
            crate::topology::draw(frame, columns[1], app, false);
        } else {
            draw_hosts(frame, rows[0], app, theme);
        }
        draw_dashboard(frame, rows[1], app, theme, false);
    } else {
        draw_hosts(frame, area, app, theme);
    }
}

fn draw_host_trends(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    if let Some(host) = app.selected_host() {
        let trends = trend_lines(
            app,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
            theme,
        );
        frame.render_widget(
            Paragraph::new(trends).block(panel_block(
                theme,
                format!("Trends | {}", ascii_safe(host.alias())),
                Tone::Inactive,
            )),
            area,
        );
    }
}

fn draw_topology_page(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    if app.selected_host().is_none() {
        crate::topology::draw(frame, area, app, true);
        return;
    }
    let trends = app.topology_shows_trends();
    let split = trends && area.width >= 96 && area.height >= 18;
    let telemetry_height = if trends {
        (area.height.saturating_mul(2) / 5).clamp(7, 22)
    } else if area.height >= 35 {
        9
    } else {
        6
    };
    let rows = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(telemetry_height.min(area.height.saturating_sub(4))),
    ])
    .split(area);
    crate::topology::draw(frame, rows[0], app, true);
    if split {
        let columns =
            Layout::horizontal([Constraint::Percentage(38), Constraint::Fill(1)]).split(rows[1]);
        draw_selected_usage(frame, columns[0], app, theme);
        draw_host_trends(frame, columns[1], app, theme);
    } else if trends {
        draw_host_trends(frame, rows[1], app, theme);
    } else {
        draw_selected_usage(frame, rows[1], app, theme);
    }
}

fn draw_selected_usage(frame: &mut Frame, gauges: Rect, app: &ClusterApp, theme: Theme) {
    let Some(host) = app.selected_host() else {
        return;
    };
    let snapshot = app.selected_snapshot();
    let status = snapshot.map_or(ConnectionState::Unknown, |s| s.connection);
    let report = snapshot.map(|s| &s.report);
    let readings = if status == ConnectionState::Online {
        "usage"
    } else if report.is_some_and(|r| !r.is_empty()) {
        "last readings"
    } else {
        "no readings"
    };
    let title = format!("{} | {readings}", ascii_safe(host.alias()));
    let meters = [
        ("CPU", report.and_then(|r| r.hardware.cpu_usage_percent)),
        ("RAM", report.and_then(ProbeReport::memory_used_percent)),
        (
            "Disk",
            report
                .and_then(|r| r.storage.as_deref())
                .and_then(crate::metrics::percent),
        ),
        (
            "GPU peak",
            report.and_then(|r| r.hardware.gpus.iter().filter_map(|g| g.usage_percent).max()),
        ),
    ];
    let block = panel_block(theme, title, Tone::Title);
    let inner = block.inner(gauges);
    frame.render_widget(block, gauges);
    let gap = if inner.height >= 9 {
        (inner.height - 4) / 5
    } else {
        0
    };
    for (i, (label, value)) in meters.into_iter().enumerate() {
        let y = inner.y + gap + i as u16 * (gap + 1);
        if y >= inner.bottom() {
            break;
        }
        let mut spans = vec![Span::styled(format!("{label:<9}"), theme.style(Tone::Key))];
        spans.extend(resource_bar(
            theme,
            value,
            usize::from(inner.width.saturating_sub(16)),
        ));
        let used = value.map_or_else(|| " --".into(), |percent| format!(" {percent:>3}%"));
        spans.push(Span::styled(used, theme.style(Tone::Value)));
        frame.render_widget(Line::from(spans), Rect::new(inner.x, y, inner.width, 1));
    }
}

fn hardware_usage(theme: Theme, label: &str, value: Option<u16>) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{label}: "),
        theme.fg(theme.palette().key),
    )];
    spans.extend(resource_bar(theme, value, 10));
    spans.push(Span::raw(
        value
            .map(|v| format!(" {v}%"))
            .unwrap_or_else(|| " unknown".into()),
    ));
    Line::from(spans)
}

fn draw_hardware(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    use crate::hardware::bytes_label;
    let content = if area.width >= 100 {
        let columns = Layout::horizontal([Constraint::Length(37), Constraint::Min(1)]).split(area);
        let capacity = columns[0].height.saturating_sub(4) as usize;
        let start = visible_start(app.cursor(), app.visible_count(), capacity);
        let mut hosts = vec![
            Line::from(format!(
                "  {:<3} {} {:>4} {:>4} {:>4}",
                "St",
                fit_column("Device", 11),
                "CPU",
                "GPU",
                "RAM"
            )),
            Line::from(Span::styled(
                "  GPU = highest device usage",
                theme.fg(theme.palette().muted),
            )),
        ];
        for (index, host) in app.visible_hosts().enumerate().skip(start).take(capacity) {
            let snapshot = app.snapshot_for(host.alias());
            let hardware = snapshot.map(|s| &s.report.hardware);
            let state = snapshot
                .map(|s| s.connection)
                .unwrap_or(ConnectionState::Unknown);
            let status = match state {
                ConnectionState::Online => "OK",
                ConnectionState::Stale => "OLD",
                ConnectionState::Checking => "CHK",
                ConnectionState::Unknown => "--",
                _ => "ERR",
            };
            let percent = |value: Option<u16>| {
                value
                    .map(|v| format!("{v}%"))
                    .unwrap_or_else(|| "--".into())
            };
            let text = format!(
                "{} {:<3} {} {:>4} {:>4} {:>4}",
                if index == app.cursor() { ">" } else { " " },
                status,
                fit_column(&ascii_safe(host.alias()), 11),
                percent(hardware.and_then(|h| h.cpu_usage_percent)),
                percent(hardware.and_then(|h| h.gpus.iter().filter_map(|g| g.usage_percent).max())),
                percent(hardware.and_then(|h| h.memory_percent())),
            );
            hosts.push(Line::from(Span::styled(
                text,
                if index == app.cursor() {
                    theme.selected()
                } else {
                    state_style(theme, state)
                },
            )));
        }
        frame.render_widget(
            Paragraph::new(hosts).block(panel_block(
                theme,
                format!(
                    "Devices | {}/{} select",
                    cluster_key(app, "cluster_detail", "down"),
                    cluster_key(app, "cluster_detail", "up")
                ),
                Tone::Inactive,
            )),
            columns[0],
        );
        columns[1]
    } else {
        area
    };
    let mut lines = Vec::new();
    if let Some(host) = app.selected_host() {
        let snapshot = app.selected_snapshot();
        let state = snapshot
            .map(|s| s.connection)
            .unwrap_or(ConnectionState::Unknown);
        lines.push(Line::from(vec![
            Span::styled(
                ascii_safe(host.alias()),
                theme.fg_bold(theme.palette().path),
            ),
            Span::styled(format!(" | {}", state.label()), state_style(theme, state)),
        ]));
        lines.push(kv_line(
            theme,
            "Last good",
            app.data_age(host.alias())
                .map(|age| {
                    format!(
                        "{}s ago{}",
                        age.as_secs(),
                        if state == ConnectionState::Online {
                            ""
                        } else {
                            "; retained reading"
                        }
                    )
                })
                .unwrap_or_else(|| "no completed observation".into()),
        ));
        if let Some(error) = snapshot.and_then(|s| s.error.as_deref()) {
            lines.push(kv_line(theme, "Error", ascii_safe(error)));
        }
        let empty = crate::cluster::ProbeReport::default();
        let report = snapshot.map(|s| &s.report).unwrap_or(&empty);
        let hardware = &report.hardware;
        let text = |value: Option<&str>| ascii_safe(value.unwrap_or("unknown"));
        lines.push(kv_line(theme, "Machine", text(hardware.model.as_deref())));
        lines.push(kv_line(theme, "System", text(report.system.as_deref())));
        lines.push(kv_line(
            theme,
            "Architecture",
            text(hardware.architecture.as_deref()),
        ));
        lines.push(section_line(theme, "CPU"));
        lines.push(kv_line(theme, "Model", text(hardware.cpu_model.as_deref())));
        lines.push(kv_line(
            theme,
            "Topology",
            format!(
                "{} cores / {} threads",
                hardware
                    .cpu_cores
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into()),
                hardware
                    .cpu_threads
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unknown".into())
            ),
        ));
        lines.push(hardware_usage(theme, "CPU use", hardware.cpu_usage_percent));
        lines.push(kv_line(
            theme,
            "Load 1/5/15m",
            text(report.cpu_load.as_deref()),
        ));
        lines.push(section_line(theme, "Memory"));
        lines.push(kv_line(
            theme,
            if hardware.is_macos_ram_estimate() {
                "RAM est. used/total"
            } else {
                "RAM used/total"
            },
            format!(
                "{} / {}",
                bytes_label(hardware.memory_used_bytes),
                bytes_label(hardware.memory_total_bytes)
            ),
        ));
        lines.push(hardware_usage(theme, "RAM use", hardware.memory_percent()));
        lines.push(kv_line(
            theme,
            "Basis",
            text(hardware.memory_basis.as_deref()),
        ));
        lines.push(section_line(theme, "GPU"));
        if hardware.gpus.is_empty() {
            lines.push(kv_line(theme, "GPU", "unknown; no supported driver data"));
            if let Some(reason) = hardware.gpu_status.as_deref() {
                lines.push(kv_line(theme, "Reason", ascii_safe(reason)));
            }
        }
        for gpu in &hardware.gpus {
            lines.push(Line::from(Span::styled(
                format!("GPU {}: {}", ascii_safe(&gpu.id), ascii_safe(&gpu.name)),
                theme.fg_bold(theme.palette().accent),
            )));
            lines.push(hardware_usage(theme, "GPU use", gpu.usage_percent));
            if gpu.shared_memory {
                lines.push(kv_line(
                    theme,
                    "Shared RAM",
                    format!(
                        "{} in use (driver); shared with CPU",
                        bytes_label(gpu.memory_used_bytes)
                    ),
                ));
            } else {
                lines.push(kv_line(
                    theme,
                    "VRAM used/total",
                    format!(
                        "{} / {}",
                        bytes_label(gpu.memory_used_bytes),
                        bytes_label(gpu.memory_total_bytes)
                    ),
                ));
            }
            lines.push(kv_line(
                theme,
                "Temperature",
                gpu.temperature_c
                    .map(|v| format!("{v} C"))
                    .unwrap_or_else(|| "unknown".into()),
            ));
        }
        lines.push(section_line(theme, "Storage / uptime"));
        lines.push(kv_line(theme, "Root disk", text(report.storage.as_deref())));
        lines.push(kv_line(theme, "Uptime", text(report.uptime.as_deref())));
    } else {
        lines.push(Line::from("No matching device; clear the host filter"));
    }
    // Wrap before computing the scroll limit so long names/diagnostics remain
    // reachable even at 40 columns. Preserve styled rows when no wrap is needed.
    let width = content.width.saturating_sub(2).max(1) as usize;
    let lines = lines
        .into_iter()
        .flat_map(|line| {
            if line.width() <= width {
                vec![line]
            } else {
                wrap_display(&line.to_string(), width)
            }
        })
        .collect::<Vec<_>>();
    let limit = lines
        .len()
        .saturating_sub(content.height.saturating_sub(2) as usize)
        .min(u16::MAX as usize) as u16;
    app.set_detail_limit(limit);
    app.set_hardware_viewport(content.height.saturating_sub(2));
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((app.detail_offset().min(limit), 0))
            .block(panel_block(
                theme,
                format!(
                    "Hardware | {}/{} scroll",
                    cluster_key(app, "cluster_detail", "detail_up"),
                    cluster_key(app, "cluster_detail", "detail_down")
                ),
                Tone::Active,
            )),
        content,
    );
}

fn draw_hosts(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let width = area.width.saturating_sub(2) as usize;
    // Reserve complete numeric columns before spending the remainder on names.
    // Drop optional columns in narrow splits as well as on phone terminals.
    let show_usage = width >= 27;
    let show_probe = width >= 36;
    let show_role = width >= 52;
    let show_address = width >= 67;
    let fixed = 8
        + usize::from(show_usage) * 10
        + usize::from(show_probe) * 7
        + usize::from(show_role) * 12;
    let alias_width = if show_address {
        19
    } else {
        width.saturating_sub(fixed)
    };
    let columns = |alias: &str, role: &str, probe: &str, mem: &str, disk: &str, address: &str| {
        let mut text = fit_column(alias, alias_width);
        if show_role {
            text.push_str(&format!(" {}", fit_column(role, 11)));
        }
        if show_probe {
            text.push_str(&format!(" {:>6}", truncate_to_width(probe, 6)));
        }
        if show_usage {
            text.push_str(&format!(" {mem:>4} {disk:>4}"));
        }
        if show_address {
            let remaining = width.saturating_sub(fixed + alias_width + 1);
            text.push_str(&format!(" {}", truncate_to_width(address, remaining)));
        }
        text
    };
    let mut lines = vec![Line::from(Span::styled(
        format!(
            "  {:<5} {}",
            "state",
            columns("alias", "role", "probe", "mem", "disk", "address")
        ),
        theme.bold(Tone::Key),
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
            .and_then(|s| s.report.memory_used_percent())
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let storage = snapshot
            .and_then(|snapshot| snapshot.report.storage.as_deref())
            .and_then(percent_from_token)
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "--".to_string());
        let selected = index == app.cursor();
        let marker = if selected { "> " } else { "  " };
        let fields = columns(
            &ascii_safe(host.alias()),
            &ascii_safe(host.role()),
            &latency,
            &memory,
            &storage,
            &ascii_safe(host.address()),
        );
        let state_label = format!("{:<5} ", state_short(state));
        let line = if selected {
            Line::styled(format!("{marker}{state_label}{fields}"), theme.selected())
        } else {
            Line::from(vec![
                Span::raw(marker),
                Span::styled(state_label, state_style(theme, state)),
                Span::styled(fields, theme.style(Tone::Text)),
            ])
        };
        lines.push(line);
    }

    if app.visible_count() == 0 {
        let message = if app.hosts().is_empty() {
            "No hosts configured".into()
        } else if app.mode() == ClusterMode::Filter {
            "No matching hosts".into()
        } else {
            format!(
                "No matching hosts; {} clears filter",
                cluster_key(app, "cluster", "clear_filter")
            )
        };
        lines.push(Line::styled(message, theme.style(Tone::Muted)));
    }
    let position = if app.visible_count() == 0 {
        0
    } else {
        app.cursor() + 1
    };
    let mut title = format!("Hosts {position}/{}", app.visible_count());
    if app.attention_only() {
        title.push_str(" | attention");
    }
    if !app.filter().is_empty() && app.mode() != ClusterMode::Filter {
        title.push_str(&format!(
            " | /{}",
            crate::fs_core::escape_display(app.filter())
        ));
    }
    if let Some(hint) = crate::bindings::hint(app.keymap(), "cluster", "open_topology", "map") {
        title.push_str(&format!(" | {hint}"));
    }
    title.push_str(&format!(" | {}", app.sort_label()));
    let paragraph = Paragraph::new(lines).block(panel_block(theme, title, Tone::Active));
    frame.render_widget(paragraph, area);
}

fn draw_dashboard(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme, active: bool) {
    if app.mode() == ClusterMode::Detail && area.height < 14 {
        draw_detail_panel(frame, area, app, theme, active, true);
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
    let event_count = app.events().len().min(8) as u16;
    let event_height = if !active && area.height >= 35 && event_count > 0 {
        let needed = detail_lines(app, false, theme).len() as u16 + 2;
        (event_count + 2).min(area.height.saturating_sub(route_height + needed))
    } else {
        0
    };
    let event_height = if event_height >= 4 { event_height } else { 0 };
    let rows = Layout::vertical([
        Constraint::Length(route_height),
        Constraint::Min(5),
        Constraint::Length(event_height),
    ])
    .split(area);
    draw_route(frame, rows[0], app, theme, active);
    draw_detail_panel(frame, rows[1], app, theme, active, event_height == 0);
    if event_height > 0 {
        draw_recent_events(frame, rows[2], app, theme);
    }
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
        memory_line(theme, report),
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
            Line::from(format!(
                "command: ssh {}",
                ascii_safe(&crate::cluster::ssh_session_args(host).join(" "))
            )),
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
                    Line::from(format!(
                        "command: ssh {}",
                        ascii_safe(&crate::cluster::ssh_session_args(host).join(" "))
                    )),
                ]
            }
        }
    }
}

fn draw_recent_events(frame: &mut Frame, area: Rect, app: &ClusterApp, theme: Theme) {
    let title = crate::bindings::hint(app.keymap(), "cluster", "open_events", "all").map_or_else(
        || "Cluster events".into(),
        |hint| format!("Cluster events | {hint}"),
    );
    let arrow = if crate::theme::unicode_graphs() {
        "→"
    } else {
        "->"
    };
    let lines = app
        .events()
        .iter()
        .rev()
        .take(usize::from(area.height.saturating_sub(2)))
        .map(|event| {
            let age = event.at.elapsed().unwrap_or_default().as_secs();
            Line::from(vec![
                Span::styled(format!("{age:>4}s  "), theme.style(Tone::Key)),
                Span::styled(
                    format!("{}   ", ascii_safe(&event.alias)),
                    theme.bold(Tone::Text),
                ),
                Span::styled(
                    format!("{} {arrow} ", event.previous.label()),
                    theme.style(Tone::Muted),
                ),
                Span::styled(event.current.label(), state_style(theme, event.current)),
            ])
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        Paragraph::new(lines).block(panel_block(theme, title, Tone::Inactive)),
        area,
    );
}

fn draw_detail_panel(
    frame: &mut Frame,
    area: Rect,
    app: &ClusterApp,
    theme: Theme,
    active: bool,
    include_logs: bool,
) {
    let mut lines = detail_lines(app, include_logs, theme);
    let trends = if app.mode() == ClusterMode::Detail
        && app
            .selected_host()
            .and_then(|host| app.history_for(host.alias()))
            .is_some_and(|history| history.len() >= 2)
    {
        let mut lines = vec![section_line(theme, "Trends")];
        let budget = if area.height >= 34 {
            14
        } else if area.height >= 23 {
            10
        } else {
            6
        };
        lines.extend(trend_lines(
            app,
            area.width.saturating_sub(2),
            budget,
            theme,
        ));
        lines
    } else {
        vec![]
    };
    if !trends.is_empty() {
        // Each resource's current value and trend share one row. Keep tasks/GPU
        // and system metadata, but do not repeat load/memory/storage elsewhere.
        lines.retain(|line| {
            let text = line.to_string();
            !["CPU load:", "Memory:", "Storage:"]
                .iter()
                .any(|label| text.starts_with(label))
        });
    }
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

/// Load and probe duration can cluster far above zero. Label the visible
/// vertical range, and allow a little headroom so tiny jitter stays small.
fn trend_range(values: &[Option<f64>], metric: usize, zoom: bool) -> (f64, f64) {
    if matches!(metric, 1 | 2) {
        return (0.0, 100.0);
    }
    let (min, max) = values
        .iter()
        .flatten()
        .copied()
        .fold((f64::INFINITY, 0.0_f64), |(low, high), x| {
            (low.min(x), high.max(x))
        });
    if !zoom || !min.is_finite() || min <= max * 0.5 {
        return (0.0, max.max(1.0));
    }
    let min_span = if metric == 0 { 0.5 } else { 10.0 };
    let span = (max - min).max(max * 0.03).max(min_span);
    let step = 10.0_f64.powf((span / 4.0).log10().floor());
    let margin = (span - (max - min)) * 0.5;
    let low = ((min - margin).max(0.0) / step).floor() * step;
    let high = ((max + margin) / step).ceil() * step;
    if low.is_finite() && high.is_finite() && high > low {
        (low, high)
    } else {
        (0.0, max.max(1.0))
    }
}

fn trend_number(value: f64, metric: usize) -> String {
    if metric == 0 {
        format!("{value:.2}")
    } else {
        format!("{value:.0}")
    }
}

fn trend_lines(app: &ClusterApp, width: u16, height: u16, theme: Theme) -> Vec<Line<'static>> {
    let Some(host) = app.selected_host() else {
        return vec![];
    };
    let Some(history) = app.history_for(host.alias()).filter(|h| !h.is_empty()) else {
        return vec![Line::from("Waiting for completed probes")];
    };
    let width = usize::from(width);
    let wide = width >= 44;
    let value_width = if wide { 24 } else { 13 };
    let graph_width = width.saturating_sub(10 + value_width).max(1);
    let count = history.len().min(graph_width);
    let samples = history.range(history.len() - count..).collect::<Vec<_>>();
    let duration = samples[count - 1]
        .at
        .duration_since(samples[0].at)
        .unwrap_or_default()
        .as_secs();
    let caption = format!("{count} samples / {duration}s");
    let header = if wide {
        let lead = 10 + graph_width;
        format!("{:<lead$} {:>7}  Scale", caption, "Latest")
    } else {
        caption
    };
    let mut lines = vec![Line::styled(header, theme.style(Tone::Muted))];
    let plot_rows = usize::from(height.saturating_sub(1));
    let rows = (plot_rows / 4).clamp(1, 4);
    let gutters = plot_rows.saturating_sub(4 * rows).min(3);
    let steps = rows * 8 - 1;
    for (metric, label, unit, tone) in [
        (0, "Load 1m", "", Tone::Accent),
        (1, "Mem used", "%", Tone::AccentAlt),
        (2, "Disk used", "%", Tone::Warn),
        (3, "Probe", "ms", Tone::Ok),
    ] {
        let values = samples
            .iter()
            .map(|sample| sample.values[metric])
            .collect::<Vec<_>>();
        let (low, high) = trend_range(&values, metric, wide);
        let heights = values
            .iter()
            .map(|v| crate::metrics::trend_level(*v, low, high, steps))
            .collect::<Vec<_>>();
        let current = values[count - 1];
        let retained = || {
            app.selected_snapshot().and_then(|snapshot| match metric {
                0 => snapshot
                    .report
                    .cpu_load
                    .as_deref()?
                    .split_whitespace()
                    .next()?
                    .parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite() && *v >= 0.0),
                1 => snapshot.report.memory_used_percent().map(f64::from),
                2 => snapshot
                    .report
                    .storage
                    .as_deref()
                    .and_then(crate::metrics::percent)
                    .map(f64::from),
                _ => values.iter().rev().find_map(|v| *v),
            })
        };
        let latest = current.or_else(retained).map_or_else(
            || "--".into(),
            |v| {
                format!(
                    "{}{unit}{}",
                    trend_number(v, metric),
                    if current.is_none() { " old" } else { "" }
                )
            },
        );
        let scale = if wide {
            let bound = |v: f64| {
                if v == v.trunc() {
                    format!("{v:.0}")
                } else {
                    trend_number(v, metric)
                }
            };
            format!("{}..{}{unit}", bound(low), bound(high))
        } else {
            format!("/{}{unit}", trend_number(high, metric))
        };
        let mut bands = crate::theme::trend_bands(theme, tone, &heights, graph_width, rows);
        for (i, band) in bands.iter_mut().enumerate() {
            let bottom = i + 1 == rows;
            let mut spans = vec![Span::styled(
                format!("{:<10}", if bottom { label } else { "" }),
                theme.style(Tone::Key),
            )];
            spans.append(band);
            if bottom {
                let reading = if wide {
                    format!(" {latest:>7}  ")
                } else {
                    format!(" {latest} ")
                };
                spans.push(Span::styled(reading, theme.style(Tone::Value)));
                spans.push(Span::styled(scale.clone(), theme.style(Tone::Muted)));
            }
            lines.push(Line::from(spans));
        }
        if metric < gutters {
            lines.push(Line::default());
        }
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
        let data = app
            .data_age(host.alias())
            .map(|age| format!("{}s ago; age at draw", age.as_secs()))
            .unwrap_or_else(|| "no valid observation yet".into());
        lines.insert(2.min(lines.len()), kv_line(theme, "Data age", data));
        if let Some(reason) = app.attention_reason(host.alias()) {
            lines.insert(
                3.min(lines.len()),
                Line::from(Span::styled(
                    format!("Attention: {reason}"),
                    theme.fg(palette.warn),
                )),
            );
        }
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
    lines.push(memory_line(theme, report));
    if let Some(index) = report.legacy_darwin_pressure_index() {
        lines.push(kv_line(theme, "Legacy pressure index", format!("{index}%")));
    } else if report.hardware.memory_percent().is_some()
        && !report.hardware.is_macos_ram_estimate()
        && let Some(native) = report.memory.as_deref().filter(|s| *s != "unknown")
    {
        lines.push(kv_line(theme, "Native memory summary", ascii_safe(native)));
    }
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

fn memory_line(theme: Theme, report: &ProbeReport) -> Line<'static> {
    let hw = &report.hardware;
    if let Some(used) = hw.memory_percent() {
        let capacity = format!(
            "{} / {}",
            crate::hardware::bytes_label(hw.memory_used_bytes),
            crate::hardware::bytes_label(hw.memory_total_bytes)
        );
        let suffix = if hw
            .memory_basis
            .as_deref()
            .is_some_and(|basis| basis.starts_with("macOS physical occupied"))
        {
            "occupied incl. cache"
        } else if hw.is_macos_ram_estimate() {
            "est. used"
        } else {
            "used"
        };
        meter_line(theme, "Memory", &capacity, Some((used, suffix)))
    } else if report.legacy_darwin_pressure_index().is_some() {
        meter_line(theme, "Memory", "usage unavailable", None)
    } else {
        resource_line(
            theme,
            "Memory",
            report.memory.as_deref(),
            MetricKind::Memory,
        )
    }
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
    meter_line(theme, label, raw, metric)
}

fn meter_line(
    theme: Theme,
    label: &str,
    raw: &str,
    metric: Option<(u16, &'static str)>,
) -> Line<'static> {
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
    if context == "cluster_topology" {
        let controls = [
            ("open_detail", "detail"),
            ("cancel", "back"),
            ("topology_parent", "parent"),
            ("topology_child", "child"),
            ("down", "next"),
            ("up", "prev"),
            ("open_actions", "actions"),
            ("refresh_all", "refresh"),
            ("open_filter", "filter"),
        ];
        let mut hints = controls
            .iter()
            .filter_map(|(id, name)| crate::bindings::hint(app.keymap(), context, id, name))
            .collect::<Vec<_>>();
        let shown = app.topology_shows_trends();
        if let Some(hint) = crate::bindings::hint(
            app.keymap(),
            context,
            "toggle_topology_trends",
            if shown { "hide trends" } else { "show trends" },
        ) {
            hints.insert(0, hint);
        }
        frame.render_widget(
            Paragraph::new(footer_rows(
                theme,
                &hints.join(" | "),
                area.width,
                area.height.saturating_sub(1) as usize,
            ))
            .block(base_block().borders(Borders::TOP)),
            area,
        );
        return;
    }
    if app.mode() == ClusterMode::Hardware && app.actions().is_none() {
        let mut pieces = vec![
            format!("{} back", cluster_key(app, context, "cancel")),
            format!("{} scroll", cluster_key(app, context, "detail_down")),
            format!("{} device", cluster_key(app, context, "down")),
            "^C".into(),
        ];
        if area.width >= 80 || area.height >= 3 {
            for (action, label) in [
                ("refresh_selected", "refresh"),
                (
                    "toggle_pause",
                    if app.refresh_paused() {
                        "resume"
                    } else {
                        "pause"
                    },
                ),
                ("open_actions", "actions"),
                ("open_detail", "detail"),
            ] {
                if let Some(hint) = crate::bindings::hint(app.keymap(), context, action, label) {
                    pieces.push(hint);
                }
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
        return;
    }
    if area.width < 50 && matches!(context, "cluster" | "cluster_detail") {
        let mut pieces = vec![
            crate::bindings::hint(
                app.keymap(),
                context,
                if context == "cluster" {
                    "quit"
                } else {
                    "cancel"
                },
                if context == "cluster" { "quit" } else { "back" },
            )
            .unwrap_or_default(),
        ];
        if app.selected_host().is_some() {
            pieces.push(
                crate::bindings::hint(app.keymap(), context, "open_workbench", "tersh")
                    .unwrap_or_default(),
            );
        }
        pieces.push(
            crate::bindings::hint(app.keymap(), context, "open_actions", "actions")
                .unwrap_or_default(),
        );
        pieces.push("^C".into());
        if context == "cluster" {
            pieces.extend(crate::bindings::hint(
                app.keymap(),
                context,
                "open_filter",
                "filter",
            ));
            if app.selected_host().is_some() {
                pieces.extend(crate::bindings::hint(
                    app.keymap(),
                    context,
                    "open_detail",
                    "detail",
                ));
            }
        } else {
            pieces.extend(crate::bindings::hint(
                app.keymap(),
                context,
                "detail_down",
                "scroll",
            ));
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
        return;
    }
    let compact = footer_compact(area.width, 64);
    let mut pieces = vec![
        match context {
            "cluster_detail" => "detail",
            "cluster_filter" => "filter",
            other => other,
        }
        .to_owned(),
    ];
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
            add("submit", "apply");
            add("up", "previous match");
            add("down", "next match");
            add("cancel", "cancel");
            add("backspace", "erase");
        }
        "places" => {
            add("cancel", "back");
            add("submit", "open");
            add("pin_place", "pin");
            add("remove_place", "remove");
        }
        _ => {
            if context == "cluster" {
                add("quit", "quit");
                add("open_help", "help");
            } else {
                add("cancel", "back");
            }
            add("open_actions", "actions");
            if context == "cluster" {
                add("open_filter", "filter");
                if !app.filter().is_empty() || app.attention_only() {
                    add("clear_filter", "clear filter");
                }
                add("cycle_sort", "sort");
                add("refresh_selected", "refresh one");
            } else {
                add("detail_down", "scroll");
            }
            add("refresh_all", "refresh");
            if app.refresh_paused() {
                add("toggle_pause", "resume");
            }
            if app.selected_host().is_some() {
                if context == "cluster" {
                    add("open_detail", "detail");
                }
                add("open_hardware", "hardware");
                add("open_workbench", "tersh");
                add("open_session", "shell/ssh");
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
    if context == "cluster" && !app.hosts().is_empty() {
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
            pieces.retain(|piece| piece != &hint);
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

fn fit_column(value: &str, width: usize) -> String {
    let value = truncate_to_width(value, width);
    let pad = width.saturating_sub(UnicodeWidthStr::width(value.as_str()));
    format!("{value}{}", " ".repeat(pad))
}

fn wrap_display(value: &str, width: usize) -> Vec<Line<'static>> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for word in value.split_whitespace() {
        if !line.is_empty() && used + 1 + UnicodeWidthStr::width(word) > width {
            lines.push(Line::from(std::mem::take(&mut line)));
            used = 0;
        }
        if !line.is_empty() {
            line.push(' ');
            used += 1;
        }
        for ch in word.chars() {
            let cells = UnicodeWidthChar::width(ch).unwrap_or(0);
            if used + cells > width && !line.is_empty() {
                lines.push(Line::from(std::mem::take(&mut line)));
                used = 0;
            }
            line.push(ch);
            used += cells;
        }
    }
    if !line.is_empty() {
        lines.push(Line::from(line));
    }
    lines
}

fn ascii_safe(value: &str) -> String {
    if crate::theme::unicode_graphs() {
        crate::fs_core::escape_display(value)
    } else {
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
    fn load_variation_uses_a_labeled_range_while_capacity_preserves_the_zero_baseline() {
        let observations = [Some(399.67), None, Some(415.48)];
        let (low, high) = trend_range(&observations, 0, true);
        assert!((390.0..=399.67).contains(&low) && (415.48..425.0).contains(&high));
        assert_eq!(trend_range(&observations, 0, false), (0.0, 415.48));
        assert_eq!(
            trend_range(&[Some(59.0), Some(60.0)], 1, true),
            (0.0, 100.0)
        );
        let (low, high) = trend_range(&[Some(1500.0), Some(1500.0)], 3, true);
        assert!(low < 1500.0 && high > 1500.0);
    }

    #[test]
    fn wide_bands_preserve_missing_intervals_and_real_sample_heights() {
        let bands = crate::theme::trend_bands(Theme::Mono, Tone::Ok, &[1, 0, 15], 60, 2);
        let text: Vec<Vec<char>> = bands
            .into_iter()
            .map(|b| Line::from(b).to_string().chars().collect())
            .collect();
        assert!(
            text.iter()
                .all(|row| row.len() == 60 && row[20..40].iter().all(|&ch| ch == ' '))
        );
        assert!(text[0][0..20].iter().all(|&ch| ch == ' '));
        assert!(text[1][0..19].iter().any(|&ch| ch != ' '));
        assert!(
            text.iter()
                .all(|row| row[40..60].iter().any(|&ch| ch != ' '))
        );
    }

    #[test]
    fn percent_parser_handles_ascii_and_fullwidth_percent_tokens() {
        assert_eq!(percent_from_token("memory=42% free"), Some(42));
        assert_eq!(percent_from_token("memory=42％ free"), Some(42));
        assert_eq!(percent_from_token("gpu util 99.6%"), Some(100));
    }
}
