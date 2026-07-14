// HANDWRITE-BEGIN gap="sift-otlp-wire-types" tracker="1658" reason="Define the bounded official OTLP protobuf wire subset shared by log, trace, metric, and profile normalization."
//! Receiver-side OTLP protobuf wire subset.
//!
//! Field numbers follow `opentelemetry-proto`. Prost ignores newer unknown
//! fields, so collectors can advance without forcing Sift to embed an exporter
//! SDK. Profiles are retained as an opaque request until the profile-store WI
//! adds the still-evolving sample/location model.

#[derive(Clone, PartialEq, prost::Message)]
pub struct AnyValue {
    #[prost(oneof = "any_value::Value", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub value: Option<any_value::Value>,
}

impl AnyValue {
    pub fn string(value: impl Into<String>) -> Self {
        Self {
            value: Some(any_value::Value::StringValue(value.into())),
        }
    }
}

pub mod any_value {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "1")]
        StringValue(String),
        #[prost(bool, tag = "2")]
        BoolValue(bool),
        #[prost(int64, tag = "3")]
        IntValue(i64),
        #[prost(double, tag = "4")]
        DoubleValue(f64),
        #[prost(message, tag = "5")]
        ArrayValue(super::ArrayValue),
        #[prost(message, tag = "6")]
        KvlistValue(super::KeyValueList),
        #[prost(bytes, tag = "7")]
        BytesValue(Vec<u8>),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ArrayValue {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<AnyValue>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct KeyValueList {
    #[prost(message, repeated, tag = "1")]
    pub values: Vec<KeyValue>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct KeyValue {
    #[prost(string, tag = "1")]
    pub key: String,
    #[prost(message, optional, tag = "2")]
    pub value: Option<AnyValue>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct InstrumentationScope {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub version: String,
    #[prost(message, repeated, tag = "3")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "4")]
    pub dropped_attributes_count: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Resource {
    #[prost(message, repeated, tag = "1")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "2")]
    pub dropped_attributes_count: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExportLogsServiceRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_logs: Vec<ResourceLogs>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ResourceLogs {
    #[prost(message, optional, tag = "1")]
    pub resource: Option<Resource>,
    #[prost(message, repeated, tag = "2")]
    pub scope_logs: Vec<ScopeLogs>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ScopeLogs {
    #[prost(message, optional, tag = "1")]
    pub scope: Option<InstrumentationScope>,
    #[prost(message, repeated, tag = "2")]
    pub log_records: Vec<LogRecord>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct LogRecord {
    #[prost(fixed64, tag = "1")]
    pub time_unix_nano: u64,
    #[prost(int32, tag = "2")]
    pub severity_number: i32,
    #[prost(string, tag = "3")]
    pub severity_text: String,
    #[prost(message, optional, tag = "5")]
    pub body: Option<AnyValue>,
    #[prost(message, repeated, tag = "6")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "7")]
    pub dropped_attributes_count: u32,
    #[prost(fixed32, tag = "8")]
    pub flags: u32,
    #[prost(bytes, tag = "9")]
    pub trace_id: Vec<u8>,
    #[prost(bytes, tag = "10")]
    pub span_id: Vec<u8>,
    #[prost(fixed64, tag = "11")]
    pub observed_time_unix_nano: u64,
    #[prost(string, tag = "12")]
    pub event_name: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExportTraceServiceRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_spans: Vec<ResourceSpans>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ResourceSpans {
    #[prost(message, optional, tag = "1")]
    pub resource: Option<Resource>,
    #[prost(message, repeated, tag = "2")]
    pub scope_spans: Vec<ScopeSpans>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ScopeSpans {
    #[prost(message, optional, tag = "1")]
    pub scope: Option<InstrumentationScope>,
    #[prost(message, repeated, tag = "2")]
    pub spans: Vec<Span>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Span {
    #[prost(bytes, tag = "1")]
    pub trace_id: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub span_id: Vec<u8>,
    #[prost(string, tag = "3")]
    pub trace_state: String,
    #[prost(bytes, tag = "4")]
    pub parent_span_id: Vec<u8>,
    #[prost(string, tag = "5")]
    pub name: String,
    #[prost(int32, tag = "6")]
    pub kind: i32,
    #[prost(fixed64, tag = "7")]
    pub start_time_unix_nano: u64,
    #[prost(fixed64, tag = "8")]
    pub end_time_unix_nano: u64,
    #[prost(message, repeated, tag = "9")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "10")]
    pub dropped_attributes_count: u32,
    #[prost(message, repeated, tag = "11")]
    pub events: Vec<SpanEvent>,
    #[prost(uint32, tag = "12")]
    pub dropped_events_count: u32,
    #[prost(message, repeated, tag = "13")]
    pub links: Vec<SpanLink>,
    #[prost(uint32, tag = "14")]
    pub dropped_links_count: u32,
    #[prost(message, optional, tag = "15")]
    pub status: Option<Status>,
    #[prost(fixed32, tag = "16")]
    pub flags: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SpanEvent {
    #[prost(fixed64, tag = "1")]
    pub time_unix_nano: u64,
    #[prost(string, tag = "2")]
    pub name: String,
    #[prost(message, repeated, tag = "3")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "4")]
    pub dropped_attributes_count: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct SpanLink {
    #[prost(bytes, tag = "1")]
    pub trace_id: Vec<u8>,
    #[prost(bytes, tag = "2")]
    pub span_id: Vec<u8>,
    #[prost(string, tag = "3")]
    pub trace_state: String,
    #[prost(message, repeated, tag = "4")]
    pub attributes: Vec<KeyValue>,
    #[prost(uint32, tag = "5")]
    pub dropped_attributes_count: u32,
    #[prost(fixed32, tag = "6")]
    pub flags: u32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Status {
    #[prost(string, tag = "2")]
    pub message: String,
    #[prost(int32, tag = "3")]
    pub code: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExportMetricsServiceRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_metrics: Vec<ResourceMetrics>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ResourceMetrics {
    #[prost(message, optional, tag = "1")]
    pub resource: Option<Resource>,
    #[prost(message, repeated, tag = "2")]
    pub scope_metrics: Vec<ScopeMetrics>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ScopeMetrics {
    #[prost(message, optional, tag = "1")]
    pub scope: Option<InstrumentationScope>,
    #[prost(message, repeated, tag = "2")]
    pub metrics: Vec<Metric>,
    #[prost(string, tag = "3")]
    pub schema_url: String,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Metric {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub description: String,
    #[prost(string, tag = "3")]
    pub unit: String,
    #[prost(oneof = "metric::Data", tags = "5, 7, 9")]
    pub data: Option<metric::Data>,
}

pub mod metric {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Data {
        #[prost(message, tag = "5")]
        Gauge(super::Gauge),
        #[prost(message, tag = "7")]
        Sum(super::Sum),
        #[prost(message, tag = "9")]
        Histogram(super::Histogram),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Gauge {
    #[prost(message, repeated, tag = "1")]
    pub data_points: Vec<NumberDataPoint>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Sum {
    #[prost(message, repeated, tag = "1")]
    pub data_points: Vec<NumberDataPoint>,
    #[prost(int32, tag = "2")]
    pub aggregation_temporality: i32,
    #[prost(bool, tag = "3")]
    pub is_monotonic: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Histogram {
    #[prost(message, repeated, tag = "1")]
    pub data_points: Vec<HistogramDataPoint>,
    #[prost(int32, tag = "2")]
    pub aggregation_temporality: i32,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct NumberDataPoint {
    #[prost(message, repeated, tag = "7")]
    pub attributes: Vec<KeyValue>,
    #[prost(fixed64, tag = "2")]
    pub start_time_unix_nano: u64,
    #[prost(fixed64, tag = "3")]
    pub time_unix_nano: u64,
    #[prost(message, repeated, tag = "5")]
    pub exemplars: Vec<Exemplar>,
    #[prost(uint32, tag = "8")]
    pub flags: u32,
    #[prost(oneof = "number_data_point::Value", tags = "4, 6")]
    pub value: Option<number_data_point::Value>,
}

pub mod number_data_point {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Value {
        #[prost(double, tag = "4")]
        AsDouble(f64),
        #[prost(sfixed64, tag = "6")]
        AsInt(i64),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct HistogramDataPoint {
    #[prost(message, repeated, tag = "9")]
    pub attributes: Vec<KeyValue>,
    #[prost(fixed64, tag = "2")]
    pub start_time_unix_nano: u64,
    #[prost(fixed64, tag = "3")]
    pub time_unix_nano: u64,
    #[prost(fixed64, tag = "4")]
    pub count: u64,
    #[prost(double, optional, tag = "5")]
    pub sum: Option<f64>,
    #[prost(fixed64, repeated, packed = "true", tag = "6")]
    pub bucket_counts: Vec<u64>,
    #[prost(double, repeated, packed = "true", tag = "7")]
    pub explicit_bounds: Vec<f64>,
    #[prost(message, repeated, tag = "8")]
    pub exemplars: Vec<Exemplar>,
    #[prost(uint32, tag = "10")]
    pub flags: u32,
    #[prost(double, optional, tag = "11")]
    pub min: Option<f64>,
    #[prost(double, optional, tag = "12")]
    pub max: Option<f64>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Exemplar {
    #[prost(message, repeated, tag = "7")]
    pub filtered_attributes: Vec<KeyValue>,
    #[prost(fixed64, tag = "2")]
    pub time_unix_nano: u64,
    #[prost(bytes, tag = "3")]
    pub span_id: Vec<u8>,
    #[prost(bytes, tag = "4")]
    pub trace_id: Vec<u8>,
    #[prost(oneof = "exemplar::Value", tags = "5, 6")]
    pub value: Option<exemplar::Value>,
}

pub mod exemplar {
    #[derive(Clone, PartialEq, prost::Oneof)]
    pub enum Value {
        #[prost(double, tag = "5")]
        AsDouble(f64),
        #[prost(sfixed64, tag = "6")]
        AsInt(i64),
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct ExportProfilesServiceRequest {
    #[prost(message, repeated, tag = "1")]
    pub resource_profiles: Vec<OpaqueResourceProfiles>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct OpaqueResourceProfiles {}

/// The three stable OTLP signals and profiles development signal all encode
/// partial success as message field 1 containing rejected count field 1 and
/// error message field 2. The field names differ only in JSON.
#[derive(Clone, PartialEq, prost::Message)]
pub struct ExportResponse {
    #[prost(message, optional, tag = "1")]
    pub partial_success: Option<PartialSuccess>,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct PartialSuccess {
    #[prost(int64, tag = "1")]
    pub rejected_items: i64,
    #[prost(string, tag = "2")]
    pub error_message: String,
}
// HANDWRITE-END
