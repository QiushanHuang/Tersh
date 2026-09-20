use tersh::fs_ops::{
    DeleteDecision, copy_path, permanent_delete, rename_path, trash_path, validate_file_name,
};

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

#[cfg(unix)]
#[test]
fn copies_symlink_without_stating_protected_target() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let protected = dir.path().join("protected");
    std::fs::create_dir(&protected).unwrap();
    let target = protected.join("target.txt");
    std::fs::write(&target, "secret").unwrap();
    let link = dir.path().join("link.txt");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    std::fs::set_permissions(&protected, std::fs::Permissions::from_mode(0o000)).unwrap();

    let copied = dir.path().join("copied-link.txt");
    let result = copy_path(&link, &copied, false);

    std::fs::set_permissions(&protected, std::fs::Permissions::from_mode(0o700)).unwrap();
    result.unwrap();
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
fn delete_rejects_canonical_work_root() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let alias = nested.join("..");

    let err = permanent_delete(&alias, dir.path()).unwrap_err();

    assert!(err.to_string().contains("work root"));
    assert!(dir.path().exists());
}

#[test]
fn trash_rejects_canonical_work_root() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("nested");
    std::fs::create_dir(&nested).unwrap();
    let alias = nested.join("..");

    let err = trash_path(&alias, dir.path()).unwrap_err();

    assert!(err.to_string().contains("work root"));
    assert!(dir.path().exists());
}

#[test]
fn delete_rejects_root_and_relative_paths() {
    let dir = tempfile::tempdir().unwrap();

    let root_err = permanent_delete(std::path::Path::new("/"), dir.path()).unwrap_err();
    let relative_err = permanent_delete(std::path::Path::new("relative"), dir.path()).unwrap_err();

    assert!(root_err.to_string().contains("filesystem root"));
    assert!(relative_err.to_string().contains("non-absolute"));
}

#[test]
fn delete_and_trash_reject_home_directory_when_home_is_available() {
    let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
        return;
    };
    let Ok(home) = home.canonicalize() else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();

    let delete_err = permanent_delete(&home, dir.path()).unwrap_err();
    let trash_err = trash_path(&home, dir.path()).unwrap_err();

    assert!(delete_err.to_string().contains("home directory"));
    assert!(trash_err.to_string().contains("home directory"));
}

#[test]
fn trash_rejects_tersh_trash_directory() {
    let dir = tempfile::tempdir().unwrap();
    let trash = dir.path().join(".tersh-trash");
    std::fs::create_dir(&trash).unwrap();

    let err = trash_path(&trash, dir.path()).unwrap_err();

    assert!(err.to_string().contains(".tersh-trash"));
    assert!(trash.exists());
}

#[test]
fn permanent_delete_rejects_tersh_trash_directory() {
    let dir = tempfile::tempdir().unwrap();
    let trash = dir.path().join(".tersh-trash");
    std::fs::create_dir(&trash).unwrap();

    let err = permanent_delete(&trash, dir.path()).unwrap_err();

    assert!(err.to_string().contains(".tersh-trash"));
    assert!(trash.exists());
}

#[test]
fn delete_and_trash_reject_tersh_trash_descendants() {
    let dir = tempfile::tempdir().unwrap();
    let trash = dir.path().join(".tersh-trash");
    std::fs::create_dir(&trash).unwrap();
    let trashed_file = trash.join("old.txt");
    std::fs::write(&trashed_file, "old").unwrap();

    let delete_err = permanent_delete(&trashed_file, dir.path()).unwrap_err();
    let trash_err = trash_path(&trashed_file, dir.path()).unwrap_err();

    assert!(delete_err.to_string().contains(".tersh-trash"));
    assert!(trash_err.to_string().contains(".tersh-trash"));
    assert!(trashed_file.exists());
}

#[cfg(unix)]
#[test]
fn trash_rejects_symlinked_tersh_trash_directory() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let trash = dir.path().join(".tersh-trash");
    std::os::unix::fs::symlink(outside.path(), &trash).unwrap();
    let file = dir.path().join("old.txt");
    std::fs::write(&file, "old").unwrap();

    let err = trash_path(&file, dir.path()).unwrap_err();

    assert!(err.to_string().contains(".tersh-trash"));
    assert!(file.exists());
}

#[cfg(unix)]
#[test]
fn delete_and_trash_reject_paths_inside_symlinked_trash_directory() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let trash = dir.path().join(".tersh-trash");
    std::os::unix::fs::symlink(outside.path(), &trash).unwrap();
    let trashed_file = trash.join("old.txt");
    std::fs::write(outside.path().join("old.txt"), "old").unwrap();

    let delete_err = permanent_delete(&trashed_file, dir.path()).unwrap_err();
    let trash_err = trash_path(&trashed_file, dir.path()).unwrap_err();

    assert!(delete_err.to_string().contains(".tersh-trash"));
    assert!(trash_err.to_string().contains(".tersh-trash"));
    assert!(outside.path().join("old.txt").exists());
}

#[cfg(unix)]
#[test]
fn permanent_delete_removes_symlink_without_deleting_target_directory() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("target-dir");
    let link = dir.path().join("link-dir");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("kept.txt"), "kept").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();

    permanent_delete(&link, dir.path()).unwrap();

    assert!(std::fs::symlink_metadata(&link).is_err());
    assert!(target.join("kept.txt").exists());
}

#[cfg(unix)]
#[test]
fn permanent_delete_removes_dangling_symlink() {
    let dir = tempfile::tempdir().unwrap();
    let link = dir.path().join("dangling");
    std::os::unix::fs::symlink(dir.path().join("missing"), &link).unwrap();

    permanent_delete(&link, dir.path()).unwrap();

    assert!(std::fs::symlink_metadata(&link).is_err());
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
fn copy_replace_preserves_existing_target_when_source_is_invalid() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.txt");
    let target = dir.path().join("target.txt");
    std::fs::write(&target, "keep").unwrap();

    let err = copy_path(&missing, &target, true).unwrap_err();

    assert!(err.to_string().contains("failed to inspect"));
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
}

#[cfg(unix)]
#[test]
fn failed_directory_copy_cleans_partial_target() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let target = dir.path().join("target");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("copied-before-error.txt"), "partial").unwrap();
    let fifo = source.join("unsupported-fifo");
    let fifo_c =
        std::ffi::CString::new(std::os::unix::ffi::OsStrExt::as_bytes(fifo.as_os_str())).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo_c.as_ptr(), 0o600) }, 0);

    let err = copy_path(&source, &target, false).unwrap_err();

    assert!(err.to_string().contains("unsupported file type"));
    assert!(!target.exists());
}

#[test]
fn rename_validation_rejects_paths_and_empty_names() {
    assert!(validate_file_name("").is_err());
    assert!(validate_file_name("../escape").is_err());
    assert!(validate_file_name("/tmp/escape").is_err());
    assert!(validate_file_name("bad\nname").is_err());
    assert!(validate_file_name("bad\u{1b}name").is_err());
    assert!(validate_file_name("safe-name.txt").is_ok());
}

#[test]
fn trash_records_restore_receipt_before_reporting_success() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("old.txt");
    std::fs::write(&file, "old").unwrap();
    trash_path(&file, dir.path()).unwrap();
    let receipts = dir.path().join(".tersh-trash/.receipts");
    assert!(
        receipts.is_dir(),
        "managed trash must persist restore metadata"
    );
    assert_eq!(std::fs::read_dir(receipts).unwrap().count(), 1);
}

#[test]
fn cancellation_during_copy_cleans_current_output() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let target = dir.path().join("target");
    std::fs::write(&source, vec![42_u8; 1024 * 1024]).unwrap();
    let mut bytes = 0;
    let result = tersh::fs_ops::copy_path_cancellable(&source, &target, false, &mut |_, count| {
        bytes += count;
        if bytes >= 128 * 1024 {
            anyhow::bail!("test cancellation");
        }
        Ok(())
    });
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("test cancellation")
    );
    assert_eq!(bytes, 128 * 1024);
    assert!(!target.exists());
    assert_eq!(std::fs::metadata(source).unwrap().len(), 1024 * 1024);
}

#[test]
fn cancelled_replacement_preserves_original_and_cleans_staging() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let target = dir.path().join("target");
    std::fs::write(&source, vec![42_u8; 1024 * 1024]).unwrap();
    std::fs::write(&target, "precious original").unwrap();
    let result = tersh::fs_ops::copy_path_cancellable(&source, &target, true, &mut |_, count| {
        if count > 0 {
            anyhow::bail!("test cancellation");
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "precious original"
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn cancelled_recursive_copy_cleans_whole_incomplete_root() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let target = dir.path().join("target");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("one"), "one").unwrap();
    std::fs::write(source.join("two"), "two").unwrap();
    let mut bytes = 0;
    let result = tersh::fs_ops::copy_path_cancellable(&source, &target, false, &mut |_, count| {
        bytes += count;
        if bytes > 3 {
            anyhow::bail!("test cancellation");
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(!target.exists());
    assert!(source.join("one").exists());
    assert!(source.join("two").exists());
}

#[cfg(unix)]
#[test]
fn cancellable_delete_never_follows_child_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    let kept = dir.path().join("kept");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(&kept, "precious original").unwrap();
    std::os::unix::fs::symlink(&kept, source.join("link")).unwrap();
    tersh::fs_ops::permanent_delete_cancellable(&source, dir.path(), &mut |_, _| Ok(())).unwrap();
    assert!(!source.exists());
    assert_eq!(std::fs::read_to_string(kept).unwrap(), "precious original");
}

#[test]
fn recursive_delete_cancellation_preserves_remaining_children() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source");
    std::fs::create_dir(&source).unwrap();
    for index in 0..8 {
        std::fs::write(source.join(index.to_string()), "value").unwrap();
    }
    let mut calls = 0;
    let result =
        tersh::fs_ops::permanent_delete_cancellable(&source, dir.path(), &mut |path, _| {
            if path != source {
                calls += 1;
            }
            if calls == 3 {
                anyhow::bail!("test cancellation");
            }
            Ok(())
        });
    assert!(result.is_err());
    assert_eq!(std::fs::read_dir(&source).unwrap().count(), 6);
}

#[cfg(unix)]
#[test]
fn copy_readonly_directory_populates_before_preserving_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    let target = root.path().join("target");
    std::fs::create_dir(&source).unwrap();
    std::fs::write(source.join("item"), "contents").unwrap();
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = copy_path(&source, &target, false);
    let mode = std::fs::metadata(&target).map(|m| m.permissions().mode() & 0o777);
    std::fs::set_permissions(&source, std::fs::Permissions::from_mode(0o700)).unwrap();
    if target.exists() {
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    result.unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join("item")).unwrap(),
        "contents"
    );
    assert_eq!(mode.unwrap(), 0o500);
}

#[cfg(unix)]
#[test]
fn cancelled_copy_cleans_completed_readonly_subdirectories() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    let child = source.join("readonly");
    let target = root.path().join("target");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(child.join("item"), "value").unwrap();
    std::fs::write(&target, "original").unwrap();
    std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o500)).unwrap();
    let mut copied = false;
    let result =
        tersh::fs_ops::copy_path_cancellable(&source, &target, true, &mut |path, count| {
            if count > 0 {
                copied = true;
            }
            if path == source && copied {
                anyhow::bail!("test cancellation");
            }
            Ok(())
        });
    std::fs::set_permissions(&child, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "original");
    assert_eq!(
        std::fs::read_dir(root.path()).unwrap().count(),
        2,
        "all staging output must be removed"
    );
}

#[cfg(unix)]
#[test]
fn cancellable_delete_pins_directory_when_ancestor_is_swapped_for_symlink() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    let displaced = root.path().join("displaced");
    let outside = root.path().join("outside");
    std::fs::create_dir(&source).unwrap();
    std::fs::create_dir(&outside).unwrap();
    std::fs::write(source.join("same-name"), "delete").unwrap();
    std::fs::write(outside.join("same-name"), "keep").unwrap();
    let mut visits = 0;
    let result =
        tersh::fs_ops::permanent_delete_cancellable(&source, root.path(), &mut |path, _| {
            if path == source {
                visits += 1;
                if visits == 2 {
                    std::fs::rename(&source, &displaced).unwrap();
                    std::os::unix::fs::symlink(&outside, &source).unwrap();
                }
            }
            Ok(())
        });
    assert!(result.is_err());
    assert_eq!(
        std::fs::read_to_string(outside.join("same-name")).unwrap(),
        "keep"
    );
    assert!(
        std::fs::symlink_metadata(source)
            .unwrap()
            .file_type()
            .is_symlink()
    );
}
