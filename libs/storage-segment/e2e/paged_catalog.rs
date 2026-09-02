use std::{
    collections::BTreeMap,
    io::Write,
    sync::{Arc, Mutex},
};

use storage_object::{
    Object, ObjectMeta, ObjectStore, ObjectStoreError, ObjectVersion, PutCondition,
};
use storage_segment::{CatalogEntry, PagedCatalog, SegmentError, MAX_ABORT_TRACKED_CATALOG_PAGES};

#[derive(Default)]
struct MemoryStore {
    objects: Mutex<BTreeMap<String, (Vec<u8>, String)>>,
    gets: Mutex<u64>,
}

impl MemoryStore {
    fn reset_gets(&self) {
        *self.gets.lock().unwrap() = 0;
    }

    fn gets(&self) -> u64 {
        *self.gets.lock().unwrap()
    }

    fn len(&self) -> usize {
        self.objects.lock().unwrap().len()
    }
}

impl ObjectStore for MemoryStore {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: PutCondition,
    ) -> storage_object::Result<ObjectMeta> {
        let mut objects = self.objects.lock().unwrap();
        if matches!(condition, PutCondition::IfAbsent) && objects.contains_key(key) {
            return Err(ObjectStoreError::PreconditionFailed { key: key.into() });
        }
        objects.insert(key.into(), (bytes.to_vec(), content_type.into()));
        Ok(ObjectMeta {
            key: key.into(),
            size: bytes.len() as u64,
            content_type: content_type.into(),
            version: ObjectVersion::new(format!("v-{}", bytes.len())),
            etag: None,
            updated: None,
        })
    }

    fn get(&self, key: &str) -> storage_object::Result<Object> {
        *self.gets.lock().unwrap() += 1;
        let objects = self.objects.lock().unwrap();
        let (bytes, content_type) = objects
            .get(key)
            .cloned()
            .ok_or_else(|| ObjectStoreError::NotFound { key: key.into() })?;
        Ok(Object {
            meta: ObjectMeta {
                key: key.into(),
                size: bytes.len() as u64,
                content_type,
                version: ObjectVersion::new(format!("v-{}", bytes.len())),
                etag: None,
                updated: None,
            },
            bytes,
        })
    }

    fn head(&self, key: &str) -> storage_object::Result<ObjectMeta> {
        self.get(key).map(|object| object.meta)
    }

    fn list(&self, prefix: &str) -> storage_object::Result<Vec<ObjectMeta>> {
        Ok(self
            .objects
            .lock()
            .unwrap()
            .iter()
            .filter(|(key, _)| key.starts_with(prefix))
            .map(|(key, (bytes, content_type))| ObjectMeta {
                key: key.clone(),
                size: bytes.len() as u64,
                content_type: content_type.clone(),
                version: ObjectVersion::new(format!("v-{}", bytes.len())),
                etag: None,
                updated: None,
            })
            .collect())
    }

    fn delete(&self, key: &str) -> storage_object::Result<()> {
        self.objects.lock().unwrap().remove(key);
        Ok(())
    }
}

fn entries(count: usize) -> impl Iterator<Item = CatalogEntry> {
    (0..count).map(|index| CatalogEntry {
        key: format!("segment/{index:09}"),
        value: (index as u64).to_le_bytes().to_vec(),
    })
}

#[test]
fn failed_streaming_build_returns_every_uncommitted_page_for_cleanup() {
    let store = Arc::new(MemoryStore::default());
    let catalog = PagedCatalog::with_page_bytes(store.clone(), "failed/catalog", 4 * 1024).unwrap();
    let input = entries(5_000).enumerate().map(|(index, entry)| {
        if index == 4_000 {
            Err(SegmentError::Serialization {
                message: "injected source failure".into(),
            })
        } else {
            Ok(entry)
        }
    });
    let abort = catalog.build_sorted_with_abort(input).unwrap_err();
    assert!(abort.error.to_string().contains("injected source failure"));
    assert!(!abort.written_page_keys.is_empty());
    for key in &abort.written_page_keys {
        store.delete(key).unwrap();
    }
    assert_eq!(store.len(), 0);
}

#[test]
fn compatibility_abort_cleanup_has_a_fixed_page_key_limit() {
    let store = Arc::new(MemoryStore::default());
    let catalog =
        PagedCatalog::with_page_bytes(store.clone(), "bounded/catalog", 4 * 1024).unwrap();
    let input = (0..MAX_ABORT_TRACKED_CATALOG_PAGES + 10).map(|index| {
        Ok(CatalogEntry {
            key: format!("segment/{index:09}"),
            value: vec![b'x'; 700],
        })
    });
    let abort = catalog.build_sorted_with_abort(input).unwrap_err();
    assert!(
        abort.error.to_string().contains("bounded limit"),
        "unexpected abort error: {}",
        abort.error
    );
    assert_eq!(
        abort.written_page_keys.len(),
        MAX_ABORT_TRACKED_CATALOG_PAGES
    );
    for key in &abort.written_page_keys {
        store.delete(key).unwrap();
    }
    assert_eq!(store.len(), 0);
}

#[test]
fn keys_that_cannot_guarantee_three_way_fanout_are_rejected_before_upload() {
    let store = Arc::new(MemoryStore::default());
    let catalog = PagedCatalog::with_page_bytes(store.clone(), "fanout/catalog", 4 * 1024).unwrap();
    let error = catalog
        .build_sorted(std::iter::once(Ok(CatalogEntry {
            key: "x".repeat(1_000),
            value: vec![1],
        })))
        .unwrap_err();
    assert!(matches!(error, SegmentError::CatalogPageTooLarge { .. }));
    assert_eq!(store.len(), 0);
}

#[test]
fn million_entry_catalog_keeps_a_small_root_logarithmic_append_and_bounded_reader() {
    let store = Arc::new(MemoryStore::default());
    let catalog = PagedCatalog::new(store.clone(), "archive/catalog").unwrap();

    let small = catalog.build(entries(1_000)).unwrap();
    let small_root_bytes = serde_json::to_vec(&small.root).unwrap().len();
    store.reset_gets();
    let _ = catalog
        .upsert(
            &small.root,
            CatalogEntry {
                key: "segment/999999998".into(),
                value: vec![1],
            },
        )
        .unwrap();
    let small_append_reads = store.gets();

    let ledger = tempfile::tempfile().unwrap();
    let mut ledger = std::io::BufWriter::new(ledger);
    let mut observer_peak_bytes = 0_usize;
    let large = catalog
        .build_sorted_observed(entries(1_000_000).map(Ok), |reference| {
            observer_peak_bytes = observer_peak_bytes.max(reference.key.len());
            writeln!(ledger, "{}", reference.key).map_err(|error| SegmentError::Serialization {
                message: error.to_string(),
            })
        })
        .unwrap();
    ledger.flush().unwrap();
    let large_root_bytes = serde_json::to_vec(&large.root).unwrap().len();
    store.reset_gets();
    let appended = catalog
        .upsert(
            &large.root,
            CatalogEntry {
                key: "segment/999999999".into(),
                value: vec![2],
            },
        )
        .unwrap();
    let large_append_reads = store.gets();

    assert!(small_root_bytes < 64 * 1024);
    assert!(large_root_bytes < 64 * 1024);
    assert!(
        large.peak_buffer_bytes <= 64 * 1024 * (large.root.height as usize + 2),
        "streaming bulk build buffered {} bytes at height {}",
        large.peak_buffer_bytes,
        large.root.height
    );
    assert!(
        observer_peak_bytes < 1_024,
        "the abort observer must hold only its current page key"
    );
    assert!(large_append_reads <= small_append_reads + 4);
    assert_eq!(appended.root.entry_count, 1_000_001);
    assert!(!appended.obsolete_page_keys.is_empty());

    store.reset_gets();
    let removed = catalog.remove(&appended.root, "segment/000500000").unwrap();
    let large_remove_reads = store.gets();
    assert!(large_remove_reads <= large_append_reads + 4);
    assert_eq!(removed.root.entry_count, 1_000_000);
    assert!(catalog
        .lookup(&removed.root, "segment/000500000")
        .unwrap()
        .is_none());
    assert!(!removed.obsolete_page_keys.is_empty());

    let missing = catalog
        .remove(&removed.root, "segment/not-present")
        .unwrap();
    assert_eq!(missing.root, removed.root);
    assert!(missing.written_page_keys.is_empty());
    assert!(missing.obsolete_page_keys.is_empty());

    store.reset_gets();
    let tail = catalog
        .reader_after(&removed.root, "segment/000999990")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(tail.len(), 10);
    assert!(
        store.gets() <= removed.root.height as u64 + 3,
        "range reader fetched {} pages at height {}",
        store.gets(),
        removed.root.height
    );

    let mut reader = catalog.reader(&removed.root).unwrap();
    let mut count = 0_u64;
    while reader.next().transpose().unwrap().is_some() {
        count += 1;
    }
    assert_eq!(count, 1_000_000);
    assert!(reader.peak_buffer_bytes() <= 64 * 1024 * (removed.root.height as usize + 1));
}

#[test]
fn removing_the_only_entry_produces_a_valid_empty_catalog() {
    let store = Arc::new(MemoryStore::default());
    let catalog = PagedCatalog::new(store, "archive/catalog").unwrap();
    let root = catalog.build(entries(1)).unwrap().root;
    let removed = catalog.remove(&root, "segment/000000000").unwrap();
    assert_eq!(removed.root.entry_count, 0);
    assert!(catalog.reader(&removed.root).unwrap().next().is_none());
}

#[test]
fn last_prefix_lookup_reads_only_one_tree_path() {
    let store = Arc::new(MemoryStore::default());
    let catalog = PagedCatalog::new(store.clone(), "archive/catalog").unwrap();
    let entries = (0..10_000).flat_map(|index| {
        ["log", "metric", "span"]
            .into_iter()
            .map(move |signal| CatalogEntry {
                key: format!("segment/{signal}/{index:09}"),
                value: (index as u64).to_le_bytes().to_vec(),
            })
    });
    let mut entries = entries.collect::<Vec<_>>();
    entries.sort_by(|left, right| left.key.cmp(&right.key));
    let root = catalog
        .build_sorted(entries.into_iter().map(Ok))
        .unwrap()
        .root;

    for signal in ["log", "metric", "span"] {
        store.reset_gets();
        let entry = catalog
            .last_with_prefix(&root, &format!("segment/{signal}/"))
            .unwrap()
            .unwrap();
        assert_eq!(entry.key, format!("segment/{signal}/000009999"));
        assert!(store.gets() <= root.height as u64 + 1);
    }
    assert!(catalog
        .last_with_prefix(&root, "segment/profile/")
        .unwrap()
        .is_none());

    let unicode_root = catalog
        .build([
            CatalogEntry {
                key: "unicode/prefix/a".into(),
                value: vec![1],
            },
            CatalogEntry {
                key: format!("unicode/prefix/{}tail", '\u{10ffff}'),
                value: vec![2],
            },
            CatalogEntry {
                key: "unicode/q".into(),
                value: vec![3],
            },
        ])
        .unwrap()
        .root;
    assert_eq!(
        catalog
            .last_with_prefix(&unicode_root, "unicode/prefix/")
            .unwrap()
            .unwrap()
            .key,
        format!("unicode/prefix/{}tail", '\u{10ffff}')
    );
}

#[test]
fn a_small_page_rejects_keys_that_cannot_form_a_two_child_branch() {
    let store = Arc::new(MemoryStore::default());
    let catalog =
        PagedCatalog::with_page_bytes(store.clone(), "archive/catalog", 4 * 1024).unwrap();
    let key = "k".repeat(950);
    let error = catalog
        .build([
            CatalogEntry {
                key: format!("a{key}"),
                value: vec![1],
            },
            CatalogEntry {
                key: format!("b{key}"),
                value: vec![2],
            },
        ])
        .unwrap_err();
    assert!(error.to_string().contains("page"));
    assert_eq!(
        store.len(),
        0,
        "fanout must be rejected before immutable leaf pages become orphans"
    );
}
