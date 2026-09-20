use anyhow::{Context, Result, anyhow, bail};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeleteDecision {
    MovedToTrash { from: PathBuf, to: PathBuf },
}

/// The observer is called at recursive boundaries and after at most 128 KiB.
/// Returning an error aborts the current root copy and removes its incomplete output.
pub fn copy_path_cancellable(
    source: &Path,
    target: &Path,
    replace: bool,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    observer(source, 0)?;
    if path_exists_no_follow(target)? {
        if replace {
            return replace_path(source, target, observer);
        }
        bail!("target already exists: {}", target.display());
    }
    copy_path_no_replace(source, target, observer)
}

pub fn copy_path(source: &Path, target: &Path, replace: bool) -> Result<()> {
    copy_path_cancellable(source, target, replace, &mut |_, _| Ok(()))
}

fn copy_path_no_replace(
    source: &Path,
    target: &Path,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    observer(source, 0)?;
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
        copy_dir_recursive(source, target, &metadata, &identity, observer)
    } else if metadata.is_file() {
        copy_regular_file(source, target, &metadata, &identity, observer)
    } else {
        bail!("unsupported file type: {}", source.display());
    }
}

fn replace_path(
    source: &Path,
    target: &Path,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    let target_identity = capture_path_identity(target)?;
    let temp = temp_sibling_path(target)?;
    let staged = (|| {
        copy_path_no_replace(source, &temp, observer)?;
        observer(source, 0)?;
        ensure_path_identity(target, &target_identity)
    })();
    if let Err(err) = staged {
        let _ = remove_copy_output(&temp);
        return Err(err);
    }
    // Commit is one non-cancellable section. Keep the old target until the new
    // complete copy is installed, with a no-clobber rollback if installation fails.
    let backup = temp_sibling_path(target)?;
    if let Err(err) = rename_no_replace(target, &backup) {
        let _ = remove_copy_output(&temp);
        return Err(err).context("failed to preserve replacement target");
    }
    if let Err(err) = rename_no_replace(&temp, target) {
        let rollback = rename_no_replace(&backup, target);
        let _ = remove_copy_output(&temp);
        return match rollback {
            Ok(()) => Err(err).context("failed to install replacement; original restored"),
            Err(rollback) => Err(anyhow!(
                "failed to install replacement: {err}; original retained at {} (rollback: {rollback})",
                backup.display()
            )),
        };
    }
    remove_existing(&backup).with_context(|| {
        format!(
            "replacement installed; failed to remove old backup {}",
            backup.display()
        )
    })?;
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
    crate::trash::trash_path(path, work_root)
}

pub(crate) fn prepare_trash_dir(work_root: &Path) -> Result<PathBuf> {
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
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
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
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .with_context(|| format!("failed to create {}", target.display()))?;
    let copied = (|| {
        let mut buffer = [0_u8; 128 * 1024];
        loop {
            observer(source, 0)?;
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            output.write_all(&buffer[..count])?;
            observer(source, count as u64)?;
        }
        ensure_path_identity(source, identity)?;
        Ok(())
    })();
    if let Err(err) = copied {
        drop(output);
        let _ = fs::remove_file(target);
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
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    create_parent_dir(target)?;
    fs::create_dir(target)
        .with_context(|| format!("failed to create directory {}", target.display()))?;
    let result = (|| {
        ensure_path_identity(source, identity)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let child_source = entry.path();
            let child_target = target.join(entry.file_name());
            observer(source, 0)?;
            ensure_same_object(source, identity)?;
            copy_path_no_replace(&child_source, &child_target, observer)?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = remove_copy_output(target);
    } else {
        fs::set_permissions(target, metadata.permissions()).ok();
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

pub(crate) fn path_exists_no_follow(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err).with_context(|| format!("failed to inspect {}", path.display())),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FileIdentity {
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
}

pub(crate) fn capture_path_identity(path: &Path) -> Result<FileIdentity> {
    fs::symlink_metadata(path)
        .map(|metadata| FileIdentity::from_metadata(&metadata))
        .with_context(|| format!("failed to inspect {}", path.display()))
}

pub(crate) fn ensure_path_identity(path: &Path, expected: &FileIdentity) -> Result<()> {
    let actual = capture_path_identity(path)?;
    if &actual != expected {
        bail!("path changed during operation: {}", path.display());
    }
    Ok(())
}

pub(crate) fn ensure_same_object(path: &Path, expected: &FileIdentity) -> Result<()> {
    let actual = capture_path_identity(path)?;
    #[cfg(unix)]
    let same = actual.dev == expected.dev
        && actual.ino == expected.ino
        && actual.is_dir == expected.is_dir
        && actual.is_symlink == expected.is_symlink;
    #[cfg(not(unix))]
    let same = actual.is_dir == expected.is_dir && actual.is_symlink == expected.is_symlink;
    if !same {
        bail!("path changed during operation: {}", path.display());
    }
    Ok(())
}

/// Cancellation of a directory deletion can leave a partially deleted tree.
/// Symlinks are removed as links and never traversed.
pub fn permanent_delete_cancellable(
    path: &Path,
    work_root: &Path,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    let guarded = guard_delete_target(path, work_root)?;
    ensure_path_identity(path, &guarded.identity)?;
    #[cfg(unix)]
    {
        delete_unix_cancellable(path, &guarded.identity, observer)
    }
    #[cfg(not(unix))]
    {
        delete_recursive_cancellable(path, observer)
    }
}

#[cfg(not(unix))]
fn delete_recursive_cancellable(
    path: &Path,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    observer(path, 0)?;
    let metadata = fs::symlink_metadata(path)?;
    let identity = FileIdentity::from_metadata(&metadata);
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        ensure_path_identity(path, &identity)?;
        for child in fs::read_dir(path)? {
            observer(path, 0)?;
            ensure_same_object(path, &identity)?;
            delete_recursive_cancellable(&child?.path(), observer)?;
        }
        observer(path, 0)?;
        ensure_same_object(path, &identity)?;
        fs::remove_dir(path)?;
    } else {
        ensure_path_identity(path, &identity)?;
        fs::remove_file(path)?;
    }
    Ok(())
}

// Pin each directory by descriptor. Path-based recursion can traverse an
// unrelated directory if another process swaps an ancestor for a symlink.
#[cfg(unix)]
fn delete_unix_cancellable(
    path: &Path,
    identity: &FileIdentity,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("delete target has no parent"))?;
    let parent_identity = capture_path_identity(parent)?;
    let parent_handle = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(parent)?;
    if FileIdentity::from_metadata(&parent_handle.metadata()?) != parent_identity {
        bail!("delete parent changed while opening");
    }
    let name = cstring_path(Path::new(
        path.file_name()
            .ok_or_else(|| anyhow!("missing file name"))?,
    ))?;
    delete_at(
        parent_handle.as_raw_fd(),
        &name,
        path,
        Some(identity),
        observer,
    )
}

#[cfg(unix)]
fn delete_at(
    parent: std::os::fd::RawFd,
    name: &std::ffi::CStr,
    display: &Path,
    expected: Option<&FileIdentity>,
    observer: &mut dyn FnMut(&Path, u64) -> Result<()>,
) -> Result<()> {
    use std::os::{
        fd::{AsRawFd, FromRawFd},
        unix::ffi::OsStrExt,
    };
    observer(display, 0)?;
    let before = stat_at(parent, name)?;
    if let Some(expected) = expected {
        if before.st_dev as u64 != expected.dev || before.st_ino != expected.ino {
            bail!("delete target changed while opening");
        }
    }
    if before.st_mode & libc::S_IFMT == libc::S_IFDIR {
        let raw = unsafe {
            libc::openat(
                parent,
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(io::Error::last_os_error().into());
        }
        let directory = unsafe { File::from_raw_fd(raw) };
        let opened = FileIdentity::from_metadata(&directory.metadata()?);
        if opened.dev != before.st_dev as u64 || opened.ino != before.st_ino {
            bail!("directory changed while opening: {}", display.display());
        }
        let duplicate = unsafe { libc::dup(directory.as_raw_fd()) };
        if duplicate < 0 {
            return Err(io::Error::last_os_error().into());
        }
        let stream = unsafe { libc::fdopendir(duplicate) };
        if stream.is_null() {
            let error = io::Error::last_os_error();
            unsafe {
                libc::close(duplicate);
            }
            return Err(error.into());
        }
        struct DirectoryStream(*mut libc::DIR);
        impl Drop for DirectoryStream {
            fn drop(&mut self) {
                unsafe {
                    libc::closedir(self.0);
                }
            }
        }
        let stream = DirectoryStream(stream);
        loop {
            // A readdir error is treated as an error; it must not look like EOF.
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
            unsafe {
                *libc::__error() = 0;
            }
            #[cfg(any(target_os = "linux", target_os = "android"))]
            unsafe {
                *libc::__errno_location() = 0;
            }
            let entry = unsafe { libc::readdir(stream.0) };
            if entry.is_null() {
                let error = io::Error::last_os_error();
                if error.raw_os_error().unwrap_or_default() != 0 {
                    return Err(error.into());
                }
                break;
            }
            let child = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_owned();
            if child.to_bytes() == b"." || child.to_bytes() == b".." {
                continue;
            }
            observer(display, 0)?;
            let child_display = display.join(std::ffi::OsStr::from_bytes(child.to_bytes()));
            delete_at(
                directory.as_raw_fd(),
                &child,
                &child_display,
                None,
                observer,
            )?;
        }
        observer(display, 0)?;
        ensure_stat_at(parent, name, &before)?;
        if unsafe { libc::unlinkat(parent, name.as_ptr(), libc::AT_REMOVEDIR) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
    } else {
        ensure_stat_at(parent, name, &before)?;
        if unsafe { libc::unlinkat(parent, name.as_ptr(), 0) } != 0 {
            return Err(io::Error::last_os_error().into());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn stat_at(parent: std::os::fd::RawFd, name: &std::ffi::CStr) -> Result<libc::stat> {
    let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe {
        libc::fstatat(
            parent,
            name.as_ptr(),
            metadata.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(io::Error::last_os_error().into());
    }
    Ok(unsafe { metadata.assume_init() })
}

#[cfg(unix)]
fn ensure_stat_at(
    parent: std::os::fd::RawFd,
    name: &std::ffi::CStr,
    expected: &libc::stat,
) -> Result<()> {
    let actual = stat_at(parent, name)?;
    if actual.st_dev != expected.st_dev
        || actual.st_ino != expected.st_ino
        || actual.st_mode & libc::S_IFMT != expected.st_mode & libc::S_IFMT
    {
        bail!("path changed during deletion");
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
        .custom_flags(libc::O_NOFOLLOW)
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

// Only use this for staging trees created by this copy. A completed child may
// already have inherited read-only source permissions when a later step aborts.
fn remove_copy_output(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                path,
                fs::Permissions::from_mode(metadata.permissions().mode() | 0o700),
            )?;
        }
        for child in fs::read_dir(path)? {
            remove_copy_output(&child?.path())?;
        }
        fs::remove_dir(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
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

pub(crate) struct GuardedDeleteTarget {
    pub(crate) identity: FileIdentity,
}

pub(crate) fn guard_delete_target(path: &Path, work_root: &Path) -> Result<GuardedDeleteTarget> {
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
pub(crate) fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
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
pub(crate) fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
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
pub(crate) fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
}

#[cfg(windows)]
pub(crate) fn rename_no_replace(source: &Path, target: &Path) -> io::Result<()> {
    fs::rename(source, target)
}

#[cfg(unix)]
fn cstring_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;

    std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
}

pub(crate) fn delete_identity(path: &Path) -> Result<PathBuf> {
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

pub(crate) fn unique_suffix() -> String {
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
