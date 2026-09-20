use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tersh::keymap::{KeyMatch, Keymap};

fn key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)
}
fn action(name: &str) -> KeyMatch {
    KeyMatch::Action(name.to_owned())
}

#[test]
fn default_navigation_and_chords_are_complete() {
    let map = Keymap::default();
    assert_eq!(map.resolve("files", &[key('j')]), action("down"));
    assert_eq!(map.resolve("files", &[key('g')]), KeyMatch::Pending);
    assert_eq!(map.resolve("files", &[key('g'), key('g')]), action("first"));
    assert_eq!(
        map.resolve("files", &[key('y'), key('f')]),
        action("copy_name")
    );
    assert_eq!(map.resolve("preview", &[key('j')]), action("half_down"));
    assert_eq!(map.resolve("files", &[key('J')]), action("open_jobs"));
    assert_eq!(map.resolve("files", &[key('u')]), action("open_trash"));
    assert_eq!(map.resolve("preview", &[key('u')]), action("open_trash"));
    assert_eq!(map.resolve("cluster", &[key('V')]), action("reverse_sort"));
}

#[test]
fn overrides_replace_instead_of_adding_legacy_keys() {
    let map = Keymap::from_json(r#"{"files":{"down":["Ctrl+n"]}}"#).unwrap();
    assert_eq!(map.resolve("files", &[key('j')]), KeyMatch::Unbound);
    assert_eq!(
        map.resolve("files", &[KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)]),
        KeyMatch::Unbound
    );
    assert_eq!(
        map.resolve(
            "files",
            &[KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL)]
        ),
        action("down")
    );
    assert_eq!(map.resolve("cluster", &[key('j')]), action("down"));
    assert_eq!(map.label("files", "down").as_deref(), Some("Ctrl+n"));
}

#[test]
fn disable_and_swap_are_merged_atomically() {
    let map = Keymap::from_json(r#"{"files":{"down":["k"],"up":["j"],"copy":[]}}"#).unwrap();
    assert_eq!(map.resolve("files", &[key('k')]), action("down"));
    assert_eq!(map.resolve("files", &[key('j')]), action("up"));
    assert_eq!(
        map.resolve("files", &[key('y'), key('y')]),
        KeyMatch::Unbound
    );
    assert_eq!(map.label("files", "copy"), None);
}

#[test]
fn invalid_maps_cannot_silently_shadow_bindings() {
    for config in [
        r#"{"unknown":{}}"#,
        r#"{"files":{},"files":{}}"#,
        r#"{"files":{"down":["z"],"down":["w"]}}"#,
        r#"{"files":{"down":["z","Shift+z"]},"files":{}}"#,
        r#"{"files":{"nonsense":["z"]}}"#,
        r#"{"files":{"down":["k"]}}"#,
        r#"{"files":{"down":["z","z"]}}"#,
        r#"{"files":{"down":["g"]}}"#,
        r#"{"files":{"down":["g g x"]}}"#,
        r#"{"files":{"down":["Hyper+x"]}}"#,
        r#"{"files":{"down":["Ctrl+"]}}"#,
        r#"{"files":{"down":["Ctrl+Ctrl+x"]}}"#,
        r#"{"files":{"down":[""]}}"#,
        r#"{"files":{"down":["a b c d e"]}}"#,
        r#"{"input":{"cancel":[]}}"#,
        r#"{"actions":{"cancel":[]}}"#,
        r#"{"trash":{"cancel":[]}}"#,
        r#"{"files":{"down":["Ctrl+c"]}}"#,
        r#"{"files":{"down":["z Ctrl+c"]}}"#,
    ] {
        assert!(Keymap::from_json(config).is_err(), "accepted {config}");
    }
}

#[test]
fn prompt_controls_and_emergency_exit_are_independent_of_text() {
    let map = Keymap::from_json(
        r#"{"input":{"submit":["Alt+s"],"cancel":["Ctrl+q"]},"files":{"force_quit":[]}}"#,
    )
    .unwrap();
    assert_eq!(map.resolve("input", &[key('q')]), KeyMatch::Unbound);
    assert_eq!(
        map.resolve(
            "input",
            &[KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)]
        ),
        action("submit")
    );
    assert_eq!(
        map.resolve(
            "input",
            &[KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL)]
        ),
        action("cancel")
    );
    for context in ["files", "input", "actions", "unknown"] {
        assert_eq!(
            map.resolve(
                context,
                &[
                    key('g'),
                    KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)
                ]
            ),
            action("force_quit")
        );
    }
    assert!(map.label("files", "force_quit").unwrap().contains("Ctrl+c"));
}

#[test]
fn terminal_shift_variants_and_release_events_are_normalized() {
    let map = Keymap::default();
    for event in [
        key('G'),
        KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::Char('g'), KeyModifiers::SHIFT),
    ] {
        assert_eq!(map.resolve("files", &[event]), action("last"));
    }
    assert_eq!(
        map.resolve(
            "files",
            &[KeyEvent::new(KeyCode::Char('j'), KeyModifiers::ALT)]
        ),
        KeyMatch::Unbound
    );
    let mut release = key('j');
    release.kind = KeyEventKind::Release;
    assert_eq!(map.resolve("files", &[release]), KeyMatch::Unbound);
    assert_eq!(
        map.resolve("files", &[key('g'), release]),
        KeyMatch::Pending
    );
}

#[test]
fn exported_defaults_roundtrip_and_describe_every_context() {
    let map = Keymap::default();
    let json = map.to_json().unwrap();
    let parsed = Keymap::from_json(&json).unwrap();
    assert_eq!(json, parsed.to_json().unwrap());
    assert_eq!(map.contexts().len(), 10);
    for context in map.contexts() {
        for (id, keys) in map.bindings(context) {
            assert_eq!(map.label(context, &id), keys.first().cloned());
        }
    }
}

#[test]
fn loading_is_bounded_and_only_accepts_regular_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("keys.json");
    std::fs::write(&path, "{}").unwrap();
    assert!(Keymap::load(&path).is_ok());
    assert!(Keymap::load(dir.path()).is_err());
    std::fs::write(&path, vec![b' '; 65537]).unwrap();
    assert!(Keymap::load(&path).is_err());
    #[cfg(unix)]
    {
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(Keymap::load(&link).is_err());
    }
}

#[test]
fn named_keys_modifiers_and_canonical_labels_resolve_consistently() {
    let map =
        Keymap::from_json(r#"{"files":{"down":["alt+F2","Ctrl++","Shift+z"],"up":["Shift+Tab"]}}"#)
            .unwrap();
    for event in [
        KeyEvent::new(KeyCode::F(2), KeyModifiers::ALT),
        KeyEvent::new(KeyCode::Char('+'), KeyModifiers::CONTROL),
        key('Z'),
    ] {
        assert_eq!(map.resolve("files", &[event]), action("down"));
    }
    for event in [
        KeyEvent::new(KeyCode::Tab, KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT),
        KeyEvent::new(KeyCode::BackTab, KeyModifiers::NONE),
    ] {
        assert_eq!(map.resolve("files", &[event]), action("up"));
    }
    assert_eq!(map.label("files", "down").as_deref(), Some("Alt+F2"));
    assert_eq!(map.label("files", "up").as_deref(), Some("BackTab"));
    assert_eq!(
        Keymap::from_json(&map.to_json().unwrap())
            .unwrap()
            .to_json()
            .unwrap(),
        map.to_json().unwrap()
    );
}

#[test]
fn every_modal_context_can_cancel_and_is_configurable() {
    for context in [
        "preview",
        "input",
        "help",
        "actions",
        "cluster_detail",
        "cluster_filter",
        "trash",
        "jobs",
    ] {
        let config = format!(r#"{{"{context}":{{"cancel":["Alt+z"]}}}}"#);
        let map = Keymap::from_json(&config).unwrap();
        assert_eq!(
            map.resolve(context, &[KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)]),
            KeyMatch::Unbound
        );
        assert_eq!(
            map.resolve(
                context,
                &[KeyEvent::new(KeyCode::Char('z'), KeyModifiers::ALT)]
            ),
            action("cancel")
        );
        let disabled = format!(r#"{{"{context}":{{"cancel":[]}}}}"#);
        assert!(Keymap::from_json(&disabled).is_err());
    }
}

#[test]
fn emergency_key_is_canonical_and_release_does_not_exit() {
    let map = Keymap::from_json(r#"{"files":{"force_quit":["Ctrl+C"]}}"#).unwrap();
    assert_eq!(map.label("files", "force_quit").as_deref(), Some("Ctrl+c"));
    for event in [
        KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL),
        KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        ),
        KeyEvent::new(
            KeyCode::Char('C'),
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ),
    ] {
        assert_eq!(map.resolve("input", &[event]), action("force_quit"));
    }
    let mut release = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    release.kind = KeyEventKind::Release;
    assert_eq!(map.resolve("input", &[release]), KeyMatch::Unbound);
    assert!(Keymap::from_json(r#"{"files":{"force_quit":["Ctrl+c","Ctrl+C"]}}"#).is_err());
}

#[test]
fn example_remap_is_collision_free_and_chords_preserve_prefix_state() {
    let map = Keymap::from_json(
        r#"{
        "files": {
            "copy": ["Ctrl+y"],
            "open_jobs": ["F5", "J"],
            "open_trash": ["F6", "u"]
        },
        "actions": {
            "down": ["Down", "Tab", "Ctrl+n"],
            "up": ["Up", "BackTab", "Ctrl+p"]
        },
        "input": {"submit": ["Enter", "z z"]}
    }"#,
    )
    .unwrap();
    assert_eq!(
        map.resolve("files", &[KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE)]),
        action("open_jobs")
    );
    assert_eq!(
        map.resolve("files", &[key('y'), key('y')]),
        KeyMatch::Unbound
    );
    assert_eq!(
        map.resolve("files", &[key('y'), key('f')]),
        action("copy_name")
    );
    assert_eq!(map.resolve("input", &[key('z')]), KeyMatch::Pending);
    assert_eq!(
        map.resolve("input", &[key('z'), key('x')]),
        KeyMatch::Unbound
    );
    assert_eq!(
        map.resolve("input", &[key('z'), key('z')]),
        action("submit")
    );
}

#[test]
fn equivalent_terminal_key_spellings_cannot_shadow_each_other() {
    for config in [
        r#"{"files":{"down":["Z","Shift+z"]}}"#,
        r#"{"files":{"down":["Shift+Tab","BackTab"]}}"#,
        r#"{"files":{"down":["Ctrl+Z"],"up":["Ctrl+Shift+z"]}}"#,
    ] {
        assert!(Keymap::from_json(config).is_err());
    }
}
