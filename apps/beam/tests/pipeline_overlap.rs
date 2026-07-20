// HANDWRITE-BEGIN gap="missing-generator:logic:b2820553" tracker="#2153" reason="scaffold for apps/beam/tests/pipeline_overlap.rs — fill in by hand and update tracker when codegen is ready"
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

use beam::domain::collection::Collection;
use beam::domain::ports::{DistanceCalculator, VectorRepository};
use beam::domain::scheduler::{PipelineScheduler, QueryBatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventKind {
    StartFetch(usize),
    EndFetch(usize),
    StartCompute(usize),
    EndCompute(usize),
}

#[derive(Debug, Clone, Copy)]
struct Event {
    kind: EventKind,
    time: std::time::Instant,
}

type EventLog = Arc<Mutex<Vec<Event>>>;

struct OverlapMockRepo {
    log: EventLog,
    delay: Duration,
    call_count: Arc<AtomicUsize>,
}

impl VectorRepository for OverlapMockRepo {
    async fn fetch_async(&self, offsets: &[u64], _vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        {
            self.log.lock().unwrap().push(Event {
                kind: EventKind::StartFetch(idx),
                time: std::time::Instant::now(),
            });
        }
        
        tokio::time::sleep(self.delay).await;
        
        {
            self.log.lock().unwrap().push(Event {
                kind: EventKind::EndFetch(idx),
                time: std::time::Instant::now(),
            });
        }
        
        Ok(vec![0; offsets.len() * 4])
    }
}

struct OverlapMockCalc {
    log: EventLog,
    delay: Duration,
    call_count: Arc<AtomicUsize>,
}

impl DistanceCalculator for OverlapMockCalc {
    async fn compute_batched(
        &self,
        _queries: &[f32],
        _targets: &[f32],
        _dim: usize,
        _metric: beam::collection::Metric,
    ) -> anyhow::Result<Vec<f32>> {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        {
            self.log.lock().unwrap().push(Event {
                kind: EventKind::StartCompute(idx),
                time: std::time::Instant::now(),
            });
        }
        
        tokio::time::sleep(self.delay).await;
        
        {
            self.log.lock().unwrap().push(Event {
                kind: EventKind::EndCompute(idx),
                time: std::time::Instant::now(),
            });
        }
        
        Ok(vec![1.0])
    }
}

#[tokio::test]
async fn test_overlap_execution() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let repo_count = Arc::new(AtomicUsize::new(0));
    let calc_count = Arc::new(AtomicUsize::new(0));
    
    let repo = OverlapMockRepo {
        log: log.clone(),
        delay: Duration::from_millis(50),
        call_count: repo_count,
    };
    
    let calc = OverlapMockCalc {
        log: log.clone(),
        delay: Duration::from_millis(50),
        call_count: calc_count,
    };
    
    let scheduler = PipelineScheduler::new(repo, calc);
    
    let mut collection = Collection::new("overlap_coll".into(), 1, beam::collection::Metric::Dot);
    collection.payload.offsets.insert("vector_0".into(), 0);
    
    let batch = QueryBatch {
        id: "overlap_batch".into(),
        queries: vec![
            vec![1.0],
            vec![2.0],
        ],
        k: 1,
    };
    
    let results = scheduler.execute_batch(&collection, &batch).await.unwrap();
    assert_eq!(results.len(), 2);
    
    let events = log.lock().unwrap().clone();
    
    // Find event times
    let start_fetch_1 = events.iter().find(|e| e.kind == EventKind::StartFetch(1)).map(|e| e.time);
    let end_compute_0 = events.iter().find(|e| e.kind == EventKind::EndCompute(0)).map(|e| e.time);
    let start_compute_0 = events.iter().find(|e| e.kind == EventKind::StartCompute(0)).map(|e| e.time);
    let end_fetch_1 = events.iter().find(|e| e.kind == EventKind::EndFetch(1)).map(|e| e.time);
    
    assert!(start_fetch_1.is_some(), "StartFetch(1) not found");
    assert!(end_compute_0.is_some(), "EndCompute(0) not found");
    assert!(start_compute_0.is_some(), "StartCompute(0) not found");
    assert!(end_fetch_1.is_some(), "EndFetch(1) not found");
    
    let start_fetch_1 = start_fetch_1.unwrap();
    let end_compute_0 = end_compute_0.unwrap();
    let start_compute_0 = start_compute_0.unwrap();
    let end_fetch_1 = end_fetch_1.unwrap();
    
    // Assert overlap: Fetch(1) starts before Compute(0) completes
    assert!(
        start_fetch_1 < end_compute_0,
        "Pipelined overlap failed: Fetch(1) started at {:?}, but Compute(0) ended at {:?}",
        start_fetch_1,
        end_compute_0
    );
    
    // Assert overlap: Fetch(1) completes after Compute(0) starts
    assert!(
        end_fetch_1 > start_compute_0,
        "Pipelined overlap failed: Fetch(1) ended at {:?}, but Compute(0) started at {:?}",
        end_fetch_1,
        start_compute_0
    );
}
// HANDWRITE-END
