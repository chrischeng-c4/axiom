// HANDWRITE-BEGIN gap="missing-generator:hand-written:32b74adb" tracker="standardize-gap-projects-mamba-src-pkgmgr-installer-archive-rs" reason="Existing hand-written code in projects/mamba/src/pkgmgr/installer/archive.rs requires tracked generator coverage."
// Produces a `WheelArchive` with the dist-info prefix and computed name/version.
// Surfaces malformed-wheel errors (missing WHEEL, missing RECORD, multiple
// dist-info dirs).

/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Schema
/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Logic
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use super::InstallerError;

/// Result of opening + structurally validating a wheel ZIP.
#[derive(Debug, Clone)]
pub struct WheelArchive {
    /// `{name}-{version}.dist-info/` prefix inside the archive (no trailing slash).
    pub dist_info_dir: String,
    pub dist_name: String,
    pub dist_version: String,
    pub has_entry_points: bool,
    pub has_data_dir: bool,
}

/// Opaque opened wheel. Phase-1.3 keeps the entry list in memory and re-opens
/// the file for extraction; cheap for the wheel sizes we target.
pub struct OpenedWheel {
    artifact_path: PathBuf,
    metadata: WheelArchive,
    #[allow(dead_code)]
    entries: Vec<String>,
}

impl OpenedWheel {
    pub fn metadata(&self) -> &WheelArchive {
        &self.metadata
    }

    /// Extract every entry into `dest`, preserving the archive's relative paths.
    /// Top-level archive entries land directly under `dest`; layout reshuffling
    /// happens later in `layout::place_files`.
    pub fn extract_to(&mut self, dest: &Path) -> Result<(), InstallerError> {
        fs::create_dir_all(dest).map_err(|e| InstallerError::Io {
            path: Some(dest.to_path_buf()),
            detail: e.to_string(),
        })?;

        let file = File::open(&self.artifact_path).map_err(|e| InstallerError::Io {
            path: Some(self.artifact_path.clone()),
            detail: e.to_string(),
        })?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| InstallerError::MalformedWheel {
            path: Some(self.artifact_path.clone()),
            detail: e.to_string(),
        })?;

        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| InstallerError::MalformedWheel {
                    path: Some(self.artifact_path.clone()),
                    detail: e.to_string(),
                })?;

            // Reject directory traversal.
            let Some(rel) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
                return Err(InstallerError::MalformedWheel {
                    path: Some(self.artifact_path.clone()),
                    detail: format!("unsafe entry path: {}", entry.name()),
                });
            };

            let out_path = dest.join(&rel);
            if entry.is_dir() {
                fs::create_dir_all(&out_path).map_err(|e| InstallerError::Io {
                    path: Some(out_path.clone()),
                    detail: e.to_string(),
                })?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(|e| InstallerError::Io {
                        path: Some(parent.to_path_buf()),
                        detail: e.to_string(),
                    })?;
                }
                let mut buf = Vec::with_capacity(entry.size() as usize);
                io::copy(&mut entry, &mut buf).map_err(|e| InstallerError::Io {
                    path: Some(out_path.clone()),
                    detail: e.to_string(),
                })?;
                fs::write(&out_path, &buf).map_err(|e| InstallerError::Io {
                    path: Some(out_path.clone()),
                    detail: e.to_string(),
                })?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Some(mode) = entry.unix_mode() {
                        let _ = fs::set_permissions(&out_path, fs::Permissions::from_mode(mode));
                    }
                }
            }
        }
        Ok(())
    }
}

/// Open the wheel at `path`, validate it has exactly one `.dist-info/` directory
/// containing both `WHEEL` and `RECORD`, and parse `{name}-{version}` out of the
/// dist-info directory name.
pub fn open_wheel(path: &Path) -> Result<OpenedWheel, InstallerError> {
    let file = File::open(path).map_err(|e| InstallerError::Io {
        path: Some(path.to_path_buf()),
        detail: e.to_string(),
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| InstallerError::MalformedWheel {
        path: Some(path.to_path_buf()),
        detail: e.to_string(),
    })?;

    let mut entries: Vec<String> = Vec::with_capacity(zip.len());
    let mut dist_info_dirs: Vec<String> = Vec::new();
    let mut has_entry_points = false;
    let mut has_data_dir = false;
    let mut has_wheel_meta = false;
    let mut has_record = false;

    for i in 0..zip.len() {
        let entry = zip
            .by_index(i)
            .map_err(|e| InstallerError::MalformedWheel {
                path: Some(path.to_path_buf()),
                detail: e.to_string(),
            })?;
        let name = entry.name().to_string();
        entries.push(name.clone());

        if let Some(top) = name.split('/').next() {
            if top.ends_with(".dist-info") && !dist_info_dirs.iter().any(|d| d == top) {
                dist_info_dirs.push(top.to_string());
            }
            if top.ends_with(".data") {
                has_data_dir = true;
            }
        }
    }

    let dist_info = match dist_info_dirs.as_slice() {
        [] => {
            return Err(InstallerError::MalformedWheel {
                path: Some(path.to_path_buf()),
                detail: "no .dist-info directory in wheel".to_string(),
            });
        }
        [one] => one.clone(),
        _ => {
            return Err(InstallerError::MalformedWheel {
                path: Some(path.to_path_buf()),
                detail: format!("multiple dist-info dirs: {:?}", dist_info_dirs),
            });
        }
    };

    let wheel_meta = format!("{}/WHEEL", dist_info);
    let record_meta = format!("{}/RECORD", dist_info);
    let entry_points = format!("{}/entry_points.txt", dist_info);
    for e in &entries {
        if e == &wheel_meta {
            has_wheel_meta = true;
        }
        if e == &record_meta {
            has_record = true;
        }
        if e == &entry_points {
            has_entry_points = true;
        }
    }

    if !has_wheel_meta {
        return Err(InstallerError::MalformedWheel {
            path: Some(path.to_path_buf()),
            detail: format!("missing {}", wheel_meta),
        });
    }
    if !has_record {
        return Err(InstallerError::MalformedWheel {
            path: Some(path.to_path_buf()),
            detail: format!("missing {}", record_meta),
        });
    }

    let stem = dist_info.trim_end_matches(".dist-info");
    let (dist_name, dist_version) =
        stem.rsplit_once('-')
            .ok_or_else(|| InstallerError::MalformedWheel {
                path: Some(path.to_path_buf()),
                detail: format!(
                    "dist-info name not '{{name}}-{{version}}.dist-info': {}",
                    dist_info
                ),
            })?;

    Ok(OpenedWheel {
        artifact_path: path.to_path_buf(),
        metadata: WheelArchive {
            dist_info_dir: dist_info.clone(),
            dist_name: dist_name.to_string(),
            dist_version: dist_version.to_string(),
            has_entry_points,
            has_data_dir,
        },
        entries,
    })
}

/// Read one named file out of an opened wheel into memory. Used by tests; the
/// install path goes through `extract_to` instead.
#[allow(dead_code)]
pub fn read_archive_file(path: &Path, archive_path: &str) -> Result<Vec<u8>, InstallerError> {
    let file = File::open(path).map_err(|e| InstallerError::Io {
        path: Some(path.to_path_buf()),
        detail: e.to_string(),
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| InstallerError::MalformedWheel {
        path: Some(path.to_path_buf()),
        detail: e.to_string(),
    })?;
    let mut entry = zip
        .by_name(archive_path)
        .map_err(|e| InstallerError::MalformedWheel {
            path: Some(path.to_path_buf()),
            detail: format!("{}: {}", archive_path, e),
        })?;
    let mut buf = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut buf)
        .map_err(|e| InstallerError::Io {
            path: Some(path.to_path_buf()),
            detail: e.to_string(),
        })?;
    Ok(buf)
}
// HANDWRITE-END
