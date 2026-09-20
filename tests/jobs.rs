use std::{
    fs,
    time::{Duration, Instant},
};
use tersh::jobs::{JobHandle, JobKind, JobRequest, JobResult};

fn wait(job: &mut JobHandle) -> JobResult {
    let started = Instant::now();
    loop {
        if let Some(result) = job.try_result() {
            return result;
        }
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "job did not complete"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn copy_job_reports_progress_success_and_conflict_without_losing_sources() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::write(&first, "copied").unwrap();
    fs::write(&second, "not copied").unwrap();
    fs::write(destination.join("second"), "existing").unwrap();
    let mut job = JobHandle::spawn(JobRequest {
        kind: JobKind::Copy {
            replace: false,
            skip_conflicts: false,
        },
        sources: vec![first.clone(), second.clone()],
        destination: Some(destination.clone()),
        work_root: root.path().into(),
    })
    .unwrap();
    let result = wait(&mut job);
    assert_eq!(result.succeeded, vec![first]);
    assert_eq!(result.failed.len(), 1);
    assert_eq!(result.failed[0].path, second);
    assert_eq!(job.progress().copied_bytes, 6);
    assert_eq!(job.progress().completed, 2);
    assert_eq!(
        fs::read_to_string(destination.join("second")).unwrap(),
        "existing"
    );
}

#[test]
fn move_skip_conflict_retains_only_unresolved_source() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    fs::write(&first, "moved").unwrap();
    fs::write(&second, "kept").unwrap();
    fs::write(destination.join("second"), "existing").unwrap();
    let mut job = JobHandle::spawn(JobRequest {
        kind: JobKind::Move {
            skip_conflicts: true,
        },
        sources: vec![first.clone(), second.clone()],
        destination: Some(destination),
        work_root: root.path().into(),
    })
    .unwrap();
    let result = wait(&mut job);
    assert_eq!(result.succeeded, vec![first.clone()]);
    assert_eq!(result.skipped, vec![second.clone()]);
    assert!(!first.exists());
    assert!(second.exists());
}

#[test]
fn cancellation_retains_sources_and_original_replacement() {
    let root = tempfile::tempdir().unwrap();
    let destination = root.path().join("destination");
    fs::create_dir(&destination).unwrap();
    let source = root.path().join("large");
    fs::File::create(&source)
        .unwrap()
        .set_len(512 * 1024 * 1024)
        .unwrap();
    fs::write(destination.join("large"), "original").unwrap();
    let mut job = JobHandle::spawn(JobRequest {
        kind: JobKind::Copy {
            replace: true,
            skip_conflicts: false,
        },
        sources: vec![source.clone()],
        destination: Some(destination.clone()),
        work_root: root.path().into(),
    })
    .unwrap();
    job.cancel();
    let result = wait(&mut job);
    assert!(result.cancelled);
    assert_eq!(result.unprocessed, vec![source]);
    assert_eq!(
        fs::read_to_string(destination.join("large")).unwrap(),
        "original"
    );
    assert_eq!(fs::read_dir(destination).unwrap().count(), 1);
}

#[test]
fn trash_and_restore_jobs_roundtrip_persisted_receipt() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("file");
    fs::write(&source, "roundtrip").unwrap();
    let mut job = JobHandle::spawn(JobRequest {
        kind: JobKind::Trash,
        sources: vec![source.clone()],
        destination: None,
        work_root: root.path().into(),
    })
    .unwrap();
    assert_eq!(wait(&mut job).succeeded, vec![source.clone()]);
    let entry = tersh::trash::list_trash(root.path()).unwrap().remove(0);
    let mut restore = JobHandle::spawn(JobRequest {
        kind: JobKind::Restore,
        sources: vec![entry.receipt_path.clone()],
        destination: None,
        work_root: root.path().into(),
    })
    .unwrap();
    assert_eq!(wait(&mut restore).succeeded, vec![entry.receipt_path]);
    assert_eq!(fs::read_to_string(source).unwrap(), "roundtrip");
}
