use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use storage_durable::{DataRoot, DataRootPolicy};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Manifest {
    version: u32,
    role: String,
}

#[derive(Clone)]
struct Policy;

impl DataRootPolicy for Policy {
    type Manifest = Manifest;

    fn product_name(&self) -> &'static str {
        "fixture"
    }

    fn directories(&self) -> &'static [&'static str] {
        &["wal/logs", "segments/logs", "tmp"]
    }

    fn legacy_markers(&self) -> &'static [&'static str] {
        &["legacy.data"]
    }

    fn create_manifest(&self, _root: &Path) -> anyhow::Result<Self::Manifest> {
        Ok(Manifest {
            version: 1,
            role: "store".into(),
        })
    }

    fn validate_manifest(&self, manifest: &Self::Manifest) -> anyhow::Result<()> {
        anyhow::ensure!(manifest.version == 1, "unsupported fixture format");
        Ok(())
    }

    fn legacy_error(&self, marker: &Path) -> anyhow::Error {
        anyhow::anyhow!("legacy fixture data at {}", marker.display())
    }
}

#[test]
fn root_is_private_versioned_and_exclusively_locked() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("data");
    let root = DataRoot::open(&path, Policy).unwrap();
    assert_eq!(root.manifest().version, 1);
    for relative in ["wal/logs", "segments/logs", "tmp"] {
        assert!(path.join(relative).is_dir());
    }
    let error = DataRoot::open(&path, Policy)
        .err()
        .expect("second owner must be refused");
    assert!(error.to_string().contains("another fixture process"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(path.join("layout.json"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn legacy_and_symlink_roots_fail_without_overwrite() {
    let legacy = tempfile::tempdir().unwrap();
    let marker = legacy.path().join("legacy.data");
    std::fs::write(&marker, b"keep-me").unwrap();
    let error = DataRoot::open(legacy.path(), Policy)
        .err()
        .expect("legacy root must fail");
    assert!(error.to_string().contains("legacy fixture data"));
    assert_eq!(std::fs::read(marker).unwrap(), b"keep-me");
    assert!(!legacy.path().join("layout.json").exists());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let parent = tempfile::tempdir().unwrap();
        let target = parent.path().join("target");
        std::fs::create_dir(&target).unwrap();
        let link: PathBuf = parent.path().join("link");
        symlink(&target, &link).unwrap();
        assert!(DataRoot::open(link, Policy).is_err());
    }
}

#[test]
fn service_can_update_its_manifest_through_the_shared_atomic_path() {
    let temp = tempfile::tempdir().unwrap();
    let mut root = DataRoot::open(temp.path(), Policy).unwrap();
    root.replace_manifest(Manifest {
        version: 1,
        role: "restored".into(),
    })
    .unwrap();
    drop(root);

    let reopened = DataRoot::open(temp.path(), Policy).unwrap();
    assert_eq!(reopened.manifest().role, "restored");
}
