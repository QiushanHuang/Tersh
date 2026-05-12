use tersh::fs_ops::{DeleteDecision, copy_path, rename_path, trash_path, validate_file_name};

#[test]
fn copies_symlink_as_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target.txt");
    std::fs::write(&target, "target").unwrap();
    let link = dir.path().join("link.txt");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&target, &link).unwrap();
    let copied = dir.path().join("copied-link.txt");

    copy_path(&link, &copied, false).unwrap();

    assert!(
        std::fs::symlink_metadata(&copied)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}

#[test]
fn trash_moves_file_into_tersh_trash() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("old.txt");
    std::fs::write(&file, "old").unwrap();

    let decision = trash_path(&file, dir.path()).unwrap();

    assert!(matches!(decision, DeleteDecision::MovedToTrash { .. }));
    assert!(!file.exists());
    assert!(dir.path().join(".tersh-trash").exists());
}

#[test]
fn rename_refuses_to_overwrite_existing_target() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.txt");
    let target = dir.path().join("target.txt");
    std::fs::write(&source, "source").unwrap();
    std::fs::write(&target, "target").unwrap();

    let err = rename_path(&source, &target).unwrap_err();

    assert!(err.to_string().contains("already exists"));
    assert!(source.exists());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "target");
}

#[test]
fn copy_rejects_directory_into_its_own_child() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("item.txt"), "item").unwrap();
    let target = source.join("nested-copy");

    let err = copy_path(&source, &target, false).unwrap_err();

    assert!(err.to_string().contains("inside itself"));
    assert!(!target.exists());
}

#[cfg(unix)]
#[test]
fn copy_refuses_to_overwrite_dangling_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.txt");
    let dangling = dir.path().join("dangling.txt");
    std::fs::write(&source, "source").unwrap();
    std::os::unix::fs::symlink(dir.path().join("missing.txt"), &dangling).unwrap();

    let err = copy_path(&source, &dangling, false).unwrap_err();

    assert!(err.to_string().contains("already exists"));
}

#[test]
fn rename_validation_rejects_paths_and_empty_names() {
    assert!(validate_file_name("").is_err());
    assert!(validate_file_name("../escape").is_err());
    assert!(validate_file_name("/tmp/escape").is_err());
    assert!(validate_file_name("safe-name.txt").is_ok());
}
