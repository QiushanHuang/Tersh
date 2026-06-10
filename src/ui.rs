use crate::{
    app::{App, Mode},
    fs_core::{FileKind, display_path, escape_display, format_size},
    theme::{
        Theme, Tone, base_block, chip, footer_compact, footer_paragraph, kv_line, modal_block,
        panel_block, section_line,
    },
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
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(2),
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
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(2),
                ])
                .split(area);
            draw_header(frame, rows[0], app, theme);
            draw_body(frame, rows[1], app, theme);
            draw_footer(frame, rows[2], app, theme);
            match app.mode() {
                Mode::Help => draw_help(frame, help_overlay_rect(area), theme),
                Mode::Filter
                | Mode::Goto
                | Mode::Rename
                | Mode::CopyTo
                | Mode::MoveTo
                | Mode::ConfirmTrash
                | Mode::ConfirmDelete
                | Mode::Conflict => {
                    draw_input_modal(frame, command_overlay_rect(area, app.mode()), app, theme)
                }
                Mode::Message | Mode::Normal | Mode::Preview | Mode::PreviewSearch => {}
            }
        }
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
    let lines = vec![Line::from(vec![
        Span::styled("Tersh", theme.fg_bold(palette.panel_title)),
        Span::raw(" | "),
        Span::styled(display_path(app.cwd()), theme.fg(palette.path)),
        Span::raw(" | "),
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
        chip("hidden", hidden, theme.chip(palette.text, palette.muted)),
        Span::raw(" "),
        chip("filter", filter, theme.chip(palette.text, palette.warn)),
        Span::raw(" "),
        chip(
            "sort",
            app.sort_label(),
            theme.chip(palette.text, palette.path),
        ),
    ])];
    let paragraph = Paragraph::new(lines).block(base_block().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_compact_header(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let filter = if app.filter().is_empty() { "-" } else { "*" };
    let text = format!(
        "Tersh | sel {} | buf {} | f {} | {}",
        app.selected_len(),
        app.copy_buffer_label(),
        filter,
        compact_path(app.cwd(), area.width.saturating_sub(34) as usize)
    );
    let paragraph = Paragraph::new(truncate_display_width(
        &text,
        area.width.saturating_sub(2) as usize,
    ))
    .style(theme.fg(theme.palette().muted))
    .block(base_block().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
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
    let mut lines = vec![Line::from(Span::styled(
        "CSB K    PERM      SIZE NAME",
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
        let prefix = format!(
            "{cursor}{mark}{buffer_mark} {:<4} {:<4} {:>8} ",
            kind_icon(entry.kind),
            perm,
            format_size(entry.size)
        );
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
            lines.push(Line::from(Span::styled(row, theme.selected())));
        } else {
            lines.push(Line::from(Span::styled(
                row,
                file_row_style(theme, entry.kind, entry.readonly, buffer_mark),
            )));
        }
    }
    let title = if app.entries().is_empty() {
        format!("Files | sort {}", app.sort_label())
    } else {
        format!(
            "Files {}/{} | sort {}",
            app.cursor().saturating_add(1),
            app.entries().len(),
            app.sort_label()
        )
    };
    let paragraph = Paragraph::new(lines).block(panel_block(theme, title, Tone::Active));
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
        .block(panel_block(theme, "Preview", Tone::Active))
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
        .block(panel_block(theme, "Preview", Tone::Inactive))
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
        .block(panel_block(theme, "Inspector", Tone::Inactive))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_compact_info(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let focused = app.entries().get(app.cursor());
    let target = focused
        .map(|entry| format!("{} {}", entry.kind_marker(), entry.name))
        .unwrap_or_else(|| "no item".to_string());
    let text = format!(
        "{target} | selected {} | buf {}",
        app.selected_len(),
        app.copy_buffer_label()
    );
    let paragraph = Paragraph::new(truncate_display_width(
        &text,
        area.width.saturating_sub(2) as usize,
    ))
    .style(theme.fg(theme.palette().muted))
    .block(panel_block(theme, "Status", Tone::Inactive));
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    let mode = format!("{:?}", app.mode()).to_lowercase();
    let compact = footer_compact(area.width, 60);
    let text = match app.mode() {
        Mode::Normal if app.pending_y() => {
            "y_ | y copy | f name | r rel | a abs | ^G cancel | ^C force".to_string()
        }
        Mode::Normal if app.pending_g() => "g_ | g top | G bottom | ^G cancel | ^C force".to_string(),
        Mode::Normal if compact && area.width < 50 => "q quit | ? help | / | ^G | ^C".to_string(),
        Mode::Normal if compact => {
            format!("next: {} | q quit | ? help | / | ^G | ^C", next_action(app))
        }
        Mode::Normal => normal_footer(app),
        Mode::Preview if compact => "preview | q | Pg | ^G | ^C".to_string(),
        Mode::Preview => {
            "preview | q close | ^G close | ^C force | j/k page | Up/Down line | PgUp/PgDn | gg/G | / find | n/N | e edit".to_string()
        }
        Mode::PreviewSearch if compact => "find | Enter | ^G | ^C".to_string(),
        Mode::PreviewSearch => format!("{mode} | Enter find | Backspace | Esc/^G cancel | ^C force"),
        Mode::Filter if compact => "filter | Enter | ^G | ^C".to_string(),
        Mode::Filter => "filter | type to narrow | Enter apply | Backspace | Esc/^G cancel | ^C force".to_string(),
        Mode::Goto if compact => "goto | Enter | ^G | ^C".to_string(),
        Mode::Goto => "goto | type directory | Enter go | Backspace | Esc/^G cancel | ^C force".to_string(),
        Mode::Rename if compact => "rename | Enter | ^G | ^C".to_string(),
        Mode::Rename => "rename | type name | Enter rename | Backspace | Esc/^G cancel | ^C force".to_string(),
        Mode::CopyTo if compact => "copy-to | Enter | ^G | ^C".to_string(),
        Mode::CopyTo => "copy-to | type destination | Enter copy | Backspace | Esc/^G cancel | ^C force".to_string(),
        Mode::MoveTo if compact => "move-to | Enter | ^G | ^C".to_string(),
        Mode::MoveTo => "move-to | type destination | Enter move | Backspace | Esc/^G cancel | ^C force".to_string(),
        Mode::ConfirmTrash if compact => "trash | Enter | ^G | ^C".to_string(),
        Mode::ConfirmTrash => "trash | type trash | Enter confirm | Esc/^G cancel | ^C force".to_string(),
        Mode::ConfirmDelete if compact => "delete | Enter | ^G | ^C".to_string(),
        Mode::ConfirmDelete => {
            "delete | type delete | Enter confirm | Esc/^G cancel | ^C force".to_string()
        }
        Mode::Conflict if compact => "conflict | replace/skip | ^G | ^C".to_string(),
        Mode::Conflict => {
            "conflict | type replace overwrite | type skip keep existing | Enter confirm | Esc/^G cancel | ^C force".to_string()
        }
        Mode::Help => "help | q/?/Enter close | ^G close | ^C force".to_string(),
        Mode::Message => format!("{mode} | Esc/^G close | ^C force"),
    };
    let paragraph = footer_paragraph(theme, &text);
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect, theme: Theme) {
    frame.render_widget(Clear, area);
    let lines = if area.width < 60 || area.height < 14 {
        vec![
            Line::from("Move: j/k arrows PgUp/PgDn"),
            Line::from("Open: h parent, l/Enter preview"),
            Line::from("Find: / filter, . hidden, r refresh"),
            Line::from("Ops: Space, y..., x cut, p paste"),
            Line::from("More: c copy-to, m move-to, n rename"),
            Line::from("Delete: d trash, D delete"),
            Line::from("Preview: / find, n/N, e edit"),
            Line::from("Exit: q, Esc/^G, ^C"),
        ]
    } else {
        vec![
            Line::from("Navigation"),
            Line::from("  j/k or arrows: move    PageUp/PageDown: page    Home/End: first/last"),
            Line::from("  h: parent    l/Enter: open directory or preview file"),
            Line::from("  /: filter    .: hidden    r: refresh    s/S: sort/reverse"),
            Line::from(""),
            Line::from("Operations"),
            Line::from("  Space: mark    yy: copy    x: cut    p: paste"),
            Line::from("  c: copy to...    m: move to...    n: rename"),
            Line::from("  yf: file name    yr: relative path    ya: absolute path"),
            Line::from("  e: edit focused regular file with $VISUAL/$EDITOR/nano"),
            Line::from("  :: goto directory"),
            Line::from("  d: trash, then type trash    D: delete, then type delete"),
            Line::from(""),
            Line::from("Preview mode"),
            Line::from("  j/k/PageUp/PageDown: page    arrows/Ctrl+F/Ctrl+B: line"),
            Line::from("  gg: top    G: bottom"),
            Line::from("  /: search    n/N: next/prev match"),
            Line::from(""),
            Line::from("Exit"),
            Line::from("  q: quit/close    Esc/Ctrl+G: cancel    Q/Ctrl+C: force quit"),
        ]
    };
    let paragraph = Paragraph::new(lines)
        .block(panel_block(theme, "Help", Tone::Active))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_input_modal(frame: &mut Frame, area: Rect, app: &App, theme: Theme) {
    frame.render_widget(Clear, area);
    let prompt = match app.mode() {
        Mode::Filter => "Filter current directory",
        Mode::Goto => "Go to directory",
        Mode::Rename => "Rename focused item",
        Mode::CopyTo => "Copy selected/focused item(s) to directory",
        Mode::MoveTo => "Move selected/focused item(s) to directory",
        Mode::ConfirmTrash => "Move to .tersh-trash: type trash then Enter. Esc/Ctrl+G cancels.",
        Mode::ConfirmDelete => "Permanent delete: type delete then Enter. Esc/Ctrl+G cancels.",
        Mode::Conflict => "Destination already exists",
        Mode::PreviewSearch => "Find in preview",
        _ => "",
    };
    let mut lines = vec![Line::from(prompt)];
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
        _ => "Command",
    };
    let block = match app.mode() {
        Mode::ConfirmDelete => modal_block(theme, title, Tone::Danger).style(theme.danger()),
        Mode::ConfirmTrash => modal_block(theme, title, Tone::Warn),
        Mode::Conflict => modal_block(theme, title, Tone::Warn),
        _ => modal_block(theme, title, Tone::Active),
    };
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn normal_footer(app: &App) -> String {
    let mut actions = vec![format!("next: {}", next_action(app))];
    if app.selected_len() > 0 {
        actions.push(format!("{} selected", app.selected_len()));
        actions.push("d trash".to_string());
        actions.push("yy copy".to_string());
    } else {
        actions.push("Space mark".to_string());
        actions.push("yy copy".to_string());
    }
    if app.copy_buffer_len() > 0 {
        actions.push("p paste".to_string());
    }
    actions.push("s sort".to_string());

    format!(
        "normal | {} | q quit | Esc/^G clear | ^C force | ? help | j/k move | / filter | h parent",
        actions.join(" | ")
    )
}

fn next_action(app: &App) -> &'static str {
    match app.entries().get(app.cursor()).map(|entry| entry.kind) {
        Some(FileKind::Directory) => "Enter open dir",
        Some(FileKind::File) => "Enter preview file | e edit",
        Some(FileKind::Symlink) => "Enter inspect link",
        Some(FileKind::Other) => "Space mark item",
        None => "r refresh",
    }
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
