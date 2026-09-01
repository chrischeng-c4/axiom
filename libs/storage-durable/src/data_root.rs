//! Private, versioned, single-process data-root mechanics.
//!
//! The library owns filesystem safety. A service supplies its manifest and
//! compatibility policy through [`DataRootPolicy`].

use std::{
    fs::{self, File, OpenOptions},
    path::{Component, Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use fs2::FileExt;
use serde::{de::DeserializeOwned, Serialize};

use crate::{atomic_write, FsyncPolicy};

/// Service-owned manifest and compatibility hooks for a shared data root.
pub trait DataRootPolicy {
    type Manifest: Clone + Serialize + DeserializeOwned;

    fn product_name(&self) -> &'static str;

    fn manifest_file(&self) -> &'static str {
        "layout.json"
    }

    fn directories(&self) -> &'static [&'static str];

    fn legacy_markers(&self) -> &'static [&'static str] {
        &[]
    }

    fn create_manifest(&self, root: &Path) -> Result<Self::Manifest>;

    fn validate_manifest(&self, manifest: &Self::Manifest) -> Result<()>;

    fn legacy_error(&self, marker: &Path) -> anyhow::Error {
        anyhow::anyhow!(
            "legacy {} data at {} is not compatible",
            self.product_name(),
            marker.display()
        )
    }
}

/// Holds the exclusive directory lock for the life of one service process.
pub struct DataRoot<P: DataRootPolicy> {
    root: PathBuf,
    manifest_path: PathBuf,
    manifest: P::Manifest,
    policy: P,
    _root_lock: File,
}

impl<P: DataRootPolicy> DataRoot<P> {
    pub fn open(root: impl AsRef<Path>, policy: P) -> Result<Self> {
        let root = root.as_ref();
        reject_symlink(root)?;
        fs::create_dir_all(root).with_context(|| {
            format!(
                "create {} data directory {}",
                policy.product_name(),
                root.display()
            )
        })?;
        require_directory(root)?;
        set_private_directory_mode(root)?;

        let manifest_path = root.join(policy.manifest_file());
        reject_symlink(&manifest_path)?;
        if !manifest_path.exists() {
            refuse_legacy_root(root, &policy)?;
        } else {
            require_regular_file(&manifest_path)?;
        }

        let root_lock = OpenOptions::new().read(true).open(root).with_context(|| {
            format!(
                "open {} data root {} for locking",
                policy.product_name(),
                root.display()
            )
        })?;
        root_lock.try_lock_exclusive().with_context(|| {
            format!(
                "lock {} data root {}; another {} process may be using it",
                policy.product_name(),
                root.display(),
                policy.product_name()
            )
        })?;

        let manifest = if manifest_path.exists() {
            let manifest = serde_json::from_slice(
                &fs::read(&manifest_path)
                    .with_context(|| format!("read layout {}", manifest_path.display()))?,
            )
            .with_context(|| format!("decode layout {}", manifest_path.display()))?;
            policy.validate_manifest(&manifest)?;
            set_private_file_mode(&manifest_path)?;
            manifest
        } else {
            let manifest = policy.create_manifest(root)?;
            policy.validate_manifest(&manifest)?;
            write_manifest(&manifest_path, &manifest)?;
            manifest
        };

        for relative in policy.directories() {
            ensure_private_relative_directory(root, relative)?;
        }

        Ok(Self {
            root: root.to_path_buf(),
            manifest_path,
            manifest,
            policy,
            _root_lock: root_lock,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &P::Manifest {
        &self.manifest
    }

    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    pub fn replace_manifest(&mut self, manifest: P::Manifest) -> Result<()> {
        self.policy.validate_manifest(&manifest)?;
        write_manifest(&self.manifest_path, &manifest)?;
        self.manifest = manifest;
        Ok(())
    }
}

fn write_manifest<T: Serialize>(path: &Path, manifest: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(manifest).context("encode data-root layout")?;
    atomic_write(path, &bytes, FsyncPolicy::Always)?;
    set_private_file_mode(path)
}

fn refuse_legacy_root<P: DataRootPolicy>(root: &Path, policy: &P) -> Result<()> {
    if let Some(marker) = policy
        .legacy_markers()
        .iter()
        .map(|marker| root.join(marker))
        .find(|path| path.exists())
    {
        return Err(policy.legacy_error(&marker));
    }
    Ok(())
}

fn ensure_private_relative_directory(root: &Path, relative: &str) -> Result<()> {
    let relative_path = Path::new(relative);
    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            !matches!(component, Component::Normal(_) | Component::CurDir)
        })
    {
        bail!("data-root directory must be a safe relative path: {relative}");
    }

    let mut current = root.to_path_buf();
    for component in relative_path.components() {
        if let Component::Normal(component) = component {
            current.push(component);
            reject_symlink(&current)?;
            match fs::symlink_metadata(&current) {
                Ok(_) => require_directory(&current)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    fs::create_dir(&current).with_context(|| {
                        format!("create storage directory {}", current.display())
                    })?;
                }
                Err(error) => {
                    return Err(error).with_context(|| format!("inspect {}", current.display()))
                }
            }
            set_private_directory_mode(&current)?;
        }
    }
    Ok(())
}

pub fn reject_symlink(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            bail!("data path must not be a symlink: {}", path.display())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

fn require_directory(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect data directory {}", path.display()))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        bail!("data path must be a real directory: {}", path.display());
    }
    Ok(())
}

fn require_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect data file {}", path.display()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        bail!("data path must be a regular file: {}", path.display());
    }
    Ok(())
}

#[cfg(unix)]
pub fn set_private_directory_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("set private directory mode on {}", path.display()))
}

#[cfg(not(unix))]
pub fn set_private_directory_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
pub fn set_private_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
        .with_context(|| format!("set private file mode on {}", path.display()))
}

#[cfg(not(unix))]
pub fn set_private_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}
