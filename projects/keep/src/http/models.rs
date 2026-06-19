//! Request / response DTOs for the HTTP surface, plus the JSON <-> engine
//! value mapping.
//!
//! Structured values travel as native JSON (`application/json`); opaque blobs
//! (claim-check payloads) travel as raw bytes (`application/octet-stream`) and
//! never round-trip through JSON. See [`json_to_kv`] / [`kv_to_json`] for the
//! structured mapping.

use std::collections::HashMap;
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::ToSchema;

use crate::types::KvValue;

// ---------------------------------------------------------------------------
// Zero-intermediate JSON value
// ---------------------------------------------------------------------------

/// A value deserialized DIRECTLY from JSON tokens into [`KvValue`], with no
/// `serde_json::Value` tree built in between — the measured hot cost on the
/// write path. The mapping matches [`json_to_kv`]. Used by internal fast-parse
/// request structs (not exposed in the OpenAPI schema; the public DTOs keep
/// `serde_json::Value` for documentation).
#[derive(Debug, Clone)]
pub struct JsonKv(pub KvValue);

impl<'de> Deserialize<'de> for JsonKv {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KvVisitor;
        impl<'de> Visitor<'de> for KvVisitor {
            type Value = KvValue;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a JSON value")
            }
            fn visit_bool<E: de::Error>(self, v: bool) -> Result<KvValue, E> {
                Ok(KvValue::Int(i64::from(v)))
            }
            fn visit_i64<E: de::Error>(self, v: i64) -> Result<KvValue, E> {
                Ok(KvValue::Int(v))
            }
            fn visit_u64<E: de::Error>(self, v: u64) -> Result<KvValue, E> {
                Ok(KvValue::Int(v as i64))
            }
            fn visit_f64<E: de::Error>(self, v: f64) -> Result<KvValue, E> {
                Ok(KvValue::Float(v))
            }
            fn visit_str<E: de::Error>(self, v: &str) -> Result<KvValue, E> {
                Ok(KvValue::String(v.to_owned()))
            }
            fn visit_string<E: de::Error>(self, v: String) -> Result<KvValue, E> {
                Ok(KvValue::String(v))
            }
            fn visit_none<E: de::Error>(self) -> Result<KvValue, E> {
                Ok(KvValue::Null)
            }
            fn visit_unit<E: de::Error>(self) -> Result<KvValue, E> {
                Ok(KvValue::Null)
            }
            fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<KvValue, D::Error> {
                Ok(JsonKv::deserialize(d)?.0)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<KvValue, A::Error> {
                let mut list = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(JsonKv(v)) = seq.next_element()? {
                    list.push(v);
                }
                Ok(KvValue::List(list))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<KvValue, A::Error> {
                let mut m = HashMap::with_capacity(map.size_hint().unwrap_or(0));
                while let Some((k, JsonKv(v))) = map.next_entry::<String, JsonKv>()? {
                    m.insert(k, v);
                }
                Ok(KvValue::Map(m))
            }
        }
        deserializer.deserialize_any(KvVisitor).map(JsonKv)
    }
}

/// Internal fast-parse form of [`SetRequest`] — parses straight into `KvValue`.
#[derive(Debug, Deserialize)]
pub struct SetRequestFast {
    pub value: JsonKv,
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}

/// Internal fast-parse form of [`MSetRequest`] — values parse straight into
/// `KvValue` (no serde_json::Value per entry).
#[derive(Debug, Deserialize)]
pub struct MSetRequestFast {
    pub entries: HashMap<String, JsonKv>,
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}

// ---------------------------------------------------------------------------
// Single-key string / scalar
// ---------------------------------------------------------------------------

/// Body of an unconditional `PUT /v1/kv/{key}` with `application/json`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct SetRequest {
    /// Value to store, as native JSON.
    pub value: serde_json::Value,
    /// Optional TTL in milliseconds. Omit for no expiry.
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}

/// A stored value returned to the caller.
#[derive(Debug, Serialize, ToSchema)]
pub struct ValueResponse {
    pub key: String,
    pub value: serde_json::Value,
}

/// Result of a successful set.
#[derive(Debug, Serialize, ToSchema)]
pub struct OkResponse {
    pub key: String,
    pub ok: bool,
}

/// Body of `POST /v1/kv/{key}/cas`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CasRequest {
    /// Value the key is expected to currently hold.
    pub expected: serde_json::Value,
    /// Value to write iff the current value matches `expected`.
    pub new: serde_json::Value,
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CasResponse {
    /// True if the swap was applied.
    pub swapped: bool,
}

/// Body of `POST /v1/kv/{key}/incr`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct IncrRequest {
    /// Amount to add; negative to decrement. Defaults to 1.
    #[serde(default = "default_delta")]
    pub delta: i64,
}
fn default_delta() -> i64 {
    1
}

#[derive(Debug, Serialize, ToSchema)]
pub struct IncrResponse {
    pub value: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteResponse {
    pub deleted: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SetNxResponse {
    /// True if the key was set (was absent), false if it already existed.
    pub set: bool,
}

// ---------------------------------------------------------------------------
// Batch
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct MGetRequest {
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MGetResponse {
    /// Parallel to the request `keys`; `null` where the key is absent.
    pub values: Vec<Option<serde_json::Value>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MSetRequest {
    pub entries: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub ttl_ms: Option<u64>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MDelRequest {
    pub keys: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CountResponse {
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Scan
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct ScanQuery {
    /// Only return keys starting with this prefix.
    pub prefix: Option<String>,
    /// Maximum number of keys to return. Defaults to 100.
    #[serde(default = "default_scan_limit")]
    pub limit: usize,
}
fn default_scan_limit() -> usize {
    100
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ScanResponse {
    pub keys: Vec<String>,
}

// ---------------------------------------------------------------------------
// Distributed locks
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct LockRequest {
    /// Identity of the lease holder.
    pub owner: String,
    /// Lease TTL in milliseconds.
    pub ttl_ms: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LockResponse {
    pub acquired: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UnlockRequest {
    pub owner: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnlockResponse {
    pub released: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExtendLockRequest {
    pub owner: String,
    pub ttl_ms: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExtendLockResponse {
    pub extended: bool,
}

// ---------------------------------------------------------------------------
// Lists
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, ToSchema)]
pub struct PushRequest {
    pub values: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PushResponse {
    /// List length after the push.
    pub length: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PopResponse {
    /// Popped value, or `null` if the list was empty / absent.
    pub value: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, ToSchema)]
pub struct InfoResponse {
    pub keys: usize,
    pub shards: usize,
    pub memory_bytes: usize,
    pub version: String,
}

// ---------------------------------------------------------------------------
// JSON <-> engine value mapping
// ---------------------------------------------------------------------------

/// Map native JSON into the engine value type.
///
/// - numbers: integral -> `Int`, fractional -> `Float`
/// - bool -> `Int` (`true`=1, `false`=0)
/// - string -> `String`; array -> `List`; object -> `Map`; null -> `Null`
///
/// Opaque byte blobs do not arrive here — they use the octet-stream path.
pub fn json_to_kv(v: serde_json::Value) -> KvValue {
    match v {
        serde_json::Value::Null => KvValue::Null,
        serde_json::Value::Bool(b) => KvValue::Int(if b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                KvValue::Int(i)
            } else {
                KvValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => KvValue::String(s),
        serde_json::Value::Array(a) => KvValue::List(a.into_iter().map(json_to_kv).collect()),
        serde_json::Value::Object(o) => {
            KvValue::Map(o.into_iter().map(|(k, v)| (k, json_to_kv(v))).collect())
        }
    }
}

/// Map an engine value back into native JSON.
///
/// `Decimal` is rendered as a string to preserve precision. `Bytes` is rendered
/// as an array of `u8` (the efficient transport for blobs is the octet-stream
/// path, not JSON).
pub fn kv_to_json(v: KvValue) -> serde_json::Value {
    use serde_json::Value as J;
    match v {
        KvValue::Null => J::Null,
        KvValue::Int(i) => J::from(i),
        KvValue::Float(f) => serde_json::Number::from_f64(f).map(J::Number).unwrap_or(J::Null),
        KvValue::Decimal(d) => J::String(d.to_string()),
        KvValue::String(s) => J::String(s),
        KvValue::Bytes(b) => J::Array(b.into_iter().map(J::from).collect()),
        KvValue::List(l) => J::Array(l.into_iter().map(kv_to_json).collect()),
        KvValue::Map(m) => J::Object(m.into_iter().map(|(k, v)| (k, kv_to_json(v))).collect()),
        KvValue::Set(s) => J::Array(s.into_iter().map(J::String).collect()),
        KvValue::SortedSet(z) => J::Object(
            z.into_iter()
                .map(|(k, score)| {
                    (
                        k,
                        serde_json::Number::from_f64(score).map(J::Number).unwrap_or(J::Null),
                    )
                })
                .collect(),
        ),
    }
}
