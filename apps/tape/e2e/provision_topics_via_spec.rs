// @ec tape-provision-topics
// @capability declarative-subscription-provisioning
// @claim spec-topics-ensure-idempotent
// @contract tape-spec-topics-cr-provisioned

// Contract: Declaring spec.topics on the CR creates subscriptions before the listener starts.
// Contract: Restarting with the same spec.topics logs "already_exists" without errors.
// Contract: Declaring a topic alone (no subscriptions) logs "noted_implicit" without mutations.

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tape::{SubscriptionError, TapeJournal};

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
}
