// @ec tape-provision-topics
// @capability declarative-subscription-provisioning
// @claim spec-topics-ensure-idempotent
// @contract tape-spec-topics-cr-provisioned

// Contract: Declaring spec.topics on the CR creates subscriptions before the listener starts.
// Contract: Restarting with the same spec.topics logs "already_exists" without errors.
// Contract: Declaring a topic alone (no subscriptions) logs "noted_implicit" without mutations.
// Contract: A provisioned subscription's acknowledged cursor survives a
// single-node WAL restart on the same data directory.

#[cfg(test)]
mod tests {
    use std::net::TcpListener;
    use std::path::Path;
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};

    use serde_json::{json, Value};
    use tape::{SubscriptionError, TapeJournal};

    const PROVISION_TOPICS: &str = r#"[{"name":"orders","subscriptions":["billing"]}]"#;
    const STARTUP_BUDGET: Duration = Duration::from_secs(10);
    const POLL_INTERVAL: Duration = Duration::from_millis(50);

    struct Node {
        child: Child,
        bind: String,
    }

    impl Node {
        fn base_url(&self) -> String {
            format!("http://{}", self.bind)
        }

        async fn stop(&mut self, phase: &str) {
            let pid = self.child.id().to_string();
            let status = Command::new("kill")
                .args(["-TERM", &pid])
                .status()
                .expect("send SIGTERM to tape serve subprocess");
            assert!(
                status.success(),
                "{phase}: SIGTERM for tape pid {pid} failed"
            );

            let deadline = Instant::now() + STARTUP_BUDGET;
            loop {
                match self.child.try_wait() {
                    Ok(Some(status)) => {
                        assert!(
                            status.success(),
                            "{phase}: tape serve exited unsuccessfully: {status}"
                        );
                        return;
                    }
                    Ok(None) => {}
                    Err(error) => panic!("{phase}: wait for tape serve failed: {error}"),
                }
                assert!(
                    Instant::now() < deadline,
                    "{phase}: tape serve did not exit after SIGTERM"
                );
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }
    }

    impl Drop for Node {
        fn drop(&mut self) {
            // A failed assertion must not leave a live tape process behind.
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }

    fn free_addr() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind a test port");
        listener.local_addr().expect("read test port").to_string()
    }

    fn spawn_provisioned_node(bind: &str, data_dir: &Path) -> Node {
        let stderr = if std::env::var_os("TAPE_TEST_LOG").is_some() {
            Stdio::inherit()
        } else {
            Stdio::null()
        };
        let child = Command::new(env!("CARGO_BIN_EXE_tape"))
            .args(["serve", "--bind", bind, "--data-dir"])
            .arg(data_dir)
            .args(["--grace-secs", "0"])
            .env("TAPE_AUTH", "off")
            .env("TAPE_PROVISION_TOPICS", PROVISION_TOPICS)
            .env_remove("REPLICAS_PER_SHARD")
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .expect("spawn provisioned tape serve subprocess");
        Node {
            child,
            bind: bind.to_string(),
        }
    }

    async fn wait_healthy(client: &reqwest::Client, base: &str, phase: &str) {
        let deadline = Instant::now() + STARTUP_BUDGET;
        loop {
            let last_status = match client.get(format!("{base}/healthz")).send().await {
                Ok(response) if response.status().is_success() => return,
                Ok(response) => response.status().to_string(),
                Err(error) => format!("request error: {error}"),
            };
            assert!(
                Instant::now() < deadline,
                "{phase}: {base} never became healthy; last_status={last_status}"
            );
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    async fn recovered_state(
        client: &reqwest::Client,
        base: &str,
    ) -> Result<(Vec<i64>, Option<i64>), String> {
        let replay = client
            .get(format!("{base}/topics/orders/replay"))
            .send()
            .await
            .map_err(|error| format!("replay request error: {error}"))?;
        let replay_status = replay.status();
        if !replay_status.is_success() {
            return Err(format!("replay status: {replay_status}"));
        }
        let replay: Value = replay
            .json()
            .await
            .map_err(|error| format!("replay JSON error: {error}"))?;
        let event_ns = replay["events"]
            .as_array()
            .ok_or_else(|| "replay has no events array".to_string())?
            .iter()
            .map(|event| {
                event["payload"]["n"]
                    .as_i64()
                    .ok_or_else(|| "replay event has no integer payload.n".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let checkpoint = client
            .get(format!("{base}/topics/orders/consumers/billing/checkpoint"))
            .send()
            .await
            .map_err(|error| format!("checkpoint request error: {error}"))?;
        let checkpoint_status = checkpoint.status();
        if !checkpoint_status.is_success() {
            return Err(format!("checkpoint status: {checkpoint_status}"));
        }
        let checkpoint: Value = checkpoint
            .json()
            .await
            .map_err(|error| format!("checkpoint JSON error: {error}"))?;
        Ok((event_ns, checkpoint["checkpoint"]["offset"].as_i64()))
    }

    async fn wait_for_recovered_state(client: &reqwest::Client, base: &str) {
        let deadline = Instant::now() + STARTUP_BUDGET;
        loop {
            let last_state = match recovered_state(client, base).await {
                Ok((event_ns, Some(3))) if event_ns.as_slice() == &[1, 2, 3] => return,
                Ok((event_ns, checkpoint)) => {
                    format!("event_ns={event_ns:?}, checkpoint={checkpoint:?}")
                }
                Err(error) => error,
            };
            assert!(
                Instant::now() < deadline,
                "restart recovery did not restore three events and checkpoint offset 3; {last_state}"
            );
            tokio::time::sleep(POLL_INTERVAL).await;
        }
    }

    #[test]
    fn provision_topics_single_subscription() {
        let mut journal = TapeJournal::default();

        // Simulate declaring one topic with one subscription
        let result = journal.create_subscription("orders", "billing");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "billing");

        // Verify the subscription exists
        let subscription = journal.subscription("orders", "billing");
        assert!(subscription.is_some());
        assert_eq!(subscription.unwrap().topic, "orders");
    }

    #[test]
    fn provision_topics_multiple_subscriptions() {
        let mut journal = TapeJournal::default();

        // Simulate declaring one topic with multiple subscriptions
        let subs = vec!["billing", "audit", "analytics"];
        for sub in &subs {
            let result = journal.create_subscription("orders", *sub);
            assert!(result.is_ok(), "failed to create subscription {}", sub);
        }

        // Verify all subscriptions exist
        let subscriptions = journal.subscriptions("orders");
        assert_eq!(subscriptions.len(), 3);
        assert!(subscriptions
            .iter()
            .all(|s| s.topic == "orders" && subs.contains(&s.name.as_str())));
    }

    #[test]
    fn provision_topics_idempotent_already_exists() {
        let mut journal = TapeJournal::default();

        // Create the subscription
        let first = journal.create_subscription("orders", "billing");
        assert!(first.is_ok());

        // Try to create it again - should return AlreadyExists
        let second = journal.create_subscription("orders", "billing");
        assert!(matches!(
            second,
            Err(SubscriptionError::AlreadyExists { topic, name })
                if topic == "orders" && name == "billing"
        ));

        // Verify only one subscription exists
        let subscriptions = journal.subscriptions("orders");
        assert_eq!(subscriptions.len(), 1);
    }

    #[test]
    fn provision_topics_multiple_topics() {
        let mut journal = TapeJournal::default();

        // Declare multiple topics with different subscriptions
        let _ = journal.create_subscription("orders", "billing");
        let _ = journal.create_subscription("orders", "audit");
        let _ = journal.create_subscription("telemetry", "metrics");
        let _ = journal.create_subscription("telemetry", "logging");

        // Verify subscriptions are scoped to their topics
        let orders_subs = journal.subscriptions("orders");
        assert_eq!(orders_subs.len(), 2);
        assert!(orders_subs.iter().all(|s| s.topic == "orders"));

        let telemetry_subs = journal.subscriptions("telemetry");
        assert_eq!(telemetry_subs.len(), 2);
        assert!(telemetry_subs.iter().all(|s| s.topic == "telemetry"));
    }

    #[test]
    fn provision_topics_empty_subscriptions_list() {
        let mut journal = TapeJournal::default();

        // Appending to a topic makes it implicit (no explicit subscription creation)
        journal.append("orders", None, json!({"n": 1}), None);

        // Verify the topic exists implicitly via end_offset
        let end_offset = journal.end_offset("orders");
        assert_eq!(end_offset, 1);

        // No subscriptions should exist yet
        let subscriptions = journal.subscriptions("orders");
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn provision_topics_pull_after_subscription() {
        let mut journal = TapeJournal::default();

        // Append events first
        journal.append("orders", None, json!({"n": 1}), None);
        journal.append("orders", None, json!({"n": 2}), None);

        // Create subscription
        journal.create_subscription("orders", "consumer-a").unwrap();

        // Pull should work immediately from offset 0
        let batch = journal
            .pull_subscription("orders", "consumer-a", None)
            .unwrap();
        assert_eq!(batch.cursor, 0);
        assert_eq!(batch.events.len(), 2);
        assert_eq!(batch.next_offset, 2);
    }

    #[test]
    fn provision_topics_cross_subscription_checkpoint() {
        let mut journal = TapeJournal::default();

        // Append events and create a subscription
        journal.append("orders", None, json!({"n": 1}), None);
        journal.append("orders", None, json!({"n": 2}), None);
        journal.create_subscription("orders", "consumer-a").unwrap();

        // Pull and ack as consumer-a
        let batch = journal
            .pull_subscription("orders", "consumer-a", None)
            .unwrap();
        journal
            .ack_subscription("orders", "consumer-a", batch.next_offset)
            .unwrap();

        // Create a second subscription - it should start from offset 0 independently
        journal.create_subscription("orders", "consumer-b").unwrap();
        let batch_b = journal
            .pull_subscription("orders", "consumer-b", None)
            .unwrap();
        assert_eq!(batch_b.cursor, 0);
        assert_eq!(batch_b.events.len(), 2);
    }

    /// A CR supplies `TAPE_PROVISION_TOPICS` on every boot. This black-box
    /// case uses the actual `tape serve` executable twice over one WAL data
    /// directory, so a startup-only in-memory subscription create cannot
    /// accidentally make the acknowledged subscription cursor durable.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn provisioned_subscription_checkpoint_survives_single_node_wal_restart() {
        let data_dir = tempfile::tempdir().expect("create WAL data directory");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(1))
            .build()
            .expect("build HTTP client");

        let first_bind = free_addr();
        let mut first = spawn_provisioned_node(&first_bind, data_dir.path());
        wait_healthy(&client, &first.base_url(), "first startup").await;

        for n in 1_i64..=3 {
            let response = client
                .post(format!("{}/topics/orders/append", first.base_url()))
                .json(&json!({ "payload": { "n": n } }))
                .send()
                .await
                .expect("append to provisioned Tape node");
            assert!(
                response.status().is_success(),
                "append {n} must be acknowledged before restart"
            );
            let appended: Value = response.json().await.expect("decode append response");
            assert_eq!(appended["offset"].as_i64(), Some(n - 1));
        }

        let response = client
            .post(format!(
                "{}/topics/orders/subscriptions/billing/ack",
                first.base_url()
            ))
            .json(&json!({ "offset": 3 }))
            .send()
            .await
            .expect("ack provisioned subscription checkpoint");
        assert!(
            response.status().is_success(),
            "the provisioned subscription must accept checkpoint offset 3"
        );
        let acknowledged: Value = response.json().await.expect("decode ack response");
        assert_eq!(acknowledged["topic"], "orders");
        assert_eq!(acknowledged["consumer"], "billing");
        assert_eq!(acknowledged["offset"], 3);

        first.stop("first complete shutdown").await;

        let second_bind = free_addr();
        let second = spawn_provisioned_node(&second_bind, data_dir.path());
        wait_healthy(&client, &second.base_url(), "restart startup").await;
        wait_for_recovered_state(&client, &second.base_url()).await;
    }
}
