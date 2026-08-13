use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;

use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftStateMachine, RaftStore,
};

pub struct TestSm {
    pub applied: AtomicU64,
}

impl TestSm {
    pub fn new() -> Arc<Self> {
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

pub async fn bind() -> (TcpListener, String) {
    let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = l.local_addr().unwrap().port();
    (l, format!("http://127.0.0.1:{port}"))
}

pub fn peers_excluding(me: u64, all: &[(u64, String)]) -> HashMap<u64, String> {
    all.iter()
        .filter(|(id, _)| *id != me)
        .map(|(id, url)| (*id, url.clone()))
        .collect()
}

pub struct Node {
    pub host: Arc<RaftHost>,
    pub sm: Arc<TestSm>,
    pub _serve: tokio::task::JoinHandle<()>,
    pub _dir: TempDir,
}

pub async fn cluster(n: u64) -> Vec<Node> {
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

pub async fn await_leader(nodes: &[Node]) -> Option<usize> {
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
