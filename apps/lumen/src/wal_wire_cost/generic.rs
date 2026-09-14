//! Cost-only serde grammar for the private, moving WAL decoder. Container
//! storage is counted here; token storage comes from the borrowing preflight.

#![allow(dead_code)]
use super::bounded_serde::Quiet;
use super::{add, allocation, mul, token_stats};
use crate::types::FieldSpec;
use crate::wal::bounded_generic::{FIELD_VALUE_WIDTH, INDEX_ITEM_WIDTH, REPLACE_ITEM_WIDTH};
use anyhow::{ensure, Result};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::{fmt, marker::PhantomData, mem::size_of};

trait HeapCost {
    fn heap(&self) -> Result<usize>;
}
impl<T: HeapCost> HeapCost for Option<T> {
    fn heap(&self) -> Result<usize> {
        self.as_ref().map_or(Ok(0), HeapCost::heap)
    }
}
impl HeapCost for FieldSpec {
    fn heap(&self) -> Result<usize> {
        Ok(0)
    }
}
fn grow_array(count: usize, width: usize) -> Result<usize> {
    if count == 0 {
        Ok(0)
    } else {
        allocation(mul(mul(count.max(4), width)?, 3)?)
    }
}
fn as_de<E: de::Error>(error: anyhow::Error) -> E {
    E::custom(error.to_string())
}

struct Text;
impl HeapCost for Text {
    fn heap(&self) -> Result<usize> {
        Ok(0)
    }
}
impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct TextVisitor;
        impl Visitor<'_> for TextVisitor {
            type Value = Text;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a string")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<Text, E> {
                let _ = s;
                Ok(Text)
            }
            fn visit_string<E: de::Error>(self, s: String) -> std::result::Result<Text, E> {
                // Structural preflight already prices this complete token.
                drop(s);
                Ok(Text)
            }
        }
        d.deserialize_string(TextVisitor)
    }
}

struct List<T, const WIDTH: usize> {
    heap: usize,
    marker: PhantomData<T>,
}
impl<T, const WIDTH: usize> HeapCost for List<T, WIDTH> {
    fn heap(&self) -> Result<usize> {
        Ok(self.heap)
    }
}
impl<'de, T: Deserialize<'de> + HeapCost, const WIDTH: usize> Deserialize<'de> for List<T, WIDTH> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct ListVisitor<T, const WIDTH: usize>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + HeapCost, const WIDTH: usize> Visitor<'de>
            for ListVisitor<T, WIDTH>
        {
            type Value = List<T, WIDTH>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an array")
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let (mut count, mut heap) = (0, 0);
                // Do not allocate from an untrusted size_hint.
                while let Some(value) = a.next_element::<T>()? {
                    count = add(count, 1).map_err(as_de)?;
                    heap = add(heap, value.heap().map_err(as_de)?).map_err(as_de)?;
                }
                // Wire item vectors can coexist with the public target vector
                // during move conversion. Count both even if collect reuses it.
                heap = add(heap, grow_array(count, WIDTH).map_err(as_de)?).map_err(as_de)?;
                heap = add(
                    heap,
                    allocation(mul(count, WIDTH).map_err(as_de)?).map_err(as_de)?,
                )
                .map_err(as_de)?;
                Ok(List {
                    heap,
                    marker: PhantomData,
                })
            }
        }
        d.deserialize_seq(ListVisitor::<T, WIDTH>(PhantomData))
    }
}

struct Fields<T, const WIDTH: usize> {
    heap: usize,
    marker: PhantomData<T>,
}
impl<T, const WIDTH: usize> HeapCost for Fields<T, WIDTH> {
    fn heap(&self) -> Result<usize> {
        Ok(self.heap)
    }
}
impl<'de, T: Deserialize<'de> + HeapCost, const WIDTH: usize> Deserialize<'de>
    for Fields<T, WIDTH>
{
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct FieldsVisitor<T, const WIDTH: usize>(PhantomData<T>);
        impl<'de, T: Deserialize<'de> + HeapCost, const WIDTH: usize> Visitor<'de>
            for FieldsVisitor<T, WIDTH>
        {
            type Value = Fields<T, WIDTH>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a field map")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let node = allocation(
                    add(
                        mul(11, add(size_of::<String>(), WIDTH).map_err(as_de)?).map_err(as_de)?,
                        mul(12, size_of::<usize>()).map_err(as_de)?,
                    )
                    .map_err(as_de)?,
                )
                .map_err(as_de)?;
                let mut heap = 0;
                while let Some(key) = a.next_key::<Text>()? {
                    let value = a.next_value::<T>()?;
                    heap = add(
                        heap,
                        add(
                            mul(node, 2).map_err(as_de)?,
                            add(key.heap().map_err(as_de)?, value.heap().map_err(as_de)?)
                                .map_err(as_de)?,
                        )
                        .map_err(as_de)?,
                    )
                    .map_err(as_de)?;
                }
                Ok(Fields {
                    heap,
                    marker: PhantomData,
                })
            }
        }
        d.deserialize_map(FieldsVisitor::<T, WIDTH>(PhantomData))
    }
}

struct Value(usize);
impl HeapCost for Value {
    fn heap(&self) -> Result<usize> {
        Ok(self.0)
    }
}
// An array may hold only numbers or strings. A scalar marker identifies its
// alternative without keeping its content or trying an owned Vec decoder.
struct Scalar {
    number: bool,
}
impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct ScalarVisitor;
        impl Visitor<'_> for ScalarVisitor {
            type Value = Scalar;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a number or string")
            }
            fn visit_str<E: de::Error>(self, _s: &str) -> std::result::Result<Scalar, E> {
                Ok(Scalar { number: false })
            }
            fn visit_string<E: de::Error>(self, _s: String) -> std::result::Result<Scalar, E> {
                Ok(Scalar { number: false })
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> std::result::Result<Scalar, E> {
                Ok(Scalar { number: true })
            }
            fn visit_i64<E: de::Error>(self, n: i64) -> std::result::Result<Scalar, E> {
                self.visit_f64(n as f64)
            }
            fn visit_u64<E: de::Error>(self, n: u64) -> std::result::Result<Scalar, E> {
                self.visit_f64(n as f64)
            }
        }
        d.deserialize_any(ScalarVisitor)
    }
}
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct ValueVisitor;
        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a field value")
            }
            fn visit_str<E: de::Error>(self, s: &str) -> std::result::Result<Value, E> {
                let _ = s;
                Ok(Value(0))
            }
            fn visit_string<E: de::Error>(self, s: String) -> std::result::Result<Value, E> {
                drop(s);
                Ok(Value(0))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> std::result::Result<Value, E> {
                Ok(Value(0))
            }
            fn visit_i64<E: de::Error>(self, n: i64) -> std::result::Result<Value, E> {
                self.visit_f64(n as f64)
            }
            fn visit_u64<E: de::Error>(self, n: u64) -> std::result::Result<Value, E> {
                self.visit_f64(n as f64)
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> std::result::Result<Value, A::Error> {
                let (mut count, mut kind) = (0, None);
                while let Some(value) = a.next_element::<Scalar>()? {
                    if kind.is_some_and(|number| number != value.number) {
                        return Err(de::Error::custom("mixed field-value array"));
                    }
                    kind = Some(value.number);
                    count = add(count, 1).map_err(as_de)?;
                }
                // The private FieldValue visitor directly grows one typed
                // vector. There is no untagged Content tree or cloned strings.
                let width = if kind == Some(false) {
                    size_of::<String>()
                } else {
                    size_of::<f32>()
                };
                let heap = grow_array(count, width).map_err(as_de)?;
                Ok(Value(heap))
            }
        }
        d.deserialize_any(ValueVisitor)
    }
}

#[derive(Deserialize)]
struct Create {
    fields: Fields<FieldSpec, { size_of::<FieldSpec>() }>,
}
impl HeapCost for Create {
    fn heap(&self) -> Result<usize> {
        self.fields.heap()
    }
}
#[derive(Deserialize)]
struct Item {
    external_id: Text,
    field: Text,
    value: Value,
    #[serde(default)]
    version: Option<u64>,
}
impl HeapCost for Item {
    fn heap(&self) -> Result<usize> {
        add(
            add(self.external_id.heap()?, self.field.heap()?)?,
            self.value.heap()?,
        )
    }
}
#[derive(Deserialize)]
struct Index {
    items: List<Item, INDEX_ITEM_WIDTH>,
    #[serde(default)]
    request_id: Option<Text>,
}
impl HeapCost for Index {
    fn heap(&self) -> Result<usize> {
        add(self.items.heap()?, self.request_id.heap()?)
    }
}
#[derive(Deserialize)]
struct Doc {
    external_id: Text,
    #[serde(default)]
    version: Option<u64>,
    fields: Fields<Value, FIELD_VALUE_WIDTH>,
}
impl HeapCost for Doc {
    fn heap(&self) -> Result<usize> {
        add(self.external_id.heap()?, self.fields.heap()?)
    }
}
#[derive(Deserialize)]
struct Replace {
    docs: List<Doc, REPLACE_ITEM_WIDTH>,
}
impl HeapCost for Replace {
    fn heap(&self) -> Result<usize> {
        self.docs.heap()
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Unindex {
    external_ids: List<Text, { size_of::<String>() }>,
}
#[derive(Deserialize)]
enum Entry {
    CreateCollection {
        collection_id: Text,
        req: Create,
    },
    Index {
        collection_id: Text,
        req: Index,
    },
    ReplaceDocs {
        collection_id: Text,
        req: Replace,
    },
    TruncateDocs {
        collection_id: Text,
    },
    UnindexDocs {
        collection_id: Text,
        req: Unindex,
    },
    Delete {
        collection_id: Text,
        external_id: Text,
        field: Option<Text>,
    },
    DropCollection {
        collection_id: Text,
        force: bool,
    },
    AddField {
        collection_id: Text,
        field_name: Text,
        spec: FieldSpec,
    },
    DropField {
        collection_id: Text,
        field_name: Text,
    },
}
impl HeapCost for Entry {
    fn heap(&self) -> Result<usize> {
        let nested = match self {
            Self::CreateCollection { collection_id, req } => {
                add(collection_id.heap()?, req.heap()?)?
            }
            Self::Index { collection_id, req } => add(collection_id.heap()?, req.heap()?)?,
            Self::ReplaceDocs { collection_id, req } => add(collection_id.heap()?, req.heap()?)?,
            Self::Delete {
                collection_id,
                external_id,
                field,
            } => add(
                add(collection_id.heap()?, external_id.heap()?)?,
                field.heap()?,
            )?,
            Self::DropCollection { collection_id, .. } => collection_id.heap()?,
            Self::AddField {
                collection_id,
                field_name,
                ..
            }
            | Self::DropField {
                collection_id,
                field_name,
            } => add(collection_id.heap()?, field_name.heap()?)?,
            Self::TruncateDocs { .. } | Self::UnindexDocs { .. } => {
                anyhow::bail!("control commands must use WAL v2 fast control encoding")
            }
        };
        add(size_of::<crate::log_entry::RaftLogEntry>(), nested)
    }
}
#[derive(Deserialize)]
struct Record {
    version: u8,
    entry: Entry,
}
pub(super) fn decoded_peak_bound(bytes: &[u8]) -> Result<usize> {
    let Quiet(record): Quiet<Record> = match ciborium::de::from_reader(bytes) {
        Ok(record) => record,
        Err(_) => serde_json::from_slice(bytes)?,
    };
    ensure!(
        record.version == 1,
        "unsupported generic WAL record version {}",
        record.version
    );
    // Strings move from wire to public records. Three aggregate token lengths
    // cover final storage and geometric token growth, including ignored fields
    // and JSON's retained escape scratch. Per-token slack covers small buffers
    // and allocator headers. This is separate from the container bound above.
    let stats = token_stats::scan(bytes)?;
    let tokens = add(stats.text_bytes, stats.byte_string_bytes)?;
    let token_heap = add(mul(tokens, 3)?, mul(stats.token_count, 40)?)?;
    add(add(record.entry.heap()?, token_heap)?, 8192)
}
