use std::collections::HashMap;
use std::sync::Arc;
use tempfile::TempDir;

use raft_runtime::{
    group::{GroupId, LEGACY_GROUP_ID},
    FsyncPolicy, HostConfig, Membership, RaftHost, RaftStateMachine, RaftStore,
};

#[path = "support/cluster.rs"]
mod cluster;
use cluster::{bind, TestSm};

fn h2c_client() -> reqwest::Client {
    reqwest::Client::builder()
        .http2_prior_knowledge()
        .build()
        .unwrap()
}

#[tokio::test]
async fn row1_request_vote_refusal() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    // foreign group id
    let req_beta = serde_json::json!({
        "group_id": "beta",
        "from": 1,
        "req": {
            "term": 2,
            "candidate": 1,
            "last_log_index": 0,
            "last_log_term": 0,
        }
    });

    let resp_beta = client
        .post(&format!("{}/raft/request-vote", url))
        .json(&req_beta)
        .send()
        .await
        .unwrap();
    assert_ne!(resp_beta.status(), reqwest::StatusCode::OK);

    // correct group id
    let req_alpha = serde_json::json!({
        "group_id": "alpha",
        "from": 1,
        "req": {
            "term": 2,
            "candidate": 1,
            "last_log_index": 0,
            "last_log_term": 0,
        }
    });
    let resp_alpha = client
        .post(&format!("{}/raft/request-vote", url))
        .json(&req_alpha)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_alpha.status(), reqwest::StatusCode::OK);

    let body_alpha: serde_json::Value = resp_alpha.json().await.unwrap();
    assert!(body_alpha.get("term").is_some());

    serve.abort();
}

#[tokio::test]
async fn row2_append_entries_refusal() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let z1: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let req_foreign = serde_json::json!({
        "group_id": "beta",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "prev_log_index": 0,
            "prev_log_term": 0,
            "entries": [
                {
                    "index": 1,
                    "term": 2,
                    "command": [1, 2, 3]
                }
            ],
            "leader_commit": 1,
        }
    });
    let resp_foreign = client
        .post(&format!("{}/raft/append-entries", url))
        .json(&req_foreign)
        .send()
        .await
        .unwrap();
    assert_ne!(resp_foreign.status(), reqwest::StatusCode::OK);

    let z2: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        z1.get("applied_index").unwrap().as_u64().unwrap(),
        z2.get("applied_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("term").unwrap().as_u64().unwrap(),
        z2.get("term").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("commit_index").unwrap().as_u64().unwrap(),
        z2.get("commit_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("last_index").unwrap().as_u64().unwrap(),
        z2.get("last_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("snapshot_index").unwrap().as_u64().unwrap(),
        z2.get("snapshot_index").unwrap().as_u64().unwrap()
    );

    let req_correct = serde_json::json!({
        "group_id": "alpha",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "prev_log_index": 0,
            "prev_log_term": 0,
            "entries": [
                {
                    "index": 1,
                    "term": 2,
                    "command": [1, 2, 3]
                }
            ],
            "leader_commit": 1,
        }
    });
    let resp_correct = client
        .post(&format!("{}/raft/append-entries", url))
        .json(&req_correct)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_correct.status(), reqwest::StatusCode::OK);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let z3: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        z3.get("term").unwrap().as_u64().unwrap() > z1.get("term").unwrap().as_u64().unwrap()
            || z3.get("commit_index").unwrap().as_u64().unwrap()
                > z1.get("commit_index").unwrap().as_u64().unwrap()
            || z3.get("last_index").unwrap().as_u64().unwrap()
                > z1.get("last_index").unwrap().as_u64().unwrap()
            || z3.get("applied_index").unwrap().as_u64().unwrap()
                > z1.get("applied_index").unwrap().as_u64().unwrap()
    );

    serve.abort();
}

#[tokio::test]
async fn row3_absent_group_id_refused() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open(dir.path().to_str().unwrap(), 0, FsyncPolicy::Os).unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn(
        0,
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let req_absent = serde_json::json!({
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "prev_log_index": 0,
            "prev_log_term": 0,
            "entries": [
                {
                    "index": 1,
                    "term": 2,
                    "command": [1, 2, 3]
                }
            ],
            "leader_commit": 1,
        }
    });

    let z1: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let resp_absent = client
        .post(&format!("{}/raft/append-entries", url))
        .json(&req_absent)
        .send()
        .await
        .unwrap();
    assert_ne!(resp_absent.status(), reqwest::StatusCode::OK);

    let z2: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        z1.get("term").unwrap().as_u64().unwrap(),
        z2.get("term").unwrap().as_u64().unwrap()
    );

    let req_explicit = serde_json::json!({
        "group_id": LEGACY_GROUP_ID,
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "prev_log_index": 0,
            "prev_log_term": 0,
            "entries": [
                {
                    "index": 1,
                    "term": 2,
                    "command": [1, 2, 3]
                }
            ],
            "leader_commit": 1,
        }
    });
    let resp_explicit = client
        .post(&format!("{}/raft/append-entries", url))
        .json(&req_explicit)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_explicit.status(), reqwest::StatusCode::OK);

    serve.abort();
}

/// The leadership-transfer endpoint (#3571) carries the same cross-group guard
/// as the four that preceded it.
///
/// The foreign-group case asserts `400 BAD_REQUEST` exactly, not merely "not
/// `200 OK`". A route that does not exist answers `404`, so the looser form —
/// which rows 1 through 3 above use — is satisfied before the endpoint is
/// added, and this row's whole purpose is to show that it was. That is the
/// shape #3566 records: a guard that lands with no row, or a row a missing
/// guard also passes.
///
/// This row measures the guard and the route, not the handoff. What the message
/// does once it is past the guard is `libs/raft-core/e2e/leadership_transfer.rs`.
#[tokio::test]
async fn row5_timeout_now_refusal() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let req_beta = serde_json::json!({
        "group_id": "beta",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
        }
    });
    let resp_beta = client
        .post(&format!("{}/raft/timeout-now", url))
        .json(&req_beta)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp_beta.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "a message addressed to another group must be refused by the guard, not \
         by the router having no such route"
    );

    let req_alpha = serde_json::json!({
        "group_id": "alpha",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
        }
    });
    let resp_alpha = client
        .post(&format!("{}/raft/timeout-now", url))
        .json(&req_alpha)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_alpha.status(), reqwest::StatusCode::OK);

    serve.abort();
}

#[test]
fn row4_durable_file_paths() {
    let dir = TempDir::new().unwrap();

    let s_alpha1 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let s_beta = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("beta".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let s_alpha2 = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    let s_adv = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("../../../foo".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();

    assert_ne!(s_alpha1.path(), s_beta.path());
    assert_eq!(s_alpha1.path(), s_alpha2.path());

    assert!(s_adv.path().starts_with(dir.path()));
    assert_eq!(s_adv.path().parent().unwrap(), dir.path());
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

#[tokio::test]
async fn row6_install_snapshot_refusal() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0, 1],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let z1: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let req_beta = serde_json::json!({
        "group_id": "beta",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "snapshot_index": 2,
            "snapshot_term": 2,
            "data": [],
        }
    });
    let resp_beta = client
        .post(&format!("{}/raft/install-snapshot", url))
        .json(&req_beta)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp_beta.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "a message addressed to another group must be refused by the guard, not \
         by the router having no such route"
    );

    let z2: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        z1.get("applied_index").unwrap().as_u64().unwrap(),
        z2.get("applied_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("term").unwrap().as_u64().unwrap(),
        z2.get("term").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("commit_index").unwrap().as_u64().unwrap(),
        z2.get("commit_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("last_index").unwrap().as_u64().unwrap(),
        z2.get("last_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("snapshot_index").unwrap().as_u64().unwrap(),
        z2.get("snapshot_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(sm.applied_index(), 0);

    let req_alpha = serde_json::json!({
        "group_id": "alpha",
        "from": 1,
        "req": {
            "term": 2,
            "leader": 1,
            "snapshot_index": 2,
            "snapshot_term": 2,
            "data": [],
        }
    });
    let resp_alpha = client
        .post(&format!("{}/raft/install-snapshot", url))
        .json(&req_alpha)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_alpha.status(), reqwest::StatusCode::OK);

    let body_alpha: serde_json::Value = resp_alpha.json().await.unwrap();
    assert!(body_alpha.get("term").is_some());

    serve.abort();
}

#[tokio::test]
async fn row7_publish_refusal() {
    let dir = TempDir::new().unwrap();
    let sm = TestSm::new();
    let store = RaftStore::open_group(
        dir.path().to_str().unwrap(),
        0,
        GroupId("alpha".to_string()),
        FsyncPolicy::Os,
    )
    .unwrap();
    let (l, url) = bind().await;
    let host = Arc::new(RaftHost::spawn_group(
        0,
        GroupId("alpha".to_string()),
        Membership {
            voters: vec![0],
            learners: vec![],
        },
        HashMap::new(),
        store,
        sm.clone() as Arc<dyn RaftStateMachine>,
        HostConfig::default(),
    ));
    wait_leader(&host).await;

    let serve = tokio::spawn({
        let r = host.router();
        async move {
            loop {
                if let Ok((stream, _)) = l.accept().await {
                    let r = r.clone();
                    tokio::spawn(async move {
                        let _ = transport_h2c::server::serve_connection(stream, r).await;
                    });
                }
            }
        }
    });

    let client = h2c_client();

    let z1: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let req_beta = serde_json::json!({
        "group_id": "beta",
        "command": vec![1u8, 2, 3],
    });
    let resp_beta = client
        .post(&format!("{}/raft/publish", url))
        .json(&req_beta)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp_beta.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "a message addressed to another group must be refused by the guard, not \
         by the router having no such route"
    );

    let z2: serde_json::Value = client
        .get(&format!("{}/raftz", url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        z1.get("applied_index").unwrap().as_u64().unwrap(),
        z2.get("applied_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("term").unwrap().as_u64().unwrap(),
        z2.get("term").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("commit_index").unwrap().as_u64().unwrap(),
        z2.get("commit_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("last_index").unwrap().as_u64().unwrap(),
        z2.get("last_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(
        z1.get("snapshot_index").unwrap().as_u64().unwrap(),
        z2.get("snapshot_index").unwrap().as_u64().unwrap()
    );
    assert_eq!(sm.applied_index(), 0);

    let req_alpha = serde_json::json!({
        "group_id": "alpha",
        "command": vec![1u8, 2, 3],
    });
    let resp_alpha = client
        .post(&format!("{}/raft/publish", url))
        .json(&req_alpha)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_alpha.status(), reqwest::StatusCode::OK);

    let body_alpha: serde_json::Value = resp_alpha.json().await.unwrap();
    assert!(body_alpha.get("seq").is_some());

    serve.abort();
}
