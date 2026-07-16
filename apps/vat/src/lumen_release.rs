// HANDWRITE-BEGIN gap="vat-versioned-native-lumen-release-cache" tracker="#1813" reason="Own target release discovery, verified caching, and executable resolution."
//! VAT-owned resolver for native Lumen release binaries.
//!
//! A configured `lumen@X.Y.Z` is cached below VAT's state root. The resolver
//! never invokes `lumen upgrade` and never changes a binary found on PATH.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

const REPO: &str = "chrischeng-c4/axiom";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLumen {
    pub tag: String,
    pub executable: PathBuf,
    pub cache_hit: bool,
}

pub fn normalize_selector(version: Option<&str>) -> Result<Option<String>> {
    let Some(version) = version else {
        return Ok(None);
    };
    let version = version.trim();
    if version.starts_with("lumen@")
        && version.len() > "lumen@".len()
        && version["lumen@".len()..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+'))
    {
        return Ok(Some(version.to_string()));
    }
    bail!(
        "lumen preset version must be an exact `lumen@<version>` release tag; got `{version}`"
    )
}

pub fn target() -> Result<&'static str> {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "macos") => Ok("aarch64-apple-darwin"),
        ("x86_64", "linux") => Ok("x86_64-unknown-linux-gnu"),
        ("aarch64", "linux") => Ok("aarch64-unknown-linux-gnu"),
        (arch, os) => bail!("lumen preset has no published native release for {arch}-{os}"),
    }
}

pub fn cache_root() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("VAT_LUMEN_CACHE_DIR") {
        return Ok(PathBuf::from(path));
    }
    Ok(crate::paths::root()?.join("cache").join("lumen"))
}

pub fn cached_binary(tag: &str, target: &str) -> Result<PathBuf> {
    Ok(cache_root()?.join(tag).join(target).join("lumen"))
}

pub fn resolve(version: Option<&str>) -> Result<ResolvedLumen> {
    let target = target()?;
    let tag = normalize_selector(version)?.unwrap_or_else(latest_tag);
    let binary = cached_binary(&tag, target)?;
    if executable(&binary) {
        return Ok(ResolvedLumen { tag, executable: binary, cache_hit: true });
    }
    materialize(&tag, target, &binary)?;
    Ok(ResolvedLumen { tag, executable: binary, cache_hit: false })
}

fn latest_tag() -> String {
    // The real lookup happens in materialize so an existing explicit cache can
    // remain fully offline. This sentinel cannot be mistaken for a release tag.
    "latest".to_string()
}

fn release_base() -> String {
    std::env::var("VAT_LUMEN_RELEASE_BASE_URL")
        .unwrap_or_else(|_| format!("https://github.com/{REPO}"))
        .trim_end_matches('/')
        .to_string()
}

fn materialize(requested_tag: &str, target: &str, binary: &Path) -> Result<()> {
    let tag = if requested_tag == "latest" { discover_latest()? } else { requested_tag.to_string() };
    let binary = if requested_tag == "latest" { cached_binary(&tag, target)? } else { binary.to_path_buf() };
    if executable(&binary) {
        return Ok(());
    }
    let parent = binary.parent().context("lumen cache binary has no parent")?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    let tmp = parent.join(format!(".download-{}", std::process::id()));
    if tmp.exists() { fs::remove_dir_all(&tmp).ok(); }
    fs::create_dir_all(&tmp)?;
    let archive = tmp.join(format!("lumen-{target}.tar.gz"));
    let asset = format!("lumen-{target}.tar.gz");
    let url = format!("{}/releases/download/{tag}/{asset}", release_base());
    curl_to(&url, &archive)?;
    verify_optional_checksum(&url, &archive)?;
    let status = Command::new("tar")
        .arg("-C").arg(&tmp).arg("-xzf").arg(&archive)
        .status().context("start tar for Lumen release archive")?;
    if !status.success() { bail!("extract Lumen release archive `{url}` failed with {status}"); }
    let extracted = tmp.join(format!("lumen-{target}")).join("lumen");
    if !executable(&extracted) { bail!("Lumen release archive `{url}` did not contain lumen-{target}/lumen"); }
    let staged = parent.join(format!(".lumen-{}", std::process::id()));
    fs::copy(&extracted, &staged).with_context(|| format!("copy {}", extracted.display()))?;
    #[cfg(unix)]
    { use std::os::unix::fs::PermissionsExt; fs::set_permissions(&staged, fs::Permissions::from_mode(0o755))?; }
    fs::rename(&staged, &binary).with_context(|| format!("promote {}", binary.display()))?;
    fs::remove_dir_all(&tmp).ok();
    Ok(())
}

fn discover_latest() -> Result<String> {
    let url = format!("{}/releases?per_page=100", release_base().replace("github.com/", "api.github.com/repos/"));
    let output = curl_stdout(&url)?;
    let releases: serde_json::Value = serde_json::from_slice(&output).context("parse Lumen release list")?;
    releases.as_array().and_then(|items| items.iter().find_map(|release| release.get("tag_name").and_then(|tag| tag.as_str()).filter(|tag| tag.starts_with("lumen@")).map(str::to_owned)))
        .context("no lumen@ release found while resolving latest")
}

fn curl_to(url: &str, path: &Path) -> Result<()> {
    let status = Command::new("curl").args(["-fsSL", url, "-o"]).arg(path).status().context("start curl for Lumen release")?;
    if status.success() { Ok(()) } else { bail!("download Lumen release asset `{url}` failed with {status}") }
}

fn curl_stdout(url: &str) -> Result<Vec<u8>> {
    let output = Command::new("curl").args(["-fsSL", url]).output().context("start curl for Lumen release metadata")?;
    if output.status.success() { Ok(output.stdout) } else { bail!("download Lumen release metadata `{url}` failed: {}", String::from_utf8_lossy(&output.stderr)) }
}

fn verify_optional_checksum(asset_url: &str, archive: &Path) -> Result<()> {
    let checksum = archive.with_extension("tar.gz.sha256");
    let sha_url = format!("{asset_url}.sha256");
    let status = Command::new("curl").args(["-fsSL", &sha_url, "-o"]).arg(&checksum).status().context("start curl for Lumen checksum")?;
    if !status.success() { return Ok(()); }
    let expected = fs::read_to_string(&checksum)?.split_whitespace().next().context("empty Lumen checksum")?.to_string();
    let output = Command::new("shasum").args(["-a", "256"]).arg(archive).output().context("start shasum for Lumen archive")?;
    if !output.status.success() { bail!("sha256 verification for {} failed", archive.display()); }
    let actual = String::from_utf8_lossy(&output.stdout).split_whitespace().next().unwrap_or_default().to_string();
    if actual != expected { bail!("Lumen archive checksum mismatch: expected {expected}, got {actual}"); }
    Ok(())
}

fn executable(path: &Path) -> bool { path.is_file() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selector_requires_lumen_release_tag() {
        assert_eq!(normalize_selector(None).unwrap(), None);
        assert_eq!(normalize_selector(Some("lumen@0.4.21")).unwrap(), Some("lumen@0.4.21".into()));
        assert!(normalize_selector(Some("0.4.21")).is_err());
        assert!(normalize_selector(Some("latest")).is_err());
    }
}
// HANDWRITE-END
