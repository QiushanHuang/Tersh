use std::fs;
use tersh::{
    fs_ops::{DeleteDecision, trash_path},
    trash::{list_trash, restore_entry},
};

#[test]
fn persisted_receipt_restores_file_and_original_nested_location() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("nested");
    fs::create_dir(&nested).unwrap();
    let original = nested.join("original.txt");
    fs::write(&original, "precious").unwrap();
    trash_path(&original, root.path()).unwrap();
    let entries = list_trash(root.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].original_path,
        original
            .canonicalize()
            .unwrap_or_else(|_| nested.canonicalize().unwrap().join("original.txt"))
    );
    restore_entry(&entries[0].receipt_path, root.path()).unwrap();
    assert_eq!(fs::read_to_string(original).unwrap(), "precious");
    assert!(list_trash(root.path()).unwrap().is_empty());
}

#[test]
fn restore_refuses_existing_destination_and_preserves_both_values() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.txt");
    fs::write(&original, "old").unwrap();
    trash_path(&original, root.path()).unwrap();
    let entry = list_trash(root.path()).unwrap().remove(0);
    fs::write(&original, "new").unwrap();
    assert!(restore_entry(&entry.receipt_path, root.path()).is_err());
    assert_eq!(fs::read_to_string(original).unwrap(), "new");
    assert_eq!(fs::read_to_string(entry.trashed_path).unwrap(), "old");
}

#[test]
fn legacy_entries_are_not_given_guessed_restore_paths() {
    let root = tempfile::tempdir().unwrap();
    let trash = root.path().join(".tersh-trash");
    fs::create_dir(&trash).unwrap();
    fs::write(trash.join("legacy.txt"), "legacy").unwrap();
    assert!(list_trash(root.path()).unwrap().is_empty());
    assert!(trash.join("legacy.txt").exists());
}

#[test]
fn altered_payload_and_malformed_receipt_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.txt");
    fs::write(&original, "old").unwrap();
    trash_path(&original, root.path()).unwrap();
    let entry = list_trash(root.path()).unwrap().remove(0);
    fs::write(&entry.trashed_path, "unexpected modification").unwrap();
    assert!(restore_entry(&entry.receipt_path, root.path()).is_err());
    assert!(!original.exists());
    fs::write(&entry.receipt_path, "not json").unwrap();
    assert!(restore_entry(&entry.receipt_path, root.path()).is_err());
    assert!(!original.exists());
}

#[test]
fn forged_metadata_cannot_escape_or_select_another_payload() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.txt");
    fs::write(&original, "old").unwrap();
    trash_path(&original, root.path()).unwrap();
    let entry = list_trash(root.path()).unwrap().remove(0);
    let saved = fs::read(&entry.receipt_path).unwrap();
    for (field, value) in [
        ("id", "../other"),
        ("original_path", "../escaped"),
        ("work_root", "/"),
    ] {
        let mut document: serde_json::Value = serde_json::from_slice(&saved).unwrap();
        document[field] = value.into();
        fs::write(&entry.receipt_path, serde_json::to_vec(&document).unwrap()).unwrap();
        assert!(restore_entry(&entry.receipt_path, root.path()).is_err());
        assert!(!original.exists());
        assert_eq!(fs::read_to_string(&entry.trashed_path).unwrap(), "old");
    }
}

#[cfg(unix)]
#[test]
fn replaced_original_parent_cannot_redirect_restore() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("parent");
    fs::create_dir(&parent).unwrap();
    let original = parent.join("original.txt");
    fs::write(&original, "old").unwrap();
    trash_path(&original, root.path()).unwrap();
    let entry = list_trash(root.path()).unwrap().remove(0);
    fs::rename(&parent, root.path().join("saved-parent")).unwrap();
    fs::create_dir(&parent).unwrap();
    assert!(restore_entry(&entry.receipt_path, root.path()).is_err());
    assert!(!original.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_original_path_is_rejected_before_moving() {
    use std::os::unix::ffi::OsStringExt;
    let root = tempfile::tempdir().unwrap();
    let path = root
        .path()
        .join(std::ffi::OsString::from_vec(vec![0x66, 0xff]));
    fs::write(&path, "kept").unwrap();
    assert!(trash_path(&path, root.path()).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), "kept");
}

#[cfg(unix)]
#[test]
fn symlink_roundtrip_preserves_link_without_touching_target() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("missing");
    let original = root.path().join("link");
    std::os::unix::fs::symlink(&target, &original).unwrap();
    let DeleteDecision::MovedToTrash { to, .. } = trash_path(&original, root.path()).unwrap();
    assert!(fs::symlink_metadata(to).unwrap().file_type().is_symlink());
    let entry = list_trash(root.path()).unwrap().remove(0);
    restore_entry(&entry.receipt_path, root.path()).unwrap();
    assert_eq!(fs::read_link(original).unwrap(), target);
}

#[test]
fn repeated_names_get_independent_receipts_and_payloads() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.txt");
    for value in ["one", "two"] {
        fs::write(&original, value).unwrap();
        trash_path(&original, root.path()).unwrap();
    }
    let entries = list_trash(root.path()).unwrap();
    assert_eq!(entries.len(), 2);
    assert_ne!(entries[0].receipt_path, entries[1].receipt_path);
    assert_ne!(entries[0].trashed_path, entries[1].trashed_path);
}

#[test]
fn scanning_mixed_valid_and_corrupt_receipts_keeps_safe_recovery_available() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::write(&first, "valid").unwrap();
    fs::write(&second, "invalid metadata").unwrap();
    trash_path(&first, root.path()).unwrap();
    trash_path(&second, root.path()).unwrap();
    let entries = list_trash(root.path()).unwrap();
    let bad = entries
        .iter()
        .find(|entry| entry.original_path.file_name().unwrap() == "second")
        .unwrap();
    fs::write(&bad.receipt_path, "invalid json").unwrap();
    let scan = tersh::trash::scan_trash(root.path()).unwrap();
    assert_eq!(scan.entries.len(), 1);
    assert_eq!(scan.entries[0].original_path.file_name().unwrap(), "first");
    assert_eq!(scan.warning_count, 1);
    assert_eq!(scan.warnings.len(), 1);
    assert!(
        list_trash(root.path()).is_err(),
        "strict API must remain fail-closed"
    );
    restore_entry(&scan.entries[0].receipt_path, root.path()).unwrap();
    assert_eq!(fs::read_to_string(first).unwrap(), "valid");
    assert!(bad.trashed_path.exists());
    assert!(!second.exists());
}

#[test]
fn scan_counts_all_invalid_receipts_but_bounds_warning_storage() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("valid");
    fs::write(&original, "kept").unwrap();
    trash_path(&original, root.path()).unwrap();
    let directory = root.path().join(".tersh-trash/.receipts");
    for index in 0..25 {
        fs::write(directory.join(format!("bad-{index}.json")), "invalid").unwrap();
    }
    let scan = tersh::trash::scan_trash(root.path()).unwrap();
    assert_eq!(scan.entries.len(), 1);
    assert_eq!(scan.warning_count, 25);
    assert_eq!(scan.warnings.len(), 20);
}

#[cfg(unix)]
#[test]
fn scan_rejects_symlinked_receipt_directory_at_top_level() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let trash = root.path().join(".tersh-trash");
    fs::create_dir(&trash).unwrap();
    std::os::unix::fs::symlink(outside.path(), trash.join(".receipts")).unwrap();
    assert!(tersh::trash::scan_trash(root.path()).is_err());
}
