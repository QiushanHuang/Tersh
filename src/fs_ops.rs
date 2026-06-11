use anyhow::{Context, Result, anyhow, bail};
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteDecision {
    MovedToTrash { from: PathBuf, to: PathBuf },
}

pub fn copy_path(source: &Path, target: &Path, replace: bool) -> Result<()> {
    if path_exists_no_follow(target)? {
        if replace {
            return replace_path(source, target);
        } else {
            bail!("target already exists: {}", target.display());
        }
    }
    copy_path_no_replace(source, target)
}

fn copy_path_no_replace(source: &Path, target: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(source)
        .with_context(|| format!("failed to inspect {}", source.display()))?;
    let identity = FileIdentity::from_metadata(&metadata);
    if metadata.is_dir() {
        reject_copy_into_self(source, target)?;
    }
    if metadata.file_type().is_symlink() {
        ensure_path_identity(source, &identity)?;
        let link_target = fs::read_link(source)
            .with_context(|| format!("failed to read symlink {}", source.display()))?;
        ensure_path_identity(source, &identity)?;
        create_parent_dir(target)?;
        let is_dir = symlink_target_is_dir_for_creation(source, &link_target)?;
        create_symlink(&link_target, target, is_dir)
    } else if metadata.is_dir() {
        copy_dir_recursive(source, target, &metadata, &identity)
    } else if metadata.is_file() {
        copy_regular_file(source, target, &metadata, &identity)
    } else {
        bail!("unsupported file type: {}", source.display());
    }
}

fn replace_path(source: &Path, target: &Path) -> Result<()> {
    let target_metadata = fs::symlink_metadata(target)
        .with_context(|| format!("failed to inspect existing {}", target.display()))?;
    let target_identity = FileIdentity::from_metadata(&target_metadata);
    if target_metadata.is_dir() && !target_metadata.file_type().is_symlink() {
        bail!("refusing to replace directory: {}", target.display());
    }

    let temp = temp_sibling_path(target)?;
    let copy_result = copy_path_no_replace(source, &temp);
    if let Err(err) = copy_result {
        let _ = remove_existing(&temp);
        return Err(err);
    }

    let backup = temp_sibling_path(target)?;
    ensure_path_identity(target, &target_identity)?;
    rename_no_replace(target, &backup)
        .with_context(|| format!("failed to preserve existing {}", target.display()))?;
    if let Err(err) = rename_no_replace(&temp, target) {
        let _ = remove_existing(&temp);
        let _ = rename_no_replace(&backup, target);
        return Err(err).with_context(|| {
            format!(
                "failed to replace {} with {}",
                target.display(),
                source.display()
            )
        });
    }
    remove_existing(&backup)
        .with_context(|| format!("failed to remove replaced {}", backup.display()))?;
    Ok(())
}

pub fn rename_path(source: &Path, target: &Path) -> Result<()> {
    if path_exists_no_follow(target)? {
        bail!("target already exists: {}", target.display());
    }
    rename_no_replace(source, target).with_context(|| {
        format!(
            "failed to rename {} to {}",
            source.display(),
            target.display()
        )
    })
}

pub fn trash_path(path: &Path, work_root: &Path) -> Result<DeleteDecision> {
    let guarded = guard_delete_target(path, work_root)?;
    let trash_dir = prepare_trash_dir(work_root)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("cannot trash path without file name: {}", path.display()))?;
    for attempt in 0..100 {
        let target = if attempt == 0 {
            trash_dir.path.join(file_name)
        } else {
            trash_dir.path.join(format!(
                "{}.{}",
                file_name.to_string_lossy(),
                unique_suffix()
            ))
        };
        ensure_path_identity(path, &guarded.identity)?;
        ensure_same_path_object(&trash_dir.path, &trash_dir.identity)?;
        match rename_no_replace(path, &target) {
            Ok(()) => {
                return Ok(DeleteDecision::MovedToTrash {
                    from: path.to_path_buf(),
                    to: target,
                });
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to move {} to trash", path.display()));
            }
        }
    }
    bail!(
        "failed to allocate unique trash target for {}",
        path.display()
    );
}

struct TrashDir {
    path: PathBuf,
    identity: FileIdentity,
}

fn prepare_trash_dir(work_root: &Path) -> Result<TrashDir> {
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
    Ok(TrashDir {
        path: trash_dir,
        identity: FileIdentity::from_metadata(&metadata),
    })
}

pub fn permanent_delete(path: &Path, work_root: &Path) -> Result<()> {
    let guarded = guard_delete_target(path, work_root)?;
    ensure_path_identity(path, &guarded.identity)?;
    if guarded.identity.is_dir && !guarded.identity.is_symlink {
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

fn copy_regular_file(
    source: &Path,
    target: &Path,
    metadata: &fs::Metadata,
    identity: &FileIdentity,
) -> Result<()> {
    create_parent_dir(target)?;
    ensure_path_identity(source, identity)?;
    let mut input = open_regular_source(source)
        .with_context(|| format!("failed to open {}", source.display()))?;
    let opened_metadata = input
        .metadata()
        .with_context(|| format!("failed to inspect opened {}", source.display()))?;
    if FileIdentity::from_metadata(&opened_metadata) != *identity {
        bail!("source changed during copy: {}", source.display());
    }
    let mut output_options = OpenOptions::new();
    output_options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        output_options.mode(metadata.permissions().mode() & 0o7777);
    }
    let mut output = output_options
        .open(target)
        .with_context(|| format!("failed to create {}", target.display()))?;
    let target_identity = capture_path_identity(target)?;
    if let Err(err) = io::copy(&mut input, &mut output).with_context(|| {
        format!(
            "failed to copy {} to {}",
            source.display(),
            target.display()
        )
    }) {
        remove_existing_if_identity_matches(target, &target_identity);
        return Err(err);
    }
    fs::set_permissions(target, metadata.permissions()).ok();
    Ok(())
}

fn copy_dir_recursive(
    source: &Path,
    target: &Path,
    metadata: &fs::Metadata,
    identity: &FileIdentity,
) -> Result<()> {
    create_parent_dir(target)?;
    fs::create_dir(target)
        .with_context(|| format!("failed to create directory {}", target.display()))?;
    fs::set_permissions(target, metadata.permissions()).ok();
    let target_identity = capture_path_identity(target)?;
    let result = (|| {
        ensure_path_identity(source, identity)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let child_source = entry.path();
            let child_target = target.join(entry.file_name());
            copy_path(&child_source, &child_target, false)?;
        }
        Ok(())
    })();
    if result.is_err() {
        remove_existing_if_identity_matches(target, &target_identity);
    }
    result
}

fn create_parent_dir(target: &Path) -> Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent {}", parent.display()))?;
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

fn path_exists_no_follow(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileIdentity {
    is_dir: bool,
    is_file: bool,
    is_symlink: bool,
    len: u64,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
}

impl FileIdentity {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;

        Self {
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            is_symlink: metadata.file_type().is_symlink(),
            len: metadata.len(),
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            dev: metadata.dev(),
            #[cfg(unix)]
            ino: metadata.ino(),
        }
    }

    fn same_path_object(&self, other: &Self) -> bool {
        let same_kind = self.is_dir == other.is_dir
            && self.is_file == other.is_file
            && self.is_symlink == other.is_symlink;
        #[cfg(unix)]
        {
            same_kind && self.dev == other.dev && self.ino == other.ino
        }
        #[cfg(not(unix))]
        {
            same_kind
        }
    }
}

fn capture_path_identity(path: &Path) -> Result<FileIdentity> {
    fs::symlink_metadata(path)
        .map(|metadata| FileIdentity::from_metadata(&metadata))
        .with_context(|| format!("failed to inspect {}", path.display()))
}

fn ensure_path_identity(path: &Path, expected: &FileIdentity) -> Result<()> {
    let actual = capture_path_identity(path)?;
    if &actual != expected {
        bail!("path changed during operation: {}", path.display());
    }
    Ok(())
}

fn ensure_same_path_object(path: &Path, expected: &FileIdentity) -> Result<()> {
    let actual = capture_path_identity(path)?;
    if !expected.same_path_object(&actual) {
        bail!("path changed during operation: {}", path.display());
    }
    Ok(())
}

fn temp_sibling_path(target: &Path) -> Result<PathBuf> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("target has no parent: {}", target.display()))?;
    let name = target
        .file_name()
        .ok_or_else(|| anyhow!("target has no file name: {}", target.display()))?
        .to_string_lossy();
    for _ in 0..100 {
        let candidate = parent.join(format!(".{name}.tersh-copy-{}", unique_suffix()));
        if !path_exists_no_follow(&candidate)? {
            return Ok(candidate);
        }
    }
    bail!(
        "failed to allocate temporary copy target for {}",
        target.display()
    );
}

#[cfg(unix)]
fn open_regular_source(path: &Path) -> Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))
}

#[cfg(not(unix))]
fn open_regular_source(path: &Path) -> Result<File> {
    File::open(path).with_context(|| format!("failed to open {}", path.display()))
}

#[cfg(unix)]
fn symlink_target_is_dir_for_creation(_source: &Path, _link_target: &Path) -> Result<bool> {
    Ok(false)
}

#[cfg(windows)]
fn symlink_target_is_dir_for_creation(source: &Path, link_target: &Path) -> Result<bool> {
    let target_path = if link_target.is_absolute() {
        link_target.to_path_buf()
    } else {
        source.parent().unwrap_or(Path::new(".")).join(link_target)
    };
    match fs::metadata(&target_path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err)
            .with_context(|| format!("failed to inspect symlink target {}", target_path.display())),
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

fn remove_existing_if_identity_matches(path: &Path, expected: &FileIdentity) {
    if ensure_same_path_object(path, expected).is_ok() {
        let _ = remove_existing(path);
    }
}

struct GuardedDeleteTarget {
    identity: FileIdentity,
}

fn guard_delete_target(path: &Path, work_root: &Path) -> Result<GuardedDeleteTarget> {
    if path.as_os_str().is_empty() || !path.is_absolute() {
        bail!("refusing to delete non-absolute path: {}", path.display());
    }
    let input_trash_root = work_root.join(".tersh-trash");
    if path == input_trash_root || path.starts_with(&input_trash_root) {
        bail!("refusing to delete .tersh-trash");
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
    if work_root.starts_with(&target) {
        bail!("refusing to delete ancestor of active work root");
    }
    let trash_root = work_root.join(".tersh-trash");
    if path == trash_root || path.starts_with(&trash_root) {
        bail!("refusing to delete .tersh-trash");
    }
    if target == trash_root || target.starts_with(&trash_root) {
        bail!("refusing to delete .tersh-trash");
    }
    Ok(GuardedDeleteTarget {
        identity: capture_path_identity(path)?,
    })
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    let source = cstring_path(source)?;
    let target = cstring_path(target)?;
    let result = unsafe { libc::renamex_np(source.as_ptr(), target.as_ptr(), libc::RENAME_EXCL) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    let source = cstring_path(source)?;
    let target = cstring_path(target)?;
    let result = unsafe {
        libc::syscall(
            libc::SYS_renameat2,
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            target.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "linux"))
))]
fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
}

#[cfg(windows)]
fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
}

#[cfg(unix)]
fn cstring_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;

    std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
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
