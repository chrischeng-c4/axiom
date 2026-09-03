//! Bounding RaftHost shutdown by caller-supplied ShutdownDeadline and reporting
//! terminal phase outcomes (#3672).

use std::collections::{HashMap, HashSet};
use std::io::ErrorKind;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use raft_runtime::{
    FsyncPolicy, HostConfig, HostShutdownReport, LeadershipHandoff, Membership, PhaseStatus,
    ProposalOutcome, RaftHost, RaftStateMachine, RaftStatus, RaftStore, ShutdownCaller,
    ShutdownPhase,
};
use server_lifecycle::ShutdownDeadline;

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::{await_leader, bind, cluster, Node, TestSm};

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

async fn status(client: &reqwest::Client, url: &str) -> RaftStatus {
    client
        .get(format!("{url}/raftz"))
        .send()
        .await
        .expect("a host serves its own status")
        .json()
        .await
        .expect("the status is the published shape")
}

async fn settle_cluster(nodes: &[Node], leader: usize) {
    let client = h2c_client();
    for i in 0..5u8 {
        nodes[leader]
            .host
            .propose(vec![i])
            .await
            .expect("the leader accepts a proposal");
    }

    let leader_last = status(&client, &nodes[leader].url).await.last_index;
    assert!(leader_last >= 5);

    let caught_up = Instant::now() + Duration::from_secs(10);
    for (i, node) in nodes.iter().enumerate() {
        if i == leader {
            continue;
        }
        loop {
            let s = status(&client, &node.url).await;
            if s.last_index == leader_last && s.commit_index == leader_last {
                break;
            }
            assert!(
                Instant::now() < caught_up,
                "follower {i} never caught up with leader"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
}

/// A test-only state machine that blocks exactly one apply while the host owns
/// its Raft-node mutex. This makes `LeadershipHandoff` wait at a real public
/// host boundary without adding a production shutdown hook.
struct BlockingApplySm {
    applied: std::sync::atomic::AtomicU64,
    gate: Mutex<BlockingApplyGate>,
    released: Condvar,
}

#[derive(Default)]
struct BlockingApplyGate {
    armed: bool,
    entered: bool,
    release: bool,
}

impl BlockingApplySm {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            applied: std::sync::atomic::AtomicU64::new(0),
            gate: Mutex::new(BlockingApplyGate::default()),
            released: Condvar::new(),
        })
    }

    fn arm(&self) {
        let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
        gate.armed = true;
        gate.entered = false;
        gate.release = false;
    }

    async fn wait_until_entered(&self) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if self
                    .gate
                    .lock()
                    .expect("blocking gate mutex poisoned")
                    .entered
                {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("the armed state-machine apply must hold the node mutex");
    }

    fn release(&self) {
        let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
        gate.release = true;
        self.released.notify_all();
    }
}

impl RaftStateMachine for BlockingApplySm {
    fn apply(&self, index: u64, _command: &[u8]) -> anyhow::Result<()> {
        let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
        if gate.armed {
            gate.armed = false;
            gate.entered = true;
            while !gate.release {
                gate = self
                    .released
                    .wait(gate)
                    .expect("blocking gate mutex poisoned");
            }
        }
        drop(gate);
        self.applied
            .store(index, std::sync::atomic::Ordering::Release);
        Ok(())
    }

    fn snapshot(&self, _writer: &mut dyn std::io::Write) -> anyhow::Result<()> {
        Ok(())
    }

    fn restore(&self, _reader: &mut dyn std::io::Read) -> anyhow::Result<()> {
        Ok(())
    }

    fn applied_index(&self) -> u64 {
        self.applied.load(std::sync::atomic::Ordering::Acquire)
    }
}

struct BlockingNode {
    host: Arc<RaftHost>,
    sm: Arc<BlockingApplySm>,
    _serve: tokio::task::JoinHandle<()>,
    _dir: TempDir,
}

async fn blocking_cluster(node_count: u64) -> Vec<BlockingNode> {
    let mut listeners = Vec::new();
    let mut all = Vec::new();
    for id in 0..node_count {
        let (listener, url) = bind().await;
        listeners.push(listener);
        all.push((id, url));
    }

    let config = HostConfig {
        tick: Duration::from_millis(10),
        ..HostConfig::default()
    };
    let voters: Vec<u64> = (0..node_count).collect();
    let mut nodes = Vec::new();
    for (index, listener) in listeners.into_iter().enumerate() {
        let id = index as u64;
        let sm = BlockingApplySm::new();
        let dir = TempDir::new().expect("temporary raft store directory");
        let store = RaftStore::open(dir.path().to_str().unwrap(), id, FsyncPolicy::Os)
            .expect("temporary raft store opens");
        let host = Arc::new(RaftHost::spawn(
            id,
            Membership {
                voters: voters.clone(),
                learners: vec![],
            },
            cluster::peers_excluding(id, &all),
            store,
            sm.clone() as Arc<dyn RaftStateMachine>,
            config,
        ));
        let router = host.router();
        let serve = tokio::spawn(async move {
            loop {
                if let Ok((stream, _)) = listener.accept().await {
                    let router = router.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, router).await;
                    });
                }
            }
        });
        nodes.push(BlockingNode {
            host,
            sm,
            _serve: serve,
            _dir: dir,
        });
    }
    nodes
}

async fn await_blocking_leader(nodes: &[BlockingNode]) -> usize {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for (index, node) in nodes.iter().enumerate() {
            if node.host.is_leader().await {
                return index;
            }
        }
        assert!(
            Instant::now() < deadline,
            "the blocking test cluster must elect a leader"
        );
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

async fn hold_leader_apply(
    nodes: &[BlockingNode],
    leader: usize,
    command: Vec<u8>,
) -> tokio::task::JoinHandle<anyhow::Result<u64>> {
    nodes[leader].sm.arm();
    let host = Arc::clone(&nodes[leader].host);
    let proposal = tokio::spawn(async move { host.propose(command).await });
    nodes[leader].sm.wait_until_entered().await;
    proposal
}

/// A three-voter cluster's leader shut down under a generous deadline records
/// all four phases `Completed` in the fixed order with `peer_listener_close_safe`
/// true and `incomplete_phase` none.
#[tokio::test]
async fn three_voter_leader_generous_deadline_completes_all_phases_in_order() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let report = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        report.phases.len(),
        4,
        "all four phases must be recorded in order"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "first phase must be Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::Completed,
        "Quiesce must complete"
    );
    assert_eq!(
        report.phases[1].phase,
        ShutdownPhase::LeadershipHandoff,
        "second phase must be LeadershipHandoff"
    );
    assert_eq!(
        report.phases[1].status,
        PhaseStatus::Completed,
        "LeadershipHandoff must complete"
    );
    assert_eq!(
        report.phases[2].phase,
        ShutdownPhase::BackgroundTasks,
        "third phase must be BackgroundTasks"
    );
    assert_eq!(
        report.phases[2].status,
        PhaseStatus::Completed,
        "BackgroundTasks must complete"
    );
    assert_eq!(
        report.phases[3].phase,
        ShutdownPhase::PeerRpcDrain,
        "fourth phase must be PeerRpcDrain"
    );
    assert_eq!(
        report.phases[3].status,
        PhaseStatus::Completed,
        "PeerRpcDrain must complete"
    );

    assert_eq!(
        report.incomplete_phase, None,
        "incomplete_phase must be None on clean shutdown"
    );
    assert!(
        report.peer_listener_close_safe,
        "peer_listener_close_safe must be true when PeerRpcDrain completes"
    );
    assert!(
        report.storage_failure.is_none(),
        "storage_failure must be None on healthy host"
    );
    assert!(
        matches!(report.handoff, LeadershipHandoff::Transferred { .. }),
        "leadership handoff must transfer to a peer"
    );
}

/// A deadline whose total equals its reserve — so usable_remaining is zero —
/// stops at the first phase, names it as incomplete, and leaves peer_listener_close_safe false.
#[tokio::test]
async fn zero_usable_budget_stops_at_first_phase_and_names_it_incomplete() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(10), Duration::from_secs(10))
        .expect("valid deadline where total equals reserve");
    assert_eq!(
        deadline.usable_remaining(),
        Duration::ZERO,
        "usable_remaining must be zero"
    );

    let report = nodes[leader].host.shutdown_within(deadline).await;

    assert_eq!(
        report.phases.len(),
        1,
        "exactly one phase record must be present on zero usable budget"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "phase must be Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::DeadlineExpired,
        "first phase status must be DeadlineExpired"
    );
    assert_eq!(
        report.incomplete_phase,
        Some(ShutdownPhase::Quiesce),
        "incomplete_phase must name ShutdownPhase::Quiesce"
    );
    assert!(
        !report.peer_listener_close_safe,
        "peer_listener_close_safe must be false"
    );
}

/// One shared usable deadline assigns `LeadershipHandoff` its cumulative 50%
/// cutoff. A prior Quiesce that uses no time cannot donate the final 50% that
/// belongs to BackgroundTasks and PeerRpcDrain. Expiry names the blocked
/// phase, and later phases remain absent from the public report.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn leadership_handoff_stops_at_its_cumulative_half_deadline_without_running_later_phases() {
    const USABLE_BUDGET: Duration = Duration::from_secs(2);
    const BEFORE_LATER_PHASES: Duration = Duration::from_millis(1_500);

    let nodes = blocking_cluster(3).await;
    let leader = await_blocking_leader(&nodes).await;
    let proposal = hold_leader_apply(&nodes, leader, b"block-handoff-phase".to_vec()).await;

    let deadline = ShutdownDeadline::from_now(USABLE_BUDGET, Duration::ZERO)
        .expect("a non-reserved usable deadline is valid");
    let started = Instant::now();
    let report = nodes[leader].host.shutdown_within(deadline).await;
    let elapsed = started.elapsed();

    nodes[leader].sm.release();
    proposal.abort();
    let _ = proposal.await;

    assert_eq!(
        report.phases.len(),
        2,
        "only Quiesce and the expired LeadershipHandoff phase may be recorded"
    );
    assert_eq!(report.phases[0].phase, ShutdownPhase::Quiesce);
    assert_eq!(report.phases[0].status, PhaseStatus::Completed);
    assert_eq!(report.phases[1].phase, ShutdownPhase::LeadershipHandoff);
    assert_eq!(report.phases[1].status, PhaseStatus::DeadlineExpired);
    assert_eq!(
        report.incomplete_phase,
        Some(ShutdownPhase::LeadershipHandoff),
        "the report must name the exact expired phase"
    );
    assert!(
        !report.peer_listener_close_safe,
        "no later PeerRpcDrain phase may make listener close safe"
    );
    assert!(
        elapsed < BEFORE_LATER_PHASES,
        "LeadershipHandoff must stop at its cumulative 50% cutoff before later \
         reserved time; elapsed {elapsed:?} exceeded {BEFORE_LATER_PHASES:?}"
    );
}

/// Unused Quiesce time rolls into the following cumulative window. A real
/// leader-side apply holds LeadershipHandoff for more than the first 25% of
/// usable time, releases before the 50% cutoff, and permits all later phases.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unused_quiesce_time_rolls_forward_to_complete_a_slow_leadership_handoff() {
    const USABLE_BUDGET: Duration = Duration::from_secs(2);
    const FIRST_QUARTER: Duration = Duration::from_millis(500);
    const HOLD_AFTER_SHUTDOWN_START: Duration = Duration::from_millis(750);

    let nodes = blocking_cluster(3).await;
    let leader = await_blocking_leader(&nodes).await;
    let proposal = hold_leader_apply(&nodes, leader, b"roll-forward-handoff-phase".to_vec()).await;

    let host = Arc::clone(&nodes[leader].host);
    let shutdown = tokio::spawn(async move {
        host.shutdown_within(
            ShutdownDeadline::from_now(USABLE_BUDGET, Duration::ZERO)
                .expect("a non-reserved usable deadline is valid"),
        )
        .await
    });

    // Shutdown can run Quiesce immediately, then waits on the node mutex in
    // LeadershipHandoff. Keep it blocked past the first 25% but before 50%.
    tokio::time::sleep(HOLD_AFTER_SHUTDOWN_START).await;
    nodes[leader].sm.release();

    let report = tokio::time::timeout(Duration::from_secs(2), shutdown)
        .await
        .expect("the released handoff completes before the shared deadline")
        .expect("the shutdown task does not panic");
    proposal.abort();
    let _ = proposal.await;

    assert_eq!(
        report.phases.len(),
        4,
        "a handoff released before its 50% cutoff must allow all four phases"
    );
    assert_eq!(report.phases[0].status, PhaseStatus::Completed);
    assert_eq!(report.phases[1].phase, ShutdownPhase::LeadershipHandoff);
    assert_eq!(report.phases[1].status, PhaseStatus::Completed);
    assert!(
        report.phases[1].elapsed > FIRST_QUARTER,
        "LeadershipHandoff must complete after consuming more than Quiesce's \
         first-quarter slice, proving unused early time rolls forward; elapsed {:?}",
        report.phases[1].elapsed
    );
    assert_eq!(report.phases[2].phase, ShutdownPhase::BackgroundTasks);
    assert_eq!(report.phases[2].status, PhaseStatus::Completed);
    assert_eq!(report.phases[3].phase, ShutdownPhase::PeerRpcDrain);
    assert_eq!(report.phases[3].status, PhaseStatus::Completed);
    assert_eq!(report.incomplete_phase, None);
    assert!(report.peer_listener_close_safe);
}

/// A host whose store has an injected save failure reports storage_failure carrying the
/// injected ErrorKind and PhaseStatus::StorageFailed, and legacy shutdown() on a separate
/// latched host returns Err naming the failure.
#[tokio::test]
async fn latched_storage_failure_reported_in_shutdown_within_and_legacy_shutdown_returns_err() {
    // Host 1: shutdown_within on latched host
    let nodes1 = cluster(1).await;
    nodes1[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    // Latch the failure via a proposal
    let outcome1 = nodes1[0]
        .host
        .propose_outcome(b"cmd-save-failure-1".to_vec())
        .await;
    assert!(
        matches!(outcome1, ProposalOutcome::DurabilityFailure { .. }),
        "proposal must fail and latch storage failure"
    );

    let deadline = ShutdownDeadline::from_now(Duration::from_secs(30), Duration::from_secs(1))
        .expect("valid deadline");
    let report = nodes1[0].host.shutdown_within(deadline).await;

    assert!(
        report.storage_failure.is_some(),
        "report must carry storage failure"
    );
    let sf = report.storage_failure.unwrap();
    assert_eq!(
        sf.kind,
        ErrorKind::StorageFull,
        "storage failure kind must match injected ErrorKind"
    );
    assert_eq!(
        report.phases[0].phase,
        ShutdownPhase::Quiesce,
        "first phase is Quiesce"
    );
    assert_eq!(
        report.phases[0].status,
        PhaseStatus::StorageFailed,
        "phase where failure was observed must be StorageFailed"
    );

    // Host 2: legacy shutdown on a separate latched host
    let nodes2 = cluster(1).await;
    nodes2[0]
        .host
        .store()
        .inject_next_save_failure_with_kind(ErrorKind::StorageFull);

    let outcome2 = nodes2[0]
        .host
        .propose_outcome(b"cmd-save-failure-2".to_vec())
        .await;
    assert!(
        matches!(outcome2, ProposalOutcome::DurabilityFailure { .. }),
        "proposal must fail and latch storage failure on second host"
    );

    let res = nodes2[0].host.shutdown().await;
    assert!(
        res.is_err(),
        "legacy shutdown() must return Err on host with latched storage failure"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("durable storage failed"),
        "error message must name durability failure, got: {err_msg}"
    );
}

/// A healthy three-voter leader shut down through legacy shutdown() returns Ok(())
/// with the group's durable state intact afterwards.
#[tokio::test]
async fn healthy_three_voter_leader_legacy_shutdown_returns_ok_with_state_intact() {
    let nodes = cluster(3).await;
    let leader = await_leader(&nodes)
        .await
        .expect("a three-voter cluster elects a leader");
    settle_cluster(&nodes, leader).await;

    let res = nodes[leader].host.shutdown().await;
    assert!(
        res.is_ok(),
        "legacy shutdown on healthy leader must return Ok(())"
    );

    // Verify durable state intact
    assert!(
        nodes[leader].host.store().path().exists(),
        "store path must remain intact after shutdown"
    );
}

/// Converting a HostShutdownReport with any incomplete phase into a Result returns
/// an Err identifying that specific phase, and all four variants produce distinct messages.
#[test]
fn report_into_result_returns_distinct_err_naming_phase_for_all_incomplete_phases() {
    let variants = [
        (ShutdownPhase::Quiesce, "Quiesce"),
        (ShutdownPhase::LeadershipHandoff, "LeadershipHandoff"),
        (ShutdownPhase::BackgroundTasks, "BackgroundTasks"),
        (ShutdownPhase::PeerRpcDrain, "PeerRpcDrain"),
    ];

    let mut messages = HashSet::new();

    for (phase, expected_name) in variants {
        let report = HostShutdownReport {
            caller: ShutdownCaller::Executed,
            phases: vec![],
            handoff: LeadershipHandoff::NotLeader,
            incomplete_phase: Some(phase),
            peer_listener_close_safe: false,
            storage_failure: None,
        };
        let res = report.into_result();
        assert!(
            res.is_err(),
            "into_result must return Err for incomplete phase {phase:?}"
        );
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains(expected_name),
            "error message for {phase:?} must contain '{expected_name}', got: '{err_msg}'"
        );
        messages.insert(err_msg);
    }

    assert_eq!(
        messages.len(),
        4,
        "all four incomplete shutdown phases must render distinct error messages"
    );
}

/// A host configured with a tiny rpc_timeout reaches the Quiesce-expiry branch on
/// legacy shutdown() and returns an Err naming Quiesce.
#[tokio::test]
async fn tiny_rpc_timeout_legacy_shutdown_returns_err_naming_quiesce() {
    let (listener, url) = bind().await;
    let sm = TestSm::new();
    let dir = TempDir::new().unwrap();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig {
            rpc_timeout: Duration::from_nanos(1),
            ..Default::default()
        },
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
    let _node = Node {
        host: host.clone(),
        sm,
        url,
        _serve: serve,
        _dir: dir,
    };

    let res = host.shutdown().await;
    assert!(
        res.is_err(),
        "legacy shutdown() with tiny rpc_timeout must return Err"
    );
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Quiesce"),
        "error message must name Quiesce, got: '{err_msg}'"
    );
}
