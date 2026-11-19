//! Updates from the official stable GitHub release. Checking only reads metadata;
//! callers must obtain consent before invoking installation.
use anyhow::{Context, Result, bail, ensure};
use semver::Version;
use serde::Deserialize;
use std::{
    cmp::Ordering,
    fs::{self, File},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tempfile::{NamedTempFile, TempDir};

const LATEST: &str = "https://api.github.com/repos/QiushanHuang/Tersh/releases/latest";
const DOWNLOAD: &str = "https://github.com/QiushanHuang/Tersh/releases/download";
const METADATA_LIMIT: u64 = 512 * 1024;
const ARCHIVE_LIMIT: u64 = 32 * 1024 * 1024;
const BINARY_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Deserialize)]
struct Asset {
    name: String,
    digest: Option<String>,
}

#[derive(Deserialize)]
struct Metadata {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<Asset>,
}

pub struct Release {
    tag: String,
    version: Version,
    assets: Vec<Asset>,
}

impl Release {
    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn is_newer(&self) -> bool {
        let current = Version::parse(env!("CARGO_PKG_VERSION")).expect("Cargo package version");
        self.version.cmp_precedence(&current) == Ordering::Greater
    }

    fn from_json(json: &[u8]) -> Result<Self> {
        let metadata: Metadata =
            serde_json::from_slice(json).context("invalid release metadata")?;
        ensure!(
            !metadata.draft && !metadata.prerelease,
            "expected a published stable release"
        );
        let version = Version::parse(
            metadata
                .tag_name
                .strip_prefix('v')
                .context("release tag lacks v prefix")?,
        )?;
        ensure!(version.pre.is_empty(), "expected a stable release tag");
        Ok(Self {
            tag: metadata.tag_name,
            version,
            assets: metadata.assets,
        })
    }

    fn archive(&self, platform: &str) -> Result<(String, Option<&str>)> {
        let name = format!("tersh-{}-{platform}.tar.gz", self.tag);
        let asset = self
            .assets
            .iter()
            .find(|a| a.name == name)
            .with_context(|| format!("release {} has no {platform} archive yet", self.tag))?;
        let sidecar = format!("{name}.sha256");
        ensure!(
            self.assets.iter().any(|a| a.name == sidecar),
            "release has no checksum for {name}"
        );
        let digest = asset
            .digest
            .as_deref()
            .map(|raw| {
                raw.strip_prefix("sha256:")
                    .filter(|v| valid_hash(v))
                    .context("invalid GitHub archive digest")
            })
            .transpose()?;
        Ok((name, digest))
    }
}

fn platform() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => Ok("macos-arm64"),
        ("linux", "aarch64") => Ok("linux-arm64"),
        ("linux", "x86_64") => Ok("linux-x86_64"),
        (os, arch) => bail!("no release binary for {os}/{arch}"),
    }
}

fn download(url: &str, destination: &Path, limit: u64, timeout: &str) -> Result<()> {
    let result = Command::new("curl")
        .args([
            "-q",
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--max-redirs",
            "5",
            "--connect-timeout",
            "1.5",
            "--max-time",
            timeout,
            "--user-agent",
            "tersh-updater",
            "--max-filesize",
            &limit.to_string(),
            "--output",
        ])
        .arg(destination)
        .arg(url)
        .stdin(Stdio::null())
        .output()
        .context("run curl to contact GitHub")?;
    ensure!(
        result.status.success(),
        "GitHub request failed: {}",
        String::from_utf8_lossy(&result.stderr).trim()
    );
    ensure!(
        fs::metadata(destination)?.len() <= limit,
        "download exceeds size limit"
    );
    Ok(())
}

pub fn check() -> Result<Release> {
    let file = NamedTempFile::new()?;
    download(LATEST, file.path(), METADATA_LIMIT, "3")?;
    let release = Release::from_json(&fs::read(file.path())?)?;
    if release.is_newer() {
        release.archive(platform()?)?;
    }
    Ok(release)
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

fn sidecar_hash<'a>(input: &'a str, name: &str) -> Result<&'a str> {
    let mut parts = input.split_whitespace();
    let digest = parts.next().context("empty release checksum")?;
    ensure!(
        valid_hash(digest)
            && parts.next().map(|n| n.strip_prefix('*').unwrap_or(n)) == Some(name)
            && parts.next().is_none(),
        "invalid release checksum or archive name"
    );
    Ok(digest)
}

fn archive_hash(file: &Path) -> Result<String> {
    let (program, args): (_, &[&str]) = if cfg!(target_os = "macos") {
        ("shasum", &["-a", "256"])
    } else {
        ("sha256sum", &[])
    };
    let result = Command::new(program)
        .args(args)
        .stdin(File::open(file)?)
        .output()
        .with_context(|| format!("run {program}"))?;
    ensure!(
        result.status.success(),
        "could not hash the release archive"
    );
    let output = std::str::from_utf8(&result.stdout)?;
    let value = output
        .split_whitespace()
        .next()
        .filter(|v| valid_hash(v))
        .context("invalid SHA-256 output")?;
    Ok(value.to_owned())
}

fn executable_version(executable: &Path) -> Result<Version> {
    let output = Command::new(executable)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("check executable version at {}", executable.display()))?;
    ensure!(output.status.success(), "executable version check failed");
    let value = std::str::from_utf8(&output.stdout)?
        .trim()
        .strip_prefix("tersh ")
        .context("unexpected executable version response")?;
    Version::parse(value).context("invalid executable version")
}

// Extract exactly one fixed member to an already-open file. No archive path,
// installer, owner or permission metadata is ever applied to the filesystem.
fn extract(archive: &Path, staged: &mut File) -> Result<()> {
    let mut tar = Command::new("tar")
        .arg("-xzOf")
        .arg(archive)
        .arg("tersh")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("run tar to unpack the release")?;
    let copied = std::io::copy(
        &mut tar.stdout.take().unwrap().take(BINARY_LIMIT + 1),
        staged,
    );
    if !matches!(copied, Ok(n) if n <= BINARY_LIMIT) {
        let _ = tar.kill();
    }
    let status = tar.wait()?;
    let count = copied?;
    ensure!(
        status.success() && count > 0 && count <= BINARY_LIMIT,
        "invalid release executable in archive"
    );
    staged.flush()?;
    Ok(())
}

/// Replace the executable that started this process. The version on disk is
/// checked under a directory lock, including when another invocation won the race.
pub fn install(release: &Release) -> Result<PathBuf> {
    ensure!(release.is_newer(), "no newer stable release to install");
    let target = std::env::current_exe()?
        .canonicalize()
        .context("resolve running Tersh executable")?;
    install_at(release, platform()?, &target, download)?;
    Ok(target)
}

fn install_at(
    release: &Release,
    platform: &str,
    target: &Path,
    fetch: impl Fn(&str, &Path, u64, &str) -> Result<()>,
) -> Result<()> {
    let parent = target
        .parent()
        .context("executable has no parent directory")?;
    let directory = File::open(parent)?;
    // The directory's inode survives an atomic replacement. A killed updater
    // drops its lock automatically and leaves the installed executable intact.
    if unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(std::io::Error::last_os_error())
            .context("cannot lock installation directory; another update may be running");
    }
    let original = fs::metadata(target)?;
    match release.version.cmp_precedence(&executable_version(target)?) {
        Ordering::Equal => return Ok(()),
        Ordering::Less => bail!("a newer Tersh version is already installed"),
        Ordering::Greater => {}
    }
    let (name, github_digest) = release.archive(platform)?;
    let work = TempDir::new()?;
    let archive = work.path().join(&name);
    let checksum = work.path().join("checksum");
    let url = format!("{DOWNLOAD}/{}/{}", release.tag, name);
    fetch(&format!("{url}.sha256"), &checksum, 4096, "15")?;
    let checksum = fs::read_to_string(checksum)?;
    let expected = sidecar_hash(&checksum, &name)?;
    ensure!(
        github_digest.is_none_or(|d| d.eq_ignore_ascii_case(expected)),
        "GitHub digest disagrees with the release checksum"
    );
    fetch(&url, &archive, ARCHIVE_LIMIT, "90")?;
    ensure!(
        archive_hash(&archive)?.eq_ignore_ascii_case(expected),
        "release checksum mismatch"
    );

    let mut staged = NamedTempFile::new_in(parent)
        .context("cannot create executable in installation directory")?;
    extract(&archive, staged.as_file_mut())?;
    staged
        .as_file()
        .set_permissions(fs::Permissions::from_mode(original.mode() & 0o777))?;
    staged.as_file().sync_all()?;
    let staged = staged.into_temp_path(); // Linux cannot exec an open writer.
    ensure!(
        executable_version(&staged)? == release.version,
        "staged executable version does not match release tag"
    );
    let disk = fs::metadata(target)?;
    ensure!(
        original.dev() == disk.dev()
            && original.ino() == disk.ino()
            && original.ctime() == disk.ctime()
            && original.ctime_nsec() == disk.ctime_nsec(),
        "installed Tersh changed while downloading; rerun the update check"
    );
    staged
        .persist(target)
        .context("atomically replace installed Tersh")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_stable_tags_from_published_releases_can_be_installed() {
        let metadata = |tag: &str, draft: bool, prerelease: bool| {
            Release::from_json(
                &serde_json::to_vec(&serde_json::json!({
                    "tag_name": tag, "draft": draft, "prerelease": prerelease, "assets":[],
                }))
                .unwrap(),
            )
        };
        for tag in ["1.4.0", "v1.4.0-rc.1", "v1.4.0/elsewhere", "v01.4.0"] {
            assert!(metadata(tag, false, false).is_err());
        }
        assert!(metadata("v2.0.0", true, false).is_err());
        assert!(metadata("v2.0.0", false, true).is_err());
        let stable = metadata("v1.10.0", false, false).unwrap();
        for earlier in ["1.9.99", "1.10.0-rc.2", "1.10.0-beta.12"] {
            assert_eq!(
                stable
                    .version
                    .cmp_precedence(&Version::parse(earlier).unwrap()),
                Ordering::Greater
            );
        }
        assert_eq!(
            stable
                .version
                .cmp_precedence(&Version::parse("1.10.0+other").unwrap()),
            Ordering::Equal
        );
    }
}
