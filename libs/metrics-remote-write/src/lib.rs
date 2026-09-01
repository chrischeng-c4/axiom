//! Prometheus Remote Write 1.0 transport and validation.
//!
//! This crate owns the wire contract. Products own the conversion from a
//! validated write request to their domain records.

use std::collections::BTreeSet;

use prost::Message;
use thiserror::Error;

/// Prometheus' reserved stale marker. Other NaN payloads are invalid.
pub const PROMETHEUS_STALE_NAN_BITS: u64 = 0x7ff0_0000_0000_0002;

pub mod proto {
    #[derive(Clone, PartialEq, prost::Message)]
    pub struct WriteRequest {
        #[prost(message, repeated, tag = "1")]
        pub timeseries: Vec<TimeSeries>,
        #[prost(message, repeated, tag = "3")]
        pub metadata: Vec<MetricMetadata>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct TimeSeries {
        #[prost(message, repeated, tag = "1")]
        pub labels: Vec<Label>,
        #[prost(message, repeated, tag = "2")]
        pub samples: Vec<Sample>,
        #[prost(message, repeated, tag = "3")]
        pub exemplars: Vec<Exemplar>,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Label {
        #[prost(string, tag = "1")]
        pub name: String,
        #[prost(string, tag = "2")]
        pub value: String,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Sample {
        #[prost(double, tag = "1")]
        pub value: f64,
        #[prost(int64, tag = "2")]
        pub timestamp: i64,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct Exemplar {
        #[prost(message, repeated, tag = "1")]
        pub labels: Vec<Label>,
        #[prost(double, tag = "2")]
        pub value: f64,
        #[prost(int64, tag = "3")]
        pub timestamp: i64,
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MetricMetadata {
        #[prost(int32, tag = "1")]
        pub r#type: i32,
        #[prost(string, tag = "2")]
        pub metric_family_name: String,
        #[prost(string, tag = "4")]
        pub help: String,
        #[prost(string, tag = "5")]
        pub unit: String,
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum HeaderError {
    #[error("Prometheus Remote Write 2.0 is not supported")]
    RemoteWriteTwo,
    #[error("Prometheus Remote Write 1.0 requires application/x-protobuf")]
    UnsupportedMediaType,
    #[error("Prometheus Remote Write 1.0 requires snappy block compression")]
    UnsupportedEncoding,
    #[error("unsupported Prometheus Remote Write version `{0}`")]
    UnsupportedVersion(String),
}

/// Validate all transport headers before any payload is decoded or written.
pub fn validate_headers(
    content_type: &str,
    content_encoding: &str,
    version: Option<&str>,
) -> Result<(), HeaderError> {
    let media = content_type.trim().to_ascii_lowercase();
    let version = version.map(str::trim).filter(|value| !value.is_empty());
    if media.contains("io.prometheus.write.v2.request")
        || version.is_some_and(|value| value.starts_with('2'))
    {
        return Err(HeaderError::RemoteWriteTwo);
    }
    if media.split(';').next().map(str::trim) != Some("application/x-protobuf") {
        return Err(HeaderError::UnsupportedMediaType);
    }
    if !content_encoding.trim().eq_ignore_ascii_case("snappy") {
        return Err(HeaderError::UnsupportedEncoding);
    }
    if let Some(version) = version {
        if version != "0.1.0" && version != "1.0" && version != "1.0.0" {
            return Err(HeaderError::UnsupportedVersion(version.to_owned()));
        }
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("decoded remote write body is {actual} bytes; limit is {limit} bytes")]
    BodyTooLarge { actual: usize, limit: usize },
    #[error("invalid Snappy block: {0}")]
    InvalidSnappy(#[from] snap::Error),
    #[error("invalid Prometheus WriteRequest protobuf: {0}")]
    InvalidProtobuf(#[from] prost::DecodeError),
    #[error("remote write request contains no time series")]
    EmptyRequest,
    #[error("remote write series contains no labels")]
    EmptyLabels,
    #[error("remote write label names must not be empty")]
    EmptyLabelName,
    #[error("remote write labels must be sorted by name")]
    UnsortedLabels,
    #[error("remote write labels must be unique")]
    DuplicateLabels,
    #[error("remote write series contains no samples")]
    EmptySamples,
    #[error("remote write sample values must be finite or the Prometheus stale marker")]
    InvalidSampleValue,
    #[error("remote write sample timestamps must be strictly increasing")]
    NonIncreasingTimestamps,
    #[error("remote write exemplar value must be finite")]
    InvalidExemplarValue,
}

/// Decompress one Snappy block with a decoded-size limit.
pub fn decode_snappy(body: &[u8], max_decoded_bytes: usize) -> Result<Vec<u8>, DecodeError> {
    let decoded_len = snap::raw::decompress_len(body)?;
    if decoded_len > max_decoded_bytes {
        return Err(DecodeError::BodyTooLarge {
            actual: decoded_len,
            limit: max_decoded_bytes,
        });
    }
    let mut decoded = vec![0; decoded_len];
    let written = snap::raw::Decoder::new().decompress(body, &mut decoded)?;
    decoded.truncate(written);
    Ok(decoded)
}

/// Encode one Remote Write Snappy block.
pub fn encode_snappy(body: &[u8]) -> Result<Vec<u8>, snap::Error> {
    snap::raw::Encoder::new().compress_vec(body)
}

#[derive(Clone, Debug)]
pub struct ValidatedWrite {
    request: proto::WriteRequest,
    sample_count: usize,
}

impl ValidatedWrite {
    pub fn request(&self) -> &proto::WriteRequest {
        &self.request
    }

    pub fn into_inner(self) -> proto::WriteRequest {
        self.request
    }

    pub fn sample_count(&self) -> usize {
        self.sample_count
    }
}

/// Decode and validate the product-neutral Remote Write 1.0 contract.
pub fn decode_write_request(body: &[u8]) -> Result<ValidatedWrite, DecodeError> {
    let request = proto::WriteRequest::decode(body)?;
    if request.timeseries.is_empty() {
        return Err(DecodeError::EmptyRequest);
    }

    let mut sample_count = 0;
    for series in &request.timeseries {
        validate_labels(&series.labels)?;
        if series.samples.is_empty() {
            return Err(DecodeError::EmptySamples);
        }
        let mut previous_timestamp = None;
        for sample in &series.samples {
            let stale = sample.value.to_bits() == PROMETHEUS_STALE_NAN_BITS;
            if !sample.value.is_finite() && !stale {
                return Err(DecodeError::InvalidSampleValue);
            }
            if previous_timestamp.is_some_and(|previous| sample.timestamp <= previous) {
                return Err(DecodeError::NonIncreasingTimestamps);
            }
            previous_timestamp = Some(sample.timestamp);
            sample_count += 1;
        }
        for exemplar in &series.exemplars {
            if !exemplar.labels.is_empty() {
                validate_labels(&exemplar.labels)?;
            }
            if !exemplar.value.is_finite() {
                return Err(DecodeError::InvalidExemplarValue);
            }
        }
    }

    Ok(ValidatedWrite {
        request,
        sample_count,
    })
}

fn validate_labels(labels: &[proto::Label]) -> Result<(), DecodeError> {
    if labels.is_empty() {
        return Err(DecodeError::EmptyLabels);
    }
    let mut previous: Option<&str> = None;
    let mut seen = BTreeSet::new();
    for label in labels {
        if label.name.is_empty() {
            return Err(DecodeError::EmptyLabelName);
        }
        if previous.is_some_and(|previous| previous > label.name.as_str()) {
            return Err(DecodeError::UnsortedLabels);
        }
        if !seen.insert(label.name.as_str()) {
            return Err(DecodeError::DuplicateLabels);
        }
        previous = Some(&label.name);
    }
    Ok(())
}

/// A product hook that converts a validated request to domain data.
pub trait RemoteWriteConsumer {
    type Output;
    type Error;

    fn consume(&self, write: ValidatedWrite) -> Result<Self::Output, Self::Error>;
}

pub fn consume_write<C: RemoteWriteConsumer>(
    body: &[u8],
    consumer: &C,
) -> Result<C::Output, ConsumeError<C::Error>> {
    let write = decode_write_request(body).map_err(ConsumeError::Decode)?;
    consumer.consume(write).map_err(ConsumeError::Consumer)
}

#[derive(Debug, Error)]
pub enum ConsumeError<E> {
    #[error(transparent)]
    Decode(DecodeError),
    #[error("remote write consumer rejected the request")]
    Consumer(E),
}
