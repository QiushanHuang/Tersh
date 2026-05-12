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
        Mode::Message | Mode::Normal => {}
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
    for (index, entry) in app.entries().iter().enumerate() {
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
    let paragraph = Paragraph::new(lines)
        .block(Block::default().title("Files").borders(Borders::ALL))
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
    let text = format!("{mode} | q quit | Esc | ^C | ? | : goto | yy/yf/yr/ya | x/p | c/m | d/D");
    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(paragraph, area);
}

fn draw_help(frame: &mut Frame, area: Rect) {
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from("Navigation"),
        Line::from("  j/k or arrows: move    h: parent    l/Enter: open"),
        Line::from("  /: filter    .: hidden    r: refresh"),
        Line::from(""),
        Line::from("Operations"),
        Line::from("  Space: mark    yy: copy    x: cut    p: paste"),
        Line::from("  c: copy to...    m: move to...    n: rename"),
        Line::from("  yf: file name    yr: relative path    ya: absolute path"),
        Line::from("  :: goto directory"),
        Line::from("  d: trash, then type trash    D: delete, then type delete"),
        Line::from(""),
        Line::from("Exit"),
        Line::from("  q: quit/close    Q: force quit    Esc: cancel    Ctrl+C: exit"),
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
        Mode::ConfirmTrash => "Type trash then Enter. Esc cancels.",
        Mode::ConfirmDelete => "Permanent delete: type delete then Enter. Esc cancels.",
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
