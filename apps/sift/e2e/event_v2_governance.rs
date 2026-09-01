// HANDWRITE-BEGIN gap="sift-v2-governance-tests" tracker="1657" reason="Golden-test phase-one typed round trips, legacy refusal, project policy isolation, and absence of sensitive WAL bytes."
use std::{collections::BTreeMap, fs};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use serde_json::json;
use sift::{
    decode_event_json, AttributeValue, DurableJournal, EventEnvelope, EventQuery, GovernancePolicy,
    GovernancePolicySet, InstrumentationScope, MetricPoint, MetricTemporality, SignalKind,
    EVENT_SCHEMA_URL, EVENT_SCHEMA_VERSION,
};

fn v2_event(project: &str, id: &str, signal: SignalKind) -> EventEnvelope {
    let mut event =
        EventEnvelope::for_project(project, "prod", id, signal, json!({"message": "accepted"}));
    event
        .resource
        .insert("service.name".to_string(), "checkout".to_string());
    event.instrumentation_scope = Some(InstrumentationScope {
        name: "checkout-sdk".to_string(),
        version: Some("1.2.3".to_string()),
        attributes: BTreeMap::from([("scope.enabled".to_string(), AttributeValue::Bool(true))]),
        schema_url: Some("https://opentelemetry.io/schemas/1.30.0".to_string()),
    });
    event.trace_id = Some("0af7651916cd43dd8448eb211c80319c".to_string());
    event.span_id = Some("b7ad6b7169203331".to_string());
    event.request_id = Some("request-1".to_string());
    event.session_id = Some("session-1".to_string());
    event
}

#[test]
fn operational_event_v2_round_trips_phase_one_signals_and_typed_attributes() {
    for signal in SignalKind::ALL {
        let mut event = v2_event("project-a", &format!("event-{signal}"), signal);
        event.attributes = BTreeMap::from([
            (
                "string".to_string(),
                AttributeValue::String("value".to_string()),
            ),
            ("bool".to_string(), AttributeValue::Bool(true)),
            ("int".to_string(), AttributeValue::Int(42)),
            ("double".to_string(), AttributeValue::Double(4.25)),
            (
                "bytes".to_string(),
                AttributeValue::Bytes("AQID".to_string()),
            ),
            (
                "array".to_string(),
                AttributeValue::Array(vec![AttributeValue::Int(1), AttributeValue::Bool(false)]),
            ),
            (
                "map".to_string(),
                AttributeValue::Map(BTreeMap::from([(
                    "nested".to_string(),
                    AttributeValue::String("yes".to_string()),
                )])),
            ),
        ]);
        if signal == SignalKind::Metric {
            event.metric = Some(MetricPoint {
                name: "request.duration".to_string(),
                value: 12.5,
                stale: false,
                unit: Some("ms".to_string()),
                temporality: MetricTemporality::Delta,
                exemplars: Vec::new(),
            });
        }

        event.validate().unwrap();
        let encoded = serde_json::to_vec(&event).unwrap();
        let decoded = decode_event_json(&encoded).unwrap();
        assert_eq!(decoded, event);
        assert_eq!(decoded.schema_version, EVENT_SCHEMA_VERSION);
        assert_eq!(decoded.schema_url, EVENT_SCHEMA_URL);
    }
}

#[test]
fn v1_events_and_legacy_roots_are_refused_without_overwrite() {
    let v1 = json!({
        "schema_version": 1,
        "event_id": "legacy-1",
        "occurred_at": "2026-07-14T00:00:00Z",
        "signal": "log",
        "resource": {"service.name": "legacy-api"},
        "payload": {"message": "legacy"}
    });
    let error = decode_event_json(&serde_json::to_vec(&v1).unwrap())
        .unwrap_err()
        .to_string();
    assert!(error.contains("unsupported schema_version 1"), "{error}");

    let temp = tempfile::tempdir().unwrap();
    let legacy = temp.path().join("raw-events.framed");
    fs::write(&legacy, b"preserve-me").unwrap();
    let error = DurableJournal::open(temp.path())
        .err()
        .expect("legacy root must be refused")
        .to_string();
    assert!(error.contains("legacy Sift 0.1.1 data"), "{error}");
    assert_eq!(fs::read(legacy).unwrap(), b"preserve-me");
}

#[test]
fn governance_redacts_and_truncates_before_raw_bytes_and_is_project_scoped() {
    let default = GovernancePolicy {
        capture_genai_content: false,
        max_string_bytes: 8,
        allowed_attribute_keys: None,
        denied_attribute_keys: ["secret".to_string()].into_iter().collect(),
        redaction_text: "[X]".to_string(),
    };
    let capture = GovernancePolicy {
        capture_genai_content: true,
        ..default.clone()
    };
    let allowlist = GovernancePolicy {
        allowed_attribute_keys: Some(["kept".to_string()].into_iter().collect()),
        ..default.clone()
    };
    let policies = GovernancePolicySet {
        default,
        projects: BTreeMap::from([
            ("capture-project".to_string(), capture),
            ("allowlist-project".to_string(), allowlist),
        ]),
    };
    let temp = tempfile::tempdir().unwrap();
    let journal = DurableJournal::open_with_governance(temp.path(), policies).unwrap();

    let mut redacted = v2_event("private-project", "private-1", SignalKind::Span);
    redacted.attributes.insert(
        "gen_ai.operation.name".to_string(),
        AttributeValue::String("chat".to_string()),
    );
    redacted.attributes.insert(
        "gen_ai.prompt".to_string(),
        AttributeValue::String("top-secret-prompt".to_string()),
    );
    redacted.attributes.insert(
        "secret".to_string(),
        AttributeValue::String("attribute-secret".to_string()),
    );
    redacted.attributes.insert(
        "long".to_string(),
        AttributeValue::String("1234567890".to_string()),
    );
    let encoded_prompt = BASE64.encode(vec![0x53; 80_000]);
    redacted.payload = json!({
        "prompt": "payload-secret-prompt",
        "promptBytesBase64": encoded_prompt,
        "response": "payload-secret-response",
        "safe": "visible",
    });
    journal.append(redacted).unwrap();

    let mut captured = v2_event("capture-project", "capture-1", SignalKind::Span);
    captured.attributes.insert(
        "gen_ai.operation.name".to_string(),
        AttributeValue::String("chat".to_string()),
    );
    captured.payload = json!({"prompt": "kept"});
    journal.append(captured).unwrap();

    let mut allowlisted = v2_event("allowlist-project", "allowlist-1", SignalKind::Log);
    allowlisted
        .attributes
        .insert("kept".to_string(), AttributeValue::String("visible".into()));
    allowlisted.attributes.insert(
        "dropped".to_string(),
        AttributeValue::String("must-redact".into()),
    );
    journal.append(allowlisted).unwrap();

    let mut raw = fs::read(temp.path().join("wal/traces/events.framed")).unwrap();
    raw.extend(fs::read(temp.path().join("wal/logs/events.framed")).unwrap());
    let raw_text = String::from_utf8_lossy(&raw);
    for forbidden in [
        "top-secret-prompt",
        "attribute-secret",
        "payload-secret-prompt",
        "payload-secret-response",
        "1234567890",
        "must-redact",
    ] {
        assert!(
            !raw_text.contains(forbidden),
            "raw journal leaked {forbidden}"
        );
    }

    let rows = journal
        .query(EventQuery {
            limit: 10,
            ..EventQuery::default()
        })
        .unwrap();
    assert_eq!(
        rows[0].event.attributes["secret"],
        AttributeValue::String("[X]".into())
    );
    assert_eq!(
        rows[0].event.attributes["long"],
        AttributeValue::String("12345678".into())
    );
    assert_eq!(rows[0].event.payload["prompt"], "[X]");
    assert_eq!(rows[0].event.payload["promptBytesBase64"], "[X]");
    assert!(rows[0].event.blob_refs.is_empty());
    assert_eq!(rows[0].event.payload["response"], "[X]");
    assert_eq!(rows[1].event.payload["prompt"], "kept");
    assert_eq!(
        rows[2].event.attributes["kept"],
        AttributeValue::String("visible".into())
    );
    assert_eq!(
        rows[2].event.attributes["dropped"],
        AttributeValue::String("[X]".into())
    );
}

// HANDWRITE-END
