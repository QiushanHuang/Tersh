use std::{path::PathBuf, time::Duration};
use tersh::{
    job_history::JobHistory,
    jobs::{JobFailure, JobKind, JobProgress, JobRequest, JobResult},
};

fn request(kind: JobKind) -> JobRequest {
    JobRequest {
        kind,
        sources: ["good", "bad", "skip", "waiting"]
            .into_iter()
            .map(PathBuf::from)
            .collect(),
        destination: Some(PathBuf::from("/destination")),
        work_root: PathBuf::from("/source"),
    }
}

fn result() -> JobResult {
    JobResult {
        succeeded: vec!["good".into()],
        failed: vec![JobFailure {
            path: "bad".into(),
            error: "cannot read source".into(),
        }],
        skipped: vec!["skip".into()],
        unprocessed: vec!["waiting".into()],
        cancelled: true,
    }
}

#[test]
fn keeps_exact_counts_elapsed_and_copied_bytes_without_the_full_request() {
    let mut history = JobHistory::default();
    let id = history.push(
        &request(JobKind::Delete),
        &result(),
        &JobProgress {
            copied_bytes: 1234,
            ..Default::default()
        },
        Duration::from_millis(1250),
    );
    let entry = history.get(id).unwrap();
    assert_eq!(entry.counts.total, 4);
    assert_eq!(entry.counts.succeeded, 1);
    assert_eq!(entry.counts.failed, 1);
    assert_eq!(entry.counts.skipped, 1);
    assert_eq!(entry.counts.remaining, 1);
    assert!(entry.cancelled);
    assert_eq!(entry.elapsed, Duration::from_millis(1250));
    assert_eq!(entry.copied_bytes, 1234);
    assert_eq!(entry.omitted_details, 0);
}

#[test]
fn retry_includes_only_unresolved_sources_and_clears_conflict_decisions() {
    let mut history = JobHistory::default();
    for kind in [
        JobKind::Copy {
            replace: true,
            skip_conflicts: true,
        },
        JobKind::Move {
            skip_conflicts: true,
        },
    ] {
        let id = history.push(
            &request(kind),
            &result(),
            &JobProgress::default(),
            Duration::ZERO,
        );
        let retry = history.get(id).unwrap().retry_request().unwrap();
        assert_eq!(
            retry.sources,
            vec![PathBuf::from("bad"), PathBuf::from("waiting")]
        );
        assert_eq!(retry.destination, Some(PathBuf::from("/destination")));
        assert_eq!(retry.work_root, PathBuf::from("/source"));
        assert!(matches!(
            retry.kind,
            JobKind::Copy {
                replace: false,
                skip_conflicts: false
            } | JobKind::Move {
                skip_conflicts: false
            }
        ));
    }
}

#[test]
fn all_success_or_skip_has_no_retry_request() {
    let mut history = JobHistory::default();
    let id = history.push(
        &request(JobKind::Trash),
        &JobResult {
            succeeded: vec!["good".into()],
            skipped: vec!["skip".into()],
            ..Default::default()
        },
        &JobProgress::default(),
        Duration::ZERO,
    );
    assert!(history.get(id).unwrap().retry_request().is_err());
}

#[test]
fn history_retains_latest_twenty_and_drops_oldest() {
    let mut history = JobHistory::default();
    let first = history.push(
        &request(JobKind::Delete),
        &result(),
        &JobProgress::default(),
        Duration::ZERO,
    );
    let mut last = first;
    for _ in 0..40 {
        last = history.push(
            &request(JobKind::Delete),
            &result(),
            &JobProgress::default(),
            Duration::ZERO,
        );
    }
    assert_eq!(history.entries().len(), 20);
    assert!(history.get(first).is_none());
    assert_eq!(history.entries().front().unwrap().id, last);
    assert!(last > first);
}

#[test]
fn detail_and_history_budgets_preserve_true_counts_and_refuse_partial_retry() {
    let mut history = JobHistory::default();
    let result = JobResult {
        failed: (0..5000)
            .map(|index| JobFailure {
                path: format!("/source/{index:05}-{}", "x".repeat(80)).into(),
                error: "cannot copy: ".repeat(200),
            })
            .collect(),
        ..Default::default()
    };
    let request = JobRequest {
        sources: result
            .failed
            .iter()
            .map(|failure| failure.path.clone())
            .collect(),
        ..request(JobKind::Delete)
    };
    for _ in 0..30 {
        history.push(&request, &result, &JobProgress::default(), Duration::ZERO);
    }
    assert!(history.retained_bytes() <= 1024 * 1024);
    assert_eq!(history.entries().len(), 20);
    for entry in history.entries() {
        assert!(entry.retained_bytes() <= 64 * 1024);
        assert_eq!(entry.counts.failed, 5000);
        assert_eq!(entry.counts.total, 5000);
        assert!(entry.omitted_details > 0);
        assert!(entry.omitted_retry_items > 0);
        assert!(entry.retry_request().is_err());
    }
}

#[test]
fn oversized_context_is_explicitly_omitted_and_never_used_for_retry() {
    let mut history = JobHistory::default();
    let request = JobRequest {
        work_root: "x".repeat(100_000).into(),
        ..request(JobKind::Delete)
    };
    let id = history.push(&request, &result(), &JobProgress::default(), Duration::ZERO);
    let entry = history.get(id).unwrap();
    assert!(!entry.retry_context_retained);
    assert!(entry.retained_bytes() <= 64 * 1024);
    assert!(entry.retry_request().is_err());
}

#[test]
fn export_preserves_counts_and_never_overwrites_existing_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("task.json");
    let mut history = JobHistory::default();
    let id = history.push(
        &request(JobKind::Delete),
        &result(),
        &JobProgress::default(),
        Duration::ZERO,
    );
    let entry = history.get(id).unwrap();
    entry.export(&path).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["counts"]["total"], 4);
    assert_eq!(json["counts"]["failed"], 1);
    assert!(json["cancelled"].as_bool().unwrap());
    assert_eq!(json["omitted_details"], 0);
    assert!(json["limitations"].as_str().unwrap().contains("journal"));
    assert!(entry.export(&path).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    assert!(bytes.len() <= 512 * 1024);
}

#[cfg(unix)]
#[test]
fn export_refuses_symlinks_and_is_private_by_default() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("keep");
    let link = dir.path().join("link");
    std::fs::write(&target, "keep me").unwrap();
    std::os::unix::fs::symlink(&target, &link).unwrap();
    let mut history = JobHistory::default();
    let id = history.push(
        &request(JobKind::Delete),
        &result(),
        &JobProgress::default(),
        Duration::ZERO,
    );
    let entry = history.get(id).unwrap();
    assert!(entry.export(&link).is_err());
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep me");
    let output = dir.path().join("new.json");
    entry.export(&output).unwrap();
    assert_eq!(
        std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn export_reports_omissions_while_retaining_complete_result_counts() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bounded.json");
    let mut history = JobHistory::default();
    let result = JobResult {
        succeeded: (0..1_000)
            .map(|index| PathBuf::from(format!("/source/file-{index}")))
            .collect(),
        ..Default::default()
    };
    let request = JobRequest {
        sources: result.succeeded.clone(),
        ..request(JobKind::Trash)
    };
    let id = history.push(&request, &result, &JobProgress::default(), Duration::ZERO);
    history.get(id).unwrap().export(&path).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(json["counts"]["total"], 1_000);
    assert_eq!(json["counts"]["succeeded"], 1_000);
    assert_eq!(
        json["details"].as_array().unwrap().len()
            + json["omitted_details"].as_u64().unwrap() as usize,
        1_000
    );
}

#[test]
fn retry_deduplicates_retained_unresolved_paths() {
    let mut history = JobHistory::default();
    let result = JobResult {
        unprocessed: vec!["waiting".into(), "waiting".into()],
        ..result()
    };
    let id = history.push(
        &request(JobKind::Restore),
        &result,
        &JobProgress::default(),
        Duration::ZERO,
    );
    let retry = history.get(id).unwrap().retry_request().unwrap();
    assert_eq!(
        retry.sources,
        vec![PathBuf::from("bad"), PathBuf::from("waiting")]
    );
    assert_eq!(retry.kind, JobKind::Restore);
}

#[cfg(unix)]
#[test]
fn export_preserves_non_utf8_source_path_bytes() {
    use base64::Engine;
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("bytes.json");
    let raw = b"/source/non-utf8-\xff".to_vec();
    let source = PathBuf::from(OsString::from_vec(raw.clone()));
    let mut history = JobHistory::default();
    let result = JobResult {
        unprocessed: vec![source.clone()],
        ..Default::default()
    };
    let request = JobRequest {
        sources: vec![source],
        ..request(JobKind::Copy {
            replace: false,
            skip_conflicts: false,
        })
    };
    let id = history.push(&request, &result, &JobProgress::default(), Duration::ZERO);
    history.get(id).unwrap().export(&output).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    let encoded = json["unprocessed"][0]["unix_bytes_base64"]
        .as_str()
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .unwrap(),
        raw
    );
}
