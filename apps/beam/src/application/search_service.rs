// SPEC-MANAGED: apps/beam/tech-design/interfaces/rest/wire-the-high-throughput-pipeline-into-production-query-serving.md#changes
use crate::domain::collection::Collection;
use crate::domain::ports::{DistanceCalculator, VectorRepository};
use crate::domain::scheduler::{PipelineScheduler, QueryBatch};

// <HANDWRITE gap="missing-generator:logic" tracker="#2153" reason="logic section in search_service.rs is hand-written pending codegen support">
/// Application Service mapping inbound search requests to the Domain Scheduler.
pub struct SearchApplicationService<R, C>
where
    R: VectorRepository + 'static,
    C: DistanceCalculator + 'static,
{
    scheduler: PipelineScheduler<R, C>,
}
// </HANDWRITE>

impl<R, C> SearchApplicationService<R, C>
where
    R: VectorRepository + 'static,
    C: DistanceCalculator + 'static,
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
