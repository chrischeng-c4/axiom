// HANDWRITE-BEGIN gap="missing-generator:hand-written:b9a36beb" tracker="standardize-gap-projects-mamba-src-pkgmgr-installer-layout-rs" reason="Existing hand-written code in projects/mamba/src/pkgmgr/installer/layout.rs requires tracked generator coverage."
// payload to the appropriate site_packages subtree; top-level archive entries
// go to `site_packages/`. Hardlink-vs-copy choice gated on `stat.st_dev`
// equality (R9 P3 optimisation).

/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Logic
use std::fs;
use std::path::{Path, PathBuf};

use super::archive::WheelArchive;
use super::InstallerError;

/// Walk every file under `staging` and copy/hardlink to the right place under
/// `site_packages` per PEP 427.
///
/// - Top-level archive entries (and `*.dist-info/`) land directly under `site_packages`.
/// - Files under `<name>-<version>.data/purelib/...` and `.../platlib/...` land under
///   `site_packages/`.
/// - Files under `<name>-<version>.data/scripts/...` land under `<site_packages>/../bin/`.
/// - Files under `<name>-<version>.data/data/...` land under the venv root (`<site_packages>/../../`).
///
/// Returns the list of placed file paths (relative to `site_packages`).
pub fn place_files(
    staging: &Path,
    site_packages: &Path,
    meta: &WheelArchive,
) -> Result<Vec<PathBuf>, InstallerError> {
    fs::create_dir_all(site_packages).map_err(|e| InstallerError::Io {
        path: Some(site_packages.to_path_buf()),
        detail: e.to_string(),
    })?;

    let stem = meta
        .dist_info_dir
        .trim_end_matches(".dist-info")
        .to_string();
    let data_root = format!("{}.data", stem);

    let bin_dir = site_packages
        .parent()
        .map(|p| p.join("bin"))
        .unwrap_or_else(|| site_packages.join("bin"));
    let venv_root = site_packages
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| site_packages.to_path_buf());

    let mut placed: Vec<PathBuf> = Vec::new();
    walk_files(staging, &mut |abs| {
        let rel = abs.strip_prefix(staging).unwrap_or(abs).to_path_buf();
        let segments: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if segments.is_empty() {
            return Ok(());
        }

        // Skip .data directory's own entries — handled below by remapping.
        let (target_root, target_rel) = if segments[0] == data_root {
            // segments: [<dist>.data, <kind>, ...]
            if segments.len() < 3 {
                return Ok(());
            }
            let kind = segments[1].as_str();
            let rest: PathBuf = segments[2..].iter().collect();
            match kind {
                "purelib" | "platlib" => (site_packages.to_path_buf(), rest),
                "scripts" => (bin_dir.clone(), rest),
                "data" => (venv_root.clone(), rest),
                other => {
                    return Err(InstallerError::MalformedWheel {
                        path: Some(abs.to_path_buf()),
                        detail: format!("unknown .data subdir '{}'", other),
                    });
                }
            }
        } else {
            (site_packages.to_path_buf(), rel.clone())
        };

        let dest = target_root.join(&target_rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| InstallerError::Io {
                path: Some(parent.to_path_buf()),
                detail: e.to_string(),
            })?;
        }

        if dest.exists() {
            return Err(InstallerError::LayoutCollision {
                path: dest.clone(),
                detail: format!("destination already exists when placing {}", abs.display()),
            });
        }

        place_one(abs, &dest)?;

        let placed_rel = dest
            .strip_prefix(site_packages)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| dest.clone());
        placed.push(placed_rel);
        Ok(())
    })?;
    let _ = meta; // unused outside data_root remap
    Ok(placed)
}

fn place_one(src: &Path, dest: &Path) -> Result<(), InstallerError> {
    // Try hardlink first (same-device fast path); fall back to copy on cross-device.
    match fs::hard_link(src, dest) {
        Ok(()) => Ok(()),
        Err(_) => fs::copy(src, dest)
            .map_err(|e| InstallerError::Io {
                path: Some(dest.to_path_buf()),
                detail: e.to_string(),
            })
            .map(|_| ()),
    }
}

fn walk_files<F>(root: &Path, callback: &mut F) -> Result<(), InstallerError>
where
    F: FnMut(&Path) -> Result<(), InstallerError>,
{
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| InstallerError::Io {
            path: Some(dir.clone()),
            detail: e.to_string(),
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            let ft = entry.file_type().map_err(|e| InstallerError::Io {
                path: Some(path.clone()),
                detail: e.to_string(),
            })?;
            if ft.is_dir() {
                stack.push(path);
            } else {
                callback(&path)?;
            }
        }
    }
    Ok(())
}
// HANDWRITE-END
