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
