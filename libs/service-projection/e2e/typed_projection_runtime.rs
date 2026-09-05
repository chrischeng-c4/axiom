use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use anyhow::Result;
use service_projection::{
    Projection, ProjectionDescriptor, ProjectionReadSession, ProjectionRecord, ProjectionRegistry,
    ProjectionRuntimeConfig, ProjectionSource,
};

#[derive(Clone)]
struct Record {
    cursor: u64,
    id: String,
    value: u64,
}

impl ProjectionRecord for Record {
    fn projection_cursor(&self) -> u64 {
        self.cursor
    }

    fn projection_event_id(&self) -> &str {
        &self.id
    }
}

#[derive(Default)]
struct Source {
    records: Mutex<Vec<Record>>,
    generation: AtomicU64,
}

impl ProjectionSource<Record> for Source {
    fn current_cursor(&self) -> u64 {
        self.records
            .lock()
            .unwrap()
            .last()
            .map_or(0, |record| record.cursor)
    }

    fn read_after(&self, after: u64, limit: usize) -> Result<Vec<Record>> {
        Ok(self
            .records
            .lock()
            .unwrap()
            .iter()
            .filter(|record| record.cursor > after)
            .take(limit)
            .cloned()
            .collect())
    }

    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }
}

struct SessionSource {
    records: Vec<Record>,
    opened: Arc<AtomicU64>,
}

struct SessionReader {
    records: Vec<Record>,
    offset: usize,
}

impl ProjectionReadSession<Record> for SessionReader {
    fn read_next(&mut self, limit: usize) -> Result<Vec<Record>> {
        let end = self.offset.saturating_add(limit).min(self.records.len());
        let page = self.records[self.offset..end].to_vec();
        self.offset = end;
        Ok(page)
    }
}

impl ProjectionSource<Record> for SessionSource {
    fn current_cursor(&self) -> u64 {
        self.records.last().map_or(0, |record| record.cursor)
    }

    fn read_after(&self, _after: u64, _limit: usize) -> Result<Vec<Record>> {
        panic!("stateful projection source must not fall back to stateless paging")
    }

    fn open_read_session(
        &self,
        after: u64,
    ) -> Result<Option<Box<dyn ProjectionReadSession<Record>>>> {
        self.opened.fetch_add(1, Ordering::AcqRel);
        Ok(Some(Box::new(SessionReader {
            records: self
                .records
                .iter()
                .filter(|record| record.cursor > after)
                .cloned()
                .collect(),
            offset: 0,
        })))
    }
}

#[test]
fn one_stateful_read_session_serves_all_projection_pages() {
    let root = tempfile::tempdir().unwrap();
    let opened = Arc::new(AtomicU64::new(0));
    let source = Arc::new(SessionSource {
        records: (1..=25)
            .map(|cursor| Record {
                cursor,
                id: format!("session-{cursor}"),
                value: 1,
            })
            .collect(),
        opened: opened.clone(),
    });
    let mut registry =
        ProjectionRegistry::new(root.path(), source, ProjectionRuntimeConfig::new(3, 100, 1))
            .unwrap();
    let handle = registry
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();

    assert_eq!(handle.catch_up().unwrap(), 25);
    assert_eq!(handle.projection().value(), 25);
    assert_eq!(opened.load(Ordering::Acquire), 1);
}

#[test]
fn source_generation_change_replaces_stale_projection_and_survives_restart() {
    let root = tempfile::tempdir().unwrap();
    let source = Arc::new(Source::default());
    source.records.lock().unwrap().extend([
        Record {
            cursor: 1,
            id: "expired".into(),
            value: 100,
        },
        Record {
            cursor: 2,
            id: "retained".into(),
            value: 7,
        },
    ]);

    let mut registry = ProjectionRegistry::new(
        root.path(),
        source.clone(),
        ProjectionRuntimeConfig::new(10, 1, 1),
    )
    .unwrap();
    let handle = registry
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    handle.catch_up().unwrap();
    handle.flush().unwrap();
    assert_eq!(handle.projection().value(), 107);

    source.records.lock().unwrap().remove(0);
    source.generation.store(1, Ordering::Release);
    assert_eq!(handle.catch_up().unwrap(), 2);
    assert_eq!(handle.projection().value(), 7);
    handle.flush().unwrap();
    drop(handle);
    drop(registry);

    let mut reopened =
        ProjectionRegistry::new(root.path(), source, ProjectionRuntimeConfig::new(10, 1, 1))
            .unwrap();
    let restored = reopened
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    assert_eq!(restored.current_cursor(), 2);
    assert_eq!(restored.projection().value(), 7);
}

#[test]
fn corrupt_rebuildable_snapshot_is_quarantined_and_rebuilt_from_source() {
    let root = tempfile::tempdir().unwrap();
    let source = Arc::new(Source::default());
    source.records.lock().unwrap().push(Record {
        cursor: 1,
        id: "retained".into(),
        value: 7,
    });
    let mut registry = ProjectionRegistry::new(
        root.path(),
        source.clone(),
        ProjectionRuntimeConfig::new(10, 1, 1),
    )
    .unwrap();
    let handle = registry
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    handle.catch_up().unwrap();
    handle.flush().unwrap();
    drop(handle);
    drop(registry);

    let state_dir = root.path().join("indexes/sum");
    std::fs::write(state_dir.join("state.json"), b"{\"truncated\":").unwrap();

    let mut reopened =
        ProjectionRegistry::new(root.path(), source, ProjectionRuntimeConfig::new(10, 1, 1))
            .unwrap();
    let rebuilt = reopened
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    assert_eq!(rebuilt.current_cursor(), 1);
    assert_eq!(rebuilt.projection().value(), 7);
    assert!(state_dir.join("state.json").is_file());
    assert_eq!(
        std::fs::read_dir(&state_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("state.corrupt-"))
            .count(),
        1
    );
}

#[derive(Default)]
struct SumProjection(Mutex<u64>);

impl SumProjection {
    fn value(&self) -> u64 {
        *self.0.lock().unwrap()
    }
}

impl Projection<Record> for SumProjection {
    fn descriptor(&self) -> ProjectionDescriptor {
        ProjectionDescriptor {
            name: "sum".to_string(),
            schema_version: 1,
            retention: "source-owned".to_string(),
        }
    }

    fn apply_idempotent(&self, record: &Record) -> Result<()> {
        *self.0.lock().unwrap() += record.value;
        Ok(())
    }

    fn snapshot(&self) -> Result<Vec<u8>> {
        Ok(self.value().to_le_bytes().to_vec())
    }

    fn restore(&self, state: &[u8]) -> Result<()> {
        *self.0.lock().unwrap() = u64::from_le_bytes(state.try_into()?);
        Ok(())
    }
}

#[tokio::test]
async fn typed_handle_restores_catches_up_flushes_and_rebuilds_without_any() {
    let root = tempfile::tempdir().unwrap();
    let source = Arc::new(Source::default());
    source.records.lock().unwrap().extend([
        Record {
            cursor: 1,
            id: "a".into(),
            value: 2,
        },
        Record {
            cursor: 2,
            id: "b".into(),
            value: 3,
        },
    ]);

    let mut registry = ProjectionRegistry::new(
        root.path(),
        source.clone(),
        ProjectionRuntimeConfig::new(10, 1, 1),
    )
    .unwrap();
    let handle = registry
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    assert_eq!(handle.catch_up().unwrap(), 2);
    assert_eq!(handle.projection().value(), 5);
    handle.flush().unwrap();
    assert!(handle.rebuild_and_compare().unwrap().equal);
    drop(handle);
    drop(registry);

    let mut reopened =
        ProjectionRegistry::new(root.path(), source, ProjectionRuntimeConfig::new(10, 1, 1))
            .unwrap();
    let restored = reopened
        .register(|| Ok(Arc::new(SumProjection::default())))
        .unwrap();
    assert_eq!(restored.current_cursor(), 2);
    assert_eq!(restored.projection().value(), 5);
    restored
        .wait_for_min_cursor(2, std::time::Duration::from_millis(1))
        .await
        .unwrap();
}
