// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-raft-scheduler" tracker="#766" reason="Real three-node h2c Raft integration for committed task state and fenced dispatch ownership."
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::{TimeZone, Utc};
use defer::{CreateTask, DeferRaft, DeferScheduler, QueuePolicy, Target, TaskStatus};
use raft_runtime::Membership;

struct Node {
    raft: Arc<DeferRaft>,
    serve: tokio::task::JoinHandle<()>,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

struct Cluster {
    urls: HashMap<u64, String>,
    listeners: Vec<Option<tokio::net::TcpListener>>,
    nodes: Vec<Option<Node>>,
    dirs: Vec<tempfile::TempDir>,
    membership: Membership,
}

impl Cluster {
    async fn new(size: usize) -> Self {
        let mut urls = HashMap::new();
        let mut listeners = Vec::new();
        for id in 0..size {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            urls.insert(
                id as u64,
                format!("http://{}", listener.local_addr().unwrap()),
            );
            listeners.push(Some(listener));
        }
        Self {
            urls,
            listeners,
            nodes: (0..size).map(|_| None).collect(),
            dirs: (0..size).map(|_| tempfile::tempdir().unwrap()).collect(),
            membership: Membership {
                voters: (0..size as u64).collect(),
                learners: vec![],
            },
        }
    }

    fn start_node(&mut self, id: usize) {
        let peers = self
            .urls
            .iter()
            .filter(|(peer, _)| **peer != id as u64)
            .map(|(peer, url)| (*peer, url.clone()))
            .collect();
        let raft = Arc::new(
            DeferRaft::spawn(
                Arc::new(Mutex::new(DeferScheduler::new())),
                &self.dirs[id].path().join("raft"),
                id as u64,
                self.membership.clone(),
                peers,
                DeferRaft::host_config(8),
            )
            .unwrap(),
        );
        let app = raft.router();
        let listener = match self.listeners[id].take() {
            Some(listener) => listener,
            None => {
                let address = self.urls[&(id as u64)]
                    .strip_prefix("http://")
                    .unwrap()
                    .parse::<std::net::SocketAddr>()
                    .unwrap();
                let listener = std::net::TcpListener::bind(address).unwrap();
                listener.set_nonblocking(true).unwrap();
                tokio::net::TcpListener::from_std(listener).unwrap()
            }
        };
        let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
        let serve = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
        self.nodes[id] = Some(Node {
            raft,
            serve,
            shutdown: Some(shutdown),
        });
    }

    fn start_all(&mut self) {
        for id in 0..self.nodes.len() {
            self.start_node(id);
        }
    }

    fn raft(&self, id: usize) -> &Arc<DeferRaft> {
        &self.nodes[id].as_ref().unwrap().raft
    }

    fn live(&self) -> impl Iterator<Item = (usize, &Node)> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(id, node)| node.as_ref().map(|node| (id, node)))
    }

    async fn wait_leader(&self) -> usize {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut stable = None;
        let mut samples = 0;
        loop {
            let leader = {
                let mut found = None;
                for (id, node) in self.live() {
                    if node.raft.is_leader().await {
                        found = Some(id);
                        break;
                    }
                }
                found
            };
            if leader == stable && leader.is_some() {
                samples += 1;
            } else {
                stable = leader;
                samples = usize::from(stable.is_some());
            }
            if samples >= 3 {
                return stable.unwrap();
            }
            assert!(Instant::now() < deadline, "no stable leader elected");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn wait_status(&self, task_id: &str, expected: impl Fn(&TaskStatus) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            let converged = self.live().all(|(_, node)| {
                node.raft
                    .scheduler()
                    .lock()
                    .unwrap()
                    .status("jobs", task_id)
                    .unwrap()
                    .as_ref()
                    .is_some_and(&expected)
            });
            if converged {
                return;
            }
            assert!(Instant::now() < deadline, "task status did not converge");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    async fn kill(&mut self, id: usize) {
        if let Some(mut node) = self.nodes[id].take() {
            node.shutdown.take();
            node.serve.abort();
            let _ = node.serve.await;
            node.raft.shutdown().await.unwrap();
        }
    }

    async fn shutdown(mut self) {
        for node in self.nodes.iter().flatten() {
            node.raft.shutdown().await.unwrap();
        }
        for node in &mut self.nodes {
            if let Some(mut node) = node.take() {
                if let Some(shutdown) = node.shutdown.take() {
                    let _ = shutdown.send(());
                }
                let _ = node.serve.await;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

fn task(id: &str, at: chrono::DateTime<Utc>) -> CreateTask {
    CreateTask {
        task_id: id.into(),
        target: Target {
            url: "http://target.test/jobs".into(),
            method: "POST".into(),
            headers: Default::default(),
        },
        payload: serde_json::json!({"task": id}),
        schedule_at: at,
        priority: 10,
        max_attempts: 3,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn committed_scheduler_converges_and_fences_cross_replica_effects() {
    let now = Utc.timestamp_millis_opt(10_000).unwrap();
    let mut cluster = Cluster::new(3).await;
    cluster.start_all();
    let leader = cluster.wait_leader().await;
    let follower = cluster
        .live()
        .map(|(id, _)| id)
        .find(|id| *id != leader)
        .unwrap();

    cluster
        .raft(leader)
        .configure_queue("jobs".into(), QueuePolicy::default())
        .await
        .unwrap();
    cluster
        .raft(leader)
        .create_task("jobs".into(), task("first", now))
        .await
        .unwrap();

    // A follower can forward the lease proposal, but the committed executor
    // is still that follower. Another replica cannot claim its HTTP result.
    let lease = cluster
        .raft(follower)
        .lease_due("jobs".into(), now, 1)
        .await
        .unwrap()
        .remove(0);
    assert_eq!(lease.executor_node, follower as u64);
    cluster
        .wait_status("first", |status| {
            matches!(status, TaskStatus::Leased { executor_node, epoch, .. } if *executor_node == follower as u64 && *epoch == lease.epoch)
        })
        .await;
    assert!(!cluster
        .raft(leader)
        .ack("jobs".into(), lease.attempt_id.clone(), lease.epoch, now)
        .await
        .unwrap());
    assert!(cluster
        .raft(follower)
        .ack("jobs".into(), lease.attempt_id, lease.epoch, now)
        .await
        .unwrap());
    cluster
        .wait_status("first", |status| matches!(status, TaskStatus::Succeeded))
        .await;

    // If the executor dies with a live lease, the new primary preserves that
    // lease, rejects the stale attempt, and only reassigns after committed
    // expiry. The new assignment advances the fence epoch.
    cluster
        .raft(leader)
        .create_task("jobs".into(), task("second", now))
        .await
        .unwrap();
    let abandoned = cluster
        .raft(leader)
        .lease_due("jobs".into(), now, 1)
        .await
        .unwrap()
        .remove(0);
    cluster.kill(leader).await;
    let new_leader = cluster.wait_leader().await;
    assert!(!cluster
        .raft(new_leader)
        .ack(
            "jobs".into(),
            abandoned.attempt_id.clone(),
            abandoned.epoch,
            now,
        )
        .await
        .unwrap());
    let after_expiry = abandoned.expires_at + chrono::Duration::milliseconds(1);
    assert_eq!(
        cluster
            .raft(new_leader)
            .reclaim_expired("jobs".into(), after_expiry)
            .await
            .unwrap(),
        vec!["second".to_string()]
    );
    let retry_at = after_expiry + chrono::Duration::milliseconds(1_000);
    let reassigned = cluster
        .raft(new_leader)
        .lease_due("jobs".into(), retry_at, 1)
        .await
        .unwrap()
        .remove(0);
    assert!(reassigned.epoch > abandoned.epoch);
    assert_ne!(reassigned.attempt_id, abandoned.attempt_id);
    assert!(!cluster
        .raft(new_leader)
        .ack(
            "jobs".into(),
            abandoned.attempt_id,
            abandoned.epoch,
            retry_at
        )
        .await
        .unwrap());
    assert!(cluster
        .raft(new_leader)
        .ack(
            "jobs".into(),
            reassigned.attempt_id,
            reassigned.epoch,
            retry_at
        )
        .await
        .unwrap());
    cluster
        .wait_status("second", |status| matches!(status, TaskStatus::Succeeded))
        .await;

    // Recover the original primary from its durable raft state and require it
    // to converge before a second leader loss. The surviving quorum must still
    // accept and complete the full replicated task lifecycle.
    cluster.start_node(leader);
    cluster
        .wait_status("first", |status| matches!(status, TaskStatus::Succeeded))
        .await;
    cluster
        .wait_status("second", |status| matches!(status, TaskStatus::Succeeded))
        .await;
    let second_leader = cluster.wait_leader().await;
    cluster.kill(second_leader).await;
    let third_leader = cluster.wait_leader().await;
    assert_ne!(third_leader, second_leader);
    cluster
        .raft(third_leader)
        .create_task("jobs".into(), task("third", retry_at))
        .await
        .unwrap();
    let third_lease = cluster
        .raft(third_leader)
        .lease_due("jobs".into(), retry_at, 1)
        .await
        .unwrap()
        .remove(0);
    assert!(cluster
        .raft(third_leader)
        .ack(
            "jobs".into(),
            third_lease.attempt_id,
            third_lease.epoch,
            retry_at,
        )
        .await
        .unwrap());
    cluster
        .wait_status("third", |status| matches!(status, TaskStatus::Succeeded))
        .await;
    cluster.shutdown().await;
}
// HANDWRITE-END
