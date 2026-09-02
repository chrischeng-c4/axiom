//! Lumen's state machine on the deterministic Raft host.
//!
//! The shared adversarial corpus proves the Raft safety rules. This test keeps
//! Lumen in that proof boundary by using its real command codec and EngineSm.

use std::collections::BTreeMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use lumen::log_entry::RaftLogEntry;
use lumen::raft_sm::EngineSm;
use lumen::storage::Engine;
use lumen::types::{CreateCollectionRequest, FieldSpec, FieldType};
use lumen::wal::WalRecord;
use raft_runtime::conformance::{
    ConformanceRole, DeterministicHost, StateMachineOperation, StepError,
};
use raft_runtime::{FsyncPolicy, Membership, NodeId, RaftStateMachine, RaftStore};
use tempfile::TempDir;

const NODE_COUNT: usize = 3;
const MAX_DELIVERIES: usize = 512;

fn open_host(
    root: &TempDir,
    id: NodeId,
    membership: &Membership,
    sm: Arc<EngineSm>,
) -> Result<DeterministicHost> {
    let path = root
        .path()
        .to_str()
        .context("temporary Raft path is not UTF-8")?;
    let store = RaftStore::open(path, id, FsyncPolicy::Always)?;
    Ok(DeterministicHost::open(
        id,
        membership.clone(),
        store,
        sm as Arc<dyn RaftStateMachine>,
    )?)
}

fn drain(hosts: &mut [Option<DeterministicHost>]) -> Result<()> {
    let mut delivered = 0;
    loop {
        let mut progressed = false;
        for from in 0..hosts.len() {
            loop {
                let Some(to) = hosts[from]
                    .as_ref()
                    .and_then(|host| host.ready_peers().into_iter().next())
                else {
                    break;
                };
                let envelope = hosts[from]
                    .as_mut()
                    .expect("source host is live")
                    .take_next(to)
                    .expect("ready peer has an envelope");
                let target = usize::try_from(to).context("peer id does not fit usize")?;
                hosts
                    .get_mut(target)
                    .and_then(Option::as_mut)
                    .context("envelope targets a stopped or unknown host")?
                    .receive(envelope)?;
                delivered += 1;
                anyhow::ensure!(
                    delivered <= MAX_DELIVERIES,
                    "deterministic delivery did not quiesce"
                );
                progressed = true;
            }
        }
        if !progressed {
            return Ok(());
        }
    }
}

fn elect(hosts: &mut [Option<DeterministicHost>], candidate: usize) -> Result<usize> {
    for _ in 0..50 + candidate {
        hosts[candidate]
            .as_mut()
            .expect("candidate host is live")
            .tick()?;
    }
    drain(hosts)?;
    let leaders: Vec<_> = hosts
        .iter()
        .enumerate()
        .filter_map(|(id, host)| {
            host.as_ref()
                .is_some_and(|host| host.view().role == ConformanceRole::Leader)
                .then_some(id)
        })
        .collect();
    anyhow::ensure!(
        leaders.len() == 1,
        "deterministic cluster elected {} leaders: {leaders:?}",
        leaders.len()
    );
    Ok(leaders[0])
}

fn schema() -> CreateCollectionRequest {
    let mut fields = BTreeMap::new();
    fields.insert(
        "title".to_owned(),
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        },
    );
    CreateCollectionRequest { fields }
}

#[test]
fn engine_sm_commits_and_recovers_on_the_deterministic_host() -> Result<()> {
    let root = TempDir::new()?;
    let membership = Membership {
        voters: (0..NODE_COUNT as NodeId).collect(),
        learners: Vec::new(),
    };
    let mut engines = Vec::with_capacity(NODE_COUNT);
    let mut sms = Vec::with_capacity(NODE_COUNT);
    let mut hosts = Vec::with_capacity(NODE_COUNT);
    for id in 0..NODE_COUNT as NodeId {
        let engine = Arc::new(Engine::new());
        let sm = EngineSm::new(engine.clone(), 0);
        hosts.push(Some(open_host(&root, id, &membership, sm.clone())?));
        engines.push(engine);
        sms.push(sm);
    }

    let before_election = hosts[0]
        .as_mut()
        .expect("node zero is live")
        .try_propose(StateMachineOperation::Command(vec![0]));
    assert_eq!(before_election, Err(StepError::NotLeader));

    let leader = elect(&mut hosts, 0)?;
    let command = WalRecord::new(RaftLogEntry::CreateCollection {
        collection_id: "docs".to_owned(),
        req: schema(),
    })
    .encode()?;
    let index = hosts[leader]
        .as_mut()
        .expect("leader is live")
        .try_propose(StateMachineOperation::Command(command))?;
    drain(&mut hosts)?;

    for (id, (engine, sm)) in engines.iter().zip(&sms).enumerate() {
        assert_eq!(sm.applied_index(), index, "node {id} did not apply");
        let stats = engine
            .stats("docs")
            .with_context(|| format!("node {id} did not create the collection"))?;
        assert_eq!(stats.fields["title"].field_type, FieldType::Keyword);
    }

    let follower = (leader + 1) % NODE_COUNT;
    drop(hosts[follower].take().expect("follower is live"));
    let reopened_engine = Arc::new(Engine::new());
    let reopened_sm = EngineSm::new(reopened_engine.clone(), 0);
    hosts[follower] = Some(open_host(
        &root,
        follower as NodeId,
        &membership,
        reopened_sm.clone(),
    )?);

    assert_eq!(reopened_sm.applied_index(), index);
    let restored = reopened_engine
        .stats("docs")
        .context("reopened follower lost the committed collection")?;
    assert_eq!(restored.fields["title"].field_type, FieldType::Keyword);
    assert_eq!(
        hosts[follower]
            .as_ref()
            .expect("follower reopened")
            .view()
            .commit_index,
        index
    );

    Ok(())
}
