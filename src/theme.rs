use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Btop,
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
    pub selected_bg: Color,
    pub selected_fg: Color,
}

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
            "contrast" | "high-contrast" | "high_contrast" => Self::Contrast,
            "mono" | "monochrome" | "no-color" | "off" => Self::Mono,
            _ => Self::Btop,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Btop => "btop",
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

    pub fn fg_bold(self, color: Color) -> Style {
        self.fg(color).add_modifier(Modifier::BOLD)
    }

    pub fn chip(self, fg: Color, bg: Color) -> Style {
        match self {
            Self::Mono => Style::default().add_modifier(Modifier::BOLD),
            _ => Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        }
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

pub fn footer_line(theme: Theme, text: &str) -> Line<'static> {
    let palette = theme.palette();
    let mut spans = Vec::new();
    for (index, raw_segment) in text.split('|').enumerate() {
        if index > 0 {
            spans.push(Span::styled(" | ", theme.fg(palette.muted)));
        }
        let segment = raw_segment.trim().to_string();
        let lower = segment.to_ascii_lowercase();
        let style = if index == 0 {
            theme.chip(palette.text, palette.accent)
        } else if lower.contains("^c")
            || lower.contains("delete")
            || lower.contains("danger")
            || lower.contains("force")
        {
            theme.fg_bold(palette.danger)
        } else if lower.contains("^g")
            || lower.contains("esc")
            || lower.contains("cancel")
            || lower.contains("back")
        {
            theme.fg(palette.warn)
        } else if lower.starts_with("next:") {
            theme.fg_bold(palette.ok)
        } else {
            theme.fg(palette.muted)
        };
        spans.push(Span::styled(segment, style));
    }
    Line::from(spans)
}

pub fn chip(label: &str, value: impl std::fmt::Display, style: Style) -> Span<'static> {
    Span::styled(format!(" {label} {value} "), style)
}
