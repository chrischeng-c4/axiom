use std::io::Write;

use flate2::{write::GzEncoder, Compression};
use opentelemetry_proto::tonic::{
    collector::logs::v1::{ExportLogsServiceRequest, ExportLogsServiceResponse},
    logs::v1::ResourceLogs,
};
use prost::Message;
use transport_otlp::{
    decode_content_encoding, decode_payload, encode_response, DecodedPayload, OtlpMediaType,
    OtlpSignal, PartialSuccess,
};

#[test]
fn official_protobuf_and_partial_success_stay_signal_specific() {
    let request = ExportLogsServiceRequest {
        resource_logs: vec![ResourceLogs::default()],
    };
    let decoded = decode_payload(
        OtlpSignal::Logs,
        OtlpMediaType::Protobuf,
        &request.encode_to_vec(),
    )
    .unwrap();
    assert!(matches!(decoded, DecodedPayload::Logs(_)));

    let encoded = encode_response(
        OtlpSignal::Logs,
        OtlpMediaType::Protobuf,
        &PartialSuccess::new(2, "two invalid logs"),
    )
    .unwrap();
    let response = ExportLogsServiceResponse::decode(encoded.body.as_slice()).unwrap();
    let partial = response.partial_success.unwrap();
    assert_eq!(partial.rejected_log_records, 2);
    assert_eq!(partial.error_message, "two invalid logs");
}

#[test]
fn bounded_gzip_decode_rejects_expansion_over_the_limit() {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&vec![b'x'; 4096]).unwrap();
    let compressed = encoder.finish().unwrap();

    assert!(decode_content_encoding(Some("gzip"), &compressed, 1024).is_err());
    assert_eq!(
        decode_content_encoding(Some("gzip"), &compressed, 4096)
            .unwrap()
            .len(),
        4096
    );
}
