use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use tersh::{
    app::{App, Command, Mode},
    places::Places,
};

fn type_text(app: &mut App, text: &str) {
    for ch in text.chars() {
        app.handle_command(Command::Input(ch));
    }
}
fn settle(app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while app.directory_loading()
        || app.preview_loading()
        || app.trash_loading()
        || app.job_active()
    {
        assert!(Instant::now() < deadline, "operation did not settle");
        app.poll_readers();
        app.poll_job();
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn places_are_scoped_searchable_pinned_and_open_existing_directories() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("project");
    fs::create_dir(&child).unwrap();
    let mut places = Places::memory();
    places.visit("local", "local", &child).unwrap();
    places
        .visit("other", "other", Path::new("/remote-only"))
        .unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.set_places(places);
    app.handle_command(Command::OpenPlaces);
    assert_eq!(app.mode(), Mode::Places);
    type_text(&mut app, "project");
    assert_eq!(app.places_entries().len(), 1);
    assert_eq!(app.places_query(), "project");
    app.handle_command(Command::PinPlace);
    assert!(app.places_entries()[0].pinned);
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert_eq!(app.cwd(), child.canonicalize().unwrap());
}

#[test]
fn structured_preview_and_inspector_are_explicit_toggles() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("sample.json");
    fs::write(&path, "{\"nested\":{\"value\":3}}").unwrap();
    let mut app = App::new_async(path).unwrap();
    settle(&mut app);
    let raw = app.preview().lines.clone();
    app.handle_command(Command::ToggleStructured);
    settle(&mut app);
    assert!(app.structured_preview());
    assert_ne!(app.preview().lines, raw);
    app.handle_command(Command::ToggleStructured);
    settle(&mut app);
    assert_eq!(app.preview().lines, raw);
    let visible = app.inspector_visible();
    app.handle_command(Command::ToggleInspector);
    assert_ne!(app.inspector_visible(), visible);
}

#[test]
fn opening_log_creates_read_session_and_pause_freezes_visible_contents() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("output.log");
    fs::write(&path, "first\n").unwrap();
    let mut app = App::new(path.clone()).unwrap();
    app.handle_command(Command::OpenLog);
    assert_eq!(app.mode(), Mode::Log);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.log_snapshot().is_none() {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    app.handle_command(Command::ToggleLogPause);
    assert!(app.log_paused());
    let snapshot = app.log_snapshot().unwrap().lines.clone();
    fs::write(&path, "first\nsecond\n").unwrap();
    let until = Instant::now() + Duration::from_millis(300);
    while Instant::now() < until {
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(app.log_snapshot().unwrap().lines, snapshot);
    app.handle_command(Command::Cancel);
    assert_eq!(app.mode(), Mode::Preview);
    assert!(app.log_snapshot().is_none());
}

#[test]
fn completed_jobs_survive_new_jobs_in_bounded_history() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::create_dir(&first).unwrap();
    fs::create_dir(&second).unwrap();
    let path = root.path().join("source.txt");
    fs::write(&path, "source").unwrap();
    let mut app = App::new(path).unwrap();
    app.handle_command(Command::Cancel);
    for destination in [&first, &second] {
        app.handle_command(Command::CopyTo);
        type_text(&mut app, destination.to_str().unwrap());
        app.handle_command(Command::Submit);
        settle(&mut app);
    }
    assert_eq!(app.job_history().entries().len(), 2);
    assert_eq!(app.selected_job().unwrap().counts.succeeded, 1);
}

#[test]
fn retry_only_rechecks_failed_items_and_never_repeats_completed_sources() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.txt"), "original-a").unwrap();
    fs::write(root.path().join("b.txt"), "original-b").unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::SelectAll);
    app.handle_command(Command::CopyTo);
    type_text(&mut app, destination.path().to_str().unwrap());
    fs::write(root.path().join("b.txt"), "changed-b-identity").unwrap();
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert_eq!(app.selected_job().unwrap().counts.succeeded, 1);
    assert_eq!(app.selected_job().unwrap().counts.failed, 1);
    fs::write(root.path().join("a.txt"), "do-not-recopy-a").unwrap();
    app.handle_command(Command::OpenJobs);
    app.handle_command(Command::RetryJob);
    settle(&mut app);
    assert_eq!(app.selected_job().unwrap().counts.total, 1);
    assert_eq!(app.selected_job().unwrap().counts.succeeded, 1);
    assert_eq!(
        fs::read_to_string(destination.path().join("a.txt")).unwrap(),
        "original-a"
    );
    assert_eq!(
        fs::read_to_string(destination.path().join("b.txt")).unwrap(),
        "changed-b-identity"
    );
}

#[test]
fn retry_copy_conflict_requires_a_new_decision_and_export_never_overwrites() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let source = root.path().join("a.txt");
    fs::write(&source, "before").unwrap();
    let mut app = App::new(source.clone()).unwrap();
    app.handle_command(Command::Cancel);
    app.handle_command(Command::CopyTo);
    type_text(&mut app, destination.path().to_str().unwrap());
    fs::write(&source, "changed-content").unwrap();
    app.handle_command(Command::Submit);
    settle(&mut app);
    fs::write(destination.path().join("a.txt"), "existing-target").unwrap();
    app.handle_command(Command::OpenJobs);
    app.handle_command(Command::RetryJob);
    assert_eq!(app.mode(), Mode::Conflict);
    assert_eq!(
        fs::read_to_string(destination.path().join("a.txt")).unwrap(),
        "existing-target"
    );
    app.handle_command(Command::Cancel);
    app.handle_command(Command::OpenJobs);
    let export = root.path().join("report.json");
    fs::write(&export, "retain me").unwrap();
    app.handle_command(Command::ExportJob);
    type_text(&mut app, export.to_str().unwrap());
    app.handle_command(Command::Submit);
    assert_eq!(app.mode(), Mode::ExportJob);
    assert_eq!(app.input(), export.to_str().unwrap());
    assert!(app.navigation_error().unwrap().contains("export failed"));
    assert_eq!(fs::read_to_string(&export).unwrap(), "retain me");
    app.handle_command(Command::Cancel);
    app.handle_command(Command::ExportJob);
    let fresh = root.path().join("fresh.json");
    type_text(&mut app, fresh.to_str().unwrap());
    app.handle_command(Command::Submit);
    assert_eq!(app.mode(), Mode::Jobs);
    assert!(!fs::read(&fresh).unwrap().is_empty());
}

#[test]
fn retry_permanent_delete_requires_typed_confirmation_again() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("keep.txt");
    fs::write(&path, "before").unwrap();
    let mut app = App::new(path.clone()).unwrap();
    app.handle_command(Command::Cancel);
    app.handle_command(Command::PermanentDelete);
    fs::write(&path, "changed-before-confirmation").unwrap();
    type_text(&mut app, "delete");
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert!(path.exists());
    assert_eq!(app.selected_job().unwrap().counts.failed, 1);
    app.handle_command(Command::OpenJobs);
    app.handle_command(Command::RetryJob);
    assert_eq!(app.mode(), Mode::ConfirmDelete);
    assert!(app.input().is_empty());
    assert!(path.exists());
    app.handle_command(Command::Cancel);
    assert!(path.exists());
}

#[test]
fn places_clear_retains_pins_and_failed_open_keeps_query() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing-project");
    let mut places = Places::memory();
    places.toggle_pin("local", "local", &missing).unwrap();
    places.visit("local", "local", root.path()).unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.set_places(places);
    app.handle_command(Command::OpenPlaces);
    app.handle_command(Command::ClearRecent);
    assert_eq!(app.places_entries().len(), 1);
    type_text(&mut app, "missing");
    app.handle_command(Command::Submit);
    assert_eq!(app.mode(), Mode::Places);
    assert_eq!(app.places_query(), "missing");
    assert!(app.navigation_error().is_some());
    assert!(!missing.exists());
    app.handle_command(Command::RemovePlace);
    assert!(app.places_entries().is_empty());
    app.set_places(Places::disabled());
    app.handle_command(Command::Cancel);
    app.handle_command(Command::PinPlace);
    assert!(!app.places_enabled());
    assert!(app.places_entries().is_empty());
}

#[test]
fn log_follow_fills_viewport_scroll_and_search_pause_and_resume_catches_up() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("output.log");
    fs::write(
        &path,
        (0..50).map(|n| format!("row-{n:02}\n")).collect::<String>(),
    )
    .unwrap();
    let mut app = App::new(path.clone()).unwrap();
    app.set_log_view_lines(10);
    app.handle_command(Command::OpenLog);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.log_snapshot().is_none() {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(app.preview_offset(), 40);
    app.handle_command(Command::Up);
    assert!(app.log_paused());
    assert_eq!(app.preview_offset(), 39);
    app.handle_command(Command::OpenPreviewSearch);
    assert_eq!(app.mode(), Mode::LogSearch);
    type_text(&mut app, "row-05");
    app.handle_command(Command::Submit);
    assert_eq!(app.mode(), Mode::Log);
    assert!(app.log_paused());
    assert_eq!(app.preview_offset(), 5);
    use std::io::Write;
    writeln!(
        fs::OpenOptions::new().append(true).open(&path).unwrap(),
        "row-50"
    )
    .unwrap();
    app.handle_command(Command::ToggleLogPause);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.log_snapshot().unwrap().lines.len() != 51 {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(!app.log_paused());
    assert_eq!(app.preview_offset(), 41);
    app.handle_command(Command::OpenJobs);
    assert!(app.log_snapshot().is_none());
}

#[test]
fn action_menu_explains_unavailable_file_features_and_has_shared_metadata() {
    let root = tempfile::tempdir().unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::OpenActions);
    let actions = app.actions().unwrap().matches();
    let log = actions
        .iter()
        .find(|action| action.command == Command::OpenLog)
        .unwrap();
    assert!(log.disabled_reason.is_some());
    assert!(!log.description.is_empty());
    assert_ne!(log.group, "Actions");
}

#[test]
fn asynchronous_missing_place_preserves_search_and_places_screen() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("missing-project");
    let mut places = Places::memory();
    places.visit("local", "local", &missing).unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    app.set_places(places);
    app.handle_command(Command::OpenPlaces);
    type_text(&mut app, "missing");
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert_eq!(app.mode(), Mode::Places);
    assert_eq!(app.places_query(), "missing");
    assert!(app.navigation_error().is_some());
    assert!(!missing.exists());
}

#[test]
fn following_log_rebuilds_search_matches_after_ring_eviction() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ring.log");
    fs::write(
        &path,
        (0..2000)
            .map(|n| format!("row-{n:04}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut app = App::new(path.clone()).unwrap();
    app.handle_command(Command::OpenLog);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.log_snapshot().is_none() {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    app.handle_command(Command::OpenPreviewSearch);
    type_text(&mut app, "row-0000");
    app.handle_command(Command::Submit);
    assert_eq!(app.preview_matches(), [0]);
    use std::io::Write;
    writeln!(
        fs::OpenOptions::new().append(true).open(&path).unwrap(),
        "row-2000"
    )
    .unwrap();
    app.handle_command(Command::ToggleLogPause);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.preview().lines.first().map(String::as_str) != Some("row-0001") {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(
        app.preview_matches().is_empty(),
        "evicted text must not leave a match on another line"
    );
    app.handle_command(Command::PreviewSearchNext);
    assert_eq!(app.preview_active_match(), None);
}

#[test]
fn finishing_background_job_does_not_replace_active_log_with_regular_preview() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let path = root.path().join("job.log");
    fs::write(
        &path,
        (0..3000)
            .map(|n| format!("row-{n:04}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let mut app = App::new_async(path).unwrap();
    settle(&mut app);
    app.handle_command(Command::Cancel);
    app.handle_command(Command::CopyTo);
    type_text(&mut app, destination.path().to_str().unwrap());
    app.handle_command(Command::Submit);
    app.handle_command(Command::OpenLog);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.log_snapshot().is_none() {
        assert!(Instant::now() < deadline);
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.job_active() || app.directory_loading() {
        assert!(Instant::now() < deadline);
        app.poll_job();
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(app.mode(), Mode::Log);
    assert!(
        !app.preview_loading(),
        "background refresh queued a regular preview behind Log"
    );
    assert_eq!(app.preview().lines, app.log_snapshot().unwrap().lines);
}

#[test]
fn finishing_background_job_preserves_pending_explicit_navigation() {
    let root = tempfile::tempdir().unwrap();
    let destination = tempfile::tempdir().unwrap();
    let wanted = tempfile::tempdir().unwrap();
    let source = root.path().join("source.txt");
    fs::write(&source, "source").unwrap();
    let mut app = App::new_async(source).unwrap();
    settle(&mut app);
    app.handle_command(Command::Cancel);
    app.handle_command(Command::CopyTo);
    type_text(&mut app, destination.path().to_str().unwrap());
    app.handle_command(Command::Submit);
    app.handle_command(Command::OpenGoto);
    type_text(&mut app, wanted.path().to_str().unwrap());
    app.handle_command(Command::Submit);
    let deadline = Instant::now() + Duration::from_secs(2);
    while app.job_active() {
        assert!(Instant::now() < deadline);
        app.poll_job();
        std::thread::sleep(Duration::from_millis(2));
    }
    settle(&mut app);
    assert_eq!(app.cwd(), wanted.path().canonicalize().unwrap());
    assert_eq!(app.mode(), Mode::Normal);
    assert!(app.input().is_empty());
}
