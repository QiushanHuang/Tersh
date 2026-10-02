//! A small, in-process action picker. It dispatches existing commands only.
use crate::theme::{Theme, base_block, panel_title};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Borders, Clear, Paragraph},
};

/// Shared task vocabulary for menus and help. Labels/keys remain caller-owned.
pub fn metadata(id: &str) -> (&'static str, &'static str, &'static str) {
    match id {
        "open_places" => (
            "Navigation",
            "Recent and pinned host directories",
            "bookmark recent location 地点 书签 最近",
        ),
        "pin_place" => (
            "Navigation",
            "Pin or unpin this working directory",
            "bookmark favorite 固定 收藏",
        ),
        "open_goto" => (
            "Navigation",
            "Enter a directory path; errors keep your input",
            "jump path cd 跳转 路径",
        ),
        "parent" => (
            "Navigation",
            "Return to the parent and keep your place",
            "back up 返回 上级",
        ),
        "open" => (
            "View",
            "Open a directory or inspect the selected file",
            "preview inspect 打开 预览",
        ),
        "open_log" => (
            "View",
            "Read a bounded tail and follow new log output",
            "tail follow log 日志 跟随",
        ),
        "toggle_structured" => (
            "View",
            "Switch raw and structured JSON, CSV or diff",
            "json csv diff table 格式 表格",
        ),
        "open_filter" => (
            "View",
            "Filter the current list without changing files",
            "search find 筛选 查找",
        ),
        "open_preview_search" => (
            "View",
            "Find text in the displayed preview",
            "find text search 查找 搜索",
        ),
        "cycle_sort" | "reverse_sort" => ("View", "Change the visible ordering", "order sort 排序"),
        "toggle_inspector" => (
            "View",
            "Show or hide the contextual inspector",
            "panel details 面板 详情",
        ),
        "copy" | "copy_to" | "paste" => (
            "Files",
            "Copy with conflict checks and cancellable progress",
            "copy paste duplicate 复制 粘贴",
        ),
        "cut" | "move_to" => (
            "Files",
            "Move items after validating source and destination",
            "move cut 移动 剪切",
        ),
        "rename" => ("Files", "Rename the focused item", "rename 名称 重命名"),
        "open_jobs" => (
            "Tasks",
            "Inspect recent results and unresolved items",
            "progress result history jobs 任务 进度 结果",
        ),
        "cancel_job" => (
            "Tasks",
            "Cancel remaining work; completed items stay completed",
            "stop cancel 停止 取消",
        ),
        "retry_job" => (
            "Tasks",
            "Revalidate failed and remaining items before retrying",
            "retry remaining failed 重试 失败",
        ),
        "export_job" => (
            "Tasks",
            "Export this result to a new file",
            "save export receipt 导出 回执",
        ),
        "open_trash" | "restore" => (
            "Recovery",
            "Restore managed trash without overwriting existing files",
            "restore recover undo 恢复 回收",
        ),
        "trash" => (
            "Recovery",
            "Confirm before moving items to managed trash",
            "remove trash 回收 删除",
        ),
        "permanent_delete" => (
            "Recovery",
            "Permanent deletion requires typed confirmation",
            "delete erase permanent 永久 删除",
        ),
        "open_workbench" => (
            "Connect",
            "Open Tersh at the selected host and directory",
            "tersh workbench remote 工作台 连接",
        ),
        "open_session" => (
            "Connect",
            "Open the selected host's shell",
            "ssh shell terminal 终端 连接",
        ),
        "toggle_attention" => (
            "Hosts",
            "Show hosts with unknown, stale or warning data",
            "warning attention issue 异常 待处理",
        ),
        "open_events" => (
            "Hosts",
            "Review bounded host state changes",
            "event change history 事件 变化",
        ),
        "toggle_pause" => (
            "Hosts",
            "Pause or resume automatic refresh",
            "pause refresh 暂停 刷新",
        ),
        "refresh_all" | "refresh_selected" | "refresh" => {
            ("View", "Request fresh data", "reload refresh 刷新")
        }
        "open_help" => (
            "Help",
            "Show the effective shortcuts for this context",
            "help keys 帮助 按键",
        ),
        _ => ("Actions", "Use the displayed shortcut or press Enter", ""),
    }
}

#[derive(Debug, Clone)]
pub struct Action<C> {
    pub label: &'static str,
    pub key: String,
    pub command: C,
    pub danger: bool,
    pub group: &'static str,
    pub description: &'static str,
    pub keywords: &'static str,
    pub disabled_reason: Option<&'static str>,
}

impl<C> Action<C> {
    pub fn new(label: &'static str, key: impl Into<String>, command: C) -> Self {
        Self {
            label,
            key: key.into(),
            command,
            danger: false,
            group: "Actions",
            description: "",
            keywords: if label.to_ascii_lowercase().contains("recovery") {
                "restore recover undo 恢复 回收"
            } else {
                ""
            },
            disabled_reason: None,
        }
    }
    pub fn described(
        mut self,
        group: &'static str,
        description: &'static str,
        keywords: &'static str,
    ) -> Self {
        self.group = group;
        self.description = description;
        self.keywords = keywords;
        self
    }
    pub fn disabled(mut self, reason: &'static str) -> Self {
        self.disabled_reason = Some(reason);
        self
    }
    pub fn with_metadata(mut self, id: &str) -> Self {
        let info = metadata(id);
        self.group = info.0;
        self.description = info.1;
        self.keywords = info.2;
        self
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
    pub fn new(mut actions: Vec<Action<C>>) -> Self {
        actions.sort_by_key(|a| match a.group {
            "Navigation" => 0,
            "Connect" => 1,
            "View" => 2,
            "Hosts" => 3,
            "Files" => 4,
            "Tasks" => 5,
            "Recovery" => 6,
            "Help" => 8,
            _ => 7,
        });
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
                if let Some(action) = self.matches().get(self.cursor)
                    && action.disabled_reason.is_none()
                {
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
        let query = self.query.to_lowercase();
        self.actions
            .iter()
            .filter(|a| {
                a.label.to_lowercase().contains(&query)
                    || a.key.to_ascii_lowercase() == query
                    || a.keywords.to_lowercase().contains(&query)
                    || a.description.to_lowercase().contains(&query)
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
                if let Some(action) = self.matches().get(self.cursor)
                    && action.disabled_reason.is_none()
                {
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
    let width = area.width.saturating_sub(2).min(84);
    let height = area.height.saturating_sub(2).min(24);
    let rect = Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, rect);
    let actions = menu.matches();
    let inner_width = width.saturating_sub(2) as usize;
    let capacity = height.saturating_sub(7) as usize;
    let mut rows = Vec::new();
    let mut previous = "";
    let mut focused_row = 0;
    for (index, action) in actions.iter().enumerate() {
        if menu.query.is_empty() && action.group != previous {
            rows.push(Line::from(Span::styled(
                action.group,
                theme.fg_bold(theme.palette().accent),
            )));
            previous = action.group;
        }
        if index == menu.cursor {
            focused_row = rows.len();
        }
        let style = if index == menu.cursor {
            theme.selected()
        } else if action.disabled_reason.is_some() {
            theme.fg(theme.palette().muted)
        } else if action.danger {
            theme.danger()
        } else {
            theme.fg(theme.palette().text)
        };
        let key = clip(&action.key, 10);
        let prefix = format!(
            "{} {:<10} {}",
            if index == menu.cursor { ">" } else { " " },
            key,
            if action.disabled_reason.is_some() {
                "- "
            } else if action.danger {
                "! "
            } else {
                ""
            }
        );
        let label = clip(
            action.label,
            inner_width.saturating_sub(unicode_width::UnicodeWidthStr::width(prefix.as_str())),
        );
        let mut spans = vec![Span::styled(prefix, style)];
        let query = menu.query.to_ascii_lowercase();
        if !query.is_empty() && query.is_ascii() {
            if let Some(at) = label.to_ascii_lowercase().find(&query) {
                spans.extend([
                    Span::styled(label[..at].to_owned(), style),
                    Span::styled(
                        label[at..at + query.len()].to_owned(),
                        style.add_modifier(ratatui::style::Modifier::UNDERLINED),
                    ),
                    Span::styled(label[at + query.len()..].to_owned(), style),
                ]);
            } else {
                spans.push(Span::styled(label, style));
            }
        } else {
            spans.push(Span::styled(label, style));
        }
        rows.push(Line::from(spans));
    }
    let start = focused_row.saturating_add(1).saturating_sub(capacity);
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Find: ", theme.fg_bold(theme.palette().accent)),
            Span::raw(menu.query.clone()),
        ]),
        Line::from(Span::styled(
            clip(
                &format!("{} select | {} run", menu.down_label, menu.submit_label),
                inner_width,
            ),
            theme.fg(theme.palette().muted),
        )),
    ];
    lines.extend(rows.into_iter().skip(start).take(capacity));
    if actions.is_empty() {
        lines.push(Line::from("No matches; edit query or cancel"));
    }
    if let Some(action) = actions.get(menu.cursor) {
        lines.push(Line::from(""));
        let description = action.disabled_reason.unwrap_or(action.description);
        lines.push(Line::from(Span::styled(
            clip(&format!("{}: {description}", action.group), inner_width),
            theme.fg(if action.disabled_reason.is_some() {
                theme.palette().warn
            } else {
                theme.palette().muted
            }),
        )));
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

fn clip(text: &str, width: usize) -> String {
    use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
    if text.width() <= width {
        return text.into();
    }
    if width < 3 {
        return ".".repeat(width);
    }
    let mut used = 0;
    let mut result = String::new();
    for ch in text.chars() {
        let n = ch.width().unwrap_or(0);
        if used + n > width - 3 {
            break;
        }
        result.push(ch);
        used += n;
    }
    result.push_str("...");
    result
}
