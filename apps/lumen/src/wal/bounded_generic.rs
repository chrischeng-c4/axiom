// Intended new file: apps/lumen/src/wal/bounded_generic.rs
//
// This module deliberately owns the generic wire representation.  The public
// RaftLogEntry and FieldValue types remain unchanged.  In particular, the
// FieldValue visitor receives owned strings from serde and moves them into the
// final record instead of asking untagged deserialization to retain Content
// and then copy it into a second String/Vec.

use std::collections::BTreeMap;
use std::fmt;

use anyhow::{anyhow, Result};
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::WalRecord;
use crate::log_entry::RaftLogEntry;
use crate::types::{
    CreateCollectionRequest, FieldSpec, FieldValue, IndexItem, IndexRequest, ReplaceDocItem,
    ReplaceDocsRequest,
};

// Keep allocation pricing correct if wire or public layouts change.
pub(crate) const INDEX_ITEM_WIDTH: usize =
    if std::mem::size_of::<WireIndexItem>() > std::mem::size_of::<IndexItem>() {
        std::mem::size_of::<WireIndexItem>()
    } else {
        std::mem::size_of::<IndexItem>()
    };
pub(crate) const REPLACE_ITEM_WIDTH: usize =
    if std::mem::size_of::<WireReplaceDocItem>() > std::mem::size_of::<ReplaceDocItem>() {
        std::mem::size_of::<WireReplaceDocItem>()
    } else {
        std::mem::size_of::<ReplaceDocItem>()
    };
pub(crate) const FIELD_VALUE_WIDTH: usize =
    if std::mem::size_of::<WireFieldValue>() > std::mem::size_of::<FieldValue>() {
        std::mem::size_of::<WireFieldValue>()
    } else {
        std::mem::size_of::<FieldValue>()
    };

pub(super) fn decode(bytes: &[u8]) -> Result<WalRecord> {
    let wire: WireRecord = match ciborium::de::from_reader(bytes) {
        Ok(wire) => wire,
        Err(cbor_err) => serde_json::from_slice(bytes).map_err(|json_err| {
            anyhow!("decode WAL record as cbor ({cbor_err}) or legacy json ({json_err})")
        })?,
    };
    Ok(wire.into_record())
}

#[derive(Deserialize)]
struct WireRecord {
    version: u8,
    entry: WireEntry,
}

impl WireRecord {
    fn into_record(self) -> WalRecord {
        WalRecord {
            version: self.version,
            entry: self.entry.into_entry(),
        }
    }
}

// Keep serde's externally tagged enum layout.  This is the layout emitted by
// the old derived RaftLogEntry decoder for both CBOR and the legacy JSON form.
#[derive(Deserialize)]
enum WireEntry {
    CreateCollection {
        collection_id: String,
        req: WireCreateCollectionRequest,
    },
    Index {
        collection_id: String,
        req: WireIndexRequest,
    },
    ReplaceDocs {
        collection_id: String,
        req: WireReplaceDocsRequest,
    },
    TruncateDocs {
        collection_id: String,
    },
    UnindexDocs {
        collection_id: String,
        req: WireBatchUnindexDocsRequest,
    },
    Delete {
        collection_id: String,
        external_id: String,
        field: Option<String>,
    },
    DropCollection {
        collection_id: String,
        force: bool,
    },
    AddField {
        collection_id: String,
        field_name: String,
        spec: FieldSpec,
    },
    DropField {
        collection_id: String,
        field_name: String,
    },
}

impl WireEntry {
    fn into_entry(self) -> RaftLogEntry {
        match self {
            Self::CreateCollection { collection_id, req } => RaftLogEntry::CreateCollection {
                collection_id,
                req: req.into_request(),
            },
            Self::Index { collection_id, req } => RaftLogEntry::Index {
                collection_id,
                req: req.into_request(),
            },
            Self::ReplaceDocs { collection_id, req } => RaftLogEntry::ReplaceDocs {
                collection_id,
                req: req.into_request(),
            },
            Self::TruncateDocs { collection_id } => RaftLogEntry::TruncateDocs { collection_id },
            Self::UnindexDocs { collection_id, req } => RaftLogEntry::UnindexDocs {
                collection_id,
                req: req.into_request(),
            },
            Self::Delete {
                collection_id,
                external_id,
                field,
            } => RaftLogEntry::Delete {
                collection_id,
                external_id,
                field,
            },
            Self::DropCollection {
                collection_id,
                force,
            } => RaftLogEntry::DropCollection {
                collection_id,
                force,
            },
            Self::AddField {
                collection_id,
                field_name,
                spec,
            } => RaftLogEntry::AddField {
                collection_id,
                field_name,
                spec,
            },
            Self::DropField {
                collection_id,
                field_name,
            } => RaftLogEntry::DropField {
                collection_id,
                field_name,
            },
        }
    }
}

#[derive(Deserialize)]
struct WireCreateCollectionRequest {
    fields: BTreeMap<String, FieldSpec>,
}

impl WireCreateCollectionRequest {
    fn into_request(self) -> CreateCollectionRequest {
        CreateCollectionRequest {
            fields: self.fields,
        }
    }
}

#[derive(Deserialize)]
struct WireIndexRequest {
    items: GrowVec<WireIndexItem>,
    #[serde(default)]
    request_id: Option<String>,
}

impl WireIndexRequest {
    fn into_request(self) -> IndexRequest {
        IndexRequest {
            items: self
                .items
                .0
                .into_iter()
                .map(WireIndexItem::into_item)
                .collect(),
            request_id: self.request_id,
        }
    }
}

#[derive(Deserialize)]
struct WireIndexItem {
    external_id: String,
    field: String,
    value: WireFieldValue,
    #[serde(default)]
    version: Option<u64>,
}

impl WireIndexItem {
    fn into_item(self) -> IndexItem {
        IndexItem {
            external_id: self.external_id,
            field: self.field,
            value: self.value.0,
            version: self.version,
        }
    }
}

#[derive(Deserialize)]
struct WireReplaceDocsRequest {
    docs: GrowVec<WireReplaceDocItem>,
}

impl WireReplaceDocsRequest {
    fn into_request(self) -> ReplaceDocsRequest {
        ReplaceDocsRequest {
            docs: self
                .docs
                .0
                .into_iter()
                .map(WireReplaceDocItem::into_item)
                .collect(),
        }
    }
}

#[derive(Deserialize)]
struct WireReplaceDocItem {
    external_id: String,
    #[serde(default)]
    version: Option<u64>,
    fields: BTreeMap<String, WireFieldValue>,
}

impl WireReplaceDocItem {
    fn into_item(self) -> ReplaceDocItem {
        ReplaceDocItem {
            external_id: self.external_id,
            version: self.version,
            fields: self
                .fields
                .into_iter()
                .map(|(key, value)| (key, value.0))
                .collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireBatchUnindexDocsRequest {
    external_ids: GrowVec<String>,
}

impl WireBatchUnindexDocsRequest {
    fn into_request(self) -> crate::types::BatchUnindexDocsRequest {
        crate::types::BatchUnindexDocsRequest {
            external_ids: self.external_ids.0,
        }
    }
}

struct GrowVec<T>(Vec<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for GrowVec<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct GrowVecVisitor<T>(std::marker::PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for GrowVecVisitor<T> {
            type Value = GrowVec<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array")
            }

            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                // serde's normal Vec visitor trusts size_hint. WAL input is
                // durable but untrusted at replay, so only observed elements
                // may make this buffer grow.
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(GrowVec(values))
            }
        }

        deserializer.deserialize_seq(GrowVecVisitor(std::marker::PhantomData))
    }
}

struct WireFieldValue(FieldValue);

enum Scalar {
    Number(f32),
    String(String),
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct ScalarVisitor;

        impl<'de> Visitor<'de> for ScalarVisitor {
            type Value = Scalar;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number or string")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
                Ok(Scalar::String(value.to_owned()))
            }

            fn visit_string<E: de::Error>(
                self,
                value: String,
            ) -> std::result::Result<Self::Value, E> {
                Ok(Scalar::String(value))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Self::Value, E> {
                Ok(Scalar::Number(value as f32))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Self::Value, E> {
                Ok(Scalar::Number(value as f32))
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Self::Value, E> {
                Ok(Scalar::Number(value as f32))
            }
        }

        deserializer.deserialize_any(ScalarVisitor)
    }
}

impl<'de> Deserialize<'de> for WireFieldValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        struct FieldValueVisitor;

        impl<'de> Visitor<'de> for FieldValueVisitor {
            type Value = WireFieldValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string, number, or homogeneous array of strings or numbers")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Self::Value, E> {
                Ok(WireFieldValue(FieldValue::String(value.to_owned())))
            }

            fn visit_string<E: de::Error>(
                self,
                value: String,
            ) -> std::result::Result<Self::Value, E> {
                // This is the ownership boundary: an owned serde token becomes
                // the final FieldValue string without Content or a second copy.
                Ok(WireFieldValue(FieldValue::String(value)))
            }

            fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Self::Value, E> {
                Ok(WireFieldValue(FieldValue::Number(value)))
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                // Do not trust `size_hint`: the input can claim an arbitrary
                // count.  Grow only after each observed element.
                let mut numbers = Vec::new();
                let mut strings = Vec::new();
                let mut kind = None;
                while let Some(value) = seq.next_element::<Scalar>()? {
                    match value {
                        Scalar::Number(value) => {
                            if matches!(kind, Some(false)) {
                                return Err(de::Error::custom("mixed field-value array"));
                            }
                            kind = Some(true);
                            numbers.push(value);
                        }
                        Scalar::String(value) => {
                            if matches!(kind, Some(true)) {
                                return Err(de::Error::custom("mixed field-value array"));
                            }
                            kind = Some(false);
                            strings.push(value);
                        }
                    }
                }
                Ok(WireFieldValue(match kind {
                    Some(true) | None => FieldValue::Vector(numbers),
                    Some(false) => FieldValue::StringList(strings),
                }))
            }
        }

        deserializer.deserialize_any(FieldValueVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Analyzer, FieldType};

    fn legacy_decode(bytes: &[u8]) -> Result<WalRecord> {
        match ciborium::de::from_reader(bytes) {
            Ok(record) => Ok(record),
            Err(cbor_err) => serde_json::from_slice(bytes).map_err(|json_err| {
                anyhow!("decode WAL record as cbor ({cbor_err}) or legacy json ({json_err})")
            }),
        }
    }

    fn assert_same_wire_result(bytes: &[u8]) {
        let old = legacy_decode(bytes);
        let new = decode(bytes);
        match (old, new) {
            (Ok(old), Ok(new)) => {
                let mut old_bytes = Vec::new();
                let mut new_bytes = Vec::new();
                ciborium::ser::into_writer(&old, &mut old_bytes).unwrap();
                ciborium::ser::into_writer(&new, &mut new_bytes).unwrap();
                assert_eq!(old_bytes, new_bytes, "generic decode changed the record");
            }
            (Err(_), Err(_)) => {}
            (old, new) => panic!("old/new decode disagree: old={old:?}, new={new:?}"),
        }
    }

    fn cbor(record: &WalRecord) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(record, &mut bytes).unwrap();
        bytes
    }

    #[test]
    fn generic_cbor_and_legacy_json_preserve_every_entry_shape_and_defaults() {
        let spec = FieldSpec {
            field_type: FieldType::Text,
            analyzer: Some(Analyzer::WhitespaceLower),
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        };
        let records = vec![
            WalRecord {
                version: 1,
                entry: RaftLogEntry::CreateCollection {
                    collection_id: "集合".into(),
                    req: CreateCollectionRequest {
                        fields: BTreeMap::from([("title".into(), spec.clone())]),
                    },
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::Index {
                    collection_id: "docs".into(),
                    req: IndexRequest {
                        request_id: None,
                        items: vec![
                            IndexItem {
                                external_id: "id".into(),
                                field: "text".into(),
                                value: FieldValue::String("escaped \\ \" 雪".into()),
                                version: None,
                            },
                            IndexItem {
                                external_id: "id".into(),
                                field: "number".into(),
                                value: FieldValue::Number(4.5),
                                version: Some(9),
                            },
                            IndexItem {
                                external_id: "id".into(),
                                field: "vector".into(),
                                value: FieldValue::Vector(vec![0.25, 0.5]),
                                version: None,
                            },
                            IndexItem {
                                external_id: "id".into(),
                                field: "set".into(),
                                value: FieldValue::StringList(vec!["é".into(), "雪".into()]),
                                version: None,
                            },
                            IndexItem {
                                external_id: "id".into(),
                                field: "empty".into(),
                                value: FieldValue::Vector(Vec::new()),
                                version: None,
                            },
                        ],
                    },
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::ReplaceDocs {
                    collection_id: "docs".into(),
                    req: ReplaceDocsRequest {
                        docs: vec![ReplaceDocItem {
                            external_id: "id".into(),
                            version: None,
                            fields: BTreeMap::from([
                                ("title".into(), FieldValue::String("多字節".into())),
                                ("embedding".into(), FieldValue::Vector(vec![1.0, 2.0])),
                            ]),
                        }],
                    },
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::Delete {
                    collection_id: "docs".into(),
                    external_id: "id".into(),
                    field: None,
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::DropCollection {
                    collection_id: "docs".into(),
                    force: true,
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::AddField {
                    collection_id: "docs".into(),
                    field_name: "title".into(),
                    spec: spec.clone(),
                },
            },
            WalRecord {
                version: 1,
                entry: RaftLogEntry::DropField {
                    collection_id: "docs".into(),
                    field_name: "title".into(),
                },
            },
            WalRecord {
                version: 2,
                entry: RaftLogEntry::TruncateDocs {
                    collection_id: "docs".into(),
                },
            },
            WalRecord {
                version: 2,
                entry: RaftLogEntry::UnindexDocs {
                    collection_id: "docs".into(),
                    req: crate::types::BatchUnindexDocsRequest {
                        external_ids: vec!["id".into()],
                    },
                },
            },
        ];
        for record in records {
            let bytes = cbor(&record);
            assert_same_wire_result(&bytes);
            assert_same_wire_result(&serde_json::to_vec(&record).unwrap());
        }

        let nonfinite = WalRecord {
            version: 1,
            entry: RaftLogEntry::Index {
                collection_id: "docs".into(),
                req: IndexRequest {
                    request_id: Some("cbor-only-nonfinite".into()),
                    items: vec![IndexItem {
                        external_id: "id".into(),
                        field: "embedding".into(),
                        value: FieldValue::Vector(vec![f32::NAN, f32::INFINITY]),
                        version: None,
                    }],
                },
            },
        };
        assert_same_wire_result(&cbor(&nonfinite));
    }

    #[test]
    fn generic_field_value_rejects_mixed_and_non_value_shapes_like_untagged_field_value() {
        for json in [
            br#"[1,"two"]"#.as_slice(),
            br#"null"#.as_slice(),
            br#"true"#.as_slice(),
            br#"{}"#.as_slice(),
        ] {
            assert!(
                serde_json::from_slice::<WireFieldValue>(json).is_err(),
                "{json:?}"
            );
            assert!(
                serde_json::from_slice::<FieldValue>(json).is_err(),
                "{json:?}"
            );
        }
        let empty: WireFieldValue = serde_json::from_slice(br#"[]"#).unwrap();
        assert!(matches!(empty.0, FieldValue::Vector(values) if values.is_empty()));
    }

    #[test]
    fn generic_unknown_unindex_keys_keep_legacy_refusal() {
        let value = serde_json::json!({
            "version": 1,
            "entry": {"UnindexDocs": {"collection_id": "c", "req": {
                "external_ids": ["id"], "unexpected": true
            }}}
        });
        let mut cbor = Vec::new();
        ciborium::ser::into_writer(&value, &mut cbor).unwrap();
        for bytes in [serde_json::to_vec(&value).unwrap(), cbor] {
            assert!(legacy_decode(&bytes).is_err());
            assert!(
                decode(&bytes).is_err(),
                "generic decoder accepted an unknown control field"
            );
        }
    }

    #[test]
    fn generic_integer_vector_tokens_keep_direct_f32_rounding() {
        let value = serde_json::json!({
            "version": 1,
            "entry": {"Index": {"collection_id": "c", "req": {"items": [{
                "external_id": "id", "field": "v",
                "value": [(1u64 << 63) + (1u64 << 39) + 1]
            }]}}}
        });
        let mut cbor = Vec::new();
        ciborium::ser::into_writer(&value, &mut cbor).unwrap();
        for bytes in [serde_json::to_vec(&value).unwrap(), cbor] {
            for record in [legacy_decode(&bytes).unwrap(), decode(&bytes).unwrap()] {
                let RaftLogEntry::Index { req, .. } = record.entry else {
                    panic!("expected Index")
                };
                let FieldValue::Vector(vector) = &req.items[0].value else {
                    panic!("expected Vector")
                };
                assert_eq!(
                    vector[0].to_bits(),
                    0x5f000001,
                    "integer vector token was double-rounded"
                );
            }
        }
    }

    #[test]
    fn owned_string_token_moves_into_final_field_value_without_content_copy() {
        let owned = "x".repeat(256 * 1024);
        let allocation = owned.as_ptr();
        let value = WireFieldValue::deserialize(serde::de::value::StringDeserializer::<
            serde::de::value::Error,
        >::new(owned))
        .unwrap();
        let FieldValue::String(final_string) = value.0 else {
            panic!("string token must stay a string");
        };
        assert_eq!(
            final_string.as_ptr(),
            allocation,
            "decoder copied an owned string token"
        );
    }
}
