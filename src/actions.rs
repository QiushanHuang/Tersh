//! A small, in-process action picker. It dispatches existing commands only.
use crate::theme::{Theme, Tone, draw_query, panel_block};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
        "open_topology" => (
            "Hosts",
            "Switch between the host list and the SSH route map",
            "network map topology route jump proxy 网络 拓扑 跳板",
        ),
        "show_overview" => (
            "Hosts",
            "Show the host table and compact route map",
            "overview list hosts 概览 列表",
        ),
        "show_topology_trends" => (
            "Hosts",
            "Show device routes with observation trends",
            "split topology history charts 分栏 拓扑 趋势",
        ),
        "show_topology_meters" => (
            "Hosts",
            "Use the full width for the selected device’s usage meters",
            "full width meters graph 全宽 用量 仪表",
        ),
        "toggle_topology_trends" => (
            "Hosts",
            "Switch between full-width usage meters and the split view with trends",
            "history usage meters charts layout 用量 趋势 布局",
        ),
        "topology_parent" => (
            "Hosts",
            "Select the configured parent on this SSH route",
            "jump upstream 上级 跳板",
        ),
        "topology_child" => (
            "Hosts",
            "Select the first visible child on this SSH route",
            "downstream 下级",
        ),
        "open_hardware" => (
            "Hosts",
            "Inspect CPU, GPU and memory inventory and current usage",
            "hardware cpu gpu ram memory device 硬件 设备 内存 显卡",
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
            "first" => self.cursor = 0,
            "last" => self.cursor = self.matches().len().saturating_sub(1),
            "backspace" => {
                self.query.pop();
                self.cursor = 0;
            }
            _ => {}
        }
        MenuResult::Pending
    }

    pub fn matches(&self) -> Vec<&Action<C>> {
        if self.query.is_empty() {
            return self.actions.iter().collect();
        }
        let query = self.query.to_lowercase();
        let mut matches = Vec::new();
        for action in &self.actions {
            let label = action.label.to_lowercase();
            let rank = if action.key == self.query || label == query {
                0
            } else if label.starts_with(&query) {
                1
            } else if label.contains(&query) || action.key.eq_ignore_ascii_case(&query) {
                2
            } else if action.keywords.to_lowercase().contains(&query)
                || action.description.to_lowercase().contains(&query)
            {
                3
            } else {
                continue;
            };
            matches.push((rank, action));
        }
        matches.sort_by_key(|(rank, _)| *rank);
        matches.into_iter().map(|(_, action)| action).collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> MenuResult<C> {
        let action = match key.code {
            KeyCode::Esc => "cancel",
            KeyCode::Char('g' | 'G') if key.modifiers.contains(KeyModifiers::CONTROL) => "cancel",
            KeyCode::Enter => "submit",
            KeyCode::Down | KeyCode::Tab => "down",
            KeyCode::Up | KeyCode::BackTab => "up",
            KeyCode::Home => "first",
            KeyCode::End => "last",
            KeyCode::Backspace => "backspace",
            _ => {
                if let Some(ch) = crate::bindings::printable(key) {
                    self.input_char(ch);
                }
                return MenuResult::Pending;
            }
        };
        self.handle_action(action)
    }
}

pub fn draw<C: Clone>(frame: &mut Frame, area: Rect, menu: &ActionMenu<C>, theme: Theme) {
    let width = if area.width < 48 {
        area.width
    } else {
        area.width.saturating_sub(4).min(84)
    };
    let height = if area.height < 12 {
        area.height
    } else {
        area.height.saturating_sub(2).min(24)
    };
    let rect = Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    );
    let actions = menu.matches();
    let position = if actions.is_empty() {
        0
    } else {
        menu.cursor + 1
    };
    let mut block = panel_block(
        theme,
        format!("Actions {position}/{}", actions.len()),
        Tone::Active,
    );
    let close = format!(" {} close ", menu.cancel_label);
    if width as usize >= 20 + close.width() {
        block = block
            .title_top(Line::from(Span::styled(close, theme.style(Tone::Key))).right_aligned());
    }
    let inner = block.inner(rect);
    frame.render_widget(Clear, rect);
    frame.render_widget(block, rect);
    if inner.is_empty() {
        return;
    }

    let row = |y| Rect::new(inner.x, y, inner.width, 1);
    draw_query(frame, row(inner.y), theme, "Find: ", &menu.query);
    let selected = actions.get(menu.cursor);
    let show_hint = inner.height >= 5;
    let show_description = inner.height >= 4 && selected.is_some();
    let list_y = inner.y + 1;
    let capacity =
        usize::from(inner.height - 1 - u16::from(show_hint) - u16::from(show_description));
    let mut rows = Vec::new();
    let mut focused_row = 0;
    let mut previous = "";
    let query = menu.query.to_ascii_lowercase();
    // Keep an action visible even when the terminal has room for a single result.
    for (index, action) in actions.iter().enumerate() {
        if capacity >= 3 && query.is_empty() && action.group != previous {
            rows.push(Line::from(Span::styled(
                action.group,
                theme.bold(Tone::Title),
            )));
            previous = action.group;
        }
        let focused = index == menu.cursor;
        if focused {
            focused_row = rows.len();
        }
        rows.push(action_row(
            action,
            focused,
            &query,
            usize::from(inner.width),
            theme,
        ));
    }
    if actions.is_empty() {
        rows.push(Line::from("No matches"));
    }
    let start = focused_row.saturating_add(1).saturating_sub(capacity);
    frame.render_widget(
        Paragraph::new(
            rows.into_iter()
                .skip(start)
                .take(capacity)
                .collect::<Vec<_>>(),
        ),
        Rect::new(inner.x, list_y, inner.width, capacity as u16),
    );
    if let Some(action) = selected.filter(|_| show_description) {
        let (description, tone) = match action.disabled_reason {
            Some(reason) => (reason, Tone::Warn),
            None if action.danger => (action.description, Tone::Danger),
            None => (action.description, Tone::Muted),
        };
        frame.render_widget(
            Line::from(Span::styled(
                clip(description, inner.width as usize),
                theme.style(tone),
            )),
            row(inner.bottom() - 1 - u16::from(show_hint)),
        );
    }
    if show_hint {
        let hint = format!("{} select | {} run", menu.down_label, menu.submit_label);
        frame.render_widget(
            Line::from(Span::styled(
                clip(&hint, inner.width as usize),
                theme.style(Tone::Key),
            )),
            row(inner.bottom() - 1),
        );
    }
}

fn action_row<C>(
    action: &Action<C>,
    focused: bool,
    query: &str,
    width: usize,
    theme: Theme,
) -> Line<'static> {
    let style = if focused {
        theme.selected()
    } else if action.disabled_reason.is_some() {
        theme.style(Tone::Muted)
    } else if action.danger {
        theme.danger()
    } else {
        theme.style(Tone::Text)
    };
    let marker = if action.disabled_reason.is_some() {
        "- "
    } else if action.danger {
        "! "
    } else {
        ""
    };
    let lead = if focused { "> " } else { "  " };
    let key = if width >= 20 {
        clip(&action.key, (width / 3).min(16))
    } else {
        String::new()
    };
    let label_space = width
        .saturating_sub(lead.len() + marker.len() + key.width() + usize::from(!key.is_empty()));
    let label = clip(action.label, label_space);
    let mut spans = vec![Span::raw(lead), Span::raw(marker)];
    if !query.is_empty()
        && query.is_ascii()
        && let Some(at) = label.to_ascii_lowercase().find(query)
    {
        spans.push(Span::raw(label[..at].to_owned()));
        spans.push(Span::styled(
            label[at..at + query.len()].to_owned(),
            ratatui::style::Modifier::UNDERLINED,
        ));
        spans.push(Span::raw(label[at + query.len()..].to_owned()));
    } else {
        spans.push(Span::raw(label.clone()));
    }
    if !key.is_empty() {
        spans.push(Span::raw(
            " ".repeat(label_space.saturating_sub(label.width()) + 1),
        ));
        spans.push(Span::styled(
            key,
            if focused {
                style
            } else {
                theme.bold(Tone::Key)
            },
        ));
    }
    Line::from(spans).style(style)
}

fn clip(text: &str, width: usize) -> String {
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
