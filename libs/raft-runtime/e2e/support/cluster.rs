use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;

use raft_core::ELECTION_TIMEOUT_FLOOR_TICKS;
use raft_runtime::{
    FsyncPolicy, HostConfig, Index, Membership, RaftHost, RaftStateMachine, RaftStore,
};

const MIN_LEADER_POLL: Duration = Duration::from_millis(1);
const MAX_LEADER_POLL: Duration = Duration::from_millis(25);

/// The time limits for one leader-election observation.
///
/// A node with ordinal `node_count - 1` can wait for the election floor plus
/// that ordinal. Two such windows cover an election and its observed result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeaderWaitBudget {
    pub max_election_ticks: u64,
    pub wait_budget: Duration,
    pub poll_interval: Duration,
}

/// Derive a leader wait budget from the runtime tick and the cluster size.
///
/// The multiplication is checked so a malformed caller size or tick cannot
/// wrap a short wait into a false timeout.
pub fn leader_wait_budget(tick: Duration, node_count: usize) -> LeaderWaitBudget {
    let max_election_ticks = ELECTION_TIMEOUT_FLOOR_TICKS
        .saturating_add(u64::try_from(node_count.saturating_sub(1)).unwrap_or(u64::MAX));
    let two_election_windows = max_election_ticks.saturating_mul(2);
    let wait_budget = u32::try_from(two_election_windows)
        .ok()
        .and_then(|windows| tick.checked_mul(windows))
        .unwrap_or(Duration::MAX);
    let poll_interval = tick.max(MIN_LEADER_POLL).min(MAX_LEADER_POLL);

    LeaderWaitBudget {
        max_election_ticks,
        wait_budget,
        poll_interval,
    }
}

pub struct TestSm {
    pub applied: AtomicU64,
    pub fail_restore: AtomicBool,
    pub restore_attempts: AtomicU64,
    pub snapshot_capable: AtomicBool,
    pub snapshot_capability_calls: AtomicU64,
    pub drop_capability_after_first_probe: AtomicBool,
}

impl TestSm {
    pub fn new() -> Arc<Self> {
        Arc::new(TestSm {
            applied: AtomicU64::new(0),
            fail_restore: AtomicBool::new(false),
            restore_attempts: AtomicU64::new(0),
            snapshot_capable: AtomicBool::new(true),
            snapshot_capability_calls: AtomicU64::new(0),
            drop_capability_after_first_probe: AtomicBool::new(false),
        })
    }
}

impl RaftStateMachine for TestSm {
    fn snapshot_capability(&self) -> Option<&'static str> {
        let call = self
            .snapshot_capability_calls
            .fetch_add(1, Ordering::AcqRel);
        let disappeared = self
            .drop_capability_after_first_probe
            .load(Ordering::Acquire)
            && call > 0;
        (self.snapshot_capable.load(Ordering::Acquire) && !disappeared)
            .then_some("test-snapshot-v1")
    }

    fn apply(&self, index: Index, _command: &[u8]) -> anyhow::Result<()> {
        self.applied.store(index, Ordering::Release);
        Ok(())
    }
    fn snapshot(&self, _writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        Ok(())
    }
    fn snapshot_at(&self, _index: Index, _writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        Ok(())
    }
    fn validate_snapshot(&self, _reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
        self.restore_attempts.fetch_add(1, Ordering::Relaxed);
        if self.fail_restore.load(Ordering::Acquire) {
            anyhow::bail!("injected snapshot validation failure");
        }
        Ok(())
    }
    fn restore(&self, _reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
        if self.fail_restore.load(Ordering::Acquire) {
            anyhow::bail!("injected snapshot restore failure");
        }
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
    pub url: String,
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
        let url = all[idx].1.clone();
        nodes.push(Node {
            host,
            sm,
            url,
            _serve: serve,
            _dir: dir,
        });
    }
    nodes
}

pub async fn await_leader(nodes: &[Node]) -> Option<usize> {
    await_leader_with_tick(nodes, HostConfig::default().tick).await
}

/// Wait for a leader using the actual tick configured for these hosts.
///
/// Callers that construct a custom [`HostConfig`] must pass that config's
/// `tick`; callers using [`HostConfig::default`] may use [`await_leader`].
pub async fn await_leader_with_tick(nodes: &[Node], tick: Duration) -> Option<usize> {
    let timing = leader_wait_budget(tick, nodes.len());
    let started = tokio::time::Instant::now();

    loop {
        for (i, n) in nodes.iter().enumerate() {
            if n.host.is_leader().await {
                return Some(i);
            }
        }

        let elapsed = started.elapsed();
        if elapsed >= timing.wait_budget {
            return None;
        }
        let remaining = timing.wait_budget.saturating_sub(elapsed);
        tokio::time::sleep(timing.poll_interval.min(remaining)).await;
    }
}
