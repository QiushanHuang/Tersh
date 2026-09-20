use crate::{
    app::{App, Mode},
    fs_core::{FileKind, display_path, escape_display, format_size},
    theme::{Theme, base_block, chip, footer_compact, footer_height, footer_rows, panel_title},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Borders, Clear, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = Theme::current();
    match app.mode() {
        Mode::Preview | Mode::PreviewSearch => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(if area.width >= 80 && area.height >= 14 {
                        4
                    } else {
                        3
                    }),
                    Constraint::Min(3),
                    Constraint::Length(footer_height(area.width, area.height)),
                ])
                .split(area);
            draw_header(frame, rows[0], app, theme);
            draw_fullscreen_preview(frame, rows[1], app, theme);
            if app.mode() == Mode::PreviewSearch {
                draw_input_modal(frame, command_overlay_rect(area, app.mode()), app, theme);
            }
            draw_footer(frame, rows[2], app, theme);
        }
        _ => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(if area.width >= 80 && area.height >= 14 {
                        4
                    } else {
                        3
                    }),
                    Constraint::Min(3),
                    Constraint::Length(footer_height(area.width, area.height)),
                ])
                .split(area);
            draw_header(frame, rows[0], app, theme);
            draw_body(frame, rows[1], app, theme);
            draw_footer(frame, rows[2], app, theme);
            match app.mode() {
                Mode::Help => draw_help(frame, help_overlay_rect(area), app, theme),
                Mode::Jobs => draw_jobs(frame, rows[1], app, theme),
                Mode::Trash => draw_trash(frame, rows[1], app, theme),
                Mode::Filter
                | Mode::Goto
                | Mode::Rename
                | Mode::CopyTo
                | Mode::MoveTo
                | Mode::ConfirmTrash
                | Mode::ConfirmDelete
                | Mode::ConfirmRestore
                | Mode::Conflict => {
                    draw_input_modal(frame, command_overlay_rect(area, app.mode()), app, theme)
                }
                Mode::Message | Mode::Normal | Mode::Preview | Mode::PreviewSearch => {}
            }
        }
    }
    if let Some(menu) = app.actions() {
        crate::actions::draw(frame, area, menu, theme);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    if area.width < 80 {
        draw_compact_header(frame, area, app, theme);
        return;
    }
    let palette = theme.palette();
    let hidden = if app.show_hidden() { "ON" } else { "OFF" };
    let filter = if app.filter().is_empty() {
        "-".to_string()
    } else {
        escape_display(app.filter())
    };
    let lines = vec![
        Line::from(vec![
            Span::styled(" Tersh ", theme.chip(palette.text, palette.accent)),
            Span::raw(" "),
            Span::styled(
                compact_path(app.cwd(), area.width.saturating_sub(10) as usize),
                theme.fg(palette.path),
            ),
        ]),
        Line::from(vec![
            chip(
                "items",
                app.entries().len(),
                theme.chip(palette.text, palette.ok),
            ),
            Span::raw(" "),
            chip(
                "sel",
                format!(
                    "{} {}",
                    app.selected_len(),
                    format_size(app.selected_total_size())
                ),
                theme.chip(palette.text, palette.accent_alt),
            ),
            Span::raw(" "),
            chip(
                "buf",
                app.copy_buffer_label(),
                theme.chip(palette.text, palette.accent),
            ),
            Span::raw(" "),
            chip("sort", app.sort_label(), theme.fg_bold(palette.path)),
            chip("hidden", hidden, theme.fg(palette.muted)),
            chip(
                "filter",
                truncate_display_width(&filter, 16),
                theme.fg(palette.warn),
            ),
        ]),
    ];
    let paragraph = Paragraph::new(lines).block(
        base_block()
            .borders(Borders::ALL)
            .title(panel_title(theme, job_banner(app))),
    );
    frame.render_widget(paragraph, area);
}

fn draw_compact_header(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let palette = theme.palette();
    let line = Line::from(vec![
        chip(
            "sel",
            app.selected_len(),
            theme.chip(palette.text, palette.accent_alt),
        ),
        Span::raw(" "),
        chip(
            "buf",
            app.copy_buffer_label(),
            theme.chip(palette.text, palette.accent),
        ),
        Span::raw(" "),
        chip("items", app.entries().len(), theme.fg(palette.muted)),
    ]);
    frame.render_widget(
        Paragraph::new(line).block(base_block().borders(Borders::ALL).title(panel_title(
            theme,
            format!(
                "Tersh | {}",
                if app.job_progress().is_some() {
                    job_banner(app)
                } else {
                    compact_path(app.cwd(), area.width.saturating_sub(12) as usize)
                }
            ),
        ))),
        area,
    );
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    if area.width >= 120 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(50),
                Constraint::Percentage(18),
            ])
            .split(area);
        draw_files(frame, columns[0], app, theme);
        draw_preview(frame, columns[1], app, theme);
        draw_info(frame, columns[2], app, theme);
    } else if area.width >= 80 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_files(frame, columns[0], app, theme);
        draw_preview(frame, columns[1], app, theme);
    } else {
        let rows = if area.height >= 7 && area.width >= 60 {
            Some(
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Min(3), Constraint::Length(3)])
                    .split(area),
            )
        } else {
            None
        };
        if let Some(rows) = rows {
            draw_files(frame, rows[0], app, theme);
            draw_compact_info(frame, rows[1], app, theme);
        } else {
            draw_files(frame, area, app, theme);
        }
    }
}

fn draw_files(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let palette = theme.palette();
    let compact = area.width < 55;
    let mut lines = vec![Line::from(Span::styled(
        if compact && area.width >= 44 {
            "CSB    SIZE NAME"
        } else if compact {
            "CSB NAME"
        } else {
            "CSB K    PERM      SIZE NAME"
        },
        theme.fg_bold(palette.key),
    ))];
    if !app.filter().is_empty() {
        lines.push(kv_line(theme, "filter", escape_display(app.filter())));
    }
    let entry_capacity = area
        .height
        .saturating_sub(2)
        .saturating_sub(lines.len() as u16) as usize;
    let visible_start = visible_entry_start(app.cursor(), app.entries().len(), entry_capacity);
    for (index, entry) in app
        .entries()
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(entry_capacity)
    {
        let cursor = if index == app.cursor() { ">" } else { " " };
        let mark = if app.is_selected(&entry.path) {
            "*"
        } else {
            " "
        };
        let buffer_mark = app.transfer_marker_for(&entry.path);
        let suffix = match entry.kind {
            crate::fs_core::FileKind::Directory => "/",
            crate::fs_core::FileKind::Symlink => "@",
            _ => "",
        };
        let perm = if entry.readonly { "RO" } else { "RW" };
        let prefix = if compact && area.width >= 44 {
            format!(
                "{cursor}{mark}{buffer_mark} {:>7} ",
                format_size(entry.size)
            )
        } else if compact {
            format!("{cursor}{mark}{buffer_mark} ")
        } else {
            format!(
                "{cursor}{mark}{buffer_mark} {:<4} {:<4} {:>8} ",
                kind_icon(entry.kind),
                perm,
                format_size(entry.size)
            )
        };
        let inner_width = area.width.saturating_sub(2) as usize;
        let name_width = inner_width
            .saturating_sub(display_width(&prefix))
            .saturating_sub(display_width(suffix));
        let row = format!(
            "{prefix}{}{}",
            truncate_display_width(&entry.name, name_width),
            suffix
        );
        if index == app.cursor() {
            lines.push(Line::from(Span::styled(
                format!(
                    "{row}{}",
                    " ".repeat(inner_width.saturating_sub(display_width(&row)))
                ),
                theme.selected(),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                row,
                file_row_style(theme, entry.kind, entry.readonly, buffer_mark),
            )));
        }
    }
    if app.entries().is_empty() {
        lines.push(Line::from(if app.filter().is_empty() {
            "Empty directory"
        } else {
            "No matching files"
        }));
        lines.push(Line::from(Span::styled(
            format!(
                "{} filter | {} refresh | {} parent",
                action_key(app, "files", "open_filter"),
                action_key(app, "files", "refresh"),
                action_key(app, "files", "parent")
            ),
            theme.fg(palette.muted),
        )));
    }
    let title = if app.entries().is_empty() {
        format!(
            "Files | {} actions | sort {}",
            action_key(app, "files", "open_actions"),
            app.sort_label()
        )
    } else {
        format!(
            "Files {}/{} | {} actions | sort {}",
            app.cursor().saturating_add(1),
            app.entries().len(),
            action_key(app, "files", "open_actions"),
            app.sort_label()
        )
    };
    let paragraph = Paragraph::new(lines).block(
        base_block()
            .title(panel_title(theme, title))
            .border_style(theme.fg(palette.accent))
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, area);
}

fn draw_fullscreen_preview(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let palette = theme.palette();
    let lines = app.preview().lines.iter().collect::<Vec<_>>();
    let total_lines = lines.len();
    let view_lines = area.height.saturating_sub(4) as usize;
    let offset = app.preview_offset().min(total_lines.saturating_sub(1));
    let query = app.preview_search_query().to_lowercase();
    let matches = app.preview_matches();
    let active_match = app.preview_active_match();
    let active_line = active_match
        .and_then(|index| matches.get(index))
        .copied()
        .unwrap_or(usize::MAX);

    let mut rendered = Vec::new();
    rendered.push(Line::from(Span::styled(
        display_path(&app.preview().path),
        theme.fg_bold(palette.path),
    )));
    let match_text = match active_match {
        Some(index) => format!("{}/{}", index + 1, matches.len()),
        None if matches.is_empty() && app.preview_search_query().is_empty() => "-".to_string(),
        None => format!("0/{}", matches.len()),
    };
    rendered.push(Line::from(vec![
        Span::styled("offset: ", theme.fg(palette.key)),
        Span::styled(
            format!("{} / {}", offset.saturating_add(1), total_lines),
            theme.fg(palette.value),
        ),
        Span::styled(" | ", theme.fg(palette.separator)),
        Span::styled("match: ", theme.fg(palette.key)),
        Span::styled(match_text, theme.fg(palette.value)),
        Span::styled(" | ", theme.fg(palette.separator)),
        Span::styled("search: ", theme.fg(palette.key)),
        Span::styled(
            escape_display(app.preview_search_query()),
            theme.fg(palette.value),
        ),
        if app.preview().truncated {
            Span::styled(" | truncated", theme.fg_bold(palette.warn))
        } else {
            Span::raw("")
        },
    ]));
    for (index, line) in lines.iter().enumerate().skip(offset).take(view_lines) {
        let mut style = Style::default();
        if !query.is_empty() && line.to_lowercase().contains(&query) {
            style = theme.chip(palette.selected_fg, palette.search_match);
        }
        if index == active_line {
            style = style
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::BOLD);
        }
        rendered.push(Line::from(Span::styled(line.as_str(), style)));
    }

    let paragraph = Paragraph::new(rendered)
        .block(
            base_block()
                .title(panel_title(theme, "Preview"))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn display_width(value: &str) -> usize {
    UnicodeWidthStr::width(value)
}

fn truncate_display_width(value: &str, max_width: usize) -> String {
    if display_width(value) <= max_width {
        return value.to_string();
    }
    if max_width <= 3 {
        return ".".repeat(max_width);
    }
    let content_width = max_width - 3;
    let mut used = 0;
    let mut truncated = String::new();
    for ch in value.chars() {
        let width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + width > content_width {
            break;
        }
        truncated.push(ch);
        used += width;
    }
    truncated.push_str("...");
    truncated
}

fn compact_path(path: &std::path::Path, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let full = display_path(path);
    if display_width(&full) <= max_width {
        return full;
    }
    let Some(name) = path.file_name() else {
        return truncate_display_width(&full, max_width);
    };
    let compact = format!(".../{}", escape_display(&name.to_string_lossy()));
    truncate_display_width(&compact, max_width)
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let mut lines = vec![Line::from(Span::styled(
        display_path(&app.preview().path),
        theme.fg(theme.palette().path),
    ))];
    lines.extend(app.preview().lines.iter().cloned().map(Line::from));
    let paragraph = Paragraph::new(lines)
        .block(
            base_block()
                .title(panel_title(theme, "Preview"))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_info(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let palette = theme.palette();
    let focused = app.entries().get(app.cursor());
    let mut lines = Vec::new();
    lines.push(section_line(theme, "TARGET"));
    if let Some(entry) = focused {
        lines.push(kv_line(theme, "kind", entry.kind_marker()));
        lines.push(kv_line(theme, "size", format_size(entry.size)));
        lines.push(kv_line(
            theme,
            "perm",
            if entry.readonly {
                "readonly"
            } else {
                "writable"
            },
        ));
        if let Some(target) = &entry.symlink_target {
            lines.push(Line::from(vec![
                Span::styled("-> ", theme.fg(palette.key)),
                Span::styled(display_path(target), theme.fg(palette.path)),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "no item",
            theme.fg(palette.inactive),
        )));
    }
    lines.push(Line::from(""));
    lines.push(section_line(theme, "BUFFER"));
    let buffer_label = app.copy_buffer_label();
    lines.push(Line::from(Span::styled(
        buffer_label.clone(),
        buffer_style(theme, &buffer_label),
    )));
    lines.push(kv_line(theme, "selected", app.selected_len().to_string()));
    lines.push(Line::from(""));
    lines.push(section_line(theme, "SEARCH"));
    lines.push(kv_line(
        theme,
        "filter",
        if app.filter().is_empty() {
            "-".to_string()
        } else {
            escape_display(app.filter())
        },
    ));
    lines.push(kv_line(theme, "sort", app.sort_label()));
    lines.push(Line::from(""));
    lines.push(section_line(theme, "LOG"));
    for log in app.logs().iter().rev().take(5) {
        lines.push(Line::from(Span::styled(
            escape_display(log),
            log_style(theme, log),
        )));
    }
    let paragraph = Paragraph::new(lines)
        .block(
            base_block()
                .title(panel_title(theme, "Inspector"))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_compact_info(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let focused = app.entries().get(app.cursor());
    let target = focused
        .map(|entry| format!("{} {}", entry.kind_marker(), entry.name))
        .unwrap_or_else(|| "no item".to_string());
    let text = format!(
        "{target} | selected {} | copy {}",
        app.selected_len(),
        app.copy_buffer_len()
    );
    let paragraph = Paragraph::new(truncate_display_width(
        &text,
        area.width.saturating_sub(2) as usize,
    ))
    .style(theme.fg(theme.palette().muted))
    .block(
        base_block()
            .title(panel_title(theme, "Status"))
            .borders(Borders::ALL),
    );
    frame.render_widget(paragraph, area);
}

fn action_key(app: &App, context: &str, action: &str) -> String {
    crate::bindings::label(app.keymap(), context, action).unwrap_or_else(|| "unbound".into())
}

fn job_banner(app: &App) -> String {
    let Some(progress) = app.job_progress() else {
        return String::new();
    };
    let status = if app.job_active() {
        if progress.cancelling {
            "cancelling"
        } else {
            "running"
        }
    } else if app.last_job().is_some_and(|r| r.cancelled) {
        "cancelled"
    } else {
        "finished"
    };
    format!(
        "{} {} {}/{} {} | {} jobs",
        progress.label,
        status,
        progress.completed,
        progress.total,
        format_size(progress.copied_bytes),
        action_key(app, "files", "open_jobs")
    )
}

fn draw_jobs(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(Clear, area);
    let mut lines = vec![Line::from(job_banner(app))];
    if let Some(progress) = app.job_progress() {
        lines.push(kv_line(
            theme,
            "Processed roots",
            format!("{} / {}", progress.completed, progress.total),
        ));
        lines.push(kv_line(theme, "Copied", format_size(progress.copied_bytes)));
        if let Some(path) = &progress.current_path {
            lines.push(kv_line(theme, "Current", display_path(path)));
        }
    } else {
        lines.push(Line::from("No file jobs yet"));
    }
    if app.exit_after_job() {
        lines.push(Line::from("Exiting after worker cancellation and cleanup"));
    }
    lines.push(Line::from(
        "Cancellation retains completed items; deletion is not undoable.",
    ));
    if let Some(result) = app.last_job() {
        lines.push(kv_line(
            theme,
            "Result",
            format!(
                "{} done / {} failed / {} skipped / {} remaining",
                result.succeeded.len(),
                result.failed.len(),
                result.skipped.len(),
                result.unprocessed.len()
            ),
        ));
        for failure in result.failed.iter().take(100) {
            lines.push(Line::from(Span::styled(
                format!(
                    "FAILED {}: {}",
                    display_path(&failure.path),
                    escape_display(&failure.error)
                ),
                theme.danger(),
            )));
        }
        for path in result.skipped.iter().take(100) {
            lines.push(Line::from(format!("SKIPPED {}", display_path(path))));
        }
        for path in result.unprocessed.iter().take(100) {
            lines.push(Line::from(format!("REMAINING {}", display_path(path))));
        }
    }
    let max_offset = lines
        .len()
        .saturating_sub(area.height.saturating_sub(2) as usize);
    frame.render_widget(
        Paragraph::new(lines)
            .scroll((
                app.help_offset().min(max_offset).min(u16::MAX as usize) as u16,
                0,
            ))
            .block(
                base_block()
                    .borders(Borders::ALL)
                    .title(panel_title(theme, "File jobs")),
            ),
        area,
    );
}

fn draw_trash(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(Clear, area);
    let mut lines = Vec::new();
    if let Some(error) = app.trash_error() {
        lines.push(Line::from(Span::styled(
            escape_display(error),
            theme.danger(),
        )));
    }
    if app.trash_entries().is_empty() {
        lines.push(Line::from("No restorable receipts in this work root"));
        lines.push(Line::from(
            "Legacy trash has no recorded original location.",
        ));
    }
    let capacity = area.height.saturating_sub(2 + lines.len() as u16) as usize;
    let start = app
        .trash_cursor()
        .saturating_add(1)
        .saturating_sub(capacity);
    for (index, entry) in app
        .trash_entries()
        .iter()
        .enumerate()
        .skip(start)
        .take(capacity)
    {
        let text = format!(
            "{} {}",
            if index == app.trash_cursor() {
                ">"
            } else {
                " "
            },
            compact_path(&entry.original_path, area.width.saturating_sub(4) as usize)
        );
        lines.push(Line::from(Span::styled(
            text,
            if index == app.trash_cursor() {
                theme.selected()
            } else {
                theme.fg(theme.palette().path)
            },
        )));
    }
    frame.render_widget(
        Paragraph::new(lines).block(base_block().borders(Borders::ALL).title(panel_title(
            theme,
            format!("Trash recovery | {} receipts", app.trash_entries().len()),
        ))),
        area,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let context = app.key_context();
    let compact = footer_compact(area.width, 60);
    let mode = if app.actions().is_some() {
        "actions".into()
    } else if app.pending_y() {
        "y_".into()
    } else if app.pending_g() {
        "g_".into()
    } else if app.mode() == Mode::PreviewSearch {
        "find".into()
    } else if app.mode() == Mode::ConfirmRestore {
        "restore".into()
    } else {
        format!("{:?}", app.mode()).to_lowercase()
    };
    let mut pieces = vec![mode];
    let mut add = |action: &str, description: &str| {
        if action == "cancel" {
            pieces.push(format!(
                "{}{}",
                crate::bindings::cancel_label(app.keymap(), context, compact),
                if compact {
                    String::new()
                } else {
                    format!(" {description}")
                }
            ));
        } else if let Some(hint) = crate::bindings::hint(app.keymap(), context, action, description)
        {
            pieces.push(if context == "files" && action == "open" {
                format!("next: {hint}")
            } else {
                hint
            });
        }
    };
    match context {
        "files" => {
            add("quit", "quit");
            add("open_help", "help");
            add("open_actions", "actions");
            add("cancel", "clear");
            add("open_filter", "filter");
            if app.pending_y() {
                add("copy", "copy");
                add("copy_name", "name");
                add("copy_relative_path", "rel");
                add("copy_absolute_path", "abs");
            } else if app.pending_g() {
                add("first", "top");
                add("last", "bottom");
            } else {
                if app.copy_buffer_len() > 0 {
                    add("paste", "paste");
                }
                add(
                    "open",
                    match app.entries().get(app.cursor()).map(|e| e.kind) {
                        Some(FileKind::Directory) => "open dir",
                        Some(FileKind::File) => "preview file",
                        _ => "inspect",
                    },
                );
                if app
                    .entries()
                    .get(app.cursor())
                    .is_some_and(|e| e.kind == FileKind::File)
                {
                    add("edit", "edit");
                }
                add("toggle_select", "mark");
                add("copy", "copy");
                add("cycle_sort", "sort");
                add("open_jobs", "jobs");
                add("open_trash", "trash recovery");
                if app.job_active() {
                    add("cancel_job", "cancel job");
                }
                if !compact {
                    add("down", "down");
                    add("up", "up");
                    add("parent", "parent");
                }
            }
        }
        "preview" => {
            add("cancel", "close");
            add("open_actions", "actions");
            add("half_down", "page down");
            add("half_up", "page up");
            add("open_preview_search", "find");
            add("preview_search_next", "next");
            add("edit", "edit");
            add("open_jobs", "jobs");
            if app.job_active() {
                add("cancel_job", "cancel job");
            }
        }
        "help" => {
            add("cancel", "close");
            add("down", "scroll");
            add("up", "up");
        }
        "actions" => {
            add("cancel", "cancel");
            add("submit", "run");
            add("down", "next");
            add("up", "previous");
        }
        "trash" => {
            add("cancel", "back");
            if !app.trash_entries().is_empty() {
                add("restore", "restore");
            }
            add("refresh", "refresh");
            add("down", "down");
            add("up", "up");
        }
        "jobs" => {
            add("cancel", "back");
            if app.job_active() {
                add("cancel_job", "cancel job");
            }
            add("down", "scroll");
        }
        _ => {
            add("cancel", "cancel");
            add(
                "submit",
                match app.mode() {
                    Mode::Filter => "apply",
                    Mode::PreviewSearch => "find",
                    Mode::ConfirmRestore => "restore",
                    _ => "confirm",
                },
            );
            add("backspace", "erase");
        }
    }
    // Emergency exit is deliberately immutable in the validated keymap.
    pieces.insert(
        1,
        if compact {
            "^C".into()
        } else if app.job_active() {
            "^C exit safely".into()
        } else {
            "^C force".into()
        },
    );
    let text = pieces.join(" | ");
    frame.render_widget(
        Paragraph::new(footer_rows(
            theme,
            &text,
            area.width,
            area.height.saturating_sub(1) as usize,
        ))
        .block(base_block().borders(Borders::TOP)),
        area,
    );
}

fn draw_help(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(Clear, area);
    let lines = crate::bindings::help_lines(app.keymap(), app.help_context())
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    let paragraph = Paragraph::new(lines)
        .scroll((app.help_offset().min(u16::MAX as usize) as u16, 0))
        .block(
            base_block()
                .title(panel_title(
                    theme,
                    format!(
                        "Help: {} | {} close",
                        app.help_context(),
                        action_key(app, "help", "cancel")
                    ),
                ))
                .borders(Borders::ALL)
                .border_style(theme.fg(theme.palette().panel_title)),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_input_modal(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(Clear, area);
    let palette = theme.palette();
    let prompt = match app.mode() {
        Mode::Filter => "Filter current directory",
        Mode::Goto => "Go to directory",
        Mode::Rename => "Rename focused item",
        Mode::CopyTo => "Copy selected/focused item(s) to directory",
        Mode::MoveTo => "Move selected/focused item(s) to directory",
        Mode::ConfirmTrash => "Move to .tersh-trash: type trash then Enter. Esc/Ctrl+G cancels.",
        Mode::ConfirmDelete => "Permanent delete: type delete then Enter. Esc/Ctrl+G cancels.",
        Mode::Conflict => "Destination already exists",
        Mode::ConfirmRestore => {
            "Restore to original location; existing files are never overwritten."
        }
        Mode::PreviewSearch => "Find in preview",
        _ => "",
    };
    let prompt = prompt
        .replace(
            "Esc/Ctrl+G",
            &crate::bindings::cancel_label(app.keymap(), "input", false),
        )
        .replace("Enter", &action_key(app, "input", "submit"));
    let mut lines = vec![Line::from(prompt)];
    if app.mode() == Mode::ConfirmRestore
        && let Some(path) = app.restore_target()
    {
        lines.push(Line::from(format!(
            "Name: {}",
            path.file_name()
                .map(|name| escape_display(&name.to_string_lossy()))
                .unwrap_or_default()
        )));
        lines.push(Line::from(format!("Original: {}", display_path(path))));
        lines.push(Line::from(format!(
            "{} restore | {} cancel",
            action_key(app, "input", "submit"),
            action_key(app, "input", "cancel")
        )));
    }
    if matches!(app.mode(), Mode::ConfirmTrash | Mode::ConfirmDelete) {
        let required = match app.mode() {
            Mode::ConfirmTrash => "trash",
            Mode::ConfirmDelete => "delete",
            _ => "",
        };
        lines.push(Line::from(format!("required: {required}")));
        lines.push(Line::from(format!(
            "typed: {}",
            if app.input().is_empty() {
                "-"
            } else {
                app.input()
            }
        )));
        lines.push(Line::from(format!(
            "targets: {} {}",
            app.operation_target_count(),
            app.operation_target_source()
        )));
        if let Some(path) = app.operation_target_first() {
            lines.push(Line::from(format!("first: {}", display_path(&path))));
        }
        for (index, label) in app.operation_target_labels(3).into_iter().enumerate() {
            lines.push(Line::from(format!("{}. {}", index + 1, label)));
        }
    }
    if app.mode() == Mode::Conflict {
        lines.push(Line::from(format!("conflicts: {}", app.conflict_count())));
        if let Some(destination) = app.conflict_destination() {
            lines.push(Line::from(format!(
                "destination: {}",
                display_path(destination)
            )));
        }
        lines.push(Line::from("type skip to keep existing targets"));
        if app.conflict_allows_replace() {
            lines.push(Line::from("type replace to overwrite existing targets"));
        } else {
            lines.push(Line::from("replace unavailable for move/cut operations"));
        }
        for (index, label) in app.conflict_target_labels(3).into_iter().enumerate() {
            lines.push(Line::from(format!("{}. {}", index + 1, label)));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(escape_display(app.input())));
    let title = match app.mode() {
        Mode::ConfirmDelete => "DANGER",
        Mode::ConfirmTrash => "TRASH",
        Mode::Conflict => "CONFLICT",
        Mode::ConfirmRestore => "Restore confirmation",
        _ => "Command",
    };
    let block = match app.mode() {
        Mode::ConfirmDelete => base_block()
            .title(panel_title(theme, title))
            .borders(Borders::ALL)
            .border_style(theme.border_danger())
            .style(theme.danger()),
        Mode::ConfirmTrash => base_block()
            .title(panel_title(theme, title))
            .borders(Borders::ALL)
            .border_style(theme.fg(palette.warn)),
        Mode::Conflict => base_block()
            .title(panel_title(theme, title))
            .borders(Borders::ALL)
            .border_style(theme.fg(palette.warn)),
        _ => base_block()
            .title(panel_title(theme, title))
            .borders(Borders::ALL),
    };
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn section_line(theme: Theme, label: &'static str) -> Line<'static> {
    Line::from(Span::styled(
        label,
        theme.fg_bold(theme.palette().panel_title),
    ))
}

fn kv_line(theme: Theme, key: &'static str, value: impl Into<String>) -> Line<'static> {
    let palette = theme.palette();
    Line::from(vec![
        Span::styled(format!("{key}: "), theme.fg(palette.key)),
        Span::styled(value.into(), theme.fg(palette.value)),
    ])
}

fn buffer_style(theme: Theme, label: &str) -> Style {
    let palette = theme.palette();
    if label.starts_with("COPY") {
        theme.fg_bold(palette.copy)
    } else if label.starts_with("CUT") {
        theme.fg_bold(palette.cut)
    } else {
        theme.fg(palette.inactive)
    }
}

fn file_row_style(
    theme: Theme,
    kind: crate::fs_core::FileKind,
    readonly: bool,
    buffer_mark: &str,
) -> Style {
    let palette = theme.palette();
    if buffer_mark == "C" {
        return theme.fg_bold(palette.copy);
    }
    if buffer_mark == "X" {
        return theme.fg_bold(palette.cut);
    }
    if readonly {
        return theme.fg(palette.warn);
    }
    match kind {
        crate::fs_core::FileKind::Directory => theme.fg(palette.accent),
        crate::fs_core::FileKind::File => theme.fg(palette.value),
        crate::fs_core::FileKind::Symlink => theme.fg(palette.accent_alt),
        crate::fs_core::FileKind::Other => theme.fg(palette.inactive),
    }
}

fn log_style(theme: Theme, log: &str) -> Style {
    let palette = theme.palette();
    let lower = log.to_ascii_lowercase();
    if lower.contains("error") || lower.contains("failed") || lower.contains("rejected") {
        theme.fg(palette.danger)
    } else if lower.contains("skipped") || lower.contains("conflict") {
        theme.fg(palette.warn)
    } else if lower.contains("copied")
        || lower.contains("moved")
        || lower.contains("renamed")
        || lower.contains("trashed")
        || lower.contains("pasted")
    {
        theme.fg(palette.ok)
    } else {
        theme.fg(palette.muted)
    }
}

fn kind_icon(kind: crate::fs_core::FileKind) -> &'static str {
    match kind {
        crate::fs_core::FileKind::Directory => "D",
        crate::fs_core::FileKind::File => "F",
        crate::fs_core::FileKind::Symlink => "L",
        crate::fs_core::FileKind::Other => "O",
    }
}

fn visible_entry_start(cursor: usize, total_entries: usize, entry_capacity: usize) -> usize {
    if entry_capacity == 0 || total_entries <= entry_capacity {
        return 0;
    }
    cursor
        .saturating_add(1)
        .saturating_sub(entry_capacity)
        .min(total_entries.saturating_sub(entry_capacity))
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

fn help_overlay_rect(area: Rect) -> Rect {
    if area.width < 60 || area.height < 14 {
        area
    } else {
        centered_rect(70, 70, area)
    }
}

fn command_overlay_rect(area: Rect, mode: Mode) -> Rect {
    if mode == Mode::ConfirmRestore {
        return if area.width < 60 || area.height < 14 {
            area
        } else {
            centered_rect(90, 70, area)
        };
    }
    if matches!(
        mode,
        Mode::ConfirmTrash | Mode::ConfirmDelete | Mode::Conflict
    ) && (area.width < 60 || area.height < 14)
    {
        area
    } else {
        centered_rect(70, 30, area)
    }
}
