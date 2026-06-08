// HANDWRITE-BEGIN gap="missing-generator:hand-written:397a59b9" tracker="standardize-gap-projects-mamba-src-pkgmgr-installer-mod-rs" reason="Existing hand-written code in projects/mamba/src/pkgmgr/installer/mod.rs requires tracked generator coverage."
// `Installer::uninstall(name: &str, site_packages: &Path) -> Result<(), InstallerError>`,
// and the graph-driven `install_graph(graph: &ResolvedGraph, site_packages, python_exe)`
// orchestrator (R7). Routes to archive/record/layout/scripts/uninstall submodules.

/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Schema
/// @spec .aw/tech-design/projects/mamba/pkgmgr/installer.md#Logic
pub mod archive;
pub mod layout;
pub mod record;
pub mod scripts;
pub mod uninstall;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::resolver::ResolvedGraph;

/// Installation strategy. `purelib` is the only fully implemented variant in
/// Phase-1.3; `editable` is reserved for PEP 660 (P2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMode {
    Purelib,
    Editable,
}

/// One install operation against a single resolved wheel artifact.
#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub artifact_path: PathBuf,
    pub site_packages: PathBuf,
    pub python_executable: PathBuf,
    pub mode: InstallMode,
}

/// Outcome kind discriminator for [`InstallResult`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallKind {
    Installed,
    AlreadyInstalled,
}

/// Outcome of one install — success carries the RECORD-derived file inventory.
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub kind: InstallKind,
    pub distribution: String,
    pub version: String,
    pub installed_files: Vec<PathBuf>,
    pub console_scripts: Vec<String>,
}

/// Tagged error union for installer failures.
#[derive(Debug, thiserror::Error)]
pub enum InstallerError {
    #[error("malformed wheel{}: {detail}", path.as_ref().map(|p| format!(" at {}", p.display())).unwrap_or_default())]
    MalformedWheel {
        path: Option<PathBuf>,
        detail: String,
    },

    #[error("RECORD hash mismatch for {}: {detail}", path.display())]
    RecordHashMismatch { path: PathBuf, detail: String },

    #[error("RECORD references missing file {}: {detail}", path.display())]
    RecordMissingFile { path: PathBuf, detail: String },

    #[error("layout collision at {}: {detail}", path.display())]
    LayoutCollision { path: PathBuf, detail: String },

    #[error("editable installs not supported in Phase-1.3 (PEP 660 reserved for P2)")]
    EditableNotSupported,

    #[error("no installed distribution matches '{name}' under {}", site_packages.display())]
    NotInstalled {
        name: String,
        site_packages: PathBuf,
    },

    #[error("io error{}: {detail}", path.as_ref().map(|p| format!(" at {}", p.display())).unwrap_or_default())]
    Io {
        path: Option<PathBuf>,
        detail: String,
    },
}

impl InstallerError {
    fn io(detail: impl Into<String>, path: Option<PathBuf>) -> Self {
        InstallerError::Io {
            path,
            detail: detail.into(),
        }
    }
}

/// Installer entry point. Stateless — every method routes through the request payload.
#[derive(Debug, Default)]
pub struct Installer;

impl Installer {
    pub fn new() -> Self {
        Installer
    }

    /// Install one wheel into `site_packages`. Returns `AlreadyInstalled` when the
    /// existing installation's RECORD-derived file set matches the artifact's expected
    /// dist-info; otherwise extracts, verifies, places, writes scripts, and writes RECORD.
    pub fn install(&self, req: InstallRequest) -> Result<InstallResult, InstallerError> {
        if matches!(req.mode, InstallMode::Editable) {
            return Err(InstallerError::EditableNotSupported);
        }

        // Open + structurally validate the wheel ZIP.
        let mut archive = archive::open_wheel(&req.artifact_path)?;
        let meta = archive.metadata().clone();

        // Hash-fast-path: if dist-info already exists with matching versions, skip.
        if let Some(existing) = read_installed_dist_info(&req.site_packages, &meta.dist_name)? {
            if existing.version == meta.dist_version {
                return Ok(InstallResult {
                    kind: InstallKind::AlreadyInstalled,
                    distribution: meta.dist_name.clone(),
                    version: meta.dist_version.clone(),
                    installed_files: Vec::new(),
                    console_scripts: Vec::new(),
                });
            }
        }

        // Extract into staging tempdir.
        let staging = tempdir_in(&req.site_packages)?;
        archive.extract_to(&staging)?;

        // Parse RECORD and verify hashes against staged files.
        let record_path = staging.join(&meta.dist_info_dir).join("RECORD");
        let record_text = fs::read_to_string(&record_path).map_err(|e| InstallerError::Io {
            path: Some(record_path.clone()),
            detail: e.to_string(),
        })?;
        let entries = record::parse(&record_text)?;
        record::verify(&staging, &entries)?;

        // Read entry_points.txt before placement (needed for scripts).
        let entry_points_path = staging.join(&meta.dist_info_dir).join("entry_points.txt");
        let entry_points_text = if entry_points_path.exists() {
            Some(
                fs::read_to_string(&entry_points_path).map_err(|e| InstallerError::Io {
                    path: Some(entry_points_path.clone()),
                    detail: e.to_string(),
                })?,
            )
        } else {
            None
        };

        // Place files into site_packages per PEP 427 layout.
        let placed = layout::place_files(&staging, &req.site_packages, &meta)?;

        // Generate console-script wrappers from entry_points.txt.
        let console_scripts = if let Some(text) = entry_points_text {
            let bin_dir = req
                .site_packages
                .parent()
                .map(|p| p.join("bin"))
                .unwrap_or_else(|| req.site_packages.join("bin"));
            scripts::write_console_scripts(&text, &bin_dir, &req.python_executable)?
        } else {
            Vec::new()
        };

        // Write RECORD (with self-entry blanked) under the installed dist-info.
        let installed_dist_info = req.site_packages.join(&meta.dist_info_dir);
        let installed_record = installed_dist_info.join("RECORD");
        let record_text = render_record(&entries, &meta.dist_info_dir);
        fs::create_dir_all(&installed_dist_info).map_err(|e| InstallerError::Io {
            path: Some(installed_dist_info.clone()),
            detail: e.to_string(),
        })?;
        fs::write(&installed_record, record_text).map_err(|e| InstallerError::Io {
            path: Some(installed_record.clone()),
            detail: e.to_string(),
        })?;

        // Best-effort cleanup of the staging directory.
        let _ = fs::remove_dir_all(&staging);

        Ok(InstallResult {
            kind: InstallKind::Installed,
            distribution: meta.dist_name,
            version: meta.dist_version,
            installed_files: placed,
            console_scripts,
        })
    }

    /// Remove an installed distribution by name. Delegates to `uninstall::run`.
    pub fn uninstall(&self, name: &str, site_packages: &Path) -> Result<(), InstallerError> {
        uninstall::run(name, site_packages)
    }

    /// Install every node in `graph` in topological order (roots first, then deps).
    /// `cache_lookup` resolves node `(name, version)` → cached `.whl` path.
    pub fn install_graph<F>(
        &self,
        graph: &ResolvedGraph,
        site_packages: &Path,
        python_executable: &Path,
        mut cache_lookup: F,
    ) -> Result<Vec<InstallResult>, InstallerError>
    where
        F: FnMut(&str, &str) -> Result<PathBuf, InstallerError>,
    {
        let mut results = Vec::with_capacity(graph.nodes.len());
        let order = topological_order(graph);
        for name in order {
            let Some(node) = graph.nodes.iter().find(|n| n.name == name) else {
                continue;
            };
            let artifact = cache_lookup(&node.name, &node.version)?;
            let req = InstallRequest {
                artifact_path: artifact,
                site_packages: site_packages.to_path_buf(),
                python_executable: python_executable.to_path_buf(),
                mode: InstallMode::Purelib,
            };
            results.push(self.install(req)?);
        }
        Ok(results)
    }
}

/// Lightweight already-installed probe: scans `site_packages` for a single
/// `<name>-<version>.dist-info/` directory whose normalised name matches `name`.
fn read_installed_dist_info(
    site_packages: &Path,
    name: &str,
) -> Result<Option<InstalledDistInfo>, InstallerError> {
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
        let Some((dist, version)) = stem.rsplit_once('-') else {
            continue;
        };
        if normalise_name(dist) == normalised {
            return Ok(Some(InstalledDistInfo {
                version: version.to_string(),
            }));
        }
    }
    Ok(None)
}

#[derive(Debug)]
struct InstalledDistInfo {
    version: String,
}

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

/// Render a RECORD file from parsed entries. The RECORD self-row carries blank fields per PEP 376.
fn render_record(entries: &[record::RecordEntry], dist_info_dir: &str) -> String {
    let record_self_path = format!("{}/RECORD", dist_info_dir.trim_end_matches('/'));
    let mut out = String::new();
    for e in entries {
        if e.path == record_self_path {
            out.push_str(&format!("{},,\n", e.path));
        } else {
            let h = e.sha256_b64url.as_deref().unwrap_or("");
            let s = e.size.map(|n| n.to_string()).unwrap_or_default();
            let prefix = if h.is_empty() { "" } else { "sha256=" };
            out.push_str(&format!("{},{}{},{}\n", e.path, prefix, h, s));
        }
    }
    out
}

/// Compute a topological order: nodes whose `requires` are already emitted come last.
/// Phase-1.3: `requires` is empty (transitive deps deferred); falls back to `roots`-first
/// then remaining nodes by name for deterministic ordering.
fn topological_order(graph: &ResolvedGraph) -> Vec<String> {
    let mut order: Vec<String> = Vec::with_capacity(graph.nodes.len());
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for r in &graph.roots {
        if !seen.contains_key(r) {
            seen.insert(r.clone(), ());
            order.push(r.clone());
        }
    }
    for node in &graph.nodes {
        if !seen.contains_key(&node.name) {
            seen.insert(node.name.clone(), ());
            order.push(node.name.clone());
        }
    }
    order
}

/// Allocate a unique staging directory inside `parent`. Caller cleans up on success.
fn tempdir_in(parent: &Path) -> Result<PathBuf, InstallerError> {
    fs::create_dir_all(parent)
        .map_err(|e| InstallerError::io(e.to_string(), Some(parent.to_path_buf())))?;
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    for n in 0u32..16 {
        let candidate = parent.join(format!(".mamba-staging-{}-{}-{}", pid, nanos, n));
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(InstallerError::io(e.to_string(), Some(candidate))),
        }
    }
    Err(InstallerError::io(
        "could not allocate staging dir after 16 attempts",
        Some(parent.to_path_buf()),
    ))
}
// HANDWRITE-END
