//! Shared key sequence state and display labels used by both workbenches.
use crate::keymap::{KeyMatch, Keymap};
use crossterm::event::{KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Default)]
pub struct KeyState {
    pub pending: Vec<KeyEvent>,
    pub replay: Vec<char>,
    context: String,
}

impl KeyState {
    pub fn clear(&mut self) {
        self.pending.clear();
    }
    pub fn feed(&mut self, map: &Keymap, context: &str, key: KeyEvent) -> KeyMatch {
        self.replay.clear();
        if key.kind == KeyEventKind::Release {
            return KeyMatch::Unbound;
        }
        if self.context != context {
            self.pending.clear();
            self.context = context.into();
        }
        // Cancellation/emergency commands interrupt any pending chord.
        if let KeyMatch::Action(action) = map.resolve(context, &[key])
            && matches!(action.as_str(), "cancel" | "force_quit" | "cancel_job")
        {
            self.pending.clear();
            return KeyMatch::Action(action);
        }
        self.pending.push(key);
        let mut result = map.resolve(context, &self.pending);
        if result == KeyMatch::Unbound && self.pending.len() > 1 {
            if matches!(context, "input" | "actions" | "cluster_filter") {
                self.replay.extend(
                    self.pending[..self.pending.len() - 1]
                        .iter()
                        .filter_map(|event| printable(*event)),
                );
            }
            self.pending.clear();
            self.pending.push(key);
            result = map.resolve(context, &self.pending);
        }
        if result != KeyMatch::Pending {
            self.pending.clear();
        }
        result
    }
}

pub fn printable(key: KeyEvent) -> Option<char> {
    use crossterm::event::{KeyCode, KeyModifiers};
    if key
        .modifiers
        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SUPER)
    {
        return None;
    }
    match key.code {
        KeyCode::Char(ch) if !ch.is_control() => {
            Some(if key.modifiers.contains(KeyModifiers::SHIFT) {
                ch.to_ascii_uppercase()
            } else {
                ch
            })
        }
        _ => None,
    }
}

pub fn short_key(label: &str) -> String {
    if let Some(key) = label.strip_prefix("Ctrl+") {
        return format!("^{}", key.to_ascii_uppercase());
    }
    match label {
        "PageDown" => "PgDn".into(),
        "PageUp" => "PgUp".into(),
        _ => label.replace(' ', ""),
    }
}

pub fn label(map: &Keymap, context: &str, action: &str) -> Option<String> {
    map.label(context, action).map(|key| short_key(&key))
}

pub fn hint(map: &Keymap, context: &str, action: &str, description: &str) -> Option<String> {
    label(map, context, action).map(|key| format!("{key} {description}"))
}

pub fn cancel_label(map: &Keymap, context: &str, compact: bool) -> String {
    let keys = map
        .bindings(context)
        .into_iter()
        .find(|(id, _)| id == "cancel")
        .map(|(_, keys)| keys)
        .unwrap_or_default();
    if compact {
        keys.iter()
            .find(|key| key.as_str() == "Ctrl+g")
            .or(keys.first())
            .map(|key| short_key(key))
            .unwrap_or_else(|| "unbound".into())
    } else {
        keys.iter()
            .take(2)
            .map(|key| short_key(key))
            .collect::<Vec<_>>()
            .join("/")
    }
}

pub fn help_lines(map: &Keymap, context: &str) -> Vec<String> {
    map.bindings(context)
        .into_iter()
        .map(|(action, keys)| {
            let keys = if keys.is_empty() {
                "unbound".into()
            } else {
                keys.join(" / ")
            };
            format!("{keys}: {}", action.replace('_', " "))
        })
        .collect()
}
