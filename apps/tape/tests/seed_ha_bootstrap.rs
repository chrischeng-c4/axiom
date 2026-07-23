//! #2436 GKE runs 0723113842/0723120246: a fresh 3-replica group where every
//! member consumed the SAME `bootstrapSeedUri` object came up Ready with an
//! EMPTY journal. Root cause (#2465): a single-node tape's `/admin/backup`
//! snapshot carries `up_to: 0` (no raft applied index exists there), and raft
//! treats an index-0 snapshot as nonexistent, so the seed silently
//! evaporated. These tests bootstrap an all-fresh 3-voter group from
//! identical seed bytes for BOTH origins — a replica-source snapshot
//! (`up_to > 0`) and the single-node-origin shape (`up_to == 0`) — and
//! require the seeded events to replay once the group elects and applies.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use raft_runtime::Membership;
use tape::raft::{prepare_bootstrap_seed, snapshot_bytes, TapeRaft};
use tape::TapeJournal;

async fn seeded_group_surfaces_data(up_to: u64) {
    let source = Arc::new(Mutex::new(TapeJournal::default()));
    for n in 1..=3 {
        source.lock().unwrap().append(
            "acceptance",
            None,
            serde_json::json!({ "n": n }),
            Some(100),
        );
    }
    let bytes = snapshot_bytes(&source, up_to).unwrap();

    let mut urls = HashMap::new();
    let mut listeners = Vec::new();
    for id in 0u64..3 {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        urls.insert(id, format!("http://{}", l.local_addr().unwrap()));
        listeners.push(l);
    }
    let membership = Membership {
        voters: vec![0, 1, 2],
        learners: vec![],
    };

    let dirs: Vec<tempfile::TempDir> = (0..3).map(|_| tempfile::tempdir().unwrap()).collect();
    let mut journals = Vec::new();
    let mut rafts = Vec::new();
    for (id, listener) in listeners.into_iter().enumerate() {
        prepare_bootstrap_seed(dirs[id].path(), id as u64, &bytes).unwrap();
        let journal = Arc::new(Mutex::new(TapeJournal::default()));
        let peers: HashMap<u64, String> = urls
            .iter()
            .filter(|(k, _)| **k != id as u64)
            .map(|(k, v)| (*k, v.clone()))
            .collect();
        let raft = Arc::new(
            TapeRaft::spawn(
                Arc::clone(&journal),
                &dirs[id].path().join("raft"),
                id as u64,
                membership.clone(),
                peers,
                TapeRaft::host_config(8),
            )
            .unwrap(),
        );
        let app = raft.router();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        journals.push(journal);
        rafts.push(raft);
    }

    // The seed must surface once the fresh group elects and applies: every
    // member's applied index reaches at least the seeded index and the
    // journal replays the seeded events.
    let want_applied = up_to.max(1);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let applied: Vec<u64> = rafts.iter().map(|r| r.applied_index()).collect();
        let replayed: Vec<usize> = journals
            .iter()
            .map(|j| j.lock().unwrap().replay("acceptance", None, None, None).len())
            .collect();
        if applied.iter().all(|&a| a >= want_applied) && replayed.iter().all(|&n| n == 3) {
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            panic!(
                "seeded 3-voter group (up_to={up_to}) never surfaced the seed: applied={applied:?} replayed={replayed:?}"
            );
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// A replica-origin seed (`up_to > 0`) bootstraps a fresh 3-voter group.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn replica_origin_seed_bootstraps_a_fresh_three_voter_group() {
    seeded_group_surfaces_data(3).await;
}

/// #2465: the single-node-origin seed shape (`up_to == 0`, exactly what a
/// 1x1 deployment's `/admin/backup` emits) must also bootstrap the group —
/// this is the documented DR path from a default deployment into HA.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn single_node_origin_seed_bootstraps_a_fresh_three_voter_group() {
    seeded_group_surfaces_data(0).await;
}
