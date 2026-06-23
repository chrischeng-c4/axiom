// SPEC-MANAGED: projects/relay/tech-design/logic/ha-via-leader-follower-log-replication-async-primary-backup.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:28986b9b" tracker="pending-tracker" reason="Real-h2c HA test: a follower replicator converges to a leader's N entries, replicates entries published after connect, serves reads, and does not duplicate on continued tailing."
//! HA replication (#135): a follower tails a real h2c leader, converges to its
//! entries, keeps replicating live, serves reads, and never duplicates.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;

use relay::server::{router, AppState};
use relay::server_config::RelayServerConfig;
use relay::{spawn_follower, Relay, RelayCoreConfig};

async fn start_leader() -> (String, Arc<Relay>) {
    let state = AppState::new(RelayServerConfig::ephemeral());
    let leader = state.relay_handle();
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (format!("http://{addr}"), leader)
}

fn publish(r: &Relay, subject: &str, range: std::ops::Range<usize>) {
    let now = Utc::now();
    for i in range {
        r.publish(
            subject,
            &format!("m{i}"),
            serde_json::json!({ "i": i }),
            BTreeMap::new(),
            now,
        )
        .unwrap();
    }
}

async fn wait_for_len(r: &Relay, subject: &str, want: u64) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let len = r.log_len(subject).unwrap();
        if len >= want || Instant::now() > deadline {
            return len;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn follower_converges_replicates_live_and_serves_reads() {
    let (leader_url, leader) = start_leader().await;

    // Seed the leader, then start the follower.
    publish(&leader, "s", 0..50);
    let follower = Arc::new(Relay::new(RelayCoreConfig::in_memory()));
    let handle = spawn_follower(Arc::clone(&follower), leader_url, vec!["s".to_string()]);

    // Converge to the seeded entries.
    assert_eq!(
        wait_for_len(&follower, "s", 50).await,
        50,
        "follower converged to leader"
    );

    // Live replication: entries published after the follower connected arrive too.
    publish(&leader, "s", 50..60);
    assert_eq!(
        wait_for_len(&follower, "s", 60).await,
        60,
        "live entries replicated"
    );

    // The follower serves reads of the replicated data (promotable).
    follower.subscribe("s", "reader", 0).unwrap();
    let got = follower.poll("s", "reader").unwrap();
    let ids: std::collections::BTreeSet<String> =
        got.iter().map(|e| e.message_id.clone()).collect();
    let expected: std::collections::BTreeSet<String> = (0..60).map(|i| format!("m{i}")).collect();
    assert_eq!(ids, expected, "follower serves every replicated message");

    // No duplicates on continued tailing (dedupe on message_id).
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(follower.log_len("s").unwrap(), 60, "no duplicates");

    handle.stop();
}
// HANDWRITE-END
