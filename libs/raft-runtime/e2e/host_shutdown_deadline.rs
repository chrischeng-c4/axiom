//! Bounding RaftHost shutdown by caller-supplied ShutdownDeadline and reporting
//! terminal phase outcomes (#3672).
//!
//! # Facets
//!
//! - Behavior: `libs/raft-runtime/e2e/host_shutdown_deadline.rs:389-411`
//!   and `libs/raft-runtime/e2e/host_shutdown_deadline.rs:510-558` assert the
//!   public durable commit, unadvanced apply head, and retry order. The declared
//!   gate is `cargo test -p raft-runtime` at `libs/raft-runtime/README.md:29-31`.
//! - Security: the apply scheduling seam at `libs/raft-runtime/src/host.rs:536-608`
//!   consumes only committed `RaftNode` entries. It adds no request parser,
//!   identity decision, file path, or secret boundary. Peer identity remains
//!   covered by the existing `cargo test -p raft-runtime --test peer_mtls` gate
//!   declared at `libs/raft-runtime/README.md:35-53`.
//! - Performance: gap. `apps/lumen/src/raft_sm.rs:310` calls `propose`, and
//!   `apps/lumen/src/bin/lumen.rs:3882` mounts the router. The shared-host
//!   promise at `libs/raft-runtime/README.md:22-33` gives no current numerical
//!   budget for an apply-blocked proposal or status request. The bounded test
//!   waits below are test cleanup limits, not a product performance promise.

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

async fn status_with_committed_index(
    client: &reqwest::Client,
    url: &str,
    expected: u64,
    limit: Duration,
) -> Result<RaftStatus, tokio::time::error::Elapsed> {
    tokio::time::timeout(limit, async {
        loop {
            let observed = status(client, url).await;
            if observed.last_index >= expected && observed.commit_index >= expected {
                return observed;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
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

/// A test-only state machine that blocks exactly one callback after its
/// durable commit. It lets shutdown tests account for an active apply worker
/// in `BackgroundTasks` without adding a production shutdown hook.
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
    fail_once_after_release: bool,
    failure_observed: bool,
    callback_indices: Vec<u64>,
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
        self.arm_inner(false);
    }

    fn arm_to_fail_once(&self) {
        self.arm_inner(true);
    }

    fn arm_inner(&self, fail_once_after_release: bool) {
        let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
        gate.armed = true;
        gate.entered = false;
        gate.release = false;
        gate.fail_once_after_release = fail_once_after_release;
        gate.failure_observed = false;
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
        .expect("the armed state-machine apply must reach the callback");
    }

    fn release(&self) {
        let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
        gate.release = true;
        self.released.notify_all();
    }

    fn callback_indices(&self) -> Vec<u64> {
        self.gate
            .lock()
            .expect("blocking gate mutex poisoned")
            .callback_indices
            .clone()
    }

    async fn failure_observed_within(&self, limit: Duration) -> bool {
        tokio::time::timeout(limit, async {
            loop {
                if self
                    .gate
                    .lock()
                    .expect("blocking gate mutex poisoned")
                    .failure_observed
                {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .is_ok()
    }
}

impl RaftStateMachine for BlockingApplySm {
    fn apply(&self, index: u64, _command: &[u8]) -> anyhow::Result<()> {
        let fail = {
            let mut gate = self.gate.lock().expect("blocking gate mutex poisoned");
            gate.callback_indices.push(index);
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
            if gate.fail_once_after_release {
                gate.fail_once_after_release = false;
                gate.failure_observed = true;
                true
            } else {
                false
            }
        };
        if fail {
            anyhow::bail!("injected state-machine apply failure at index {index}");
        }
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

/// Releases the test callback during unwinding so no timeout can strand it.
struct ReleaseApplyGate(Arc<BlockingApplySm>);

impl Drop for ReleaseApplyGate {
    fn drop(&mut self) {
        self.0.release();
    }
}

struct BlockingNode {
    host: Arc<RaftHost>,
    sm: Arc<BlockingApplySm>,
    url: String,
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
            url: all[index].1.clone(),
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

/// A state-machine callback that has reached a durable commit does not keep
/// /raftz or a follower-to-leader proposal route behind the node mutex.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn blocked_apply_keeps_status_and_follower_route_live() {
    const OBSERVE_LIMIT: Duration = Duration::from_secs(2);
    const JOIN_LIMIT: Duration = Duration::from_secs(3);

    let nodes = blocking_cluster(3).await;
    let leader = await_blocking_leader(&nodes).await;
    let follower = (leader + 1) % nodes.len();
    let client = h2c_client();
    let sm = nodes[leader].sm.clone();
    let release = ReleaseApplyGate(sm.clone());

    let first = hold_leader_apply(&nodes, leader, b"first".to_vec()).await;
    let first_committed =
        tokio::time::timeout(OBSERVE_LIMIT, status(&client, &nodes[leader].url)).await;

    // A follower call proves the public route can enter its elected leader.
    let follower_host = nodes[follower].host.clone();
    let second = tokio::spawn(async move { follower_host.propose(b"second".to_vec()).await });
    let second_committed =
        status_with_committed_index(&client, &nodes[leader].url, 2, OBSERVE_LIMIT).await;
    let applied_while_blocked = sm.applied_index();
    let callbacks_while_blocked = sm.callback_indices();

    // All possible errors above are observations only. Release before joining
    // and before asserting so this test never leaves an SM callback blocked.
    drop(release);
    let first_join = tokio::time::timeout(JOIN_LIMIT, first).await;
    let second_join = tokio::time::timeout(JOIN_LIMIT, second).await;

    let first_status = first_committed.expect("/raftz must return while apply is blocked");
    let second_status =
        second_committed.expect("a follower proposal must reach the leader while apply is blocked");
    let first_index = first_join
        .expect("released first proposal must join")
        .expect("first proposal task must not panic")
        .expect("released first proposal must complete");
    let second_index = second_join
        .expect("released follower proposal must join")
        .expect("follower proposal task must not panic")
        .expect("released follower proposal must complete");

    assert!(
        first_status.last_index >= 1 && first_status.commit_index >= 1,
        "the first callback starts only after its durable commit"
    );
    assert_eq!(
        first_status.applied_index, 0,
        "a blocked callback must not publish an applied index"
    );
    assert!(
        second_status.last_index >= 2 && second_status.commit_index >= 2,
        "the follower-routed proposal must allocate and commit its own index"
    );
    assert_eq!(
        applied_while_blocked, 0,
        "the first blocked callback is not falsely applied"
    );
    assert_eq!(
        callbacks_while_blocked,
        vec![1],
        "a single worker must not begin the second callback before the first releases"
    );
    assert_eq!(first_index, 1);
    assert_eq!(second_index, 2);
}

/// A callback Err is an explicit failed apply, not a normal domain outcome.
/// It retains index 1 as the source head, does not run index 2, and a restart
/// retries index 1 before it reaches index 2. The test deliberately does not
/// prescribe a ProposalOutcome for the failed callers.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn failed_apply_retains_the_head_and_replays_it_before_later_entries() {
    const OBSERVE_LIMIT: Duration = Duration::from_secs(2);
    const JOIN_LIMIT: Duration = Duration::from_secs(3);

    let nodes = blocking_cluster(1).await;
    let leader = await_blocking_leader(&nodes).await;
    let client = h2c_client();
    let sm = nodes[leader].sm.clone();
    let restart_dir = nodes[leader]._dir.path().to_path_buf();
    sm.arm_to_fail_once();
    let release = ReleaseApplyGate(sm.clone());

    let first_host = nodes[leader].host.clone();
    let first = tokio::spawn(async move { first_host.propose(b"first".to_vec()).await });
    sm.wait_until_entered().await;
    let first_committed =
        tokio::time::timeout(OBSERVE_LIMIT, status(&client, &nodes[leader].url)).await;

    let second_host = nodes[leader].host.clone();
    let second = tokio::spawn(async move { second_host.propose(b"second".to_vec()).await });
    let two_committed =
        status_with_committed_index(&client, &nodes[leader].url, 2, OBSERVE_LIMIT).await;
    let applied_while_blocked = sm.applied_index();
    let callbacks_while_blocked = sm.callback_indices();

    drop(release);
    let failure_observed = sm.failure_observed_within(OBSERVE_LIMIT).await;
    let status_after_error =
        tokio::time::timeout(OBSERVE_LIMIT, status(&client, &nodes[leader].url)).await;
    let callbacks_before_restart = sm.callback_indices();
    let applied_before_restart = sm.applied_index();

    // Failed proposal callers can wait for an implementation-defined error
    // result. Cancel both only after the failed callback is observed, then
    // bound the joins before any assertion.
    first.abort();
    second.abort();
    let first_join = tokio::time::timeout(JOIN_LIMIT, first).await;
    let second_join = tokio::time::timeout(JOIN_LIMIT, second).await;
    let stopped = tokio::time::timeout(
        JOIN_LIMIT,
        nodes[leader].host.shutdown_within(
            ShutdownDeadline::from_now(JOIN_LIMIT, Duration::ZERO)
                .expect("test shutdown deadline is valid"),
        ),
    )
    .await;
    let stopped_cleanly = matches!(
        &stopped,
        Ok(report) if report.incomplete_phase.is_none() && report.peer_listener_close_safe
    );

    // Once the old background tasks stop, reopen the same durable directory.
    // The fail-once fixture now returns Ok, so cold replay must call 1 then 2.
    let mut restart_applied = false;
    let mut callbacks_after_restart = Vec::new();
    if stopped_cleanly {
        let restarted = RaftHost::spawn(
            0,
            Membership {
                voters: vec![0],
                learners: vec![],
            },
            HashMap::new(),
            RaftStore::open(
                restart_dir.to_str().expect("temporary directory is UTF-8"),
                0,
                FsyncPolicy::Os,
            )
            .expect("restart reopens the durable store"),
            sm.clone() as Arc<dyn RaftStateMachine>,
            HostConfig {
                tick: Duration::from_millis(10),
                ..HostConfig::default()
            },
        );
        restart_applied = tokio::time::timeout(JOIN_LIMIT, async {
            while sm.applied_index() < 2 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .is_ok();
        callbacks_after_restart = sm.callback_indices();
        drop(restarted);
    }

    let first_status = first_committed.expect("/raftz must expose the first durable commit");
    let two_status = two_committed.expect("both commands must commit while first apply is gated");
    let after_error = status_after_error.expect("/raftz must stay live after an apply Err");

    assert!(
        first_status.last_index >= 1 && first_status.commit_index >= 1,
        "the first callback starts after durable commit"
    );
    assert!(
        two_status.last_index >= 2 && two_status.commit_index >= 2,
        "the second command commits before the first callback returns Err"
    );
    assert_eq!(applied_while_blocked, 0);
    assert_eq!(callbacks_while_blocked, vec![1]);
    assert!(
        failure_observed,
        "the first callback must return the injected Err"
    );
    assert!(
        after_error.last_index >= 2 && after_error.commit_index >= 2,
        "the durable source still contains both committed commands"
    );
    assert!(
        after_error.applied_index < 1 && applied_before_restart < 1,
        "an Err must not falsely publish the failed head as applied"
    );
    assert_eq!(
        callbacks_before_restart,
        vec![1],
        "the second callback must not run after the first callback Err"
    );
    assert!(
        first_join.is_ok(),
        "first caller must join after cancellation"
    );
    assert!(
        second_join.is_ok(),
        "second caller must join after cancellation"
    );
    assert!(
        stopped_cleanly,
        "the old host must complete shutdown before durable replay"
    );
    assert!(
        restart_applied,
        "restart must replay the retained committed head"
    );
    assert_eq!(
        callbacks_after_restart,
        vec![1, 1, 2],
        "restart must retry the failed head before it applies the later command"
    );
    assert_eq!(sm.applied_index(), 2);
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

/// One shared usable deadline assigns an active apply worker to
/// `BackgroundTasks` at its cumulative 75% cutoff. Quiesce and leadership
/// handoff complete first. Expiry names the blocked phase, and PeerRpcDrain is
/// absent from the public report.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn background_tasks_stops_at_its_cumulative_three_quarter_deadline_without_peer_drain() {
    const USABLE_BUDGET: Duration = Duration::from_secs(2);
    const BEFORE_PEER_DRAIN: Duration = Duration::from_millis(1_750);
    const CLEANUP_LIMIT: Duration = Duration::from_secs(3);

    let nodes = blocking_cluster(3).await;
    let leader = await_blocking_leader(&nodes).await;
    let proposal = hold_leader_apply(&nodes, leader, b"block-background-tasks".to_vec()).await;

    let deadline = ShutdownDeadline::from_now(USABLE_BUDGET, Duration::ZERO)
        .expect("a non-reserved usable deadline is valid");
    let started = Instant::now();
    let shutdown =
        tokio::time::timeout(CLEANUP_LIMIT, nodes[leader].host.shutdown_within(deadline)).await;
    let elapsed = started.elapsed();

    nodes[leader].sm.release();
    proposal.abort();
    let proposal_join = tokio::time::timeout(CLEANUP_LIMIT, proposal).await;
    let report = shutdown.expect("shutdown must stop at its shared deadline");

    assert_eq!(
        report.phases.len(),
        3,
        "only Quiesce, LeadershipHandoff, and expired BackgroundTasks may be recorded"
    );
    assert_eq!(report.phases[0].phase, ShutdownPhase::Quiesce);
    assert_eq!(report.phases[0].status, PhaseStatus::Completed);
    assert_eq!(report.phases[1].phase, ShutdownPhase::LeadershipHandoff);
    assert_eq!(report.phases[1].status, PhaseStatus::Completed);
    assert_eq!(report.phases[2].phase, ShutdownPhase::BackgroundTasks);
    assert_eq!(report.phases[2].status, PhaseStatus::DeadlineExpired);
    assert_eq!(
        report.incomplete_phase,
        Some(ShutdownPhase::BackgroundTasks),
        "the report must name the exact expired phase"
    );
    assert!(
        !report.peer_listener_close_safe,
        "absent PeerRpcDrain may not make listener close safe"
    );
    assert!(
        proposal_join.is_ok(),
        "the released blocked proposal must join during cleanup"
    );
    assert!(
        elapsed < BEFORE_PEER_DRAIN,
        "BackgroundTasks must stop at its cumulative 75% cutoff before PeerRpcDrain; \
         elapsed {elapsed:?} exceeded {BEFORE_PEER_DRAIN:?}"
    );
}

/// An active apply worker may consume time past the old LeadershipHandoff
/// cutoff. Releasing it before the BackgroundTasks cutoff permits every later
/// shutdown phase to complete.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_apply_worker_released_before_background_cutoff_completes_shutdown() {
    const USABLE_BUDGET: Duration = Duration::from_secs(2);
    const LEADERSHIP_HANDOFF_CUTOFF: Duration = Duration::from_secs(1);
    const HOLD_AFTER_SHUTDOWN_START: Duration = Duration::from_millis(1_250);
    const CLEANUP_LIMIT: Duration = Duration::from_secs(3);

    let nodes = blocking_cluster(3).await;
    let leader = await_blocking_leader(&nodes).await;
    let proposal = hold_leader_apply(
        &nodes,
        leader,
        b"release-background-tasks-before-cutoff".to_vec(),
    )
    .await;

    let host = Arc::clone(&nodes[leader].host);
    let shutdown = tokio::spawn(async move {
        host.shutdown_within(
            ShutdownDeadline::from_now(USABLE_BUDGET, Duration::ZERO)
                .expect("a non-reserved usable deadline is valid"),
        )
        .await
    });

    // Quiesce and handoff may complete while the apply worker remains active.
    // Release it past the handoff cutoff but before BackgroundTasks expires.
    tokio::time::sleep(HOLD_AFTER_SHUTDOWN_START).await;
    nodes[leader].sm.release();

    let shutdown_result = tokio::time::timeout(CLEANUP_LIMIT, shutdown).await;
    proposal.abort();
    let proposal_join = tokio::time::timeout(CLEANUP_LIMIT, proposal).await;
    let report = shutdown_result
        .expect("the released worker completes before the shared deadline")
        .expect("the shutdown task does not panic");

    assert_eq!(
        report.phases.len(),
        4,
        "a worker released before the 75% cutoff must allow all four phases"
    );
    assert_eq!(report.phases[0].status, PhaseStatus::Completed);
    assert_eq!(report.phases[1].phase, ShutdownPhase::LeadershipHandoff);
    assert_eq!(report.phases[1].status, PhaseStatus::Completed);
    assert_eq!(report.phases[2].phase, ShutdownPhase::BackgroundTasks);
    assert_eq!(report.phases[2].status, PhaseStatus::Completed);
    let elapsed_through_background =
        report.phases[0].elapsed + report.phases[1].elapsed + report.phases[2].elapsed;
    assert!(
        elapsed_through_background > LEADERSHIP_HANDOFF_CUTOFF,
        "the shutdown must reach BackgroundTasks after the LeadershipHandoff cutoff; elapsed {elapsed_through_background:?}"
    );
    assert_eq!(report.phases[3].phase, ShutdownPhase::PeerRpcDrain);
    assert_eq!(report.phases[3].status, PhaseStatus::Completed);
    assert_eq!(report.incomplete_phase, None);
    assert!(report.peer_listener_close_safe);
    assert!(
        proposal_join.is_ok(),
        "the released blocked proposal must join during cleanup"
    );
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
