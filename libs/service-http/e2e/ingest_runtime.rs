use std::{io::Write, time::Duration};

use axum::http::HeaderMap;
use flate2::{write::GzEncoder, Compression};
use service_http::{
    decode_request_body, ContentDecodeErrorKind, ContentDecodeLimits, WeightedAdmission,
    WeightedAdmissionConfig, WeightedAdmissionError,
};

#[test]
fn gzip_decode_has_independent_compressed_and_decoded_limits() {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&vec![b'x'; 512]).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(compressed.len() < 64);

    let mut headers = HeaderMap::new();
    headers.insert("content-encoding", "gzip".parse().unwrap());
    let limits = ContentDecodeLimits::new(64, 256).unwrap();
    let error = decode_request_body(&headers, &compressed, limits).unwrap_err();
    assert_eq!(error.kind(), ContentDecodeErrorKind::DecodedBodyTooLarge);

    let error = decode_request_body(&headers, b"not gzip", limits).unwrap_err();
    assert_eq!(error.kind(), ContentDecodeErrorKind::InvalidGzip);
}

#[test]
fn weighted_quota_and_raii_concurrency_are_one_shared_flow() {
    let admission = WeightedAdmission::new(
        WeightedAdmissionConfig::new(1, 3, Duration::from_secs(60), 8).unwrap(),
    );

    let first = admission
        .acquire_at("project-a".to_string(), 2, false, Duration::ZERO)
        .unwrap();
    let error = admission
        .acquire_at("project-a".to_string(), 1, false, Duration::ZERO)
        .unwrap_err();
    assert!(matches!(
        error,
        WeightedAdmissionError::ConcurrencyExceeded { .. }
    ));

    drop(first);
    let second = admission
        .acquire_at("project-a".to_string(), 1, false, Duration::ZERO)
        .unwrap();
    drop(second);
    let error = admission
        .acquire_at("project-a".to_string(), 1, false, Duration::ZERO)
        .unwrap_err();
    assert!(matches!(
        error,
        WeightedAdmissionError::QuotaExceeded { .. }
    ));
    assert_eq!(error.retry_after(), Some(Duration::from_secs(60)));
}

#[test]
fn draining_and_key_bound_fail_before_a_lease_is_created() {
    let admission = WeightedAdmission::new(
        WeightedAdmissionConfig::new(2, 10, Duration::from_secs(60), 1).unwrap(),
    );
    let error = admission
        .acquire_at("project-a".to_string(), 1, true, Duration::ZERO)
        .unwrap_err();
    assert!(matches!(error, WeightedAdmissionError::Draining));

    let lease = admission
        .acquire_at("project-a".to_string(), 1, false, Duration::ZERO)
        .unwrap();
    let error = admission
        .acquire_at("project-b".to_string(), 1, false, Duration::ZERO)
        .unwrap_err();
    assert!(matches!(error, WeightedAdmissionError::KeyLimitExceeded));
    drop(lease);
}
