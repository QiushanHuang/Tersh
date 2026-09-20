//! A small, in-process action picker. It dispatches existing commands only.
use crate::theme::{Theme, base_block, panel_title};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Borders, Clear, Paragraph},
};

#[derive(Debug, Clone)]
pub struct Action<C> {
    pub label: &'static str,
    pub key: String,
    pub command: C,
    pub danger: bool,
}

impl<C> Action<C> {
    pub fn new(label: &'static str, key: impl Into<String>, command: C) -> Self {
        Self {
            label,
            key: key.into(),
            command,
            danger: false,
        }
    }
    pub fn dangerous(mut self) -> Self {
        self.danger = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct ActionMenu<C> {
    actions: Vec<Action<C>>,
    query: String,
    cursor: usize,
    cancel_label: String,
    submit_label: String,
    down_label: String,
}

pub enum MenuResult<C> {
    Pending,
    Cancel,
    Run(C),
}

impl<C: Clone> ActionMenu<C> {
    pub fn new(actions: Vec<Action<C>>) -> Self {
        Self {
            actions,
            query: String::new(),
            cursor: 0,
            cancel_label: "Esc".into(),
            submit_label: "Enter".into(),
            down_label: "Down/Tab".into(),
        }
    }

    pub fn with_bindings(mut self, map: &crate::keymap::Keymap) -> Self {
        self.cancel_label =
            crate::bindings::label(map, "actions", "cancel").unwrap_or_else(|| "unbound".into());
        self.submit_label =
            crate::bindings::label(map, "actions", "submit").unwrap_or_else(|| "unbound".into());
        self.down_label =
            crate::bindings::label(map, "actions", "down").unwrap_or_else(|| "unbound".into());
        self
    }

    pub fn input_char(&mut self, ch: char) {
        if !ch.is_control() && self.query.chars().count() < 64 {
            self.query.push(ch);
            self.cursor = 0;
        }
    }

    pub fn handle_action(&mut self, action: &str) -> MenuResult<C> {
        match action {
            "cancel" => return MenuResult::Cancel,
            "submit" => {
                if let Some(action) = self.matches().get(self.cursor) {
                    return MenuResult::Run(action.command.clone());
                }
            }
            "down" => {
                self.cursor = self
                    .cursor
                    .saturating_add(1)
                    .min(self.matches().len().saturating_sub(1))
            }
            "up" => self.cursor = self.cursor.saturating_sub(1),
            "backspace" => {
                self.query.pop();
                self.cursor = 0;
            }
            _ => {}
        }
        MenuResult::Pending
    }

    pub fn matches(&self) -> Vec<&Action<C>> {
        let query = self.query.to_ascii_lowercase();
        self.actions
            .iter()
            .filter(|a| {
                a.label.to_ascii_lowercase().contains(&query) || a.key.to_ascii_lowercase() == query
            })
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> MenuResult<C> {
        if key.code == KeyCode::Esc
            || (key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char('g' | 'G')))
        {
            return MenuResult::Cancel;
        }
        match key.code {
            KeyCode::Enter => {
                if let Some(action) = self.matches().get(self.cursor) {
                    return MenuResult::Run(action.command.clone());
                }
            }
            KeyCode::Down | KeyCode::Tab => {
                self.cursor = self
                    .cursor
                    .saturating_add(1)
                    .min(self.matches().len().saturating_sub(1))
            }
            KeyCode::Up | KeyCode::BackTab => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Backspace => {
                self.query.pop();
                self.cursor = 0;
            }
            KeyCode::Char(ch)
                if !ch.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && self.query.chars().count() < 64 =>
            {
                self.query.push(ch);
                self.cursor = 0;
            }
            _ => {}
        }
        MenuResult::Pending
    }
}

pub fn draw<C: Clone>(frame: &mut Frame, area: Rect, menu: &ActionMenu<C>, theme: Theme) {
    let width = area.width.saturating_sub(2).min(64);
    let height = area.height.saturating_sub(3).min(18);
    let rect = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, rect);
    let actions = menu.matches();
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Find: ", theme.fg_bold(theme.palette().accent)),
            Span::raw(menu.query.clone()),
        ]),
        Line::from(Span::styled(
            format!(
                "Type to search; {} next, {} run",
                menu.down_label, menu.submit_label
            ),
            theme.fg(theme.palette().muted),
        )),
    ];
    let capacity = height.saturating_sub(4) as usize;
    let start = menu.cursor.saturating_add(1).saturating_sub(capacity);
    for (index, action) in actions.iter().enumerate().skip(start).take(capacity) {
        let style = if index == menu.cursor {
            theme.selected()
        } else if action.danger {
            theme.danger()
        } else {
            theme.fg(theme.palette().text)
        };
        lines.push(Line::from(Span::styled(
            format!(
                "{} {:<8} {}{}",
                if index == menu.cursor { ">" } else { " " },
                action.key,
                if action.danger { "! " } else { "" },
                action.label
            ),
            style,
        )));
    }
    if actions.is_empty() {
        lines.push(Line::from("No matching actions; edit query or cancel"));
    }
    frame.render_widget(
        Paragraph::new(lines).block(
            base_block()
                .borders(Borders::ALL)
                .border_style(theme.fg(theme.palette().accent))
                .title(panel_title(
                    theme,
                    format!(
                        "Actions {}/{} | {} close",
                        if actions.is_empty() {
                            0
                        } else {
                            menu.cursor + 1
                        },
                        actions.len(),
                        menu.cancel_label
                    ),
                )),
        ),
        rect,
    );
}
