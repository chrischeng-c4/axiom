use metrics_remote_write::{
    decode_snappy, decode_write_request,
    proto::{Label, Sample, TimeSeries, WriteRequest},
    validate_headers, HeaderError, PROMETHEUS_STALE_NAN_BITS,
};
use prost::Message;

fn request(value: f64) -> WriteRequest {
    WriteRequest {
        timeseries: vec![TimeSeries {
            labels: vec![Label {
                name: "__name__".into(),
                value: "requests_total".into(),
            }],
            samples: vec![Sample {
                value,
                timestamp: 1,
            }],
            exemplars: Vec::new(),
        }],
        metadata: Vec::new(),
    }
}

#[test]
fn remote_write_one_headers_snappy_and_stale_marker_validate() {
    validate_headers("application/x-protobuf", "snappy", Some("0.1.0")).unwrap();
    let payload = snap::raw::Encoder::new()
        .compress_vec(&request(f64::from_bits(PROMETHEUS_STALE_NAN_BITS)).encode_to_vec())
        .unwrap();
    let protobuf = decode_snappy(&payload, 1024).unwrap();
    let decoded = decode_write_request(&protobuf).unwrap();
    assert_eq!(decoded.sample_count(), 1);
}

#[test]
fn remote_write_two_is_rejected_before_decode() {
    let error = validate_headers(
        "application/x-protobuf;proto=io.prometheus.write.v2.Request",
        "snappy",
        Some("2.0.0"),
    )
    .unwrap_err();
    assert!(matches!(error, HeaderError::RemoteWriteTwo));
}
