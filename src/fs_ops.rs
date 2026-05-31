use anyhow::{Context, Result, anyhow, bail};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteDecision {
    MovedToTrash { from: PathBuf, to: PathBuf },
}

pub fn copy_path(source: &Path, target: &Path, replace: bool) -> Result<()> {
    if path_exists_no_follow(target) {
        if replace {
            remove_existing(target)?;
        } else {
            bail!("target already exists: {}", target.display());
        }
    }
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("failed to inspect {}", source.display()))?;
    if metadata.is_dir() {
        reject_copy_into_self(source, target)?;
    }
    if metadata.file_type().is_symlink() {
        let link_target = fs::read_link(source)
            .with_context(|| format!("failed to read symlink {}", source.display()))?;
        create_symlink(&link_target, target, false)
    } else if metadata.is_dir() {
        copy_dir_recursive(source, target)
    } else if metadata.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, target).with_context(|| {
            format!(
                "failed to copy {} to {}",
                source.display(),
                target.display()
            )
        })?;
        fs::set_permissions(target, metadata.permissions()).ok();
        Ok(())
    } else {
        bail!("unsupported file type: {}", source.display());
    }
}

pub fn rename_path(source: &Path, target: &Path) -> Result<()> {
    if path_exists_no_follow(target) {
        bail!("target already exists: {}", target.display());
    }
    fs::rename(source, target).with_context(|| {
        format!(
            "failed to rename {} to {}",
            source.display(),
            target.display()
        )
    })
}

pub fn trash_path(path: &Path, work_root: &Path) -> Result<DeleteDecision> {
    guard_delete_target(path, work_root)?;
    let trash_dir = prepare_trash_dir(work_root)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("cannot trash path without file name: {}", path.display()))?;
    let mut target = trash_dir.join(file_name);
    if path_exists_no_follow(&target) {
        target = trash_dir.join(format!(
            "{}.{}",
            file_name.to_string_lossy(),
            unique_suffix()
        ));
    }
    fs::rename(path, &target)
        .with_context(|| format!("failed to move {} to trash", path.display()))?;
    Ok(DeleteDecision::MovedToTrash {
        from: path.to_path_buf(),
        to: target,
    })
}

fn prepare_trash_dir(work_root: &Path) -> Result<PathBuf> {
    let trash_dir = work_root.join(".tersh-trash");
    match fs::symlink_metadata(&trash_dir) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() {
                bail!("refusing to use symlinked .tersh-trash");
            }
            if !metadata.is_dir() {
                bail!(".tersh-trash exists and is not a directory");
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir(&trash_dir)
                .with_context(|| format!("failed to create trash {}", trash_dir.display()))?;
        }
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to inspect trash {}", trash_dir.display()));
        }
    }
    let metadata = fs::symlink_metadata(&trash_dir)
        .with_context(|| format!("failed to inspect trash {}", trash_dir.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("refusing to use unsafe .tersh-trash");
    }
    Ok(trash_dir)
}

pub fn permanent_delete(path: &Path, work_root: &Path) -> Result<()> {
    guard_delete_target(path, work_root)?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to delete directory {}", path.display()))
    } else {
        fs::remove_file(path).with_context(|| format!("failed to delete {}", path.display()))
    }
}

pub fn destination_for_paste(source: &Path, target_dir: &Path) -> Result<PathBuf> {
    let name = source
        .file_name()
        .ok_or_else(|| anyhow!("cannot paste path without file name: {}", source.display()))?;
    Ok(target_dir.join(name))
}

pub fn validate_file_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("file name cannot be empty");
    }
    if name.chars().any(char::is_control) {
        bail!("file name must not contain control characters");
    }
    let path = Path::new(name);
    if path.is_absolute() {
        bail!("file name must not be an absolute path");
    }
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => bail!("file name must be a single path component"),
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)
        .with_context(|| format!("failed to create directory {}", target.display()))?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child_source = entry.path();
        let child_target = target.join(entry.file_name());
        copy_path(&child_source, &child_target, false)?;
    }
    Ok(())
}

fn reject_copy_into_self(source: &Path, target: &Path) -> Result<()> {
    let source = source.canonicalize()?;
    let target_parent = target
        .parent()
        .ok_or_else(|| anyhow!("target has no parent: {}", target.display()))?
        .canonicalize()?;
    let target_name = target
        .file_name()
        .ok_or_else(|| anyhow!("target has no file name: {}", target.display()))?;
    let target = target_parent.join(target_name);
    if target.starts_with(&source) {
        bail!("refusing to copy directory inside itself");
    }
    Ok(())
}

fn path_exists_no_follow(path: &Path) -> bool {
    match fs::symlink_metadata(path) {
        Ok(_) => true,
        Err(err) => err.kind() != std::io::ErrorKind::NotFound,
    }
}

fn remove_existing(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn guard_delete_target(path: &Path, work_root: &Path) -> Result<()> {
    if path.as_os_str().is_empty() || !path.is_absolute() {
        bail!("refusing to delete non-absolute path: {}", path.display());
    }
    let target = delete_identity(path)?;
    if target.parent().is_none() {
        bail!("refusing to delete filesystem root");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .and_then(|home| home.canonicalize().ok());
    if home.as_deref() == Some(target.as_path()) {
        bail!("refusing to delete home directory");
    }
    let work_root = work_root
        .canonicalize()
        .with_context(|| format!("failed to resolve work root {}", work_root.display()))?;
    if target == work_root {
        bail!("refusing to delete active work root");
    }
    if target == work_root.join(".tersh-trash") {
        bail!("refusing to delete .tersh-trash");
    }
    Ok(())
}

fn delete_identity(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to inspect delete target {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        let parent = path
            .parent()
            .ok_or_else(|| anyhow!("delete target has no parent: {}", path.display()))?
            .canonicalize()
            .with_context(|| format!("failed to resolve parent for {}", path.display()))?;
        let name = path
            .file_name()
            .ok_or_else(|| anyhow!("delete target has no file name: {}", path.display()))?;
        return Ok(parent.join(name));
    }
    path.canonicalize()
        .with_context(|| format!("failed to resolve delete target {}", path.display()))
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path, _is_dir: bool) -> Result<()> {
    std::os::unix::fs::symlink(source, target)
        .with_context(|| format!("failed to create symlink {}", target.display()))
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path, is_dir: bool) -> Result<()> {
    if is_dir {
        std::os::windows::fs::symlink_dir(source, target)
    } else {
        std::os::windows::fs::symlink_file(source, target)
    }
    .with_context(|| format!("failed to create symlink {}", target.display()))
}
