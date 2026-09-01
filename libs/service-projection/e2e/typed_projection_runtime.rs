use std::sync::{Arc, Mutex};

use anyhow::Result;
use service_projection::{
    Projection, ProjectionDescriptor, ProjectionRecord, ProjectionRegistry,
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
