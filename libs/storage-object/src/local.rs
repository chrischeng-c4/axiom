use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use storage_durable::{atomic_write, FsyncPolicy};

use crate::{
    validate_key, Object, ObjectMeta, ObjectStore, ObjectStoreError, ObjectVersion, PutCondition,
    Result,
};

const META_DIR: &str = ".object-meta";
const LOCK_FILE: &str = ".object-store.lock";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Sidecar {
    key: String,
    version: ObjectVersion,
    content_type: String,
}

pub struct LocalObjectStore {
    root: PathBuf,
    lock: File,
}

impl LocalObjectStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        if let Ok(metadata) = std::fs::symlink_metadata(root) {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err(ObjectStoreError::UnsafePath {
                    path: root.display().to_string(),
                });
            }
        } else {
            std::fs::create_dir_all(root).map_err(io_error)?;
        }
        set_dir_mode(root)?;
        let meta = root.join(META_DIR);
        std::fs::create_dir_all(&meta).map_err(io_error)?;
        set_dir_mode(&meta)?;
        let lock_path = root.join(LOCK_FILE);
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(io_error)?;
        set_file_mode(&lock_path)?;
        Ok(Self {
            root: root.to_path_buf(),
            lock,
        })
    }

    fn object_path(&self, key: &str, create_parent: bool) -> Result<PathBuf> {
        let key = validate_key(key)?;
        let mut current = self.root.clone();
        let parts = key.split('/').collect::<Vec<_>>();
        for component in &parts[..parts.len().saturating_sub(1)] {
            current.push(component);
            match std::fs::symlink_metadata(&current) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    return Err(ObjectStoreError::UnsafePath {
                        path: current.display().to_string(),
                    })
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound && create_parent => {
                    std::fs::create_dir(&current).map_err(io_error)?;
                    set_dir_mode(&current)?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(io_error(error)),
            }
        }
        let path = self.root.join(key);
        if let Ok(metadata) = std::fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(ObjectStoreError::UnsafePath {
                    path: path.display().to_string(),
                });
            }
        }
        Ok(path)
    }

    fn sidecar_path(&self, key: &str) -> PathBuf {
        let digest = Sha256::digest(key.as_bytes());
        self.root.join(META_DIR).join(format!("{digest:x}.json"))
    }

    fn locked<T>(&self, action: impl FnOnce() -> Result<T>) -> Result<T> {
        self.lock.lock_exclusive().map_err(io_error)?;
        let result = action();
        FileExt::unlock(&self.lock).map_err(io_error)?;
        result
    }

    fn head_inner(&self, key: &str) -> Result<ObjectMeta> {
        let key = validate_key(key)?;
        let path = self.object_path(key, false)?;
        let metadata = std::fs::metadata(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ObjectStoreError::NotFound {
                    key: key.to_string(),
                }
            } else {
                io_error(error)
            }
        })?;
        let mut file = File::open(&path).map_err(io_error)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(io_error)?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let version = ObjectVersion::new(format!("{:x}", hasher.finalize()));
        let sidecar = std::fs::read(self.sidecar_path(key))
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Sidecar>(&bytes).ok())
            .filter(|sidecar| sidecar.key == key && sidecar.version == version);
        let updated = metadata.modified().ok().map(|time| {
            let time: chrono::DateTime<chrono::Utc> = time.into();
            time.to_rfc3339()
        });
        Ok(ObjectMeta {
            key: key.to_string(),
            size: metadata.len(),
            content_type: sidecar
                .map(|sidecar| sidecar.content_type)
                .unwrap_or_else(|| "application/octet-stream".into()),
            version: version.clone(),
            etag: Some(version.as_str().to_string()),
            updated,
        })
    }
}

impl ObjectStore for LocalObjectStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: PutCondition,
    ) -> Result<ObjectMeta> {
        self.locked(|| {
            let key = validate_key(key)?;
            let current = self.head_inner(key);
            match condition {
                PutCondition::Any => {}
                PutCondition::IfAbsent => {
                    if current.is_ok() {
                        return Err(ObjectStoreError::PreconditionFailed {
                            key: key.to_string(),
                        });
                    }
                    if !matches!(current, Err(ObjectStoreError::NotFound { .. })) {
                        current?;
                    }
                }
                PutCondition::IfVersion(expected) => match current {
                    Ok(meta) if meta.version == expected => {}
                    Ok(_) | Err(ObjectStoreError::NotFound { .. }) => {
                        return Err(ObjectStoreError::PreconditionFailed {
                            key: key.to_string(),
                        })
                    }
                    Err(error) => return Err(error),
                },
            }
            let path = self.object_path(key, true)?;
            atomic_write(&path, bytes, FsyncPolicy::Always).map_err(io_error)?;
            set_file_mode(&path)?;
            let version = ObjectVersion::new(format!("{:x}", Sha256::digest(bytes)));
            let sidecar = Sidecar {
                key: key.to_string(),
                version,
                content_type: content_type.to_string(),
            };
            let sidecar_path = self.sidecar_path(key);
            atomic_write(
                &sidecar_path,
                &serde_json::to_vec(&sidecar).map_err(|error| ObjectStoreError::Corrupt {
                    message: error.to_string(),
                })?,
                FsyncPolicy::Always,
            )
            .map_err(io_error)?;
            set_file_mode(&sidecar_path)?;
            self.head_inner(key)
        })
    }

    fn get(&self, key: &str) -> Result<Object> {
        self.locked(|| {
            let meta = self.head_inner(key)?;
            let bytes = std::fs::read(self.object_path(key, false)?).map_err(io_error)?;
            Ok(Object { meta, bytes })
        })
    }

    fn head(&self, key: &str) -> Result<ObjectMeta> {
        self.locked(|| self.head_inner(key))
    }

    fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>> {
        self.locked(|| {
            let prefix = prefix.trim_matches('/');
            if !prefix.is_empty() {
                validate_key(prefix)?;
            }
            let mut objects = Vec::new();
            for entry in walkdir::WalkDir::new(&self.root).follow_links(false) {
                let entry = entry.map_err(|error| ObjectStoreError::Io {
                    message: error.to_string(),
                })?;
                if entry.file_type().is_symlink() {
                    return Err(ObjectStoreError::UnsafePath {
                        path: entry.path().display().to_string(),
                    });
                }
                if !entry.file_type().is_file() {
                    continue;
                }
                let relative = entry
                    .path()
                    .strip_prefix(&self.root)
                    .expect("walk entry remains below root")
                    .to_string_lossy()
                    .replace('\\', "/");
                if relative == LOCK_FILE || relative.starts_with(&format!("{META_DIR}/")) {
                    continue;
                }
                if relative.starts_with(prefix) {
                    objects.push(self.head_inner(&relative)?);
                }
            }
            objects.sort_by(|left, right| left.key.cmp(&right.key));
            Ok(objects)
        })
    }

    fn delete(&self, key: &str) -> Result<()> {
        self.locked(|| {
            let key = validate_key(key)?;
            let path = self.object_path(key, false)?;
            match std::fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
                Err(error) => return Err(io_error(error)),
            }
            let _ = std::fs::remove_file(self.sidecar_path(key));
            Ok(())
        })
    }
}

fn io_error(error: impl std::fmt::Display) -> ObjectStoreError {
    ObjectStoreError::Io {
        message: error.to_string(),
    }
}

#[cfg(unix)]
fn set_dir_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).map_err(io_error)
}

#[cfg(not(unix))]
fn set_dir_mode(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn set_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).map_err(io_error)
}

#[cfg(not(unix))]
fn set_file_mode(_path: &Path) -> Result<()> {
    Ok(())
}
