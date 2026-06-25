// HANDWRITE-BEGIN gap="missing-generator:source:4dc10013" tracker="pending-tracker" reason="Port lumen's upgrade self-update implementation (target detect, release query, version select, download, sha256 verify, extract, atomic replace) to vat."
//! `vat upgrade` — self-update the installed binary from its own GitHub
//! releases.
//!
//! vat publishes one `vat-<target>.tar.gz` (+ `.sha256`) per release tag
//! (`vat@X.Y.Z`), with the inner layout `vat-<target>/{vat,README.md}`. This
//! module resolves the running target + version, picks the latest stable (or a
//! pinned) release, downloads the matching asset, verifies its sha256, extracts
//! the inner binary, and atomically replaces the running executable.
//!
//! The version compare / asset-name / checksum / extraction logic is pure and
//! unit-tested without any network or filesystem mutation; only the HTTPS
//! download + self-replacement path is gated behind the `self-update` feature
//! (it needs the otherwise-optional HTTP client + tokio runtime).
//!
//! @spec projects/vat/tech-design/interfaces/cli/vat-upgrade-and-report-issue-subcommands-for-the-mandatory-cli-c.md#cli

use std::io::Read;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use semver::Version;
use sha2::{Digest, Sha256};

/// GitHub repository that publishes vat release assets.
pub const DEFAULT_REPO: &str = "chrischeng-c4/axiom";

/// Release-tag prefix; tags look like `vat@0.3.62`.
const TAG_PREFIX: &str = "vat@";

/// Options for an upgrade run (mirrors the CLI flags).
pub struct Options {
    /// Report current vs latest without changing the binary.
    pub check: bool,
    /// Install this exact version (`0.3.62` or `vat@0.3.62`) instead of the latest.
    pub tag: Option<String>,
    /// Reinstall even if already on the selected version.
    pub force: bool,
    /// Skip the interactive confirmation prompt.
    pub yes: bool,
}

/// The decision after comparing the installed and selected versions.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    /// Already on the selected version (and `--force` was not given).
    UpToDate,
    /// A replacement should be installed.
    Install,
}

/// The exact target triple cargo built this binary for, stamped by `build.rs`.
pub fn current_target() -> &'static str {
    env!("VAT_TARGET")
}

/// The release-asset filename for a target, e.g. `vat-aarch64-apple-darwin.tar.gz`.
pub fn asset_name(target: &str) -> String {
    format!("vat-{target}.tar.gz")
}

/// The published checksum sidecar name for an asset.
pub fn sha_name(asset: &str) -> String {
    format!("{asset}.sha256")
}

/// Parse a release tag (`vat@X.Y.Z`) into a semver. Returns `None` for tags that
/// are not vat releases or are not valid semver — callers skip those rather than
/// treating them as an error.
pub fn parse_tag(tag: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix(TAG_PREFIX)?).ok()
}

/// Pick the `(tag, version)` to install. With `pin` set, match that exact version
/// (accepting either `X.Y.Z` or `vat@X.Y.Z`); otherwise choose the highest stable
/// (non-prerelease) release.
pub fn select_version(tags: &[String], pin: Option<&str>) -> Option<(String, Version)> {
    if let Some(pin) = pin {
        let want = pin.strip_prefix(TAG_PREFIX).unwrap_or(pin);
        return tags.iter().find_map(|t| {
            let v = parse_tag(t)?;
            (v.to_string() == want).then(|| (t.clone(), v))
        });
    }
    tags.iter()
        .filter_map(|t| parse_tag(t).map(|v| (t.clone(), v)))
        .filter(|(_, v)| v.pre.is_empty())
        .max_by(|a, b| a.1.cmp(&b.1))
}

/// Verify that `bytes` hash to the expected sha256 hex digest (case-insensitive;
/// surrounding whitespace on the expected string is ignored).
pub fn verify_sha256(bytes: &[u8], expected: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_lower(&hasher.finalize()).eq_ignore_ascii_case(expected.trim())
}

fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Extract the inner `vat-<target>/vat` binary bytes from a gzip tarball. Errors
/// if the expected entry is absent.
pub fn extract_binary(tar_gz: &[u8], target: &str) -> Result<Vec<u8>> {
    let want = format!("vat-{target}/vat");
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tar_gz));
    for entry in archive.entries().context("read tar entries")? {
        let mut entry = entry.context("read tar entry")?;
        let path = entry.path().context("read tar entry path")?;
        if path.to_string_lossy() == want {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).context("read inner binary")?;
            return Ok(buf);
        }
    }
    bail!("release tarball does not contain `{want}`")
}

/// Decide whether to install: a no-op when already on `selected` unless `force`.
pub fn decide_action(current: &Version, selected: &Version, force: bool) -> Action {
    if !force && selected == current {
        Action::UpToDate
    } else {
        Action::Install
    }
}

/// The version this binary was compiled as.
pub fn current_version() -> Result<Version> {
    Version::parse(env!("CARGO_PKG_VERSION")).context("parse built-in version")
}

/// Sync CLI entry: build a tokio runtime and run the async self-update.
#[cfg(feature = "self-update")]
pub fn exec(opts: Options) -> Result<ExitCode> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(run_with_repo(opts, DEFAULT_REPO))?;
    Ok(ExitCode::SUCCESS)
}

/// Without the `self-update` feature the HTTP client is not linked, so the
/// command degrades clearly instead of silently missing.
#[cfg(not(feature = "self-update"))]
pub fn exec(_opts: Options) -> Result<ExitCode> {
    bail!(
        "this vat build was compiled without self-update support; rebuild with \
         default features (the published binary includes it)"
    )
}

#[cfg(feature = "self-update")]
async fn run_with_repo(opts: Options, repo: &str) -> Result<()> {
    let target = current_target();
    let current = current_version()?;

    let client = reqwest::Client::builder()
        .user_agent(concat!("vat/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("build HTTP client")?;

    let tags = list_release_tags(&client, repo).await?;
    let Some((tag, selected)) = select_version(&tags, opts.tag.as_deref()) else {
        match opts.tag.as_deref() {
            Some(t) => bail!(
                "no vat release matching `{t}` (scanned {} tags)",
                tags.len()
            ),
            None => bail!("no stable vat release found (scanned {} tags)", tags.len()),
        }
    };

    if opts.check {
        println!("current: {current}");
        println!("latest:  {selected} ({tag})");
        println!(
            "{}",
            if selected > current {
                "→ run `vat upgrade` to update"
            } else {
                "→ up to date"
            }
        );
        return Ok(());
    }

    if decide_action(&current, &selected, opts.force) == Action::UpToDate {
        println!("already up to date ({current})");
        return Ok(());
    }

    let asset = asset_name(target);
    let (tar_url, sha_url) = asset_urls(&client, repo, &tag, &asset).await?;

    if !opts.yes && !confirm(&current, &selected)? {
        println!("aborted");
        return Ok(());
    }

    let tar_bytes = download_bytes(&client, &tar_url).await?;
    let sha_text = download_text(&client, &sha_url).await?;
    let expected = sha_text.split_whitespace().next().unwrap_or_default();
    if !verify_sha256(&tar_bytes, expected) {
        bail!("checksum mismatch for {asset}; aborting (existing binary unchanged)");
    }

    let bin = extract_binary(&tar_bytes, target)?;
    install_binary(&bin)?;
    println!("upgraded {current} → {selected}");
    Ok(())
}

/// Prompt on an interactive terminal; non-interactive sessions proceed (callers
/// gate on `--yes` first, so this only runs when a human is present).
#[cfg(feature = "self-update")]
fn confirm(current: &Version, selected: &Version) -> Result<bool> {
    use std::io::{IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Ok(true);
    }
    print!("upgrade vat {current} → {selected}? [y/N] ");
    std::io::stdout().flush().ok();
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .context("read confirmation")?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "Yes"))
}

#[cfg(feature = "self-update")]
async fn list_release_tags(client: &reqwest::Client, repo: &str) -> Result<Vec<String>> {
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=100");
    let value: serde_json::Value = github_get(client, &url)
        .await?
        .json()
        .await
        .context("parse releases")?;
    Ok(value
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|r| r.get("tag_name")?.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default())
}

/// Resolve the `(tarball, sha256)` download URLs for `asset` in release `tag`.
#[cfg(feature = "self-update")]
async fn asset_urls(
    client: &reqwest::Client,
    repo: &str,
    tag: &str,
    asset: &str,
) -> Result<(String, String)> {
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let value: serde_json::Value = github_get(client, &url)
        .await?
        .json()
        .await
        .context("parse release")?;
    let assets = value.get("assets").and_then(|a| a.as_array());
    let find = |name: &str| -> Option<String> {
        assets?.iter().find_map(|a| {
            (a.get("name")?.as_str()? == name)
                .then(|| a.get("browser_download_url")?.as_str().map(String::from))
                .flatten()
        })
    };
    let sha = sha_name(asset);
    let tar_url = find(asset)
        .with_context(|| format!("release {tag} has no asset `{asset}` for this target"))?;
    let sha_url =
        find(&sha).with_context(|| format!("release {tag} is missing the checksum `{sha}`"))?;
    Ok((tar_url, sha_url))
}

#[cfg(feature = "self-update")]
async fn github_get(client: &reqwest::Client, url: &str) -> Result<reqwest::Response> {
    let mut req = client
        .get(url)
        .header("Accept", "application/vnd.github+json");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }
    }
    req.send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("GitHub API error for {url}"))
}

#[cfg(feature = "self-update")]
async fn download_bytes(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    Ok(resp.bytes().await.context("read download body")?.to_vec())
}

#[cfg(feature = "self-update")]
async fn download_text(client: &reqwest::Client, url: &str) -> Result<String> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    resp.text().await.context("read download body")
}

/// Atomically replace the running executable with `bin`: write a sibling temp
/// file in the same directory (same filesystem ⇒ atomic `rename`), make it
/// executable, then rename over the current binary. On a permission failure the
/// existing binary is left untouched and a remediation hint is surfaced.
#[cfg(feature = "self-update")]
fn install_binary(bin: &[u8]) -> Result<()> {
    let exe = std::env::current_exe().context("locate current executable")?;
    let exe = exe.canonicalize().unwrap_or(exe);
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("current executable has no parent directory"))?;
    let tmp = dir.join(format!(".vat-upgrade-{}.tmp", std::process::id()));

    let write = || -> std::io::Result<()> {
        std::fs::write(&tmp, bin)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))?;
        }
        Ok(())
    };
    if let Err(e) = write() {
        let _ = std::fs::remove_file(&tmp);
        return Err(install_error(e, &exe));
    }
    if let Err(e) = std::fs::rename(&tmp, &exe) {
        let _ = std::fs::remove_file(&tmp);
        return Err(install_error(e, &exe));
    }
    Ok(())
}

#[cfg(feature = "self-update")]
fn install_error(e: std::io::Error, exe: &std::path::Path) -> anyhow::Error {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        anyhow::anyhow!(
            "cannot replace {}: permission denied. Re-run with elevated permissions \
             (e.g. `sudo vat upgrade`) or reinstall manually.",
            exe.display()
        )
    } else {
        anyhow::anyhow!("failed to install new binary at {}: {e}", exe.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_and_sha_names() {
        assert_eq!(
            asset_name("aarch64-apple-darwin"),
            "vat-aarch64-apple-darwin.tar.gz"
        );
        assert_eq!(
            sha_name("vat-aarch64-apple-darwin.tar.gz"),
            "vat-aarch64-apple-darwin.tar.gz.sha256"
        );
    }

    #[test]
    fn parse_tag_skips_non_vat_and_malformed() {
        assert_eq!(parse_tag("vat@1.2.3"), Some(Version::new(1, 2, 3)));
        assert!(parse_tag("lumen@0.4.3").is_none());
        assert!(parse_tag("vat@not-semver").is_none());
        assert!(parse_tag("0.3.62").is_none());
    }

    #[test]
    fn select_latest_stable() {
        let tags = vec![
            "vat@0.3.0".to_string(),
            "vat@0.3.62".to_string(),
            "vat@0.3.100".to_string(),
            "lumen@9.9.9".to_string(),
        ];
        let (tag, v) = select_version(&tags, None).unwrap();
        assert_eq!(tag, "vat@0.3.100");
        assert_eq!(v, Version::new(0, 3, 100));
    }

    #[test]
    fn select_exact_tag_override() {
        let tags = vec![
            "vat@0.3.0".to_string(),
            "vat@0.3.62".to_string(),
            "vat@0.3.100".to_string(),
        ];
        let (tag, v) = select_version(&tags, Some("0.3.62")).unwrap();
        assert_eq!(tag, "vat@0.3.62");
        assert_eq!(v, Version::new(0, 3, 62));
        let (tag2, _) = select_version(&tags, Some("vat@0.3.0")).unwrap();
        assert_eq!(tag2, "vat@0.3.0");
        assert!(select_version(&tags, Some("1.0.0")).is_none());
    }

    #[test]
    fn prerelease_excluded_from_latest_but_pinnable() {
        let tags = vec!["vat@1.0.0".to_string(), "vat@1.1.0-rc.1".to_string()];
        let (tag, _) = select_version(&tags, None).unwrap();
        assert_eq!(tag, "vat@1.0.0");
        let (pin, _) = select_version(&tags, Some("1.1.0-rc.1")).unwrap();
        assert_eq!(pin, "vat@1.1.0-rc.1");
    }

    #[test]
    fn verify_sha256_match_and_mismatch() {
        // sha256("hello")
        let expected = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert!(verify_sha256(b"hello", expected));
        assert!(verify_sha256(b"hello", &expected.to_uppercase()));
        assert!(verify_sha256(b"hello", &format!("  {expected}\n")));
        assert!(!verify_sha256(b"hello!", expected));
    }

    #[test]
    fn decide_action_uptodate_vs_install() {
        let v = Version::new(0, 3, 62);
        assert_eq!(decide_action(&v, &v, false), Action::UpToDate);
        assert_eq!(decide_action(&v, &v, true), Action::Install);
        assert_eq!(
            decide_action(&Version::new(0, 3, 61), &v, false),
            Action::Install
        );
    }

    #[test]
    fn extract_binary_reads_inner_and_errs_when_absent() {
        let target = "test-triple";
        let payload = b"#!/bin/sh\necho vat\n";
        let gz = make_tar_gz(&[(format!("vat-{target}/vat"), payload.as_slice())]);
        assert_eq!(extract_binary(&gz, target).unwrap(), payload);

        let gz_missing = make_tar_gz(&[("vat-other/vat".to_string(), payload.as_slice())]);
        assert!(extract_binary(&gz_missing, target).is_err());
    }

    fn make_tar_gz(files: &[(String, &[u8])]) -> Vec<u8> {
        use flate2::{write::GzEncoder, Compression};
        let mut enc = GzEncoder::new(Vec::new(), Compression::fast());
        {
            let mut builder = tar::Builder::new(&mut enc);
            for (name, data) in files {
                let mut header = tar::Header::new_gnu();
                header.set_size(data.len() as u64);
                header.set_mode(0o755);
                header.set_cksum();
                builder.append_data(&mut header, name, *data).unwrap();
            }
            builder.finish().unwrap();
        }
        enc.finish().unwrap()
    }
}
// HANDWRITE-END
