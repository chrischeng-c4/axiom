use std::sync::{Arc, Mutex};

use storage_object::{
    Object, ObjectMeta, ObjectStore, ObjectStoreError, ObjectVersion, PutCondition,
};
use storage_segment::{ArchiveCoordinator, ArchiveObject};

#[derive(Default)]
struct RecordingStore {
    puts: Mutex<Vec<String>>,
    fail_key: Mutex<Option<String>>,
}

impl RecordingStore {
    fn fail_on(&self, key: &str) {
        *self.fail_key.lock().unwrap() = Some(key.to_string());
    }
}

impl ObjectStore for RecordingStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        _condition: PutCondition,
    ) -> storage_object::Result<ObjectMeta> {
        self.puts.lock().unwrap().push(key.to_string());
        if self.fail_key.lock().unwrap().as_deref() == Some(key) {
            return Err(ObjectStoreError::Unavailable {
                message: "injected outage".to_string(),
            });
        }
        Ok(ObjectMeta {
            key: key.to_string(),
            size: bytes.len() as u64,
            content_type: content_type.to_string(),
            version: ObjectVersion::new(format!("v-{}", bytes.len())),
            etag: None,
            updated: None,
        })
    }

    fn get(&self, key: &str) -> storage_object::Result<Object> {
        Err(ObjectStoreError::NotFound {
            key: key.to_string(),
        })
    }

    fn head(&self, key: &str) -> storage_object::Result<ObjectMeta> {
        Err(ObjectStoreError::NotFound {
            key: key.to_string(),
        })
    }

    fn list(&self, _prefix: &str) -> storage_object::Result<Vec<ObjectMeta>> {
        Ok(Vec::new())
    }

    fn delete(&self, _key: &str) -> storage_object::Result<()> {
        Ok(())
    }
}

#[test]
fn immutable_objects_are_written_before_the_manifest_receipt() {
    let store = Arc::new(RecordingStore::default());
    let coordinator = ArchiveCoordinator::new(store.clone());
    let mut transaction = coordinator.begin();
    transaction
        .put(ArchiveObject::new(
            "archive/segment.parquet",
            b"segment".to_vec(),
            "application/vnd.apache.parquet",
        ))
        .unwrap();
    let receipt = transaction
        .commit(ArchiveObject::new(
            "archive/manifest.json",
            br#"{"ok":true}"#.to_vec(),
            "application/json",
        ))
        .unwrap();

    assert_eq!(
        store.puts.lock().unwrap().as_slice(),
        ["archive/segment.parquet", "archive/manifest.json"]
    );
    assert_eq!(receipt.objects.len(), 1);
    assert_eq!(receipt.manifest.key, "archive/manifest.json");
}

#[test]
fn failed_segment_upload_cannot_create_an_archive_commit() {
    let store = Arc::new(RecordingStore::default());
    store.fail_on("archive/segment.parquet");
    let coordinator = ArchiveCoordinator::new(store.clone());
    let mut transaction = coordinator.begin();
    assert!(transaction
        .put(ArchiveObject::new(
            "archive/segment.parquet",
            b"segment".to_vec(),
            "application/vnd.apache.parquet",
        ))
        .is_err());
    assert_eq!(
        store.puts.lock().unwrap().as_slice(),
        ["archive/segment.parquet"]
    );
}
