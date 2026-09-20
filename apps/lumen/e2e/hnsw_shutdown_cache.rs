//! Public-process recovery oracle. Graph caches never replace durable data.
#![cfg(unix)]

#[path = "support/serve_budget_support.rs"]
mod support;

use serde_json::{json, Value};
use std::{path::{Path, PathBuf}, thread, time::{Duration, Instant}};
use support::{LumenProcess, ServeMode};

const SNAPSHOT_SECS: u64 = 3600;

fn start(root: &Path) -> LumenProcess {
    let mut process = LumenProcess::spawn(Some(root), ServeMode::Segment, SNAPSHOT_SECS);
    process.wait_until_ready(SNAPSHOT_SECS);
    process
}

fn seed(process: &LumenProcess) {
    let created = process.put_json("/collections/graph-cache", &json!({
        "fields": {"v": {"type": "vector", "dim": 3, "metric": "l2", "backend": "hnsw-cpu"}}
    }));
    assert_eq!(created.status, 200, "{}", created.body);
    let items: Vec<_> = (0..256).map(|i| json!({
        "external_id": format!("row-{i}"), "field": "v", "value": [i as f32, 1.0, 2.0]
    })).collect();
    let indexed = process.post_json("/collections/graph-cache/index", &json!({"items": items}));
    assert_eq!(indexed.status, 200, "{}", indexed.body);
    let checkpoint = process.post_json("/admin/checkpoint", &json!({}));
    assert_eq!(checkpoint.status, 200, "{}", checkpoint.body);
}

fn query(process: &LumenProcess, x: f32) -> Value {
    let response = process.post_json("/collections/graph-cache/search", &json!({
        "query": {"knn": {"field": "v", "vector": [x, 1.0, 2.0], "k": 5}}, "limit": 5
    }));
    assert_eq!(response.status, 200, "{}", response.body);
    response.body["hits"].clone()
}

fn cache_before_docker_stop_edge(process: &mut LumenProcess) {
    let deadline = Instant::now() + Duration::from_secs(10);
    process.send_sigterm();
    loop {
        let logs = process.logs();
        if logs.contains("HNSW shutdown cache saved") {
            break;
        }
        assert!(Instant::now() < deadline, "shutdown must publish the optional graph cache before Docker's existing stop edge; logs:\n{logs}");
        thread::sleep(Duration::from_millis(20));
    }
    // The default 30-second listener grace remains unchanged. Model Docker's
    // eventual kill after the cache is complete, without extending its budget.
    process.stop_and_logs();
}

fn cache_graphs(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).expect("read test cache directory") {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                pending.push(entry.path());
            } else if entry.file_name().to_string_lossy().ends_with(".hnsw.graph") {
                found.push(entry.path());
            }
        }
    }
    found
}

#[test]
fn shutdown_cache_reopens_real_graph_and_replays_a_later_unclean_tail() {
    let root = tempfile::tempdir().unwrap();
    let mut live = start(root.path());
    seed(&live);
    let expected = query(&live, 91.0);
    cache_before_docker_stop_edge(&mut live);
    assert!(!cache_graphs(root.path()).is_empty(), "a saved event must have graph bytes");

    let mut cold = start(root.path());
    assert!(cold.logs().contains("HNSW graph cache loaded"), "cold path must use the saved graph, not rebuild it:\n{}", cold.logs());
    assert_eq!(query(&cold, 91.0), expected, "cache must preserve IDs, order and scores");
    let appended = cold.post_json("/collections/graph-cache/index", &json!({"items": [{
        "external_id": "after-cache", "field": "v", "value": [1000.0, 1.0, 2.0]
    }]}));
    assert_eq!(appended.status, 200, "{}", appended.body);
    cold.stop_and_logs();
    let recovered = start(root.path());
    assert_eq!(query(&recovered, 1000.0)[0]["external_id"], "after-cache", "unclean AOF tail must replay over the cached checkpoint graph");
}

#[test]
fn shutdown_cache_matches_replayed_tail_without_rewriting_current() {
    let root = tempfile::tempdir().unwrap();
    let mut live = start(root.path());
    seed(&live);
    let current = std::fs::read(root.path().join("CURRENT")).unwrap();
    let changed = live.post_json("/collections/graph-cache/index", &json!({"items": [{
        "external_id": "row-91", "field": "v", "value": [999.0, 1.0, 2.0]
    }, {
        "external_id": "shutdown-tail", "field": "v", "value": [1000.0, 1.0, 2.0]
    }]}));
    assert_eq!(changed.status, 200, "{}", changed.body);
    let expected = query(&live, 1000.0);
    cache_before_docker_stop_edge(&mut live);
    assert_eq!(std::fs::read(root.path().join("CURRENT")).unwrap(), current,
        "optional shutdown cache must not spend the stop budget rewriting an authoritative checkpoint");
    let recovered = start(root.path());
    assert!(recovered.logs().contains("HNSW graph cache loaded"),
        "cache fingerprint must be checked after authoritative AOF replay:\n{}", recovered.logs());
    assert_eq!(query(&recovered, 1000.0), expected);
    assert_eq!(query(&recovered, 999.0)[0]["external_id"], "row-91");
}

#[test]
fn corrupt_optional_graph_falls_back_to_authoritative_checkpoint() {
    let root = tempfile::tempdir().unwrap();
    let mut live = start(root.path());
    seed(&live);
    let expected = query(&live, 64.0);
    cache_before_docker_stop_edge(&mut live);
    let graphs = cache_graphs(root.path());
    assert!(!graphs.is_empty());
    for graph in graphs {
        std::fs::write(graph, b"damaged optional cache").unwrap();
    }
    let recovered = start(root.path());
    assert!(!recovered.logs().contains("HNSW graph cache loaded"), "corrupt bytes cannot be accepted");
    assert_eq!(query(&recovered, 64.0), expected, "cache damage must not lose durable data");
}

#[test]
fn unclean_shutdown_without_a_cache_keeps_old_recovery_path() {
    let root = tempfile::tempdir().unwrap();
    let mut live = start(root.path());
    seed(&live);
    let expected = query(&live, 17.0);
    live.stop_and_logs();
    assert!(cache_graphs(root.path()).is_empty(), "ordinary checkpoints must not dump graphs in the request interval");
    let recovered = start(root.path());
    assert_eq!(query(&recovered, 17.0), expected);
}

#[test]
fn stale_graph_cannot_override_a_newer_checkpoint_vector() {
    let root = tempfile::tempdir().unwrap();
    let mut live = start(root.path());
    seed(&live);
    cache_before_docker_stop_edge(&mut live);
    let mut changed = start(root.path());
    let indexed = changed.post_json("/collections/graph-cache/index", &json!({"items": [{
        "external_id": "row-91", "field": "v", "value": [999.0, 1.0, 2.0]
    }]}));
    assert_eq!(indexed.status, 200, "{}", indexed.body);
    let checkpoint = changed.post_json("/admin/checkpoint", &json!({}));
    assert_eq!(checkpoint.status, 200, "{}", checkpoint.body);
    changed.stop_and_logs();
    let recovered = start(root.path());
    assert!(!recovered.logs().contains("HNSW graph cache loaded"), "old graph content must not match the new checkpoint");
    assert_eq!(query(&recovered, 999.0)[0]["external_id"], "row-91");
}

#[test]
fn invalid_cache_root_is_ignored_without_following_a_symlink() {
    for link in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("sentinel"), b"keep").unwrap();
        let mut live = start(root.path());
        seed(&live);
        let expected = query(&live, 17.0);
        live.stop_and_logs();
        let cache = root.path().join("hnsw-graph-cache");
        if link {
            std::os::unix::fs::symlink(outside.path(), &cache).unwrap();
        } else {
            std::fs::write(&cache, b"not a cache directory").unwrap();
        }
        let recovered = start(root.path());
        assert_eq!(query(&recovered, 17.0), expected);
        assert_eq!(std::fs::read(outside.path().join("sentinel")).unwrap(), b"keep");
        assert_eq!(std::fs::read_dir(outside.path()).unwrap().count(), 1);
    }
}
