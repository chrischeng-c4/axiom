// HANDWRITE-BEGIN gap="missing-generator:hand-written:e1c4453f" tracker="standardize-gap-projects-mamba-src-pkgmgr-installer-uninstall-rs" reason="Existing hand-written code in projects/mamba/src/pkgmgr/installer/uninstall.rs requires tracked generator coverage."
// site_packages, deletes every listed file (idempotent on missing entries),
// removes the dist-info directory. Errors when no dist-info matches the name.

/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Logic
use std::fs;
use std::path::{Path, PathBuf};

use super::record;
use super::InstallerError;

/// PEP 503 name normalisation: lowercase + runs of `[-_.]` → `-`.
fn normalise_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        if c == '-' || c == '_' || c == '.' {
            if !prev_sep {
                out.push('-');
                prev_sep = true;
            }
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    out
}

fn locate_dist_info(name: &str, site_packages: &Path) -> Result<Option<PathBuf>, InstallerError> {
    if !site_packages.exists() {
        return Ok(None);
    }
    let normalised = normalise_name(name);
    let entries = fs::read_dir(site_packages).map_err(|e| InstallerError::Io {
        path: Some(site_packages.to_path_buf()),
        detail: e.to_string(),
    })?;
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let s = file_name.to_string_lossy();
        if !s.ends_with(".dist-info") {
            continue;
        }
        let stem = &s[..s.len() - ".dist-info".len()];
        let Some((dist, _ver)) = stem.rsplit_once('-') else {
            continue;
        };
        if normalise_name(dist) == normalised {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

/// Remove every RECORD-listed file plus the dist-info directory itself.
/// Missing files are tolerated (PEP 376 is silent — pip ignores them too).
pub fn run(name: &str, site_packages: &Path) -> Result<(), InstallerError> {
    let dist_info =
        locate_dist_info(name, site_packages)?.ok_or_else(|| InstallerError::NotInstalled {
            name: name.to_string(),
            site_packages: site_packages.to_path_buf(),
        })?;

    let record_path = dist_info.join("RECORD");
    let record_text = fs::read_to_string(&record_path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            InstallerError::MalformedWheel {
                path: Some(record_path.clone()),
                detail: format!("dist-info has no RECORD: {}", dist_info.display()),
            }
        } else {
            InstallerError::Io {
                path: Some(record_path.clone()),
                detail: e.to_string(),
            }
        }
    })?;
    let entries = record::parse(&record_text)?;

    // Delete every file the RECORD listed (RECORD's own row is its self-entry; remove it last with the dir).
    let dist_info_name = dist_info
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let record_self = format!("{}/RECORD", dist_info_name);

    for entry in &entries {
        if entry.path == record_self {
            continue;
        }
        let abs = site_packages.join(&entry.path);
        match fs::remove_file(&abs) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(InstallerError::Io {
                    path: Some(abs),
                    detail: e.to_string(),
                });
            }
        }
        // Best-effort: drop the parent dir if it became empty.
        if let Some(parent) = abs.parent() {
            if parent != site_packages && parent.exists() {
                let _ = fs::remove_dir(parent);
            }
        }
    }

    fs::remove_dir_all(&dist_info).map_err(|e| InstallerError::Io {
        path: Some(dist_info.clone()),
        detail: e.to_string(),
    })?;

    Ok(())
}
// HANDWRITE-END
