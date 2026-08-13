// HANDWRITE-BEGIN gap="missing-generator:unit-test:91f7cd40" tracker="1868" reason="Capture real tracing output and verify per-line framing, stable schema, inherited correlation, validation, redaction, bounds, static-schema drift, and exporter independence."
use std::io;
use std::sync::{Arc, Mutex};

use service_observability::{
    collector_compatible, service_log_schema_v1, LogFormat, ServiceIdentity, ServiceJsonFormatter,
    ServiceLogEventV1, ServiceLogIdentityV1, MAX_ATTRIBUTES, MAX_ATTRIBUTE_VALUE_BYTES,
    SERVICE_LOG_SCHEMA_V1,
};
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

#[derive(Clone, Default)]
struct SharedWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

struct SharedWriterGuard {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl io::Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'writer> MakeWriter<'writer> for SharedWriter {
    type Writer = SharedWriterGuard;

    fn make_writer(&'writer self) -> Self::Writer {
        SharedWriterGuard {
            bytes: Arc::clone(&self.bytes),
        }
    }
}

impl SharedWriter {
    fn output(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

fn capture(events: impl FnOnce()) -> (String, Vec<ServiceLogEventV1>) {
    let writer = SharedWriter::default();
    let identity = ServiceIdentity::new("fixture-service", "1.2.3").unwrap();
    let layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .fmt_fields(JsonFields::new())
        .event_format(ServiceJsonFormatter::new(identity))
        .with_writer(writer.clone());
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::with_default(subscriber, events);

    let output = writer.output();
    let records = output
        .lines()
        .map(|line| serde_json::from_str::<ServiceLogEventV1>(line).unwrap())
        .collect();
    (output, records)
}

#[test]
fn jsonl_lines_parse_independently_with_identity() {
    let (output, records) = capture(|| {
        tracing::info!(event = "startup", component = "fixture", "ready");
        tracing::warn!(event = "retry", retries = 2_u64, "retrying");
    });

    assert_eq!(output.lines().count(), 2);
    assert_eq!(records.len(), 2);
    for record in &records {
        assert_eq!(record.schema, SERVICE_LOG_SCHEMA_V1);
        assert_eq!(record.service.name, "fixture-service");
        assert_eq!(record.service.version, "1.2.3");
        assert!(record.timestamp.contains('T'));
        assert!(record.timestamp.ends_with('Z'));
        assert!(!record.event.is_empty());
        assert!(!record.message.is_empty());
        assert!(record.attributes.len() <= MAX_ATTRIBUTES);
    }
    assert_eq!(records[0].event, "startup");
    assert!(records[0].message.contains("ready"));
    assert_eq!(
        records[0].attributes.get("component"),
        Some(&serde_json::Value::String("fixture".to_string()))
    );
    assert_eq!(records[1].severity, "WARN");
    assert_eq!(
        records[1].attributes.get("retries"),
        Some(&serde_json::Value::Number(2_u64.into()))
    );
}

#[test]
fn active_span_fields_become_valid_correlation() {
    const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
    const SPAN_ID: &str = "b7ad6b7169203331";
    const PARENT_SPAN_ID: &str = "00f067aa0ba902b7";

    let (_output, records) = capture(|| {
        let span = tracing::info_span!(
            "request",
            trace_id = TRACE_ID,
            span_id = SPAN_ID,
            parent_span_id = PARENT_SPAN_ID,
            trace_flags = "01",
            request_id = "request-42"
        );
        let _entered = span.enter();
        tracing::info!(
            event = "indexed",
            trace_id = "INVALID",
            documents = 3_u64,
            "indexed documents"
        );
    });

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.trace_id.as_deref(), Some(TRACE_ID));
    assert_eq!(record.span_id.as_deref(), Some(SPAN_ID));
    assert_eq!(record.parent_span_id.as_deref(), Some(PARENT_SPAN_ID));
    assert_eq!(record.trace_flags.as_deref(), Some("01"));
    assert_eq!(record.request_id.as_deref(), Some("request-42"));
    assert!(!record.attributes.contains_key("trace_id"));
    assert!(!record.attributes.contains_key("request_id"));
}

#[test]
fn sensitive_and_oversized_attributes_are_safe() {
    let oversized = "x".repeat(MAX_ATTRIBUTE_VALUE_BYTES + 300);
    let (output, records) = capture(|| {
        tracing::info!(
            event = "bounded",
            trace_id = "00000000000000000000000000000000",
            span_id = "ABCDEFABCDEFABCD",
            request_id = "bad\nid",
            authorization = "Bearer super-secret",
            proxy_authorization = "Basic hidden",
            cookie = "session=secret",
            set_cookie = "session=secret",
            baggage = "tenant=secret",
            tracestate = "vendor=secret",
            a00_oversized = %oversized,
            a01 = 1_u64,
            a02 = 2_u64,
            a03 = 3_u64,
            a04 = 4_u64,
            a05 = 5_u64,
            a06 = 6_u64,
            a07 = 7_u64,
            a08 = 8_u64,
            a09 = 9_u64,
            a10 = 10_u64,
            a11 = 11_u64,
            a12 = 12_u64,
            a13 = 13_u64,
            a14 = 14_u64,
            a15 = 15_u64,
            a16 = 16_u64,
            a17 = 17_u64,
            a18 = 18_u64,
            a19 = 19_u64,
            a20 = 20_u64,
            a21 = 21_u64,
            a22 = 22_u64,
            a23 = 23_u64,
            a24 = 24_u64,
            a25 = 25_u64,
            a26 = 26_u64,
            a27 = 27_u64,
            a28 = 28_u64,
            a29 = 29_u64,
            a30 = 30_u64,
            a31 = 31_u64,
            a32 = 32_u64,
            a33 = 33_u64,
            a34 = 34_u64,
            a35 = 35_u64,
            a36 = 36_u64,
            a37 = 37_u64,
            a38 = 38_u64,
            a39 = 39_u64,
            a40 = 40_u64,
            a41 = 41_u64,
            a42 = 42_u64,
            a43 = 43_u64,
            a44 = 44_u64,
            a45 = 45_u64,
            a46 = 46_u64,
            a47 = 47_u64,
            a48 = 48_u64,
            a49 = 49_u64,
            a50 = 50_u64,
            a51 = 51_u64,
            a52 = 52_u64,
            a53 = 53_u64,
            a54 = 54_u64,
            a55 = 55_u64,
            a56 = 56_u64,
            a57 = 57_u64,
            a58 = 58_u64,
            a59 = 59_u64,
            a60 = 60_u64,
            a61 = 61_u64,
            a62 = 62_u64,
            a63 = 63_u64,
            a64 = 64_u64,
            a65 = 65_u64,
            "bounded event"
        );
    });

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.attributes.len(), MAX_ATTRIBUTES);
    assert_eq!(record.trace_id, None);
    assert_eq!(record.span_id, None);
    assert_eq!(record.request_id, None);
    for sensitive in [
        "authorization",
        "proxy_authorization",
        "cookie",
        "set_cookie",
        "baggage",
        "tracestate",
    ] {
        assert!(!record.attributes.contains_key(sensitive));
    }
    let bounded = record.attributes["a00_oversized"].as_str().unwrap();
    assert_eq!(bounded.len(), MAX_ATTRIBUTE_VALUE_BYTES);
    assert!(!output.contains("super-secret"));
    assert_eq!(output.lines().count(), 1);
    assert!(serde_json::from_str::<serde_json::Value>(output.trim()).is_ok());
}

#[test]
fn schema_contract_matches_rust_event() {
    let schema = service_log_schema_v1();
    assert_eq!(
        schema["properties"]["schema"]["const"],
        SERVICE_LOG_SCHEMA_V1
    );
    assert_eq!(
        schema["properties"]["attributes"]["maxProperties"],
        MAX_ATTRIBUTES
    );
    assert_eq!(schema["additionalProperties"], false);

    let sample = ServiceLogEventV1 {
        schema: SERVICE_LOG_SCHEMA_V1.to_string(),
        timestamp: "2026-07-17T00:00:00.000000Z".to_string(),
        severity: "INFO".to_string(),
        service: ServiceLogIdentityV1 {
            name: "fixture-service".to_string(),
            version: "1.2.3".to_string(),
        },
        event: "sample".to_string(),
        message: "sample event".to_string(),
        trace_id: Some("0af7651916cd43dd8448eb211c80319c".to_string()),
        span_id: Some("b7ad6b7169203331".to_string()),
        parent_span_id: Some("00f067aa0ba902b7".to_string()),
        trace_flags: Some("01".to_string()),
        request_id: Some("request-42".to_string()),
        attributes: Default::default(),
    };
    let event_keys = serde_json::to_value(sample)
        .unwrap()
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let schema_keys = schema["properties"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(event_keys, schema_keys);
    assert!(collector_compatible(LogFormat::Json));
    assert!(!collector_compatible(LogFormat::Pretty));
}

// HANDWRITE-END
