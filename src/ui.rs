use crate::{
    app::{App, Mode},
    fs_core::format_size,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    match app.mode() {
        Mode::Preview | Mode::PreviewSearch => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(area);
            draw_fullscreen_preview(frame, rows[0], app);
            if app.mode() == Mode::PreviewSearch {
                draw_input_modal(frame, centered_rect(70, 30, area), app);
            }
            draw_footer(frame, rows[1], app);
        }
        _ => {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(2)])
                .split(area);
            draw_body(frame, rows[0], app);
            draw_footer(frame, rows[1], app);
            match app.mode() {
                Mode::Help => draw_help(frame, centered_rect(70, 70, area)),
                Mode::Filter
                | Mode::Goto
                | Mode::Rename
                | Mode::CopyTo
                | Mode::MoveTo
                | Mode::ConfirmTrash
                | Mode::ConfirmDelete => draw_input_modal(frame, centered_rect(70, 30, area), app),
                Mode::Message | Mode::Normal | Mode::Preview | Mode::PreviewSearch => {}
            }
        }
    }
}

fn draw_body(frame: &mut Frame, area: Rect, app: &App) {
    if area.width >= 120 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(50),
                Constraint::Percentage(18),
            ])
            .split(area);
        draw_files(frame, columns[0], app);
        draw_preview(frame, columns[1], app);
        draw_info(frame, columns[2], app);
    } else if area.width >= 80 {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
            .split(area);
        draw_files(frame, columns[0], app);
        draw_preview(frame, columns[1], app);
    } else {
        draw_files(frame, area, app);
    }
}

fn draw_files(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![Line::from(Span::styled(
        app.cwd().display().to_string(),
        Style::default().fg(Color::Cyan),
    ))];
    if !app.filter().is_empty() {
        lines.push(Line::from(format!("filter: {}", app.filter())));
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
        let suffix = match entry.kind {
            crate::fs_core::FileKind::Directory => "/",
            crate::fs_core::FileKind::Symlink => "@",
            _ => "",
        };
        let row = format!(
            "{cursor}{mark} {:<5} {:>8} {}{}",
            entry.kind_marker(),
            format_size(entry.size),
            entry.name,
            suffix
        );
        if index == app.cursor() {
            lines.push(Line::from(Span::styled(
                row,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(row));
        }
    }
    let title = if app.entries().is_empty() {
        "Files".to_string()
    } else {
        format!(
            "Files {}/{}",
            app.cursor().saturating_add(1),
            app.entries().len()
        )
    };
    let paragraph =
        Paragraph::new(lines).block(Block::default().title(title).borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

fn draw_fullscreen_preview(frame: &mut Frame, area: Rect, app: &App) {
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
        app.preview().path.display().to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )));
    rendered.push(Line::from(format!(
        "offset: {} / {} | search: {}",
        offset.saturating_add(1),
        total_lines,
        app.preview_search_query(),
    )));
    for (index, line) in lines.iter().enumerate().skip(offset).take(view_lines) {
        let mut style = Style::default();
        if !query.is_empty() && line.to_lowercase().contains(&query) {
            style = style.fg(Color::Black).bg(Color::Yellow);
        }
        if index == active_line {
            style = style
                .add_modifier(Modifier::UNDERLINED)
                .add_modifier(Modifier::BOLD);
        }
        rendered.push(Line::from(Span::styled(line.as_str(), style)));
    }

    let paragraph = Paragraph::new(rendered)
        .block(Block::default().title("Preview").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_preview(frame: &mut Frame, area: Rect, app: &App) {
    let mut lines = vec![Line::from(Span::styled(
        app.preview().path.display().to_string(),
        Style::default().fg(Color::Yellow),
    ))];
    lines.extend(app.preview().lines.iter().cloned().map(Line::from));
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Preview").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_info(frame: &mut Frame, area: Rect, app: &App) {
    let focused = app.entries().get(app.cursor());
    let mut lines = Vec::new();
    if let Some(entry) = focused {
        lines.push(Line::from(format!("kind: {}", entry.kind_marker())));
        lines.push(Line::from(format!("size: {}", format_size(entry.size))));
        lines.push(Line::from(format!(
            "perm: {}",
            if entry.readonly {
                "readonly"
            } else {
                "writable"
            }
        )));
        if let Some(target) = &entry.symlink_target {
            lines.push(Line::from(format!("-> {}", target.display())));
        }
    } else {
        lines.push(Line::from("no item"));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(format!("copy: {}", app.copy_buffer_len())));
    lines.push(Line::from(""));
    lines.push(Line::from("Log"));
    for log in app.logs().iter().rev().take(5) {
        lines.push(Line::from(log.clone()));
    }
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Info").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, app: &App) {
    let mode = format!("{:?}", app.mode()).to_lowercase();
    let text = match app.mode() {
        Mode::Preview | Mode::PreviewSearch => {
            "preview | j/k/↑/↓ scroll  PgUp/PgDn  gg top  G bottom  / search  n/N  e edit  q/^G close".to_string()
        }
        _ => {
            format!(
                "{mode} | q quit | ^G cancel | ^C force | ? | PgUp/PgDn | : goto | / filter | yy/yf/yr/ya | x/p | c/m | d/D | e edit"
            )
        }
    };
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from("Navigation"),
        Line::from("  j/k or arrows: move    PageUp/PageDown: page    Home/End: first/last"),
        Line::from("  h: parent    l: open directory"),
        Line::from("  Enter on file: fullscreen preview"),
        Line::from("  /: filter    .: hidden    r: refresh"),
        Line::from(""),
        Line::from("Operations"),
        Line::from("  Space: mark    yy: copy    x: cut    p: paste"),
        Line::from("  c: copy to...    m: move to...    n: rename"),
        Line::from("  yf: file name    yr: relative path    ya: absolute path"),
        Line::from("  e: edit focused file with nano"),
        Line::from("  :: goto directory"),
        Line::from("  d: trash, then type trash    D: delete, then type delete"),
        Line::from(""),
        Line::from("Preview mode"),
        Line::from("  j/k/↑/↓: scroll    PageUp/PageDown: page    gg: top    G: bottom"),
        Line::from("  /: search    n/N: next/prev match"),
        Line::from(""),
        Line::from("Exit"),
        Line::from("  q: quit/close    Ctrl+G: cancel    Q/Ctrl+C: force quit"),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

fn draw_input_modal(frame: &mut Frame, area: Rect, app: &App) {
    frame.render_widget(Clear, area);
    let prompt = match app.mode() {
        Mode::Filter => "Filter current directory",
        Mode::Goto => "Go to directory",
        Mode::Rename => "Rename focused item",
        Mode::CopyTo => "Copy selected/focused item(s) to directory",
        Mode::MoveTo => "Move selected/focused item(s) to directory",
        Mode::ConfirmTrash => "Type trash then Enter. Ctrl+G cancels.",
        Mode::ConfirmDelete => "Permanent delete: type delete then Enter. Ctrl+G cancels.",
        Mode::PreviewSearch => "Find in preview",
        _ => "",
    };
    let lines = vec![
        Line::from(prompt),
        Line::from(""),
        Line::from(app.input().to_string()),
    ];
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Command").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
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
