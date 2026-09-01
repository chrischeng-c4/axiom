use std::process::Command;

use prost::Message as _;
use prost14::Message as _;
use sift::{ingest::otlp::wire::ExportLogsServiceRequest, prometheus::remote::WriteRequest};

#[test]
fn emits_valid_otlp_logs_protobuf_with_unique_event_ids() {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "acceptance-payload",
            "--kind",
            "otlp-logs-protobuf",
            "--items",
            "3",
            "--project",
            "project-a",
            "--event-prefix",
            "fixture",
            "--timestamp-unix-nano",
            "1700000000000000000",
        ])
        .output()
        .expect("run acceptance payload generator");
    assert!(output.status.success(), "{output:?}");

    let request = ExportLogsServiceRequest::decode(output.stdout.as_slice())
        .expect("decode generated OTLP protobuf");
    let records = &request.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].time_unix_nano, 1_700_000_000_000_000_000);
    assert_eq!(records[2].time_unix_nano, 1_700_000_000_000_000_002);
    let event_ids = records
        .iter()
        .map(|record| {
            record
                .attributes
                .iter()
                .find(|attribute| attribute.key == "sift.event_id")
                .and_then(|attribute| attribute.value.as_ref())
                .and_then(|value| value.value.as_ref())
                .map(|value| match value {
                    sift::ingest::otlp::wire::any_value::Value::StringValue(value) => {
                        value.as_str()
                    }
                    _ => panic!("event id must be a string"),
                })
                .expect("event id attribute")
        })
        .collect::<Vec<_>>();
    assert_eq!(event_ids, ["fixture-0", "fixture-1", "fixture-2"]);
}

#[test]
fn emits_valid_prometheus_remote_write_v1_snappy_block() {
    let output = Command::new(env!("CARGO_BIN_EXE_sift"))
        .args([
            "acceptance-payload",
            "--kind",
            "prometheus-remote-write-v1",
            "--items",
            "3",
            "--project",
            "project-a",
            "--event-prefix",
            "fixture",
            "--timestamp-unix-nano",
            "1700000000000000000",
        ])
        .output()
        .expect("run acceptance payload generator");
    assert!(output.status.success(), "{output:?}");

    let protobuf = metrics_remote_write::decode_snappy(&output.stdout, 16 * 1024 * 1024)
        .expect("decode generated Snappy block");
    let validated = metrics_remote_write::decode_write_request(&protobuf)
        .expect("validate generated Prometheus Remote Write 1.0 request");
    assert_eq!(validated.sample_count(), 3);
    let request = WriteRequest::decode(protobuf.as_slice())
        .expect("decode generated Prometheus WriteRequest");
    assert_eq!(request.timeseries.len(), 1);
    assert_eq!(request.timeseries[0].samples.len(), 3);
    assert_eq!(
        request.timeseries[0].samples[0].timestamp,
        1_700_000_000_000
    );
    assert_eq!(
        request.timeseries[0].samples[2].timestamp,
        1_700_000_000_002
    );
    assert!(request.timeseries[0]
        .labels
        .iter()
        .any(|label| label.name == "project" && label.value == "project-a"));
}
