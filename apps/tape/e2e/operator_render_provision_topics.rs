// SPEC-MANAGED: apps/tape/external-contracts/claim-closure/production-claims.md#tape-operator-provision

// Operator render test: verify that spec.topics is threaded to TAPE_PROVISION_TOPICS env.

#[cfg(all(test, feature = "operator"))]
mod tests {
    use serde_json::Value;
    use tape::operator::render;

    #[test]
    fn operator_render_threads_provision_topics_env() {
        // Construct a Tape CR with spec.topics
        let tape_yaml = r#"
apiVersion: tape.dev/v1alpha1
kind: Tape
metadata:
  name: test-tape
  namespace: default
  uid: test-uid-123
spec:
  image: tape:latest
  replicasPerShard: 1
  voterCount: 1
  storage: 1Gi
  topics:
    - name: orders
      subscriptions:
        - billing
        - audit
    - name: telemetry
"#;

        let tape: tape::operator::Tape = serde_yaml::from_str(tape_yaml).expect("parse tape CR");
        let rendered = render::render(&tape);

        // Find the StatefulSet in the rendered objects
        let statefulset = rendered
            .iter()
            .find(|obj| obj.get("kind").and_then(|k| k.as_str()) == Some("StatefulSet"))
            .expect("statefulset should be rendered");

        // Navigate to the container env section
        let containers = statefulset
            .get("spec")
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|s| s.get("containers"))
            .and_then(|c| c.as_array())
            .expect("containers should be an array");

        let container = containers.iter().next().expect("at least one container");
        let env = container
            .get("env")
            .and_then(|e| e.as_array())
            .expect("env should be an array");

        // Find TAPE_PROVISION_TOPICS in the env
        let provision_topics_env = env
            .iter()
            .find(|e| e.get("name").and_then(|n| n.as_str()) == Some("TAPE_PROVISION_TOPICS"))
            .expect("TAPE_PROVISION_TOPICS should be in env");

        let env_value = provision_topics_env
            .get("value")
            .and_then(|v| v.as_str())
            .expect("env value should be a string");

        // Parse and verify the JSON
        let topics: Vec<Value> =
            serde_json::from_str(env_value).expect("TAPE_PROVISION_TOPICS should be valid JSON");
        assert_eq!(topics.len(), 2);

        // First topic: orders with billing and audit subscriptions
        let orders = &topics[0];
        assert_eq!(orders.get("name").and_then(|n| n.as_str()), Some("orders"));
        let subs = orders
            .get("subscriptions")
            .and_then(|s| s.as_array())
            .expect("subscriptions should be an array");
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].as_str(), Some("billing"));
        assert_eq!(subs[1].as_str(), Some("audit"));

        // Second topic: telemetry with no subscriptions
        let telemetry = &topics[1];
        assert_eq!(
            telemetry.get("name").and_then(|n| n.as_str()),
            Some("telemetry")
        );
        let telemetry_subs = telemetry
            .get("subscriptions")
            .and_then(|s| s.as_array())
            .expect("subscriptions should be an array");
        assert_eq!(telemetry_subs.len(), 0);
    }

    #[test]
    fn operator_render_omits_provision_topics_when_empty() {
        // Construct a Tape CR without spec.topics
        let tape_yaml = r#"
apiVersion: tape.dev/v1alpha1
kind: Tape
metadata:
  name: test-tape
  namespace: default
  uid: test-uid-123
spec:
  image: tape:latest
  replicasPerShard: 1
  voterCount: 1
  storage: 1Gi
"#;

        let tape: tape::operator::Tape = serde_yaml::from_str(tape_yaml).expect("parse tape CR");
        let rendered = render::render(&tape);

        // Find the StatefulSet in the rendered objects
        let statefulset = rendered
            .iter()
            .find(|obj| obj.get("kind").and_then(|k| k.as_str()) == Some("StatefulSet"))
            .expect("statefulset should be rendered");

        // Navigate to the container env section
        let containers = statefulset
            .get("spec")
            .and_then(|s| s.get("template"))
            .and_then(|t| t.get("spec"))
            .and_then(|s| s.get("containers"))
            .and_then(|c| c.as_array())
            .expect("containers should be an array");

        let container = containers.iter().next().expect("at least one container");
        let env = container
            .get("env")
            .and_then(|e| e.as_array())
            .expect("env should be an array");

        // TAPE_PROVISION_TOPICS should not be present when spec.topics is absent/empty
        let provision_topics_env = env
            .iter()
            .find(|e| e.get("name").and_then(|n| n.as_str()) == Some("TAPE_PROVISION_TOPICS"));
        assert!(
            provision_topics_env.is_none(),
            "TAPE_PROVISION_TOPICS should not be set when spec.topics is absent"
        );
    }
}
