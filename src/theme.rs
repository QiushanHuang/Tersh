use ratatui::{
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Btop,
    Aurora,
    Contrast,
    Mono,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub accent: Color,
    pub accent_alt: Color,
    pub ok: Color,
    pub warn: Color,
    pub danger: Color,
    pub text: Color,
    pub muted: Color,
    pub path: Color,
    pub panel_title: Color,
    pub key: Color,
    pub value: Color,
    pub separator: Color,
    pub active: Color,
    pub inactive: Color,
    pub copy: Color,
    pub cut: Color,
    pub search_match: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Text,
    Muted,
    Key,
    Value,
    Path,
    Title,
    Separator,
    Ok,
    Warn,
    Danger,
    Accent,
    AccentAlt,
    Active,
    Inactive,
    Copy,
    Cut,
    SearchMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipTone {
    Ok,
    Warn,
    Danger,
    Accent,
    AccentAlt,
    Muted,
    Path,
    Copy,
    Cut,
}

const ASCII_BORDER: border::Set = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

const ROUNDED_BORDER: border::Set = border::Set {
    top_left: "╭",
    top_right: "╮",
    bottom_left: "╰",
    bottom_right: "╯",
    vertical_left: "│",
    vertical_right: "│",
    horizontal_top: "─",
    horizontal_bottom: "─",
};

const THICK_BORDER: border::Set = border::Set {
    top_left: "┏",
    top_right: "┓",
    bottom_left: "┗",
    bottom_right: "┛",
    vertical_left: "┃",
    vertical_right: "┃",
    horizontal_top: "━",
    horizontal_bottom: "━",
};

impl Theme {
    pub fn current() -> Self {
        if let Ok(value) = std::env::var("TERSH_COLOR")
            && matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "off" | "false" | "no" | "mono"
            )
        {
            return Self::Mono;
        }
        std::env::var("TERSH_THEME")
            .ok()
            .as_deref()
            .map(Self::from_name)
            .unwrap_or(Self::Btop)
    }

    pub fn from_name(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "aurora" | "neon" => Self::Aurora,
            "contrast" | "high-contrast" | "high_contrast" => Self::Contrast,
            "mono" | "monochrome" | "no-color" | "off" => Self::Mono,
            _ => Self::Btop,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Btop => "btop",
            Self::Aurora => "aurora",
            Self::Contrast => "contrast",
            Self::Mono => "mono",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Self::Btop | Self::Mono => Palette {
                accent: Color::Cyan,
                accent_alt: Color::Magenta,
                ok: Color::Green,
                warn: Color::Yellow,
                danger: Color::Red,
                text: Color::White,
                muted: Color::Gray,
                path: Color::Yellow,
                panel_title: Color::Cyan,
                key: Color::Gray,
                value: Color::White,
                separator: Color::DarkGray,
                active: Color::LightCyan,
                inactive: Color::DarkGray,
                copy: Color::LightCyan,
                cut: Color::LightMagenta,
                search_match: Color::Yellow,
                selected_bg: Color::Cyan,
                selected_fg: Color::Black,
            },
            Self::Aurora => Palette {
                accent: Color::LightCyan,
                accent_alt: Color::LightBlue,
                ok: Color::LightGreen,
                warn: Color::LightYellow,
                danger: Color::LightRed,
                text: Color::White,
                muted: Color::Gray,
                path: Color::LightMagenta,
                panel_title: Color::LightBlue,
                key: Color::LightCyan,
                value: Color::White,
                separator: Color::Blue,
                active: Color::LightGreen,
                inactive: Color::DarkGray,
                copy: Color::LightCyan,
                cut: Color::LightYellow,
                search_match: Color::LightYellow,
                selected_bg: Color::Cyan,
                selected_fg: Color::Black,
            },
            Self::Contrast => Palette {
                accent: Color::LightCyan,
                accent_alt: Color::LightMagenta,
                ok: Color::LightGreen,
                warn: Color::LightYellow,
                danger: Color::LightRed,
                text: Color::White,
                muted: Color::Gray,
                path: Color::LightYellow,
                panel_title: Color::LightCyan,
                key: Color::LightYellow,
                value: Color::White,
                separator: Color::White,
                active: Color::LightGreen,
                inactive: Color::Gray,
                copy: Color::LightCyan,
                cut: Color::LightYellow,
                search_match: Color::LightYellow,
                selected_bg: Color::White,
                selected_fg: Color::Black,
            },
        }
    }

    pub fn fg(self, color: Color) -> Style {
        match self {
            Self::Mono => Style::default(),
            _ => Style::default().fg(color),
        }
    }

    pub fn color(self, tone: Tone) -> Color {
        let palette = self.palette();
        match tone {
            Tone::Text => palette.text,
            Tone::Muted => palette.muted,
            Tone::Key => palette.key,
            Tone::Value => palette.value,
            Tone::Path => palette.path,
            Tone::Title => palette.panel_title,
            Tone::Separator => palette.separator,
            Tone::Ok => palette.ok,
            Tone::Warn => palette.warn,
            Tone::Danger => palette.danger,
            Tone::Accent => palette.accent,
            Tone::AccentAlt => palette.accent_alt,
            Tone::Active => palette.active,
            Tone::Inactive => palette.inactive,
            Tone::Copy => palette.copy,
            Tone::Cut => palette.cut,
            Tone::SearchMatch => palette.search_match,
        }
    }

    pub fn style(self, tone: Tone) -> Style {
        self.fg(self.color(tone))
    }

    pub fn bold(self, tone: Tone) -> Style {
        self.style(tone).add_modifier(Modifier::BOLD)
    }

    pub fn fg_bold(self, color: Color) -> Style {
        self.fg(color).add_modifier(Modifier::BOLD)
    }

    pub fn chip(self, fg: Color, bg: Color) -> Style {
        match self {
            Self::Mono => Style::default().add_modifier(Modifier::BOLD),
            _ => Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        }
    }

    pub fn filled(self, pair: ColorPair) -> Style {
        match self {
            Self::Mono => Style::default(),
            _ => Style::default().fg(pair.fg).bg(pair.bg),
        }
    }

    pub fn chip_pair(self, tone: ChipTone) -> ColorPair {
        let palette = self.palette();
        ColorPair {
            fg: palette.selected_fg,
            bg: self.color(tone.tone()),
        }
    }

    pub fn chip_tone(self, tone: ChipTone) -> Style {
        self.filled(self.chip_pair(tone))
            .add_modifier(Modifier::BOLD)
    }

    pub fn subtle_chip_tone(self, tone: ChipTone) -> Style {
        self.filled(self.chip_pair(tone))
    }

    pub fn search_match(self) -> Style {
        let palette = self.palette();
        self.filled(ColorPair {
            fg: palette.selected_fg,
            bg: palette.search_match,
        })
        .add_modifier(Modifier::BOLD)
    }

    pub fn selected(self) -> Style {
        let palette = self.palette();
        match self {
            Self::Mono => Style::default()
                .add_modifier(Modifier::REVERSED)
                .add_modifier(Modifier::BOLD),
            _ => Style::default()
                .fg(palette.selected_fg)
                .bg(palette.selected_bg)
                .add_modifier(Modifier::BOLD),
        }
    }

    pub fn danger(self) -> Style {
        let palette = self.palette();
        self.fg(palette.danger)
    }

    pub fn border_danger(self) -> Style {
        self.danger()
    }
}

impl ChipTone {
    fn tone(self) -> Tone {
        match self {
            Self::Ok => Tone::Ok,
            Self::Warn => Tone::Warn,
            Self::Danger => Tone::Danger,
            Self::Accent => Tone::Accent,
            Self::AccentAlt => Tone::AccentAlt,
            Self::Muted => Tone::Muted,
            Self::Path => Tone::Path,
            Self::Copy => Tone::Copy,
            Self::Cut => Tone::Cut,
        }
    }
}

pub fn base_block() -> Block<'static> {
    Block::default().border_set(border_set())
}

pub fn panel_title(theme: Theme, title: impl Into<String>) -> Line<'static> {
    panel_title_with_tone(theme, title, Tone::Title)
}

pub fn panel_title_with_tone(theme: Theme, title: impl Into<String>, tone: Tone) -> Line<'static> {
    Line::from(Span::styled(title.into(), theme.bold(tone)))
}

pub fn panel_block(theme: Theme, title: impl Into<String>, tone: Tone) -> Block<'static> {
    base_block()
        .title(panel_title_with_tone(theme, title, tone))
        .borders(Borders::ALL)
        .border_style(theme.style(tone))
}

pub fn modal_block(theme: Theme, title: impl Into<String>, tone: Tone) -> Block<'static> {
    panel_block(theme, title, tone)
}

pub fn border_set() -> border::Set {
    match std::env::var("TERSH_BORDER")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("rounded" | "round" | "unicode") => ROUNDED_BORDER,
        Some("thick" | "heavy" | "bold") => THICK_BORDER,
        _ => ASCII_BORDER,
    }
}

pub fn footer_compact(width: u16, threshold: u16) -> bool {
    match std::env::var("TERSH_FOOTER")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("compact" | "mobile" | "short") => true,
        Some("full" | "wide" | "long") => false,
        _ => width < threshold,
    }
}

pub fn section_line(theme: Theme, label: &'static str) -> Line<'static> {
    Line::from(Span::styled(label, theme.bold(Tone::Title)))
}

pub fn kv_line(theme: Theme, key: &'static str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{key}: "), theme.style(Tone::Key)),
        Span::styled(value.into(), theme.style(Tone::Value)),
    ])
}

pub fn footer_line(theme: Theme, text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    for (index, raw_segment) in text.split('|').enumerate() {
        if index > 0 {
            spans.push(Span::styled(" | ", theme.style(Tone::Separator)));
        }
        let segment = raw_segment.trim().to_string();
        let lower = segment.to_ascii_lowercase();
        let style = if index == 0 {
            theme.chip_tone(ChipTone::Accent)
        } else if lower.contains("^c")
            || lower.contains("delete")
            || lower.contains("danger")
            || lower.contains("force")
        {
            theme.bold(Tone::Danger)
        } else if lower.contains("^g")
            || lower.contains("esc")
            || lower.contains("cancel")
            || lower.contains("back")
        {
            theme.style(Tone::Warn)
        } else if lower.starts_with("next:") {
            theme.bold(Tone::Ok)
        } else {
            theme.style(Tone::Muted)
        };
        spans.push(Span::styled(segment, style));
    }
    Line::from(spans)
}

pub fn footer_paragraph(theme: Theme, text: &str) -> Paragraph<'static> {
    Paragraph::new(footer_line(theme, text)).block(base_block().borders(Borders::TOP))
}

pub fn resource_bar(theme: Theme, percent: Option<u16>, width: usize) -> Vec<Span<'static>> {
    let filled = percent
        .map(|value| ((clamp_percent(value) as usize * width) + 50) / 100)
        .unwrap_or(0)
        .min(width);
    let empty = width.saturating_sub(filled);
    let filled_tone = percent.map(metric_tone).unwrap_or(Tone::Muted);

    vec![
        Span::raw("["),
        Span::styled("#".repeat(filled), theme.style(filled_tone)),
        Span::styled("-".repeat(empty), theme.style(Tone::Inactive)),
        Span::raw("]"),
    ]
}

fn metric_tone(percent: u16) -> Tone {
    match clamp_percent(percent) {
        0..=69 => Tone::Ok,
        70..=89 => Tone::Warn,
        _ => Tone::Danger,
    }
}

fn clamp_percent(value: u16) -> u16 {
    value.min(100)
}

pub fn chip(label: &str, value: impl std::fmt::Display, style: Style) -> Span<'static> {
    Span::styled(format!(" {label} {value} "), style)
}
