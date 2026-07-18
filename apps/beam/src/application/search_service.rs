//! Application Layer Search Service.

use crate::domain::collection::Collection;
use crate::domain::ports::{DistanceCalculator, VectorRepository};
use crate::domain::scheduler::{PipelineScheduler, QueryBatch};

/// Application Service mapping inbound search requests to the Domain Scheduler.
pub struct SearchApplicationService<R, C>
where
    R: VectorRepository,
    C: DistanceCalculator,
{
    scheduler: PipelineScheduler<R, C>,
}

impl<R, C> SearchApplicationService<R, C>
where
    R: VectorRepository,
    C: DistanceCalculator,
{
    /// Create a new SearchApplicationService.
    pub fn new(repo: R, calc: C) -> Self {
        Self {
            scheduler: PipelineScheduler::new(repo, calc),
        }
    }

    /// Perform a high-throughput search over a collection.
    pub async fn search(
        &self,
        collection: &Collection,
        queries: Vec<Vec<f32>>,
        k: usize,
    ) -> anyhow::Result<Vec<Vec<(String, f32)>>> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let batch = QueryBatch {
            id: format!("batch_{}", timestamp),
            queries,
            k,
        };
        self.scheduler.execute_batch(collection, &batch).await
    }
}
