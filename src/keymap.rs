//! Context-aware, bounded key configuration. `Ctrl+c` is an unconditional
//! emergency exit in every context; it cannot be reassigned or disabled.
//!
//! JSON contains context → action ID → key sequence strings. An override replaces
//! that action's defaults; `[]` disables it. Chords use spaces (`"g g"`). Printable
//! text not matched in `input`, `actions`, or `cluster_filter` belongs to the caller.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, bail};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

const MAX_CONFIG_BYTES: u64 = 65_536;
const MAX_CHORD_KEYS: usize = 4;
const MAX_BINDINGS_PER_ACTION: usize = 16;
const MODAL_CONTEXTS: &[&str] = &[
    "preview",
    "input",
    "help",
    "actions",
    "cluster_detail",
    "cluster_filter",
    "trash",
    "jobs",
];

type Config = BTreeMap<String, BTreeMap<String, Vec<String>>>;

// Reject duplicate JSON object fields rather than accepting a silent last-value
// override. This also catches duplicate context names before configuration merge.
struct UniqueMap<T>(BTreeMap<String, T>);

impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for UniqueMap<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: serde::Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
            type Value = UniqueMap<T>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a keymap object without duplicate fields")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut access: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, T>()? {
                    if entries.contains_key(&key) {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate keymap field {key:?}"
                        )));
                    }
                    entries.insert(key, value);
                }
                Ok(UniqueMap(entries))
            }
        }
        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyMatch {
    Action(String),
    Pending,
    Unbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Stroke {
    code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug, Clone)]
struct Binding {
    keys: Vec<Stroke>,
    label: String,
}

/// Immutable, validated key bindings. Cloning is cheap enough for startup;
/// normal event handling only reads the pre-parsed bindings.
#[derive(Debug, Clone)]
pub struct Keymap {
    contexts: BTreeMap<String, BTreeMap<String, Vec<Binding>>>,
}

impl Default for Keymap {
    fn default() -> Self {
        Self::compile(default_config()).expect("built-in key bindings must be valid")
    }
}

impl Keymap {
    /// Apply an atomic set of per-action overrides to the built-in defaults.
    pub fn from_json(json: &str) -> Result<Self> {
        if json.len() as u64 > MAX_CONFIG_BYTES {
            bail!("keymap exceeds the 64 KiB limit");
        }
        let overrides: UniqueMap<UniqueMap<Vec<String>>> =
            serde_json::from_str(json).context("invalid keymap JSON")?;
        let mut merged = default_config();
        for (context, actions) in overrides.0 {
            let target = merged
                .get_mut(&context)
                .with_context(|| format!("unknown keymap context {context:?}"))?;
            for (action, keys) in actions.0 {
                if !target.contains_key(&action) {
                    bail!("unknown action {action:?} in context {context:?}");
                }
                target.insert(action, keys);
            }
        }
        Self::compile(merged)
    }

    /// Read at most 64 KiB from a regular file without following its final symlink.
    pub fn load(path: &Path) -> Result<Self> {
        let metadata = std::fs::symlink_metadata(path)
            .with_context(|| format!("cannot inspect keymap {}", path.display()))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            bail!("keymap must be a regular file, not a directory or symlink");
        }
        let mut options = OpenOptions::new();
        options.read(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
        }
        let file = options
            .open(path)
            .with_context(|| format!("cannot open keymap {}", path.display()))?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.len() > MAX_CONFIG_BYTES {
            bail!("keymap must be a regular file no larger than 64 KiB");
        }
        let mut bytes = Vec::new();
        file.take(MAX_CONFIG_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            bail!("keymap exceeds the 64 KiB limit");
        }
        let json = std::str::from_utf8(&bytes).context("keymap must contain UTF-8 JSON")?;
        Self::from_json(json).with_context(|| format!("invalid keymap {}", path.display()))
    }

    /// Resolve a complete sequence, or report that more keys are needed.
    /// Release events are ignored. The caller owns pending chord state and may
    /// retry an unmatched final key as a fresh sequence for responsive navigation.
    pub fn resolve(&self, context: &str, sequence: &[KeyEvent]) -> KeyMatch {
        let mut buffer = [Stroke {
            code: KeyCode::Null,
            modifiers: KeyModifiers::NONE,
        }; MAX_CHORD_KEYS];
        let mut length = 0;
        for event in sequence
            .iter()
            .filter(|event| event.kind != KeyEventKind::Release)
        {
            let key = normalize(event.code, event.modifiers);
            if is_emergency(&key) {
                return KeyMatch::Action("force_quit".to_owned());
            }
            if length < MAX_CHORD_KEYS {
                buffer[length] = key;
            }
            length += 1;
        }
        if length == 0 || length > MAX_CHORD_KEYS {
            return KeyMatch::Unbound;
        }
        let keys = &buffer[..length];
        let Some(actions) = self.contexts.get(context) else {
            return KeyMatch::Unbound;
        };
        for (action, bindings) in actions {
            for binding in bindings {
                if binding.keys == keys {
                    return KeyMatch::Action(action.clone());
                }
                if binding.keys.starts_with(keys) {
                    return KeyMatch::Pending;
                }
            }
        }
        KeyMatch::Unbound
    }

    /// Preferred shortcut for an action, suitable for menus and compact footers.
    /// Use `bindings` to render every alias in a full help view.
    pub fn label(&self, context: &str, action: &str) -> Option<String> {
        self.contexts
            .get(context)?
            .get(action)?
            .first()
            .map(|binding| binding.label.clone())
    }

    /// Stable action-ID order, including disabled actions with empty key lists.
    pub fn bindings(&self, context: &str) -> Vec<(String, Vec<String>)> {
        self.contexts
            .get(context)
            .into_iter()
            .flat_map(|actions| actions.iter())
            .map(|(action, bindings)| {
                (
                    action.clone(),
                    bindings.iter().map(|b| b.label.clone()).collect(),
                )
            })
            .collect()
    }

    pub fn contexts(&self) -> Vec<&str> {
        self.contexts.keys().map(String::as_str).collect()
    }

    /// Export a complete, editable JSON configuration with canonical key names.
    pub fn to_json(&self) -> Result<String> {
        let config: Config = self
            .contexts()
            .into_iter()
            .map(|context| {
                (
                    context.to_owned(),
                    self.bindings(context).into_iter().collect(),
                )
            })
            .collect();
        Ok(serde_json::to_string_pretty(&config)?)
    }

    fn compile(mut config: Config) -> Result<Self> {
        let mut contexts = BTreeMap::new();
        for (context, actions) in &mut config {
            // Keep emergency exit visible even when all other force-quit keys
            // are disabled. No other action or chord may contain this key.
            let emergency = parse_stroke("Ctrl+c")?;
            let force_keys = actions.entry("force_quit".to_owned()).or_default();
            let has_emergency = force_keys.iter().any(|s| {
                parse_sequence(s).is_ok_and(|keys| keys.len() == 1 && is_emergency(&keys[0]))
            });
            if !has_emergency {
                force_keys.push("Ctrl+c".to_owned());
            }
            let mut parsed = BTreeMap::new();
            let mut used: Vec<(String, Vec<Stroke>)> = Vec::new();
            for (action, bindings) in actions.iter() {
                if bindings.len() > MAX_BINDINGS_PER_ACTION {
                    bail!("{context}.{action} has more than 16 bindings");
                }
                let mut result = Vec::new();
                for binding in bindings {
                    let keys = parse_sequence(binding)
                        .with_context(|| format!("invalid binding for {context}.{action}"))?;
                    if keys.iter().any(is_emergency)
                        && !(action == "force_quit" && keys.len() == 1 && keys[0] == emergency)
                    {
                        bail!("Ctrl+c is reserved for emergency exit and cannot appear in chords");
                    }
                    for (other_action, other_keys) in &used {
                        if keys.starts_with(other_keys) || other_keys.starts_with(&keys) {
                            bail!(
                                "conflicting or ambiguous bindings in {context}: {action} ({binding}) and {other_action}"
                            );
                        }
                    }
                    used.push((action.clone(), keys.clone()));
                    result.push(Binding {
                        label: keys.iter().map(stroke_label).collect::<Vec<_>>().join(" "),
                        keys,
                    });
                }
                parsed.insert(action.clone(), result);
            }
            if MODAL_CONTEXTS.contains(&context.as_str())
                && parsed.get("cancel").is_none_or(Vec::is_empty)
            {
                bail!("{context}.cancel must retain at least one reachable shortcut");
            }
            contexts.insert(context.clone(), parsed);
        }
        Ok(Self { contexts })
    }
}

fn parse_sequence(sequence: &str) -> Result<Vec<Stroke>> {
    let tokens: Vec<_> = sequence.split_whitespace().collect();
    if tokens.is_empty() || tokens.len() > MAX_CHORD_KEYS {
        bail!("a key sequence must contain between 1 and 4 keys");
    }
    tokens.into_iter().map(parse_stroke).collect()
}

fn parse_stroke(token: &str) -> Result<Stroke> {
    let mut rest = token;
    let mut modifiers = KeyModifiers::NONE;
    while rest != "+" {
        let Some((prefix, tail)) = rest.split_once('+') else {
            break;
        };
        let modifier = match prefix.to_ascii_lowercase().as_str() {
            "ctrl" => KeyModifiers::CONTROL,
            "alt" => KeyModifiers::ALT,
            "shift" => KeyModifiers::SHIFT,
            "super" => KeyModifiers::SUPER,
            _ => bail!("unknown modifier {prefix:?}"),
        };
        if modifiers.contains(modifier) {
            bail!("repeated modifier {prefix:?}");
        }
        modifiers.insert(modifier);
        rest = tail;
    }
    let code = match rest.to_ascii_lowercase().as_str() {
        "enter" => KeyCode::Enter,
        "esc" => KeyCode::Esc,
        "space" => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        _ if rest.len() > 1 && rest.starts_with(['F', 'f']) => {
            let number: u8 = rest[1..].parse().context("invalid function key")?;
            if !(1..=24).contains(&number) {
                bail!("function keys must be F1 through F24");
            }
            KeyCode::F(number)
        }
        _ => {
            let mut chars = rest.chars();
            let ch = chars.next().context("missing key after modifier")?;
            if chars.next().is_some() || ch.is_control() || ch.is_whitespace() {
                bail!("unknown key {rest:?}; use a single character or a named key");
            }
            KeyCode::Char(ch)
        }
    };
    Ok(normalize(code, modifiers))
}

fn normalize(mut code: KeyCode, mut modifiers: KeyModifiers) -> Stroke {
    if let KeyCode::Char(ch) = code {
        if modifiers.contains(KeyModifiers::SHIFT) {
            code = KeyCode::Char(ch.to_ascii_uppercase());
            modifiers.remove(KeyModifiers::SHIFT);
        }
    } else if code == KeyCode::Tab && modifiers.contains(KeyModifiers::SHIFT) {
        code = KeyCode::BackTab;
        modifiers.remove(KeyModifiers::SHIFT);
    } else if code == KeyCode::BackTab {
        modifiers.remove(KeyModifiers::SHIFT);
    }
    // Terminals disagree about whether Ctrl+C retains uppercase/Shift. Treat
    // every Ctrl+C variant as the same reserved emergency key.
    if modifiers.contains(KeyModifiers::CONTROL) && matches!(code, KeyCode::Char('c' | 'C')) {
        code = KeyCode::Char('c');
        modifiers = KeyModifiers::CONTROL;
    }
    Stroke { code, modifiers }
}

fn is_emergency(key: &Stroke) -> bool {
    key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c' | 'C'))
}

fn stroke_label(stroke: &Stroke) -> String {
    let mut label = String::new();
    for (modifier, name) in [
        (KeyModifiers::CONTROL, "Ctrl+"),
        (KeyModifiers::ALT, "Alt+"),
        (KeyModifiers::SHIFT, "Shift+"),
        (KeyModifiers::SUPER, "Super+"),
    ] {
        if stroke.modifiers.contains(modifier) {
            label.push_str(name);
        }
    }
    let key = match stroke.code {
        KeyCode::Char(' ') => "Space".to_owned(),
        KeyCode::Char(ch) => ch.to_string(),
        KeyCode::F(n) => format!("F{n}"),
        KeyCode::Enter => "Enter".to_owned(),
        KeyCode::Esc => "Esc".to_owned(),
        KeyCode::Tab => "Tab".to_owned(),
        KeyCode::BackTab => "BackTab".to_owned(),
        KeyCode::Backspace => "Backspace".to_owned(),
        KeyCode::Delete => "Delete".to_owned(),
        KeyCode::Insert => "Insert".to_owned(),
        KeyCode::Home => "Home".to_owned(),
        KeyCode::End => "End".to_owned(),
        KeyCode::Up => "Up".to_owned(),
        KeyCode::Down => "Down".to_owned(),
        KeyCode::Left => "Left".to_owned(),
        KeyCode::Right => "Right".to_owned(),
        KeyCode::PageUp => "PageUp".to_owned(),
        KeyCode::PageDown => "PageDown".to_owned(),
        _ => unreachable!("only configurable keys are formatted"),
    };
    label.push_str(&key);
    label
}

fn map(entries: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    entries
        .iter()
        .map(|(action, keys)| {
            (
                (*action).to_owned(),
                keys.iter().map(|key| (*key).to_owned()).collect(),
            )
        })
        .collect()
}

fn default_config() -> Config {
    let files = map(&[
        ("down", &["j", "Down"]),
        ("up", &["k", "Up"]),
        ("half_down", &["Ctrl+d", "PageDown"]),
        ("half_up", &["Ctrl+u", "PageUp"]),
        ("first", &["g g", "Home"]),
        ("last", &["G", "End"]),
        ("parent", &["h", "Backspace"]),
        ("open", &["Enter", "l"]),
        ("open_goto", &[":"]),
        ("open_filter", &["/"]),
        ("toggle_hidden", &["."]),
        ("toggle_select", &["Space"]),
        ("select_all", &["a"]),
        ("clear_selection", &["A"]),
        ("copy", &["y y"]),
        ("copy_name", &["y f"]),
        ("copy_relative_path", &["y r"]),
        ("copy_absolute_path", &["y a"]),
        ("cut", &["x"]),
        ("paste", &["p"]),
        ("copy_to", &["c"]),
        ("move_to", &["m"]),
        ("rename", &["n"]),
        ("trash", &["d"]),
        ("permanent_delete", &["D"]),
        ("refresh", &["r"]),
        ("cycle_sort", &["s"]),
        ("reverse_sort", &["S"]),
        ("edit", &["e"]),
        ("open_help", &["?"]),
        ("quit", &["q"]),
        ("force_quit", &["Q", "Ctrl+c"]),
        ("cancel", &["Esc", "Ctrl+g"]),
        ("open_actions", &["o"]),
        ("open_jobs", &["J"]),
        ("cancel_job", &["Ctrl+x"]),
        ("open_trash", &["u"]),
    ]);
    let preview = map(&[
        ("down", &["Down", "Ctrl+f"]),
        ("up", &["Up", "Ctrl+b"]),
        ("half_down", &["j", "PageDown", "Space"]),
        ("half_up", &["k", "PageUp"]),
        ("first", &["g g", "Home"]),
        ("last", &["G", "End"]),
        ("cancel", &["Esc", "Ctrl+g", "Enter", "q"]),
        ("force_quit", &["Q", "Ctrl+c"]),
        ("edit", &["e"]),
        ("open_preview_search", &["/"]),
        ("preview_search_next", &["n"]),
        ("preview_search_prev", &["N"]),
        ("open_actions", &["o"]),
        ("open_jobs", &["J"]),
        ("open_trash", &["u"]),
        ("cancel_job", &["Ctrl+x"]),
    ]);
    let input = map(&[
        ("submit", &["Enter"]),
        ("backspace", &["Backspace"]),
        ("cancel", &["Esc", "Ctrl+g"]),
        ("force_quit", &["Ctrl+c"]),
    ]);
    let help = map(&[
        ("cancel", &["Esc", "Ctrl+g", "Enter", "q", "?"]),
        ("force_quit", &["Q", "Ctrl+c"]),
        ("down", &["j", "Down", "PageDown"]),
        ("up", &["k", "Up", "PageUp"]),
        ("first", &["g g", "Home"]),
        ("last", &["G", "End"]),
    ]);
    let actions = map(&[
        ("submit", &["Enter"]),
        ("backspace", &["Backspace"]),
        ("down", &["Down", "Tab"]),
        ("up", &["Up", "BackTab"]),
        ("cancel", &["Esc", "Ctrl+g"]),
        ("force_quit", &["Ctrl+c"]),
    ]);
    let cluster = map(&[
        ("down", &["j", "Down"]),
        ("up", &["k", "Up"]),
        ("first", &["Home", "g g"]),
        ("last", &["End", "G"]),
        ("refresh_all", &["r"]),
        ("refresh_selected", &["Enter"]),
        ("open_session", &["s"]),
        ("open_workbench", &["t"]),
        ("open_detail", &["l"]),
        ("open_help", &["?"]),
        ("quit", &["q"]),
        ("force_quit", &["Q", "Ctrl+c"]),
        ("cancel", &["Esc", "Ctrl+g"]),
        ("open_actions", &["o"]),
        ("open_filter", &["/"]),
        ("cycle_sort", &["v"]),
        ("reverse_sort", &["V"]),
        ("clear_filter", &["Backspace"]),
    ]);
    let mut detail = cluster.clone();
    detail.insert("quit".to_owned(), Vec::new());
    detail.insert(
        "cancel".to_owned(),
        vec!["Esc".to_owned(), "Ctrl+g".to_owned(), "q".to_owned()],
    );
    detail.insert("detail_down".to_owned(), vec!["PageDown".to_owned()]);
    detail.insert("detail_up".to_owned(), vec!["PageUp".to_owned()]);
    let trash = map(&[
        ("down", &["j", "Down"]),
        ("up", &["k", "Up"]),
        ("first", &["g g", "Home"]),
        ("last", &["G", "End"]),
        ("restore", &["Enter"]),
        ("refresh", &["r"]),
        ("cancel", &["Esc", "Ctrl+g", "q"]),
        ("force_quit", &["Q", "Ctrl+c"]),
    ]);
    let jobs = map(&[
        ("down", &["j", "Down"]),
        ("up", &["k", "Up"]),
        ("first", &["g g", "Home"]),
        ("last", &["G", "End"]),
        ("cancel_job", &["Ctrl+x", "x"]),
        ("cancel", &["Esc", "Ctrl+g", "q", "Enter"]),
        ("force_quit", &["Q", "Ctrl+c"]),
    ]);
    [
        ("files", files),
        ("preview", preview),
        ("input", input.clone()),
        ("help", help),
        ("actions", actions),
        ("cluster", cluster),
        ("cluster_detail", detail),
        ("cluster_filter", input),
        ("trash", trash),
        ("jobs", jobs),
    ]
    .into_iter()
    .map(|(context, actions)| (context.to_owned(), actions))
    .collect()
}
