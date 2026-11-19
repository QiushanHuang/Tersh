//! SSH routes declared by the inventory. A synthetic origin has no probe state.
use crate::{
    bindings,
    cluster::{ClusterApp, ConnectionState, HostConfig, HostKind},
    theme::{self, Theme, Tone},
};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Borders, Paragraph},
};
use std::{borrow::Cow, collections::BTreeMap};
use unicode_width::UnicodeWidthStr;

pub(crate) fn parent<'a>(app: &'a ClusterApp, host: &HostConfig) -> Option<&'a HostConfig> {
    if host.kind() == HostKind::Local {
        None
    } else if let Some(jump) = host.proxy_jump() {
        app.hosts()
            .iter()
            .find(|h| h.kind() == HostKind::Jump && h.alias() == jump)
    } else {
        app.hosts().iter().find(|h| h.kind() == HostKind::Local)
    }
}

struct Node<'a> {
    host: Option<&'a HostConfig>,
    visible: bool,
    parent: Option<usize>,
    depth: usize,
    last: bool,
    // A parent shares its horizontal lane with its first child.
    lane: usize,
}

impl Node<'_> {
    fn alias(&self) -> &str {
        self.host.map_or("Local session", HostConfig::alias)
    }
    fn kind(&self) -> &'static str {
        match self.host.map(HostConfig::kind) {
            None | Some(HostKind::Local) => "LOCAL",
            Some(HostKind::Jump) => "JUMP",
            Some(HostKind::Server) => "SSH",
        }
    }
}

fn nodes(app: &ClusterApp) -> Vec<Node<'_>> {
    if app.visible_count() == 0 {
        return vec![];
    }
    let local = app.hosts().iter().find(|h| h.kind() == HostKind::Local);
    let mut local_visible = false;
    let mut branches = Vec::new();
    let mut children: BTreeMap<&str, Vec<&HostConfig>> = BTreeMap::new();
    for host in app.visible_hosts() {
        if host.kind() == HostKind::Local {
            local_visible = true;
        } else if let Some(jump) = host.proxy_jump() {
            children.entry(jump).or_default().push(host);
        } else {
            branches.push((host, true));
        }
    }
    // Filtering a gateway out must not turn its descendants into direct routes.
    for jump in app.hosts().iter().filter(|h| h.kind() == HostKind::Jump) {
        if children.contains_key(jump.alias())
            && !branches.iter().any(|(h, _)| h.alias() == jump.alias())
        {
            branches.push((jump, false));
        }
    }
    let mut rows = vec![Node {
        host: local,
        visible: local_visible,
        parent: None,
        depth: 0,
        last: true,
        lane: 0,
    }];
    let total = branches.len();
    let mut lane = 0;
    for (index, (branch, visible)) in branches.into_iter().enumerate() {
        if index > 0 {
            lane += 1;
        }
        let branch_index = rows.len();
        rows.push(Node {
            host: Some(branch),
            visible,
            parent: Some(0),
            depth: 1,
            last: index + 1 == total,
            lane,
        });
        if let Some(leaves) = children.remove(branch.alias()) {
            let count = leaves.len();
            for (index, host) in leaves.into_iter().enumerate() {
                if index > 0 {
                    lane += 1;
                }
                rows.push(Node {
                    host: Some(host),
                    visible: true,
                    parent: Some(branch_index),
                    depth: 2,
                    last: index + 1 == count,
                    lane,
                });
            }
        }
    }
    rows
}

// Direct devices can share rows with a gateway's descendants: the two
// groups occupy different columns and have separate incoming connectors.
fn pack_lanes(nodes: &mut [Node<'_>]) -> usize {
    let mut next = [0; 3];
    for i in 1..nodes.len() {
        let depth = nodes[i].depth;
        let parent_lane = nodes[nodes[i].parent.expect("only the root has no parent")].lane;
        let mut lane = next[depth].max(parent_lane);
        if nodes.get(i + 1).is_some_and(|n| n.depth > depth) {
            lane = lane.max(next[depth + 1]);
        }
        nodes[i].lane = lane;
        next[depth] = lane + 1;
    }
    next.into_iter().max().unwrap_or(1)
}

/// Selectable nodes in the same order as the drawn route tree.
pub fn ordered_hosts(app: &ClusterApp) -> Vec<&HostConfig> {
    nodes(app)
        .into_iter()
        .filter(|n| n.visible)
        .filter_map(|n| n.host)
        .collect()
}

struct MapStyle {
    expanded: bool,
    theme: Theme,
    unicode: bool,
    motion: bool,
    phase: usize,
}

impl MapStyle {
    fn symbol(&self, unicode: &'static str, ascii: &'static str) -> &'static str {
        if self.unicode { unicode } else { ascii }
    }
}

pub fn draw(frame: &mut Frame, area: Rect, app: &ClusterApp, expanded: bool) {
    let style = MapStyle {
        expanded,
        theme: Theme::current(),
        unicode: theme::unicode_graphs(),
        motion: theme::motion_enabled(),
        phase: app.animation_phase(),
    };
    let mut block = theme::panel_block(
        style.theme,
        "Topology",
        if expanded { Tone::Active } else { Tone::Title },
    );
    if !expanded
        && let Some(hint) = bindings::hint(app.keymap(), "cluster", "open_topology", "expand")
        && hint.width() + 12 <= usize::from(area.width)
    {
        block = block.title_top(
            Line::styled(format!(" {hint} "), style.theme.style(Tone::Key)).right_aligned(),
        );
    }
    let inner = block.inner(area);
    let mut nodes = nodes(app);
    let focus = nodes
        .iter()
        .position(|node| node.visible && node.host == app.selected_host());
    frame.render_widget(block, area);
    if inner.is_empty() {
        return;
    }
    let Some(last) = nodes.last() else {
        frame.render_widget(Line::from("No matching hosts"), inner);
        return;
    };
    let columns = nodes.iter().map(|node| node.depth + 1).max().unwrap_or(1);
    let mut lanes = last.lane + 1;
    let width = usize::from(inner.width);
    let height = usize::from(inner.height);
    // Diagrams show complete routes. Use a scrollable outline when even one
    // line per endpoint would not fit, or names would be too short to identify.
    let wide_enough = width >= columns * 17 + (columns - 1) * 6 && width >= 48;
    if wide_enough && columns == 3 && lanes * 3 > height {
        lanes = pack_lanes(&mut nodes);
    }
    if wide_enough && lanes <= height {
        let step = if lanes * 3 <= height {
            3
        } else if lanes * 2 <= height {
            2
        } else {
            1
        };
        let grid = Columns::fit(inner, columns as u16, lanes as u16, step, expanded);
        draw_diagram(frame, grid, app, &nodes, focus, &style);
    } else {
        draw_outline(frame, inner, app, &nodes, focus, &style);
    }
}

fn state(app: &ClusterApp, node: &Node<'_>) -> ConnectionState {
    node.host
        .and_then(|h| app.snapshot_for(h.alias()))
        .map_or(ConnectionState::Unknown, |s| s.connection)
}

fn state_tone(state: ConnectionState) -> Tone {
    match state {
        ConnectionState::Online => Tone::Ok,
        ConnectionState::Checking => Tone::Accent,
        ConnectionState::Unknown => Tone::Muted,
        ConnectionState::Stale => Tone::Warn,
        ConnectionState::Offline | ConnectionState::Timeout | ConnectionState::AuthFailed => {
            Tone::Danger
        }
    }
}

fn draw_node(
    frame: &mut Frame,
    area: Rect,
    node: &Node<'_>,
    app: &ClusterApp,
    selected: bool,
    style: &MapStyle,
) {
    let theme = style.theme;
    let state = state(app, node);
    let status = if node.host.is_none() {
        ""
    } else if state == ConnectionState::Checking && node.depth == 0 && style.motion {
        ["| checking", "/ checking", "- checking", "\\ checking"][style.phase]
    } else {
        state.label()
    };
    let status_style = theme.bold(state_tone(state));
    let marker = if selected { "> " } else { "  " };
    let name_style = if selected {
        theme.selected()
    } else if node.visible {
        theme.bold(Tone::Text)
    } else {
        theme.style(Tone::Muted)
    };
    if area.is_empty() {
        return;
    }
    let border = theme.style(if selected {
        Tone::Active
    } else {
        Tone::Inactive
    });
    if area.height >= 2 {
        let kind = if !node.visible && node.host.is_some() {
            format!("{} via", node.kind())
        } else {
            node.kind().to_owned()
        };
        let alias = clip(
            node.alias(),
            usize::from(area.width.saturating_sub(4)),
            style,
        );
        let name = Line::from(vec![
            Span::styled(marker, theme.bold(Tone::Active)),
            Span::styled(alias, name_style),
        ]);
        let mut block = theme::base_block()
            .borders(Borders::ALL)
            .border_style(border);
        let mut labels = if area.height == 2 {
            vec![Span::styled(format!(" {kind} "), theme.style(Tone::Title))]
        } else {
            Vec::new()
        };
        if !status.is_empty() {
            labels.push(Span::styled(format!(" {status} "), status_style));
            if style.expanded
                && let Some(mem) = app
                    .snapshot_for(node.alias())
                    .filter(|s| s.connection == ConnectionState::Online)
                    .and_then(|s| s.report.memory_used_percent())
            {
                let mem = format!("RAM {mem}% ");
                if mem.width() + status.width() + 4 <= usize::from(area.width) {
                    labels.push(Span::styled(mem, theme.style(Tone::Key)));
                }
            }
        }
        if !labels.is_empty() {
            block = block.title_bottom(Line::from(labels));
        }
        if area.height == 2 {
            frame.render_widget(block.title_top(name), area);
        } else {
            block = block.title_top(Span::styled(
                format!(" {kind} "),
                theme.bold(if node.visible {
                    Tone::Title
                } else {
                    Tone::Muted
                }),
            ));
            frame.render_widget(Paragraph::new(name).block(block), area);
        }
    } else {
        let width = usize::from(area.width);
        let kind = match node.host.map(HostConfig::kind) {
            Some(HostKind::Jump) => "[J] ",
            Some(HostKind::Local) => "[L] ",
            _ => "",
        };
        let suffix = if status.is_empty() || width <= status.len() + 5 {
            ""
        } else {
            status
        };
        let left = width.saturating_sub(suffix.len() + usize::from(!suffix.is_empty()));
        let marker = if left >= marker.len() + node.alias().width() {
            marker
        } else if selected {
            ">"
        } else {
            ""
        };
        let kind = if kind.len() + marker.len() + node.alias().width() <= left {
            kind
        } else {
            ""
        };
        let name = clip(
            node.alias(),
            left.saturating_sub(marker.len() + kind.len()),
            style,
        );
        let pad = left.saturating_sub(marker.len() + kind.len() + name.width())
            + usize::from(!suffix.is_empty());
        let mut spans = vec![
            Span::styled(marker, theme.bold(Tone::Active)),
            Span::styled(kind, theme.style(Tone::Key)),
            Span::styled(name, name_style),
        ];
        if !suffix.is_empty() {
            spans.push(Span::raw(" ".repeat(pad)));
            spans.push(Span::styled(suffix, status_style));
        }
        frame.render_widget(Line::from(spans), area);
    }
}

fn clip<'a>(value: &'a str, width: usize, style: &MapStyle) -> Cow<'a, str> {
    if value.width() <= width {
        return Cow::Borrowed(value);
    }
    if width == 0 {
        return Cow::Borrowed("");
    }
    let span = Span::raw(value);
    let mut bytes = 0;
    let mut remaining = width - 1;
    for grapheme in span.styled_graphemes(Style::default()) {
        let cells = grapheme.symbol.width();
        if cells > remaining {
            break;
        }
        bytes += grapheme.symbol.len();
        remaining -= cells;
    }
    Cow::Owned(format!("{}{}", &value[..bytes], style.symbol("…", "~")))
}

fn draw_outline(
    frame: &mut Frame,
    area: Rect,
    app: &ClusterApp,
    nodes: &[Node<'_>],
    focus: Option<usize>,
    style: &MapStyle,
) {
    let height = usize::from(area.height);
    let start = focus.unwrap_or(0).saturating_add(1).saturating_sub(height);
    // Pin the selected device's parent when a short viewport would scroll
    // the gateway off-screen. The remaining rows still follow the selection.
    let sticky = focus
        .and_then(|i| nodes[i].parent)
        .filter(|&i| i < start && height >= 2);
    let window = start + usize::from(sticky.is_some())..(start + height).min(nodes.len());
    for (line, slot) in sticky.into_iter().chain(window).enumerate() {
        let node = &nodes[slot];
        let y = area.y + line as u16;
        let mut prefix = String::new();
        if node.depth == 2
            && let Some(parent) = node.parent
        {
            prefix.push_str(if nodes[parent].last {
                "   "
            } else {
                style.symbol("│  ", "|  ")
            });
        }
        if node.depth > 0 {
            prefix.push_str(if node.last {
                style.symbol("└", "`")
            } else {
                style.symbol("├", "+")
            });
            let active = style.motion && state(app, node) == ConnectionState::Checking;
            let link = if active {
                [
                    style.symbol("•─", "*-"),
                    style.symbol("─•", "-*"),
                    style.symbol("─›", "->"),
                    style.symbol("──", "--"),
                ][style.phase]
            } else {
                style.symbol("──", "--")
            };
            prefix.push_str(link);
        }
        let indent = (prefix.width() as u16).min(area.width);
        frame.render_widget(
            Line::styled(prefix, style.theme.style(Tone::Accent)),
            Rect::new(area.x, y, indent, 1),
        );
        draw_node(
            frame,
            Rect::new(area.x + indent, y, area.width - indent, 1),
            node,
            app,
            focus == Some(slot),
            style,
        );
    }
}

struct Columns {
    left: u16,
    top: u16,
    card: u16,
    gap: u16,
    height: u16,
    pitch: u16,
}

impl Columns {
    fn fit(area: Rect, columns: u16, lanes: u16, height: u16, expanded: bool) -> Self {
        let card = ((area.width - (columns - 1) * 6) / columns).min(42);
        let mut grid = Self {
            left: area.x,
            top: area.y,
            card,
            gap: 6,
            height,
            pitch: height,
        };
        // The overview keeps a compact route sketch; space its two-line
        // labels on tall displays, while short overview maps stay flush.
        if !expanded && height == 2 && area.height >= 14 && lanes > 1 {
            grid.pitch = ((area.height - height) / (lanes - 1)).min(height + 2);
            grid.top += (area.height - height - (lanes - 1) * grid.pitch) / 2;
        }
        if expanded {
            if columns > 1 {
                grid.gap = ((area.width - columns * card) / (columns - 1)).min(18);
            }
            if lanes > 1 {
                grid.pitch = ((area.height - height) / (lanes - 1)).min(height + 4);
            }
            let used_width = columns * card + (columns - 1) * grid.gap;
            let used_height = height + (lanes - 1) * grid.pitch;
            grid.left += (area.width - used_width) / 2;
            grid.top += (area.height - used_height) / 2;
        }
        grid
    }
    fn rect(&self, node: &Node<'_>) -> Rect {
        Rect::new(
            self.left + node.depth as u16 * (self.card + self.gap),
            self.top + node.lane as u16 * self.pitch,
            self.card,
            self.height,
        )
    }
}

fn socket_y(rect: Rect) -> u16 {
    rect.y + u16::from(rect.height == 3)
}

fn junction(left: bool, right: bool, up: bool, down: bool, style: &MapStyle) -> &'static str {
    match (left, right, up, down) {
        (true, true, true, true) => style.symbol("┼", "+"),
        (true, true, true, false) => style.symbol("┴", "+"),
        (true, true, false, true) => style.symbol("┬", "+"),
        (true, false, true, true) => style.symbol("┤", "+"),
        (false, true, true, true) => style.symbol("├", "+"),
        (true, false, false, true) => style.symbol("┐", "+"),
        (true, false, true, false) => style.symbol("┘", "+"),
        (false, true, false, true) => style.symbol("┌", "+"),
        (false, true, true, false) => style.symbol("└", "`"),
        (false, false, _, _) => style.symbol("│", "|"),
        _ => style.symbol("─", "-"),
    }
}

fn draw_diagram(
    frame: &mut Frame,
    grid: Columns,
    app: &ClusterApp,
    nodes: &[Node<'_>],
    focus: Option<usize>,
    style: &MapStyle,
) {
    let mut children = vec![Vec::new(); nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        if let Some(p) = node.parent {
            children[p].push(i);
        }
    }
    let mut rects: Vec<_> = nodes.iter().map(|n| grid.rect(n)).collect();
    // Align each hub with the middle of its visible branches. Packed columns
    // may also contain direct devices, which keep their own rows.
    if grid.height == 3 {
        for (i, group) in children.iter().enumerate().rev() {
            if let (Some(&first), Some(&last)) = (group.first(), group.last()) {
                let middle = rects[first].y + (rects[last].y - rects[first].y) / 2;
                let next_peer = (i + 1..nodes.len()).find(|&j| nodes[j].depth == nodes[i].depth);
                let limit = next_peer.map_or(middle, |j| rects[j].y.saturating_sub(grid.height));
                rects[i].y = middle.min(limit).max(rects[i].y);
            }
        }
    }
    let neutral = style.theme.style(Tone::Inactive);
    let active = style.theme.bold(Tone::Active);
    let horizontal = style.symbol("─", "-");
    let buf = frame.buffer_mut();
    for (i, group) in children.iter().enumerate() {
        let (Some(&first), Some(&last)) = (group.first(), group.last()) else {
            continue;
        };
        let source = rects[i];
        let source_y = socket_y(source);
        let trunk = source.right() + (grid.gap - 1) / 2;
        let top = socket_y(rects[first]).min(source_y);
        let bottom = socket_y(rects[last]).max(source_y);
        let selected_child = group
            .iter()
            .copied()
            .find(|&j| focus == Some(j) || focus.is_some_and(|f| nodes[f].parent == Some(j)));
        let route_range = selected_child.map(|j| {
            let y = socket_y(rects[j]);
            source_y.min(y)..=source_y.max(y)
        });
        for x in source.right()..trunk {
            buf[(x, source_y)]
                .set_symbol(horizontal)
                .set_style(if selected_child.is_some() {
                    active
                } else {
                    neutral
                });
        }
        let mut siblings = group.iter().copied().peekable();
        for y in top..=bottom {
            let child = siblings.next_if(|&j| socket_y(rects[j]) == y);
            // Draw the entire highlighted path, including the trunk between
            // the parent's socket and the selected child's branch.
            let junction_style = if route_range.as_ref().is_some_and(|r| r.contains(&y)) {
                active
            } else {
                neutral
            };
            buf[(trunk, y)]
                .set_symbol(junction(
                    y == source_y,
                    child.is_some(),
                    y > top,
                    y < bottom,
                    style,
                ))
                .set_style(junction_style);
            if let Some(j) = child {
                let start = trunk + 1;
                let end = rects[j].x;
                let moving = style.motion && state(app, &nodes[j]) == ConnectionState::Checking;
                let pulse = usize::from(end - start) * style.phase / 3;
                for x in start..end {
                    let (symbol, color) = if moving && usize::from(x - start) == pulse {
                        (style.symbol("●", "*"), active)
                    } else {
                        (
                            horizontal,
                            if selected_child == Some(j) {
                                active
                            } else {
                                neutral
                            },
                        )
                    };
                    buf[(x, y)].set_symbol(symbol).set_style(color);
                }
            }
        }
    }
    for (i, (node, mut rect)) in nodes.iter().zip(rects).enumerate() {
        if rect.height == 1 {
            let edge = if focus == Some(i) { active } else { neutral };
            let buf = frame.buffer_mut();
            buf[(rect.x, rect.y)].set_symbol("[").set_style(edge);
            buf[(rect.right() - 1, rect.y)]
                .set_symbol("]")
                .set_style(edge);
            rect.x += 1;
            rect.width -= 2;
        }
        draw_node(frame, rect, node, app, focus == Some(i), style);
        if rect.height == 3 {
            let y = socket_y(rect);
            let buf = frame.buffer_mut();
            if node.parent.is_some() {
                buf[(rect.x, y)].set_symbol(style.symbol("┤", "+"));
            }
            if !children[i].is_empty() {
                buf[(rect.right() - 1, y)].set_symbol(style.symbol("├", "+"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cluster::{ClusterCommand, ClusterInventory, HostSnapshot};
    use crossterm::event::{KeyCode, KeyEvent};
    use ratatui::{Terminal, backend::TestBackend};

    fn fixture() -> ClusterApp {
        ClusterApp::from_inventory(
            ClusterInventory::from_json(
                r#"{
            "jump_host":{"alias":"gateway"},
            "servers":[{"alias":"hpc-a","proxy_jump":"gateway"},
                       {"alias":"storage"},{"alias":"hpc-b","proxy_jump":"gateway"},
                       {"alias":"studio","proxy_jump":"gateway"}]
        }"#,
            )
            .unwrap(),
        )
    }

    #[test]
    fn filter_retains_gateway_and_unmonitored_origin_only_as_route_context() {
        let mut app = fixture();
        app.apply(ClusterCommand::OpenFilter);
        for c in "hpc".chars() {
            app.handle_key(KeyEvent::from(KeyCode::Char(c)));
        }
        let rows = nodes(&app);
        assert_eq!(
            rows.iter()
                .map(|n| (n.alias(), n.parent, n.visible))
                .collect::<Vec<_>>(),
            [
                ("Local session", None, false),
                ("gateway", Some(0), false),
                ("hpc-a", Some(1), true),
                ("hpc-b", Some(1), true)
            ]
        );
        assert_eq!(
            ordered_hosts(&app)
                .iter()
                .map(|h| h.alias())
                .collect::<Vec<_>>(),
            ["hpc-a", "hpc-b"]
        );
        let storage = app.hosts().iter().find(|h| h.alias() == "storage").unwrap();
        assert!(parent(&app, storage).is_none());
    }

    fn diagram(app: &ClusterApp, phase: usize, motion: bool, unicode: bool) -> Vec<String> {
        let nodes = nodes(app);
        let grid = Columns {
            left: 0,
            top: 0,
            card: 22,
            gap: 6,
            height: 3,
            pitch: 3,
        };
        let style = MapStyle {
            theme: Theme::Mono,
            expanded: true,
            phase,
            motion,
            unicode,
        };
        let mut terminal = Terminal::new(TestBackend::new(84, 16)).unwrap();
        terminal
            .draw(|frame| draw_diagram(frame, grid, app, &nodes, Some(2), &style))
            .unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .chunks(84)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect()
    }

    #[test]
    fn gateway_middle_branch_stays_connected_and_only_a_checking_branch_pulses() {
        let mut app = fixture();
        app.apply_snapshot(HostSnapshot::offline("studio", "demo"));
        app.begin_refresh(&["hpc-b".to_string()]);
        let first = diagram(&app, 0, true, true);
        let next = diagram(&app, 1, true, true);
        assert!(first[4].contains("┼●──") && first[4].contains("gateway"));
        assert!(next[4].contains("┼─●─"));
        assert!(first[1].contains("┌───") && first[1].contains("hpc-a"));
        assert!(first[7].contains("└───") && first[7].contains("studio"));
        assert!(first[8].contains("offline"));
        let reduced = diagram(&app, 0, false, false);
        assert_eq!(reduced, diagram(&app, 1, false, false));
        assert!(reduced[5].contains("checking"));
    }

    #[test]
    fn expanded_large_diagram_balances_margins_and_spaces_device_cards() {
        let app = fixture();
        for (width, height) in [(182, 48), (82, 14)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|f| draw(f, f.area(), &app, true)).unwrap();
            let buf = terminal.backend().buffer();
            let content = (1..height - 1)
                .flat_map(|y| {
                    (1..width - 1)
                        .filter_map(move |x| (buf[(x, y)].symbol() != " ").then_some((x, y)))
                })
                .collect::<Vec<_>>();
            let x0 = content.iter().map(|p| p.0).min().unwrap();
            let x1 = content.iter().map(|p| p.0).max().unwrap();
            let y0 = content.iter().map(|p| p.1).min().unwrap();
            let y1 = content.iter().map(|p| p.1).max().unwrap();
            assert!(x0.abs_diff(width - x1 - 1) <= 1);
            assert!(y0.abs_diff(height - y1 - 1) <= 1);
            if width > 160 {
                assert!(y1 - y0 >= 19, "rows need breathing room: {y0}..{y1}");
            }
        }
    }

    #[test]
    fn overview_has_framed_devices_and_selection_is_confined_to_the_alias() {
        use ratatui::style::Color;
        let app = fixture();
        for (w, h) in [(91, 24), (72, 11), (72, 7)] {
            let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
            term.draw(|f| draw(f, f.area(), &app, false)).unwrap();
            let buf = term.backend().buffer();
            let highlighted = buf
                .content()
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    c.bg != Color::Reset || c.modifier.contains(ratatui::style::Modifier::REVERSED)
                })
                .collect::<Vec<_>>();
            assert_eq!(
                highlighted
                    .iter()
                    .map(|(_, c)| c.symbol())
                    .collect::<String>(),
                "gateway"
            );
            assert!(
                highlighted
                    .windows(2)
                    .all(|pair| pair[1].0 == pair[0].0 + 1)
            );
            if h >= 11 {
                let count = buf
                    .content()
                    .iter()
                    .filter(|c| matches!(c.symbol(), "╭" | "┏" | "+"))
                    .count();
                assert!(count >= 7, "map needs a border for each station: {count}");
            }
        }
    }

    #[test]
    fn clipped_phone_outline_keeps_last_selected_host_visible() {
        let mut app = fixture();
        app.apply(ClusterCommand::Last);
        let selected = app.selected_host().unwrap().alias().to_owned();
        let mut terminal = Terminal::new(TestBackend::new(25, 5)).unwrap();
        terminal.draw(|f| draw(f, f.area(), &app, true)).unwrap();
        let buf = terminal.backend().buffer();
        assert!(buf.content().chunks(25).any(|row| {
            let text = row.iter().map(|c| c.symbol()).collect::<String>();
            text.contains('>') && text.contains(&selected)
        }));
    }
}
