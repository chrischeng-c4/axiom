use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;

use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftStateMachine, RaftStore,
};

struct TestSm {
    applied: AtomicU64,
}

impl TestSm {
    fn new() -> Arc<Self> {
        Arc::new(TestSm {
            applied: AtomicU64::new(0),
        })
    }
}

impl RaftStateMachine for TestSm {
    fn apply(&self, index: Index, _command: &[u8]) -> anyhow::Result<()> {
        self.applied.store(index, Ordering::Release);
        Ok(())
    }
    fn snapshot(&self) -> anyhow::Result<Vec<u8>> {
        Ok(vec![])
    }
    fn restore(&self, _snapshot: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

#[tokio::test]
async fn single_voter_late_subscriber() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let host = RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    );

    for v in 1..=4u64 {
        let idx = host.propose(v.to_le_bytes().to_vec()).await.unwrap();
        assert_eq!(idx, v);
    }

    let watch_val = *host.applied_watch().borrow();
    let sm_val = sm.applied_index();
    println!("sm=[{sm_val}] fresh_watch=[{watch_val}]");

    assert_eq!(sm_val, 4);
    assert_eq!(watch_val, 4);
}

async fn bind() -> (TcpListener, String) {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    (l, format!("http://127.0.0.1:{port}"))
}

fn peers_excluding(me: u64, all: &[(u64, String)]) -> HashMap<u64, String> {
    all.iter()
        .filter(|(id, _)| *id != me)
        .map(|(id, url)| (*id, url.clone()))
        .collect()
}

struct Node {
    host: Arc<RaftHost>,
    sm: Arc<TestSm>,
    _serve: tokio::task::JoinHandle<()>,
    _dir: TempDir,
}

async fn cluster(n: u64) -> Vec<Node> {
    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..n {
        let (l, url) = bind().await;
        listeners.push(l);
        all.push((id, url));
    }
    let voters: Vec<u64> = (0..n).collect();
    let mut nodes = Vec::new();
    for (idx, listener) in listeners.into_iter().enumerate() {
        let id = idx as u64;
        let peers = peers_excluding(id, &all);
        let sm = TestSm::new();
        let dir = TempDir::new().unwrap();
        let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os).unwrap();
        let host = Arc::new(RaftHost::spawn(
            id,
            Membership {
                voters: voters.clone(),
                learners: vec![],
            },
            peers,
            store,
            sm.clone() as Arc<dyn RaftStateMachine>,
            HostConfig::default(),
        ));
        let router = host.router();
        let serve = tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let r = router.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        });
        nodes.push(Node {
            host,
            sm,
            _serve: serve,
            _dir: dir,
        });
    }
    nodes
}

async fn await_leader(nodes: &[Node]) -> Option<usize> {
    for _ in 0..400 {
        for (i, n) in nodes.iter().enumerate() {
            if n.host.is_leader().await {
                return Some(i);
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    None
}

#[tokio::test]
async fn held_subscriber() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");

    let mut receivers = Vec::new();
    for n in &nodes {
        receivers.push(n.host.applied_watch());
    }

    for v in 1..=4u64 {
        let idx = nodes[leader]
            .host
            .propose(v.to_le_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(idx, v);
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    for n in &nodes {
        while n.sm.applied_index() < 4 {
            if std::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let mut sm_vals = Vec::new();
    let mut watch_vals = Vec::new();
    for (i, n) in nodes.iter().enumerate() {
        sm_vals.push(n.sm.applied_index());
        watch_vals.push(*receivers[i].borrow());
    }
    println!("sm={sm_vals:?} fresh_watch={watch_vals:?}");

    for sm_val in &sm_vals {
        assert_eq!(*sm_val, 4);
    }
    for watch_val in &watch_vals {
        assert_eq!(*watch_val, 4);
    }
}

#[tokio::test]
async fn three_voter_late_subscriber() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes).await.expect("a leader is elected");

    for v in 1..=4u64 {
        let idx = nodes[leader]
            .host
            .propose(v.to_le_bytes().to_vec())
            .await
            .unwrap();
        assert_eq!(idx, v);
    }

    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    for n in &nodes {
        while n.sm.applied_index() < 4 {
            if std::time::Instant::now() >= deadline {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    let mut sm_vals = Vec::new();
    let mut watch_vals = Vec::new();
    for n in &nodes {
        sm_vals.push(n.sm.applied_index());
        watch_vals.push(*n.host.applied_watch().borrow());
    }
    println!("sm={sm_vals:?} fresh_watch={watch_vals:?}");

    for sm_val in &sm_vals {
        assert_eq!(*sm_val, 4);
    }
    for watch_val in &watch_vals {
        assert_eq!(*watch_val, 4);
    }
}
