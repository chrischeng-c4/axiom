use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use service_executor::{spawn_group_commit, GroupCommitConfig, GroupCommitRequest};

struct Numbers {
    key: &'static str,
    values: Vec<u64>,
    bytes: usize,
}

impl GroupCommitRequest for Numbers {
    type Item = u64;
    type Key = &'static str;

    fn key(&self) -> Self::Key {
        self.key
    }

    fn item_count(&self) -> usize {
        self.values.len()
    }

    fn encoded_bytes(&self) -> usize {
        self.bytes
    }

    fn into_items(self) -> Vec<Self::Item> {
        self.values
    }
}

#[tokio::test]
async fn one_window_executes_once_and_fans_results_back_to_each_request() {
    let calls = Arc::new(AtomicUsize::new(0));
    let batches = Arc::new(Mutex::new(Vec::new()));
    let (queue, worker) = spawn_group_commit(
        GroupCommitConfig::new(Duration::from_millis(20), 10, 1_024).unwrap(),
        {
            let calls = calls.clone();
            let batches = batches.clone();
            move |items: Vec<u64>| {
                let calls = calls.clone();
                let batches = batches.clone();
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    batches.lock().unwrap().push(items.clone());
                    Ok::<_, &'static str>(items.into_iter().map(|item| item * 10).collect())
                }
            }
        },
    );

    let (first, second) = tokio::join!(
        queue.submit(Numbers {
            key: "logs",
            values: vec![1, 2],
            bytes: 20,
        }),
        queue.submit(Numbers {
            key: "logs",
            values: vec![3],
            bytes: 10,
        })
    );
    assert_eq!(first.unwrap(), vec![10, 20]);
    assert_eq!(second.unwrap(), vec![30]);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(*batches.lock().unwrap(), vec![vec![1, 2, 3]]);

    drop(queue);
    worker.join().await.unwrap();
}

#[tokio::test]
async fn key_item_and_byte_boundaries_create_separate_batches() {
    let batches = Arc::new(Mutex::new(Vec::new()));
    let (queue, worker) = spawn_group_commit(
        GroupCommitConfig::new(Duration::from_millis(5), 2, 20).unwrap(),
        {
            let batches = batches.clone();
            move |items: Vec<u64>| {
                let batches = batches.clone();
                async move {
                    batches.lock().unwrap().push(items.clone());
                    Ok::<_, &'static str>(items)
                }
            }
        },
    );

    let (first, second, third) = tokio::join!(
        queue.submit(Numbers {
            key: "logs",
            values: vec![1, 2],
            bytes: 20,
        }),
        queue.submit(Numbers {
            key: "logs",
            values: vec![3],
            bytes: 10,
        }),
        queue.submit(Numbers {
            key: "traces",
            values: vec![4],
            bytes: 10,
        })
    );
    assert_eq!(first.unwrap(), vec![1, 2]);
    assert_eq!(second.unwrap(), vec![3]);
    assert_eq!(third.unwrap(), vec![4]);

    drop(queue);
    worker.join().await.unwrap();
    assert_eq!(*batches.lock().unwrap(), vec![vec![1, 2], vec![3], vec![4]]);
}

#[tokio::test]
async fn one_sink_failure_is_fanned_out_without_losing_the_original_error() {
    let (queue, worker) = spawn_group_commit(
        GroupCommitConfig::new(Duration::from_millis(5), 10, 100).unwrap(),
        |_items: Vec<u64>| async { Err::<Vec<u64>, _>("raft proposal failed") },
    );

    let (first, second) = tokio::join!(
        queue.submit(Numbers {
            key: "metrics",
            values: vec![1],
            bytes: 10,
        }),
        queue.submit(Numbers {
            key: "metrics",
            values: vec![2],
            bytes: 10,
        })
    );
    assert!(first.unwrap_err().to_string().contains("raft proposal failed"));
    assert!(second.unwrap_err().to_string().contains("raft proposal failed"));

    drop(queue);
    worker.join().await.unwrap();
}
