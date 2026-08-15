//! Each hosted group reports its own membership: which voters count today,
//! which voters count once a transition in flight completes, which members are
//! learners, and which phase of a transition the group is in (#3573).
//!
//! Every row reads the multi-group `/raftz` endpoint rather than a per-host
//! accessor, because the isolation these rows buy is a property of one process
//! hosting several groups, and a per-host reader cannot observe one group's
//! answer arriving under another group's key.
//!
//! A group is placed in its configuration through its durable record rather
//! than by driving a promotion: `RaftHost` exposes no membership-mutation API,
//! so a joint configuration reaches a hosted group the same way it reaches one
//! after a restart — `RaftStore::save` writes it and `RaftNode::from_persisted`
//! adopts it. The joint group's local node is a minority of both its sets, so
//! it never becomes leader, never reaches `check_leave_joint`, and the joint
//! configuration it reports is stable for the length of the row rather than a
//! window a sleep has to hit.

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tempfile::TempDir;

use raft_core::ConfState;
use raft_runtime::{
    group::GroupId, FsyncPolicy, HostConfig, Index, Membership, MembershipPhase, RaftHost,
    RaftRegistry, RaftStateMachine, RaftStatus, RaftStore,
};

#[allow(dead_code)]
#[path = "support/cluster.rs"]
mod cluster;
use cluster::bind;

/// A state machine that keeps only its applied index: these rows observe
/// membership, and no row proposes a command.
struct NullSm {
    applied: AtomicU64,
}

impl NullSm {
    fn new() -> Arc<Self> {
        Arc::new(NullSm {
            applied: AtomicU64::new(0),
        })
    }
}

impl RaftStateMachine for NullSm {
    fn apply(&self, index: Index, _command: &[u8]) -> anyhow::Result<()> {
        self.applied.store(index, Ordering::Release);
        Ok(())
    }

    fn snapshot(&self) -> anyhow::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn restore(&self, _snapshot: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }

    fn applied_index(&self) -> Index {
        self.applied.load(Ordering::Acquire)
    }
}

fn open(dir: &TempDir, group: &str) -> RaftStore {
    RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId(group.to_string()),
        FsyncPolicy::Always,
    )
    .unwrap()
}

/// Write a configuration into a group's durable record before its host opens
/// it. `RaftNode::from_persisted` prefers the record's `conf` over the
/// membership passed at spawn, so this is how a group comes up already holding
/// a configuration it agreed to in an earlier life.
fn seed_conf(store: &RaftStore, conf: ConfState) {
    let mut state = store.load().unwrap().unwrap_or_default();
    state.conf = Some(conf);
    store.save(&state).unwrap();
}

/// A joint configuration whose two sets differ: `[0, 1]` counted before the
/// transition and `[0, 1, 2]` counts once it completes. Node 0 alone is a
/// minority of both, so the host holding this never wins an election and the
/// joint phase does not dissolve underneath the assertions.
fn joint_conf() -> ConfState {
    ConfState {
        membership: Membership {
            voters: vec![0, 1, 2],
            learners: vec![],
        },
        outgoing: Some(vec![0, 1]),
        generation: 7,
    }
}

fn spawn(group: &str, membership: Membership, store: RaftStore) -> Arc<RaftHost> {
    Arc::new(RaftHost::spawn_group(
        0,
        GroupId(group.to_string()),
        membership,
        HashMap::new(),
        store,
        NullSm::new() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ))
}

fn voters(v: &[u64]) -> Vec<u64> {
    v.to_vec()
}

/// Serve one registry's router on an ephemeral port and return a client plus
/// the base URL. The caller aborts the returned task.
async fn serve(registry: &RaftRegistry) -> (tokio::task::JoinHandle<()>, reqwest::Client, String) {
    let (l, url) = bind().await;
    let router = registry.router();
    let task = tokio::spawn(async move {
        loop {
            if let Ok((stream, _)) = l.accept().await {
                let r = router.clone();
                tokio::spawn(async move {
                    let _ = transport_h2c::server::serve_connection(stream, r).await;
                });
            }
        }
    });
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap();
    (task, client, url)
}

async fn statuses(client: &reqwest::Client, url: &str) -> BTreeMap<String, RaftStatus> {
    client
        .get(format!("{url}/raftz"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn wait_leader(host: &RaftHost) {
    for _ in 0..200 {
        if host.is_leader().await {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("leader was not elected within timeout");
}

/// The isolation half of AC4. A group at rest keeps reporting its own single
/// voter after a group holding a joint configuration joins the same process,
/// and the two groups' answers are not each other's.
#[tokio::test]
async fn a_resting_group_keeps_its_own_committed_set_when_a_joint_group_joins_the_same_process() {
    let dir = TempDir::new().unwrap();

    let beta = spawn(
        "beta",
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        open(&dir, "beta"),
    );
    let registry = RaftRegistry::new();
    registry.register(beta.clone()).unwrap();
    wait_leader(&beta).await;

    let (task, client, url) = serve(&registry).await;

    let before = statuses(&client, &url).await;
    let b0 = before.get("beta").expect("beta reports before alpha exists");
    assert_eq!(before.len(), 1, "only beta is hosted yet");
    assert_eq!(b0.committed_voters, voters(&[0]));
    assert_eq!(b0.incoming_voters, None);
    assert_eq!(b0.learners, Vec::<u64>::new());
    assert_eq!(b0.membership_phase, MembershipPhase::Stable);

    let alpha_store = open(&dir, "alpha");
    seed_conf(&alpha_store, joint_conf());
    let alpha = spawn(
        "alpha",
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        alpha_store,
    );
    registry.register(alpha.clone()).unwrap();

    let after = statuses(&client, &url).await;
    assert_eq!(after.len(), 2, "both groups report");
    let a1 = after.get("alpha").expect("alpha reports");
    let b1 = after.get("beta").expect("beta still reports");

    // Beta is untouched by alpha's transition, field by field.
    assert_eq!(b1.committed_voters, b0.committed_voters);
    assert_eq!(b1.incoming_voters, b0.incoming_voters);
    assert_eq!(b1.learners, b0.learners);
    assert_eq!(b1.membership_phase, b0.membership_phase);
    assert_eq!(b1.membership_phase, MembershipPhase::Stable);

    // And the row cannot pass by both groups resting on the same answer.
    assert_eq!(a1.membership_phase, MembershipPhase::Joint);
    assert_ne!(
        a1.committed_voters, b1.committed_voters,
        "alpha reported beta's committed set: {:?}",
        a1.committed_voters
    );

    task.abort();
}

/// A group whose transition is in flight reports both sets, and they differ at
/// the one moment a consumer has to tell them apart.
#[tokio::test]
async fn a_group_in_a_joint_configuration_reports_both_sets_and_names_the_joint_phase() {
    let dir = TempDir::new().unwrap();

    let store = open(&dir, "alpha");
    seed_conf(&store, joint_conf());
    let alpha = spawn(
        "alpha",
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        store,
    );
    let registry = RaftRegistry::new();
    registry.register(alpha.clone()).unwrap();

    let (task, client, url) = serve(&registry).await;
    let status = statuses(&client, &url).await;
    let a = status.get("alpha").expect("alpha reports");

    assert_eq!(
        a.committed_voters,
        voters(&[0, 1]),
        "the committed set is the one agreed before the transition, not the one it is heading for"
    );
    assert_eq!(
        a.incoming_voters,
        Some(voters(&[0, 1, 2])),
        "the incoming set is reported while the transition is in flight"
    );
    assert_ne!(
        Some(a.committed_voters.clone()),
        a.incoming_voters,
        "both sets are reported but hold the same value {:?}, so a consumer cannot tell a \
         transition from rest",
        a.committed_voters
    );
    assert_eq!(a.membership_phase, MembershipPhase::Joint);

    task.abort();
}

/// The role vocabulary the status can produce covers a learner, which the debug
/// format of `raft_core::Role` cannot: that enum has Follower, Candidate and
/// Leader and no fourth variant.
#[tokio::test]
async fn a_node_that_is_a_learner_is_reported_as_a_learner_rather_than_a_follower() {
    let dir = TempDir::new().unwrap();

    let gamma = spawn(
        "gamma",
        Membership {
            voters: vec![1],
            learners: vec![0],
        },
        open(&dir, "gamma"),
    );
    let registry = RaftRegistry::new();
    registry.register(gamma.clone()).unwrap();

    let (task, client, url) = serve(&registry).await;
    let status = statuses(&client, &url).await;
    let g = status.get("gamma").expect("gamma reports");

    assert_eq!(g.id, 0);
    assert_eq!(
        g.role, "Learner",
        "node 0 is a learner of this group and is reported as {:?}",
        g.role
    );
    assert_eq!(g.learners, voters(&[0]));
    assert_eq!(g.committed_voters, voters(&[1]));
    assert_eq!(g.membership_phase, MembershipPhase::Stable);
    assert!(!g.is_leader, "a learner does not lead");

    task.abort();
}

/// The reporting half of AC4: three groups behind one endpoint, each key
/// carrying that group's own sets rather than one group's values repeated.
#[tokio::test]
async fn the_multi_group_status_endpoint_gives_each_group_its_own_sets() {
    let dir = TempDir::new().unwrap();

    let alpha_store = open(&dir, "alpha");
    seed_conf(&alpha_store, joint_conf());
    let alpha = spawn(
        "alpha",
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        alpha_store,
    );
    let beta = spawn(
        "beta",
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        open(&dir, "beta"),
    );
    let gamma = spawn(
        "gamma",
        Membership {
            voters: vec![1],
            learners: vec![0],
        },
        open(&dir, "gamma"),
    );

    let registry = RaftRegistry::new();
    registry.register(alpha.clone()).unwrap();
    registry.register(beta.clone()).unwrap();
    registry.register(gamma.clone()).unwrap();
    wait_leader(&beta).await;

    let (task, client, url) = serve(&registry).await;
    let status = statuses(&client, &url).await;

    assert_eq!(
        status.keys().cloned().collect::<Vec<_>>(),
        vec![
            "alpha".to_string(),
            "beta".to_string(),
            "gamma".to_string()
        ]
    );

    let a = &status["alpha"];
    let b = &status["beta"];
    let g = &status["gamma"];

    assert_eq!(a.committed_voters, voters(&[0, 1]));
    assert_eq!(a.incoming_voters, Some(voters(&[0, 1, 2])));
    assert_eq!(a.membership_phase, MembershipPhase::Joint);
    assert_eq!(a.learners, Vec::<u64>::new());

    assert_eq!(b.committed_voters, voters(&[0]));
    assert_eq!(b.incoming_voters, None);
    assert_eq!(b.membership_phase, MembershipPhase::Stable);
    assert_eq!(b.learners, Vec::<u64>::new());

    assert_eq!(g.committed_voters, voters(&[1]));
    assert_eq!(g.incoming_voters, None);
    assert_eq!(g.membership_phase, MembershipPhase::Stable);
    assert_eq!(g.learners, voters(&[0]));

    task.abort();
}
