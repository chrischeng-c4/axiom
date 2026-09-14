//! Private streaming codec for a committed [`crate::wal::WalRecord`] stage.
//!
//! This format is only for the pre-apply durable-stage bridge. A caller must
//! already hold bounded record-memory admission before it calls
//! [`read_staged_wal_record`]; this decoder does not add a replay-size limit.
//! The header prevents a staged payload from being confused with the public
//! WAL wire format. CBOR retains non-finite floating-point bits, unlike JSON.

use std::io::{self, Read, Write};

use crate::change_admission::StagePayload;
use crate::log_entry::RaftLogEntry;
use crate::wal::{WalRecord, WAL_CONTROL_FORMAT_VERSION, WAL_FORMAT_VERSION};

const MAGIC: [u8; 4] = *b"LWCS";
const FORMAT_VERSION: u8 = 1;
const HEADER_LEN: usize = MAGIC.len() + 1;

impl StagePayload for WalRecord {
    fn write_stage(&mut self, output: &mut dyn Write) -> io::Result<()> {
        output.write_all(&MAGIC)?;
        output.write_all(&[FORMAT_VERSION])?;
        ciborium::ser::into_writer(&*self, output)
            .map_err(|error| io::Error::other(format!("encode staged WAL record: {error}")))
    }
}

/// Decode one exact private staged record. The caller owns already-admitted
/// record memory and must account for the returned `WalRecord` before use.
pub(crate) fn read_staged_wal_record(input: &mut dyn Read) -> io::Result<WalRecord> {
    let mut header = [0_u8; HEADER_LEN];
    input.read_exact(&mut header)?;
    if header[..MAGIC.len()] != MAGIC {
        return Err(invalid("unknown committed-stage WAL magic"));
    }
    if header[MAGIC.len()] != FORMAT_VERSION {
        return Err(invalid("unsupported committed-stage WAL version"));
    }

    let record: WalRecord = ciborium::de::from_reader(&mut *input)
        .map_err(|error| invalid(format!("decode staged WAL record: {error}")))?;
    validate_record_version(&record)?;
    let mut trailing = [0_u8; 1];
    if input.read(&mut trailing)? != 0 {
        return Err(invalid("trailing bytes in committed-stage WAL record"));
    }
    Ok(record)
}

/// Validate the private stage envelope without decoding a `WalRecord`, then
/// lend its generic CBOR body. This keeps the mapper allocation-free with
/// respect to payload strings, vectors, maps, and record values.
///
/// The public generic WAL decoder intentionally accepts a CBOR item followed
/// by bytes. A private staged record does not: its receipt names exactly one
/// durable payload, so a suffix is corruption rather than forward data.
pub(crate) fn staged_generic_cbor_payload(input: &[u8]) -> io::Result<&[u8]> {
    if input.len() < HEADER_LEN {
        return Err(invalid("truncated committed-stage WAL header"));
    }
    if input[..MAGIC.len()] != MAGIC {
        return Err(invalid("unknown committed-stage WAL magic"));
    }
    if input[MAGIC.len()] != FORMAT_VERSION {
        return Err(invalid("unsupported committed-stage WAL version"));
    }

    let payload = &input[HEADER_LEN..];
    crate::wal_wire_cost::validate_exact_cbor(payload)
        .map_err(|error| invalid(format!("decode staged generic WAL CBOR: {error}")))?;
    Ok(payload)
}

fn validate_record_version(record: &WalRecord) -> io::Result<()> {
    let control = matches!(
        record.entry,
        RaftLogEntry::TruncateDocs { .. } | RaftLogEntry::UnindexDocs { .. }
    );
    let expected = if control {
        WAL_CONTROL_FORMAT_VERSION
    } else {
        WAL_FORMAT_VERSION
    };
    if record.version != expected {
        return Err(invalid(format!(
            "staged WAL record version {} is invalid for its command (expected {expected})",
            record.version
        )));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        BatchUnindexDocsRequest, CreateCollectionRequest, FieldSpec, FieldType, FieldValue,
        IndexItem, IndexRequest, ReplaceDocItem, ReplaceDocsRequest,
    };
    use std::collections::BTreeMap;
    use std::io::Cursor;

    struct ChunkedWrite {
        bytes: Vec<u8>,
        limit: usize,
    }
    impl Write for ChunkedWrite {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let count = bytes.len().min(self.limit);
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn encode(record: &WalRecord) -> Vec<u8> {
        let mut record = record.clone();
        let mut output = ChunkedWrite {
            bytes: Vec::new(),
            limit: 3,
        };
        record.write_stage(&mut output).unwrap();
        output.bytes
    }

    fn decode(bytes: &[u8]) -> io::Result<WalRecord> {
        read_staged_wal_record(&mut Cursor::new(bytes))
    }

    #[test]
    fn mapped_generic_body_requires_one_exact_private_cbor_item() {
        let bytes = encode(&index_record());
        assert_eq!(staged_generic_cbor_payload(&bytes).unwrap(), &bytes[HEADER_LEN..]);

        let mut bad_magic = bytes.clone();
        bad_magic[0] = b'X';
        assert!(staged_generic_cbor_payload(&bad_magic).is_err());

        let mut bad_version = bytes.clone();
        bad_version[MAGIC.len()] = FORMAT_VERSION + 1;
        assert!(staged_generic_cbor_payload(&bad_version).is_err());

        let mut truncated = bytes.clone();
        truncated.truncate(HEADER_LEN);
        assert!(staged_generic_cbor_payload(&truncated).is_err());

        let mut suffix = bytes;
        suffix.push(0);
        assert!(staged_generic_cbor_payload(&suffix).is_err());
    }

    fn keyword_spec() -> FieldSpec {
        FieldSpec {
            field_type: FieldType::Keyword,
            analyzer: None,
            multi: None,
            dim: None,
            metric: None,
            backend: None,
            quantize: None,
        }
    }

    fn index_record() -> WalRecord {
        WalRecord::new(RaftLogEntry::Index {
            collection_id: "orders".into(),
            req: IndexRequest {
                request_id: Some("request-1".into()),
                items: vec![
                    IndexItem {
                        external_id: "a".into(),
                        field: "keyword".into(),
                        value: FieldValue::String("prefix".into()),
                        version: Some(8),
                    },
                    IndexItem {
                        external_id: "b".into(),
                        field: "vector".into(),
                        value: FieldValue::Vector(vec![
                            f32::from_bits(0x7fc0_0123),
                            f32::INFINITY,
                            f32::NEG_INFINITY,
                        ]),
                        version: None,
                    },
                ],
            },
        })
    }

    #[test]
    fn streams_mixed_prefix_and_non_finite_vector_bits_without_json() {
        let decoded = decode(&encode(&index_record())).unwrap();
        let RaftLogEntry::Index { collection_id, req } = decoded.entry else {
            panic!("expected index record");
        };
        assert_eq!(collection_id, "orders");
        assert_eq!(req.items[0].external_id, "a");
        assert!(matches!(req.items[0].value, FieldValue::String(ref value) if value == "prefix"));
        let FieldValue::Vector(values) = &req.items[1].value else {
            panic!("expected vector");
        };
        assert_eq!(values[0].to_bits(), 0x7fc0_0123);
        assert_eq!(values[1].to_bits(), f32::INFINITY.to_bits());
        assert_eq!(values[2].to_bits(), f32::NEG_INFINITY.to_bits());
    }

    #[test]
    fn accepts_v2_control_records_and_refuses_wrong_control_version() {
        let record = WalRecord::new(RaftLogEntry::UnindexDocs {
            collection_id: "orders".into(),
            req: BatchUnindexDocsRequest {
                external_ids: vec!["a".into(), "b".into()],
            },
        });
        assert_eq!(
            decode(&encode(&record)).unwrap().version,
            WAL_CONTROL_FORMAT_VERSION
        );
        let invalid_control = WalRecord {
            version: WAL_FORMAT_VERSION,
            entry: RaftLogEntry::TruncateDocs {
                collection_id: "orders".into(),
            },
        };
        assert_eq!(
            decode(&encode(&invalid_control)).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn round_trips_every_mutation_variant_through_the_private_codec() {
        let mut fields = BTreeMap::new();
        fields.insert("keyword".into(), keyword_spec());
        let variants = vec![
            WalRecord::new(RaftLogEntry::CreateCollection {
                collection_id: "orders".into(),
                req: CreateCollectionRequest { fields },
            }),
            index_record(),
            WalRecord::new(RaftLogEntry::ReplaceDocs {
                collection_id: "orders".into(),
                req: ReplaceDocsRequest {
                    docs: vec![ReplaceDocItem {
                        external_id: "a".into(),
                        version: Some(3),
                        fields: BTreeMap::new(),
                    }],
                },
            }),
            WalRecord::new(RaftLogEntry::TruncateDocs {
                collection_id: "orders".into(),
            }),
            WalRecord::new(RaftLogEntry::UnindexDocs {
                collection_id: "orders".into(),
                req: BatchUnindexDocsRequest {
                    external_ids: vec!["a".into()],
                },
            }),
            WalRecord::new(RaftLogEntry::Delete {
                collection_id: "orders".into(),
                external_id: "a".into(),
                field: Some("keyword".into()),
            }),
            WalRecord::new(RaftLogEntry::DropCollection {
                collection_id: "orders".into(),
                force: true,
            }),
            WalRecord::new(RaftLogEntry::AddField {
                collection_id: "orders".into(),
                field_name: "keyword".into(),
                spec: keyword_spec(),
            }),
            WalRecord::new(RaftLogEntry::DropField {
                collection_id: "orders".into(),
                field_name: "keyword".into(),
            }),
        ];
        for record in variants {
            let decoded = decode(&encode(&record)).unwrap();
            assert_eq!(decoded.version, record.version);
            let mut expected_cbor = Vec::new();
            let mut actual_cbor = Vec::new();
            ciborium::ser::into_writer(&record, &mut expected_cbor).unwrap();
            ciborium::ser::into_writer(&decoded, &mut actual_cbor).unwrap();
            assert_eq!(actual_cbor, expected_cbor);
        }
    }

    #[test]
    fn refuses_unknown_header_and_private_version() {
        let bytes = encode(&index_record());
        let mut wrong_magic = bytes.clone();
        wrong_magic[0] ^= 1;
        assert_eq!(
            decode(&wrong_magic).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        let mut wrong_version = bytes;
        wrong_version[MAGIC.len()] = FORMAT_VERSION + 1;
        assert_eq!(
            decode(&wrong_version).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn refuses_truncated_and_trailing_records() {
        let bytes = encode(&index_record());
        for end in [0, HEADER_LEN - 1, HEADER_LEN, bytes.len() - 1] {
            assert!(decode(&bytes[..end]).is_err(), "must reject length {end}");
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert_eq!(
            decode(&trailing).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
