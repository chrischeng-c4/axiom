//! Sift adapters for the shared typed projection runtime.

use std::{path::Path, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::{DurableJournal, EventQuery, StoredEvent};

use super::{
    logging::{LogPage, LogQuery, LoggingProjection, PROJECTION_LOGGING_STORE},
    metric::{MetricPage, MetricProjection, MetricQuery, PROJECTION_METRIC_STORE},
    model::{ProjectionLag, RebuildComparison},
    trace::{TracePage, TraceProjection, TraceQuery, TraceResultV1, PROJECTION_TRACE_STORE},
};

pub const PROJECTION_BATCH_SIZE: usize = 1_000;
pub const PROJECTION_RETRY_AFTER_SECONDS: u64 = 1;
pub const PROJECTION_SNAPSHOT_INTERVAL_EVENTS: u64 = 100_000;

/// Sift's domain projection hook. The shared runtime owns its control flow.
pub trait Projection: Send + Sync + 'static {
    fn descriptor(&self) -> service_projection::ProjectionDescriptor;
    fn apply_idempotent(&self, event: &StoredEvent) -> Result<()>;
    fn snapshot(&self) -> Result<Vec<u8>>;
    fn restore(&self, state: &[u8]) -> Result<()>;

    fn semantic_digest(&self) -> Result<String> {
        Ok(hex::encode(Sha256::digest(self.snapshot()?)))
    }
}

macro_rules! impl_shared_projection {
    ($projection:ty) => {
        impl service_projection::Projection<StoredEvent> for $projection {
            fn descriptor(&self) -> service_projection::ProjectionDescriptor {
                Projection::descriptor(self)
            }

            fn apply_idempotent(&self, event: &StoredEvent) -> Result<()> {
                Projection::apply_idempotent(self, event)
            }

            fn snapshot(&self) -> Result<Vec<u8>> {
                Projection::snapshot(self)
            }

            fn restore(&self, state: &[u8]) -> Result<()> {
                Projection::restore(self, state)
            }

            fn semantic_digest(&self) -> Result<String> {
                Projection::semantic_digest(self)
            }
        }
    };
}

impl_shared_projection!(LoggingProjection);
impl_shared_projection!(MetricProjection);
impl_shared_projection!(TraceProjection);

impl service_projection::ProjectionRecord for StoredEvent {
    fn projection_cursor(&self) -> u64 {
        self.cursor
    }

    fn projection_event_id(&self) -> &str {
        &self.event.event_id
    }
}

struct JournalProjectionSource {
    journal: Arc<DurableJournal>,
}

impl service_projection::ProjectionSource<StoredEvent> for JournalProjectionSource {
    fn current_cursor(&self) -> u64 {
        self.journal.last_cursor()
    }

    fn read_after(&self, after: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        self.journal.query(EventQuery {
            signal: None,
            after,
            limit,
        })
    }

    fn generation(&self) -> u64 {
        self.journal.projection_generation()
    }
}

pub struct ProjectionRuntime {
    registry: service_projection::ProjectionRegistry<StoredEvent>,
    logging: Arc<service_projection::ProjectionHandle<StoredEvent, LoggingProjection>>,
    metrics: Arc<service_projection::ProjectionHandle<StoredEvent, MetricProjection>>,
    traces: Arc<service_projection::ProjectionHandle<StoredEvent, TraceProjection>>,
}

impl ProjectionRuntime {
    pub fn open(data_dir: impl AsRef<Path>, journal: Arc<DurableJournal>) -> Result<Self> {
        let source: Arc<dyn service_projection::ProjectionSource<StoredEvent>> =
            Arc::new(JournalProjectionSource { journal });
        let config = service_projection::ProjectionRuntimeConfig::new(
            PROJECTION_BATCH_SIZE,
            PROJECTION_SNAPSHOT_INTERVAL_EVENTS,
            PROJECTION_RETRY_AFTER_SECONDS,
        );
        let mut registry = service_projection::ProjectionRegistry::new(data_dir, source, config)?;
        let logging = registry.register(|| Ok(Arc::new(LoggingProjection::new()?)))?;
        let metrics = registry.register(|| Ok(Arc::new(MetricProjection::new())))?;
        let traces = registry.register(|| Ok(Arc::new(TraceProjection::new())))?;
        Ok(Self {
            registry,
            logging,
            metrics,
            traces,
        })
    }

    pub fn projection_names(&self) -> Vec<String> {
        self.registry.projection_names()
    }

    pub fn has_projection(&self, name: &str) -> bool {
        self.registry.has_projection(name)
    }

    pub fn current_cursor(&self, name: &str) -> Result<u64> {
        self.registry.current_cursor(name)
    }

    pub fn semantic_digest(&self, name: &str) -> Result<String> {
        self.registry.semantic_digest(name)
    }

    pub fn query_logs(&self, query: &LogQuery) -> Result<LogPage> {
        self.logging.catch_up()?;
        self.logging.projection().query(query)
    }

    pub fn get_trace(&self, project: &str, trace_id: &str) -> Result<Option<TraceResultV1>> {
        self.traces.catch_up()?;
        self.traces.projection().get_trace(project, trace_id)
    }

    pub fn query_traces(&self, query: &TraceQuery) -> Result<TracePage> {
        self.traces.catch_up()?;
        self.traces.projection().query(query)
    }

    pub fn query_metrics(&self, query: &MetricQuery) -> Result<MetricPage> {
        self.metrics.catch_up()?;
        self.metrics.projection().query(query)
    }

    pub fn catch_up(&self, name: &str) -> Result<u64> {
        self.registry.catch_up(name)
    }

    pub async fn wait_for_min_cursor(
        &self,
        name: &str,
        required_cursor: u64,
        timeout: Duration,
    ) -> std::result::Result<u64, ProjectionLag> {
        match name {
            PROJECTION_LOGGING_STORE => {
                self.logging
                    .wait_for_min_cursor(required_cursor, timeout)
                    .await
            }
            PROJECTION_METRIC_STORE => {
                self.metrics
                    .wait_for_min_cursor(required_cursor, timeout)
                    .await
            }
            PROJECTION_TRACE_STORE => {
                self.traces
                    .wait_for_min_cursor(required_cursor, timeout)
                    .await
            }
            _ => Err(ProjectionLag::new(
                name,
                required_cursor,
                0,
                PROJECTION_RETRY_AFTER_SECONDS,
            )),
        }
    }

    pub fn rebuild_and_compare(&self, name: &str) -> Result<RebuildComparison> {
        self.registry.rebuild_and_compare(name)
    }

    pub fn persist_all(&self) -> Result<()> {
        self.registry
            .flush_all()
            .context("flush typed Sift projections")
    }
}
