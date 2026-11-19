use ratatui::{
    Frame,
    layout::Rect,
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
        if std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty()) {
            return Self::Mono;
        }
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
        let mut palette = match self {
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
                separator: Color::Gray,
                active: Color::LightCyan,
                inactive: Color::Gray,
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
                inactive: Color::Gray,
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
        };
        if supports_btop_rgb(self) {
            // Several terminal palettes map ANSI bright black to the window
            // background. Structural edges and labels need independent inks.
            palette.inactive = Color::from_u32(0x8aa6af);
            palette.separator = Color::from_u32(0x74939e);
            palette.active = Color::from_u32(0xb7eff0);
            palette.panel_title = Color::from_u32(0xa2d6dd);
            palette.text = Color::from_u32(0xe4edef);
            palette.value = palette.text;
            palette.key = Color::from_u32(0xb8c9ce);
            palette.muted = Color::from_u32(0xa6bdc4);
            palette.accent = Color::from_u32(0x78c7d0);
            palette.ok = Color::from_u32(0x9dd1a5);
            palette.warn = Color::from_u32(0xe6c286);
            palette.danger = Color::from_u32(0xff9d97);
            palette.selected_bg = Color::from_u32(0x88cdd3);
            palette.selected_fg = Color::from_u32(0x10252e);
        }
        palette
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
        // ANSI bright badge backgrounds need dark text for readable contrast.
        let fg = if fg == self.palette().text {
            Color::Black
        } else {
            fg
        };
        match self {
            Self::Mono => Style::default().add_modifier(Modifier::BOLD),
            _ => Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD),
        }
    }

    pub fn chip_tone(self, tone: ChipTone) -> Style {
        let palette = self.palette();
        self.chip(palette.text, self.color(tone.tone()))
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

/// Draw an append-only query field with its insertion point inside the visible row.
pub fn draw_query(frame: &mut Frame, area: Rect, theme: Theme, prompt: &str, value: &str) {
    use unicode_width::UnicodeWidthStr;

    if area.is_empty() {
        return;
    }
    let width = usize::from(area.width);
    let prompt = if prompt.width() < width { prompt } else { "" };
    let available = width - prompt.width() - 1;
    let text = Span::raw(value);
    let mut visible_width: usize = text
        .styled_graphemes(Style::default())
        .map(|g| g.symbol.width())
        .sum();
    let mut start = 0;
    if visible_width > available {
        for grapheme in text.styled_graphemes(Style::default()) {
            start += grapheme.symbol.len();
            visible_width = visible_width.saturating_sub(grapheme.symbol.width());
            if visible_width <= available {
                break;
            }
        }
    }
    frame.render_widget(
        Line::from(vec![
            Span::styled(prompt, theme.bold(Tone::Accent)),
            Span::raw(&value[start..]),
        ]),
        Rect::new(area.x, area.y, area.width, 1),
    );
    frame.set_cursor_position((area.x + (prompt.width() + visible_width) as u16, area.y));
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

// Ghostty draws block elements flush to the cell edges and may render light
// shade as alpha coverage. Use explicit gaps and dots for the desktop meter.
fn supports_btop_rgb(theme: Theme) -> bool {
    theme == Theme::Btop
        && (std::env::var("TERM_PROGRAM").is_ok_and(|s| s.eq_ignore_ascii_case("ghostty"))
            || std::env::var("TERM").as_deref() == Ok("xterm-ghostty")
            || std::env::var("COLORTERM")
                .is_ok_and(|s| matches!(s.to_ascii_lowercase().as_str(), "truecolor" | "24bit")))
}

fn chart_style(theme: Theme, tone: Tone, truecolor: bool) -> Style {
    if !truecolor {
        return theme.style(tone);
    }
    let rgb = match tone {
        Tone::Accent => 0x78c7d0,
        Tone::AccentAlt => 0xb68acf,
        Tone::Ok => 0x88c08c,
        Tone::Warn => 0xe1ba75,
        Tone::Danger => 0xe56f74,
        Tone::Inactive => 0x536171,
        _ => return theme.style(tone),
    };
    theme.fg(Color::from_u32(rgb))
}

pub fn resource_bar(theme: Theme, percent: Option<u16>, width: usize) -> Vec<Span<'static>> {
    let value = usize::from(clamp_percent(percent.unwrap_or(0)));
    let filled = (value * width + 50) / 100;
    let tone = percent.map(metric_tone).unwrap_or(Tone::Muted);
    let unicode = unicode_graphs();
    let rgb = unicode && supports_btop_rgb(theme);
    // Six-eighths height leaves space between consecutive resource rows.
    let (used, track) = if unicode { ("▆", "⣿") } else { ("#", "-") };
    let mut spans = vec![Span::raw("[")];
    spans.extend(chart_cells(
        used.repeat(filled),
        chart_style(theme, tone, rgb),
        0,
    ));
    spans.push(Span::styled(
        track.repeat(width - filled),
        chart_style(theme, Tone::Inactive, rgb),
    ));
    spans.push(Span::raw("]"));
    spans
}

/// Connecting heights describe the line between consecutive observations,
/// never additional observations. Leave the interval to a failed probe empty.
pub(crate) fn trend_bands(
    theme: Theme,
    tone: Tone,
    heights: &[usize],
    width: usize,
    rows: usize,
) -> Vec<Vec<Span<'static>>> {
    let unicode = unicode_graphs();
    let glyphs = if unicode {
        [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']
    } else {
        [' ', '_', '.', ':', '-', '=', '+', '*', '#']
    };
    let ink = chart_style(theme, tone, unicode && supports_btop_rgb(theme));
    let muted = alternate_ink(ink);
    let interior = alternate_ink(muted);
    // Store the height and whether the column is an actual measured point.
    let mut columns = vec![(0_usize, false); width];
    let mut previous = None;
    for (i, &height) in heights.iter().enumerate() {
        if width == 0 {
            break;
        }
        if height == 0 {
            previous = None;
            continue;
        }
        let x = if heights.len() == 1 {
            width - 1
        } else {
            i * (width - 1) / (heights.len() - 1)
        };
        if let Some((left, left_height)) = previous
            && x > left + 1
        {
            let distance = x - left;
            for (k, column) in columns[left..x].iter_mut().enumerate().skip(1) {
                column.0 = (left_height * (distance - k) + height * k + distance / 2) / distance;
            }
        }
        columns[x] = (height, true);
        previous = Some((x, height));
    }

    (0..rows)
        .map(|row| {
            let below = 8 * (rows - row - 1);
            let mut spans = Vec::new();
            let mut segment = String::new();
            let mut last_style = Style::default();
            for (x, &(height, measured)) in columns.iter().enumerate() {
                let level = height.saturating_sub(below).min(8);
                let style = if level == 0 {
                    Style::default()
                } else if measured {
                    ink.add_modifier(Modifier::BOLD)
                } else if x.is_multiple_of(2) {
                    interior
                } else {
                    muted
                };
                if style != last_style && !segment.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut segment), last_style));
                }
                segment.push(glyphs[level]);
                last_style = style;
            }
            spans.push(Span::styled(segment, last_style));
            spans
        })
        .collect()
}

fn alternate_ink(style: Style) -> Style {
    let Some(Color::Rgb(r, g, b)) = style.fg else {
        return style;
    };
    let shade = |component: u8| (u16::from(component) * 7 / 8) as u8;
    style.fg(Color::Rgb(shade(r), shade(g), shade(b)))
}

/// Give neighboring columns a discernible boundary even when the terminal
/// draws block glyphs flush. Index history by observation so it never flickers
/// when a new sample scrolls into view.
fn chart_cells(text: String, style: Style, first: u64) -> Vec<Span<'static>> {
    let alternate = alternate_ink(style);
    if alternate == style {
        return vec![Span::styled(text, style)];
    }
    text.chars()
        .enumerate()
        .map(|(i, ch)| {
            let ink = if first.wrapping_add(i as u64).is_multiple_of(2) {
                style
            } else {
                alternate
            };
            Span::styled(ch.to_string(), ink)
        })
        .collect()
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

/// Fixed cells, no partial shortcuts. Extra actions remain in help/action menu.
pub fn footer_rows(theme: Theme, text: &str, width: u16, rows: usize) -> Vec<Line<'static>> {
    use unicode_width::UnicodeWidthStr;
    let mut segments = text
        .split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    segments.sort_by_key(|segment| {
        if segment.starts_with("next:")
            || segment.ends_with(" paste")
            || segment.ends_with(" open")
            || segment.ends_with(" view")
        {
            0
        } else if segment.ends_with(" actions") {
            1
        } else if segment.contains("^G") || segment.contains("Esc") {
            2
        } else if segment.starts_with("q ") || segment.starts_with("?") || segment.contains("^C") {
            3
        } else {
            4
        }
    });
    let mut content = vec![String::new(); rows];
    for segment in segments {
        for line in &mut content {
            let separator = if line.is_empty() { "" } else { " | " };
            if line.width() + separator.len() + segment.width() <= width as usize {
                line.push_str(separator);
                line.push_str(segment);
                break;
            }
        }
    }
    content
        .iter()
        .map(|line| footer_line(theme, line))
        .collect()
}

pub fn footer_height(width: u16, height: u16) -> u16 {
    if width >= 40 && height >= 10 { 3 } else { 2 }
}

pub fn unicode_graphs() -> bool {
    std::env::var("TERSH_GLYPHS").is_ok_and(|value| value.eq_ignore_ascii_case("unicode"))
}

pub fn motion_enabled() -> bool {
    !std::env::var("TERSH_MOTION")
        .is_ok_and(|value| matches!(value.to_ascii_lowercase().as_str(), "off" | "0" | "reduced"))
}

pub fn profile_settings(profile: &str) -> &'static [(&'static str, &'static str)] {
    match profile {
        "desktop" => &[
            ("TERSH_FOOTER", "full"),
            ("TERSH_BORDER", "rounded"),
            ("TERSH_GLYPHS", "unicode"),
            ("TERSH_MOTION", "on"),
        ],
        "mobile" => &[
            ("TERSH_FOOTER", "compact"),
            ("TERSH_BORDER", "ascii"),
            ("TERSH_GLYPHS", "ascii"),
            ("TERSH_MOTION", "off"),
        ],
        "ssh" => &[
            ("TERSH_FOOTER", "auto"),
            ("TERSH_BORDER", "ascii"),
            ("TERSH_GLYPHS", "ascii"),
            ("TERSH_MOTION", "off"),
        ],
        _ => &[],
    }
}

#[cfg(test)]
mod plot_tests {
    use super::*;

    #[test]
    fn connected_history_marks_observations_without_painting_over_failed_probes() {
        let bands = trend_bands(Theme::Mono, Tone::Ok, &[1, 14, 0, 9, 1], 53, 2);
        let mut highs = Vec::new();
        for row in bands {
            let mut observed = Vec::new();
            let mut text = Vec::new();
            for span in row {
                for glyph in span.content.chars() {
                    if span.style.add_modifier.contains(Modifier::BOLD) {
                        observed.push(text.len());
                    }
                    text.push(glyph);
                }
            }
            assert_eq!(text.len(), 53);
            assert!(
                text[15..38].iter().all(|&ch| ch == ' '),
                "gap must break the trace"
            );
            highs.push(observed);
        }
        let anchors: Vec<_> = highs.into_iter().flatten().collect();
        for at in [0, 13, 39, 52] {
            assert!(anchors.contains(&at), "{anchors:?}");
        }
    }
}
