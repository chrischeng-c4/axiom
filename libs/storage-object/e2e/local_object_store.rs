use storage_object::{LocalObjectStore, ObjectStore, ObjectStoreError, PutCondition};

#[test]
fn local_store_preserves_bytes_versions_and_conditional_puts() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalObjectStore::open(dir.path()).unwrap();
    let first = store
        .put(
            "segments/logs/one.parquet",
            b"first",
            "application/vnd.apache.parquet",
            PutCondition::IfAbsent,
        )
        .unwrap();
    assert_eq!(store.get(&first.key).unwrap().bytes, b"first");
    assert_eq!(store.head(&first.key).unwrap(), first);
    assert!(matches!(
        store.put(&first.key, b"bad", "text/plain", PutCondition::IfAbsent),
        Err(ObjectStoreError::PreconditionFailed { .. })
    ));
    let second = store
        .put(
            &first.key,
            b"second",
            "application/octet-stream",
            PutCondition::IfVersion(first.version.clone()),
        )
        .unwrap();
    assert_ne!(first.version, second.version);
    assert_eq!(store.list("segments/logs").unwrap(), vec![second.clone()]);
    store.delete(&second.key).unwrap();
    assert!(matches!(
        store.get(&second.key),
        Err(ObjectStoreError::NotFound { .. })
    ));
}

#[test]
fn local_store_rejects_path_escape_and_symlink_components() {
    let dir = tempfile::tempdir().unwrap();
    let store = LocalObjectStore::open(dir.path()).unwrap();
    assert!(matches!(
        store.put("../escape", b"x", "text/plain", PutCondition::Any),
        Err(ObjectStoreError::InvalidKey { .. })
    ));
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(dir.path(), dir.path().join("link")).unwrap();
        assert!(matches!(
            store.put("link/escape", b"x", "text/plain", PutCondition::Any),
            Err(ObjectStoreError::UnsafePath { .. })
        ));
    }
}
