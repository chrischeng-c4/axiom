//! `<tool> upgrade` — self-update the installed binary from the tool's own
//! GitHub releases (`<project>@X.Y.Z` tags, `<project>-<target>.tar.gz` assets
//! with a `.sha256` sidecar, inner layout `<project>-<target>/<project>`).
//!
//! Pure version/asset/checksum/extraction logic is unit-tested; the HTTPS
//! download + atomic self-replacement live behind the `online` feature.

use crate::ToolInfo;
use anyhow::{bail, Context, Result};
use semver::Version;
use std::io::Read;

/// Flags for an upgrade run.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// Report current vs latest without changing the binary.
    pub check: bool,
    /// Install this exact version (`X.Y.Z` or `<project>@X.Y.Z`).
    pub tag: Option<String>,
    /// Reinstall even when already on the selected version.
    pub force: bool,
    /// Skip the confirmation prompt.
    pub yes: bool,
}

/// Decision after comparing the installed and selected versions.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    UpToDate,
    Install,
}

/// Parse a release tag (`<prefix>X.Y.Z`) into a semver, or `None` if it is not
/// this tool's tag or not valid semver.
pub fn parse_tag(tag: &str, prefix: &str) -> Option<Version> {
    Version::parse(tag.strip_prefix(prefix)?).ok()
}

/// Pick the `(tag, version)` to install: an exact `pin` (accepting `X.Y.Z` or
/// `<prefix>X.Y.Z`), else the highest stable (non-prerelease) release.
pub fn select_version(
    tags: &[String],
    prefix: &str,
    pin: Option<&str>,
) -> Option<(String, Version)> {
    if let Some(pin) = pin {
        let want = pin.strip_prefix(prefix).unwrap_or(pin);
        return tags.iter().find_map(|t| {
            let v = parse_tag(t, prefix)?;
            (v.to_string() == want).then(|| (t.clone(), v))
        });
    }
    tags.iter()
        .filter_map(|t| parse_tag(t, prefix).map(|v| (t.clone(), v)))
        .filter(|(_, v)| v.pre.is_empty())
        .max_by(|a, b| a.1.cmp(&b.1))
}

/// Verify `bytes` hash to the expected sha256 hex digest (case-insensitive;
/// surrounding whitespace ignored).
pub fn verify_sha256(bytes: &[u8], expected: &str) -> bool {
    use sha2::{Digest, Sha256};
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

/// Extract the inner binary bytes (`<inner_path>`) from a gzip tarball.
pub fn extract_binary(tar_gz: &[u8], inner_path: &str) -> Result<Vec<u8>> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tar_gz));
    for entry in archive.entries().context("read tar entries")? {
        let mut entry = entry.context("read tar entry")?;
        if entry
            .path()
            .context("read tar entry path")?
            .to_string_lossy()
            == inner_path
        {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).context("read inner binary")?;
            return Ok(buf);
        }
    }
    bail!("release tarball does not contain `{inner_path}`")
}

/// Decide whether to install: a no-op when already on `selected` unless `force`.
pub fn decide_action(current: &Version, selected: &Version, force: bool) -> Action {
    if !force && selected == current {
        Action::UpToDate
    } else {
        Action::Install
    }
}

/// Run `<tool> upgrade`. Offline builds (no `online` feature) only support the
/// install path through a clear error; `--check` still degrades clearly.
#[cfg(feature = "online")]
pub async fn run(tool: &ToolInfo, opts: Options) -> Result<()> {
    let prefix = tool.tag_prefix();
    let current = Version::parse(tool.version).context("parse current version")?;
    let client = reqwest::Client::builder()
        .user_agent(format!("{}-upgrade/{}", tool.project, tool.version))
        .build()
        .context("build HTTP client")?;

    let tags = list_release_tags(&client, tool.repo).await?;
    let Some((tag, selected)) = select_version(&tags, &prefix, opts.tag.as_deref()) else {
        if opts.check && opts.tag.is_none() {
            println!("current: {current}");
            println!("latest:  none");
            println!(
                "→ no stable {} release found (scanned {} tags)",
                tool.project,
                tags.len()
            );
            return Ok(());
        }
        match opts.tag.as_deref() {
            Some(t) => bail!(
                "no {} release matching `{t}` (scanned {} tags)",
                tool.project,
                tags.len()
            ),
            None => bail!(
                "no stable {} release found (scanned {} tags)",
                tool.project,
                tags.len()
            ),
        }
    };

    if opts.check {
        println!("current: {current}");
        println!("latest:  {selected} ({tag})");
        println!(
            "{}",
            if selected > current {
                format!("→ run `{} upgrade` to update", tool.project)
            } else {
                "→ up to date".to_string()
            }
        );
        return Ok(());
    }

    if decide_action(&current, &selected, opts.force) == Action::UpToDate {
        println!("already up to date ({current})");
        return Ok(());
    }

    let asset = tool.asset_name();
    let (tar_url, sha_url) = asset_urls(&client, tool.repo, &tag, &asset).await?;

    if !opts.yes && !crate::confirm(&format!("upgrade {} {current} → {selected}?", tool.project))?
    {
        println!("aborted");
        return Ok(());
    }

    let tar_bytes = crate::download_bytes(&client, &tar_url).await?;
    let expected = crate::download_text(&client, &sha_url)
        .await?
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .to_string();
    if !verify_sha256(&tar_bytes, &expected) {
        bail!("checksum mismatch for {asset}; aborting (existing binary unchanged)");
    }

    let bin = extract_binary(&tar_bytes, &tool.inner_binary_path())?;
    crate::install_over_self(&bin, &format!("{}-upgrade", tool.project))?;
    println!("upgraded {current} → {selected}");
    Ok(())
}

/// Without the `online` feature the HTTP client is not linked.
#[cfg(not(feature = "online"))]
pub async fn run(tool: &ToolInfo, _opts: Options) -> Result<()> {
    anyhow::bail!(
        "this {} build was compiled without self-update support (the `online` feature)",
        tool.project
    )
}

#[cfg(feature = "online")]
async fn list_release_tags(client: &reqwest::Client, repo: &str) -> Result<Vec<String>> {
    let url = format!("https://api.github.com/repos/{repo}/releases?per_page=100");
    let value: serde_json::Value = crate::github_get(client, &url)
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

#[cfg(feature = "online")]
async fn asset_urls(
    client: &reqwest::Client,
    repo: &str,
    tag: &str,
    asset: &str,
) -> Result<(String, String)> {
    let url = format!("https://api.github.com/repos/{repo}/releases/tags/{tag}");
    let value: serde_json::Value = crate::github_get(client, &url)
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
    let sha = format!("{asset}.sha256");
    let tar_url = find(asset)
        .with_context(|| format!("release {tag} has no asset `{asset}` for this target"))?;
    let sha_url =
        find(&sha).with_context(|| format!("release {tag} is missing the checksum `{sha}`"))?;
    Ok((tar_url, sha_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_and_select() {
        let tags = vec![
            "lumen@0.4.0".into(),
            "lumen@0.4.10".into(),
            "vat@9.9.9".into(),
        ];
        assert_eq!(
            parse_tag("lumen@1.2.3", "lumen@"),
            Some(Version::new(1, 2, 3))
        );
        assert!(parse_tag("vat@1.0.0", "lumen@").is_none());
        let (tag, v) = select_version(&tags, "lumen@", None).unwrap();
        assert_eq!(tag, "lumen@0.4.10");
        assert_eq!(v, Version::new(0, 4, 10));
        let (pin, _) = select_version(&tags, "lumen@", Some("0.4.0")).unwrap();
        assert_eq!(pin, "lumen@0.4.0");
    }

    #[test]
    fn sha_and_action() {
        let exp = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";
        assert!(verify_sha256(b"hello", exp));
        assert!(verify_sha256(b"hello", &exp.to_uppercase()));
        assert!(!verify_sha256(b"nope", exp));
        let v = Version::new(1, 0, 0);
        assert_eq!(decide_action(&v, &v, false), Action::UpToDate);
        assert_eq!(decide_action(&v, &v, true), Action::Install);
    }

    #[test]
    fn extract_inner() {
        use flate2::{write::GzEncoder, Compression};
        let inner = "lumen-t/lumen";
        let payload = b"ELF...";
        let mut enc = GzEncoder::new(Vec::new(), Compression::fast());
        {
            let mut b = tar::Builder::new(&mut enc);
            let mut h = tar::Header::new_gnu();
            h.set_size(payload.len() as u64);
            h.set_mode(0o755);
            h.set_cksum();
            b.append_data(&mut h, inner, payload.as_slice()).unwrap();
            b.finish().unwrap();
        }
        let gz = enc.finish().unwrap();
        assert_eq!(extract_binary(&gz, inner).unwrap(), payload);
        assert!(extract_binary(&gz, "lumen-t/other").is_err());
    }
}
