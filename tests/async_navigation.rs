use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use tersh::app::{App, Command, Mode};

fn settle(app: &mut App) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while app.directory_loading()
        || app.preview_loading()
        || app.trash_loading()
        || app.job_active()
    {
        assert!(Instant::now() < deadline, "reader did not settle");
        app.poll_readers();
        app.poll_job();
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn goto(app: &mut App, path: &Path) {
    app.handle_command(Command::OpenGoto);
    for ch in path.to_str().unwrap().chars() {
        app.handle_command(Command::Input(ch));
    }
    app.handle_command(Command::Submit);
}

#[test]
fn async_startup_defers_listing_and_loads_real_contents() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("file.txt"), "hello").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    assert!(
        app.directory_loading(),
        "interactive startup must defer directory I/O"
    );
    assert!(app.entries().is_empty());
    settle(&mut app);
    assert_eq!(app.entries().len(), 1);
    assert!(
        app.preview()
            .lines
            .iter()
            .any(|line| line.contains("hello"))
    );
}

#[test]
fn parent_restores_the_exited_child_instead_of_first_row() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("a-first")).unwrap();
    fs::create_dir(root.path().join("z-child")).unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    app.handle_command(Command::Last);
    app.handle_command(Command::Open);
    app.handle_command(Command::Parent);
    assert_eq!(app.entries()[app.cursor()].name, "z-child");
}

#[test]
fn failed_goto_keeps_editable_input_and_current_directory() {
    let root = tempfile::tempdir().unwrap();
    let mut app = App::new(root.path().into()).unwrap();
    let missing = root.path().join("missing");
    goto(&mut app, &missing);
    assert_eq!(app.mode(), Mode::Goto);
    assert_eq!(app.input(), missing.to_str().unwrap());
    assert_eq!(app.cwd(), root.path().canonicalize().unwrap());
    assert!(app.logs().iter().any(|line| line.contains("goto failed")));
}

#[test]
fn asynchronous_goto_failure_retains_input_and_previous_rows() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("keep.txt"), "keep").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    let missing = root.path().join("missing");
    goto(&mut app, &missing);
    assert!(app.directory_loading());
    assert!(
        app.entries().is_empty(),
        "old rows must not act as new-directory targets"
    );
    settle(&mut app);
    assert_eq!(app.mode(), Mode::Goto);
    assert_eq!(app.input(), missing.to_str().unwrap());
    assert_eq!(app.entries()[0].name, "keep.txt");
    assert!(app.navigation_error().unwrap().contains("goto failed"));
}

#[test]
fn asynchronous_navigation_restores_each_directory_cursor() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("z-child");
    fs::create_dir(root.path().join("a-first")).unwrap();
    fs::create_dir(&child).unwrap();
    fs::write(child.join("a.txt"), "first").unwrap();
    fs::write(child.join("z.txt"), "last").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    app.handle_command(Command::Last);
    app.handle_command(Command::Open);
    settle(&mut app);
    app.handle_command(Command::Last);
    app.handle_command(Command::Parent);
    settle(&mut app);
    assert_eq!(app.entries()[app.cursor()].name, "z-child");
    app.handle_command(Command::Open);
    settle(&mut app);
    assert_eq!(app.entries()[app.cursor()].name, "z.txt");
}

#[test]
fn cached_preview_is_shown_immediately_but_same_metadata_content_is_reverified() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("a.txt");
    fs::write(&path, "old text").unwrap();
    fs::write(root.path().join("b.txt"), "second").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    app.handle_command(Command::Down);
    settle(&mut app);
    fs::write(&path, "new text").unwrap();
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    app.handle_command(Command::Up);
    assert!(app.preview_loading());
    assert!(
        app.preview()
            .lines
            .iter()
            .any(|line| line.contains("old text"))
    );
    settle(&mut app);
    assert!(
        app.preview()
            .lines
            .iter()
            .any(|line| line.contains("new text"))
    );
}

#[test]
fn loading_directory_cannot_trash_a_previously_selected_row() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("keep.txt");
    fs::write(&original, "keep").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    app.handle_command(Command::ToggleSelect);
    app.handle_command(Command::Refresh);
    app.handle_command(Command::Trash);
    assert_eq!(app.mode(), Mode::Normal);
    app.apply(Command::Trash);
    assert_eq!(app.mode(), Mode::Normal);
    assert!(original.exists());
    settle(&mut app);
}

#[test]
fn cancelled_async_goto_cannot_navigate_when_late_result_arrives() {
    let root = tempfile::tempdir().unwrap();
    let child = root.path().join("child");
    fs::create_dir(&child).unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    goto(&mut app, &child);
    app.handle_command(Command::Cancel);
    let until = Instant::now() + Duration::from_millis(150);
    while Instant::now() < until {
        app.poll_readers();
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(app.cwd(), root.path().canonicalize().unwrap());
    assert_eq!(app.mode(), Mode::Normal);
}

#[test]
fn asynchronous_file_startup_focuses_file_and_enters_preview() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("a.txt"), "first").unwrap();
    let target = root.path().join("z.txt");
    fs::write(&target, "selected content").unwrap();
    let mut app = App::new_async(target).unwrap();
    settle(&mut app);
    assert_eq!(app.mode(), Mode::Preview);
    assert_eq!(app.entries()[app.cursor()].name, "z.txt");
    assert!(
        app.preview()
            .lines
            .iter()
            .any(|line| line.contains("selected content"))
    );
}

#[test]
fn restore_refreshes_both_directory_and_recovery_screen_with_one_reader() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("restore.txt");
    fs::write(&original, "original bytes").unwrap();
    let mut app = App::new_async(root.path().into()).unwrap();
    settle(&mut app);
    app.handle_command(Command::Trash);
    for ch in "trash".chars() {
        app.handle_command(Command::Input(ch));
    }
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert!(!original.exists());
    app.handle_command(Command::OpenTrash);
    settle(&mut app);
    assert_eq!(app.trash_entries().len(), 1);
    app.handle_command(Command::RestoreTrash);
    app.handle_command(Command::Submit);
    settle(&mut app);
    assert!(app.trash_entries().is_empty());
    app.handle_command(Command::Cancel);
    settle(&mut app);
    assert!(
        app.entries()
            .iter()
            .any(|entry| entry.name == "restore.txt")
    );
    assert_eq!(fs::read_to_string(original).unwrap(), "original bytes");
}
