//! Field-level immutable segment compaction for `segment_rdb` staging trees.
//!
//! This is a child of `segment_rdb.rs`: it deliberately uses that module's
//! private catalog identities and checksum helpers. It reads every input before
//! changing either output pathname. Streaming writers publish a new inode by
//! rename; local-row and EID sidecars use the same discipline here.

use super::*;
use crate::composed_segment::compose_checkpoint_layers;
use crate::segment::{self, stream, SegmentReader};
use crate::types::FieldType;
use std::sync::atomic::{AtomicU64, Ordering};

static COMPACTION_TEMP_NONCE: AtomicU64 = AtomicU64::new(0);

/// One field output prepared for a later catalog replacement. This function
/// never changes `collection.segments`, a manifest, or `CURRENT`.
#[derive(Debug, Clone)]
pub(super) struct CompactedField {
    pub(super) output: SegmentReference,
    pub(super) vector_eids: Option<SegmentReference>,
    pub(super) inputs: Vec<SegmentReference>,
    pub(super) logical_read_bytes: u64,
    pub(super) logical_write_bytes: u64,
}

/// Compact one field from immutable inputs in oldest-to-newest order.
///
/// `includes_base` accepts exactly the field's base plus every captured delta.
/// Without it, `inputs` must be one adjacent delta run. The returned reference
/// is only a candidate: the caller owns catalog replacement, validation, and
/// publication after this function returns successfully.
pub(super) fn write_compacted_field(
    root: &Path,
    sequence: u64,
    collection: &CollectionCatalog,
    field: &str,
    inputs: &[SegmentReference],
    includes_base: bool,
) -> Result<CompactedField> {
    validate_compaction_inputs(collection, field, inputs, includes_base)?;
    let specs: BTreeMap<String, crate::types::FieldSpec> =
        serde_json::from_value(collection.schema.clone()).context("decode compaction schema")?;
    let spec = specs
        .get(field)
        .ok_or_else(|| anyhow!("compacted field is absent from schema: {field}"))?;
    if !matches!(
        spec.field_type,
        FieldType::Keyword
            | FieldType::Number
            | FieldType::Set
            | FieldType::Hash
            | FieldType::Text
            | FieldType::Vector
    ) {
        bail!("compaction does not support field type for {field}");
    }

    let mut logical_read_bytes = 0u64;
    let mut layers = Vec::with_capacity(inputs.len());
    for input in inputs {
        let segment_path = confined(root, &input.path)?;
        logical_read_bytes = checked_bytes(logical_read_bytes, file_len(&segment_path)?)?;
        let reader = Arc::new(SegmentReader::open(&segment_path)?);
        let ids = input_external_ids(root, collection, field, input, &mut logical_read_bytes)?;
        if reader.n_docs() as usize != ids.len() {
            bail!("compaction input row map does not match segment row count");
        }
        layers.push((reader, ids));
    }
    let (view, ids) = compose_checkpoint_layers(layers)?;

    let sidecar: serde_json::Value = serde_json::from_slice(
        &std::fs::read(collection_schema_path(root, collection))
            .context("read staged checkpoint schema for compaction")?,
    )?;
    if sidecar.get("fields") != Some(&collection.schema) {
        bail!("staged checkpoint schema differs from catalog during compaction");
    }
    let layout = crate::storage::CheckpointLayout::from_sidecar(&sidecar)?;
    let stem = layout.field_stem(field);
    let last = inputs.last().expect("validated non-empty");
    let (segment_rel, rows_rel, kind, ordinal) = if includes_base {
        let existing = inputs
            .iter()
            .find(|input| matches!(input.kind, SegmentKind::Base))
            .map(|input| input.path.clone());
        (
            existing
                .unwrap_or_else(|| collection_output_relative(collection, &format!("{stem}.lseg"))),
            collection_output_relative(collection, &format!("{stem}.rows.cbor")),
            SegmentKind::Base,
            0,
        )
    } else {
        let field_dir = collection_checkpoint_dir_name(field);
        let (segment_rel, rows_rel) = if flat_layout(collection) {
            (
                collection_output_relative(
                    collection,
                    &format!("__delta/{field_dir}/{}.lseg", last.ordinal),
                ),
                collection_output_relative(
                    collection,
                    &format!("__delta/{field_dir}/{}.rows.cbor", last.ordinal),
                ),
            )
        } else {
            let prefix = delta_path_prefix(&collection.collection_id, field, last.ordinal);
            (format!("{prefix}.lseg"), format!("{prefix}.rows.cbor"))
        };
        (segment_rel, rows_rel, SegmentKind::Delta, last.ordinal)
    };
    let segment_path = confined(root, &segment_rel)?;
    match spec.field_type {
        FieldType::Text => stream::write_text_stream(&segment_path, sequence, &view)?,
        FieldType::Keyword => stream::write_keyword_stream(&segment_path, sequence, &view)?,
        FieldType::Set => stream::write_set_stream(&segment_path, sequence, &view)?,
        FieldType::Number => stream::write_number_stream(&segment_path, sequence, &view)?,
        FieldType::Hash => stream::write_hash_stream(&segment_path, sequence, &view)?,
        FieldType::Vector => {
            let dim = spec
                .dim
                .ok_or_else(|| anyhow!("vector compaction dimension missing"))?
                as usize;
            stream::write_vector_stream(&segment_path, sequence, view.n_docs(), dim, |row| {
                Ok(view.vector_at(row, dim).map(ToOwned::to_owned))
            })?;
        }
        _ => unreachable!("checked above"),
    }
    write_sparse_rows_atomic(&confined(root, &rows_rel)?, &ids)?;

    let vector_eids = if spec.field_type == FieldType::Vector && includes_base {
        let rel = collection_output_relative(collection, &format!("{stem}.eids.lseg"));
        let path = confined(root, &rel)?;
        write_eids_atomic(&path, sequence, &ids)?;
        Some(SegmentReference {
            role: SegmentRole::VectorEids,
            field: Some(field.to_owned()),
            ordinal: 0,
            kind: SegmentKind::Base,
            format: SegmentFormat::LsegV1,
            path: rel,
            local_rows: None,
            applied_seq: Some(sequence),
            payload_sha256: Some(base_payload_sha256(&path)?),
        })
    } else {
        None
    };
    let rows_path = confined(root, &rows_rel)?;
    let mut logical_write_bytes = checked_bytes(file_len(&segment_path)?, file_len(&rows_path)?)?;
    if let Some(sidecar) = &vector_eids {
        logical_write_bytes = checked_bytes(
            logical_write_bytes,
            file_len(&confined(root, &sidecar.path)?)?,
        )?;
    }
    // A local row map changes the base's identity too. Base-only, unmapped
    // files retain `base_payload_sha256`; every output here has a row map.
    let payload_sha256 = delta_payload_sha256(&segment_path, &rows_path)?;
    Ok(CompactedField {
        output: SegmentReference {
            role: SegmentRole::Field,
            field: Some(field.to_owned()),
            ordinal,
            kind,
            format: SegmentFormat::LsegV1,
            path: segment_rel,
            local_rows: Some(LocalRowsReference {
                format: "lumen-local-eids-cbor-v1".to_owned(),
                path: rows_rel,
                count: u32::try_from(ids.len()).context("compacted rows exceed u32")?,
            }),
            applied_seq: Some(sequence),
            payload_sha256: Some(payload_sha256),
        },
        vector_eids,
        inputs: inputs.to_vec(),
        logical_read_bytes,
        logical_write_bytes,
    })
}

fn validate_compaction_inputs(
    collection: &CollectionCatalog,
    field: &str,
    inputs: &[SegmentReference],
    includes_base: bool,
) -> Result<()> {
    if inputs.is_empty() {
        bail!("compaction needs at least one input");
    }
    let field_segments: Vec<_> = collection
        .segments
        .iter()
        .filter(|reference| {
            matches!(reference.role, SegmentRole::Field)
                && reference.field.as_deref() == Some(field)
        })
        .collect();
    if includes_base {
        if inputs.len() != field_segments.len() {
            bail!("base compaction must include the base and every captured delta");
        }
        for (input, expected) in inputs.iter().zip(field_segments) {
            if !same_reference(input, expected)? {
                bail!("base compaction inputs do not match catalog identity");
            }
        }
    } else {
        for input in inputs {
            if !matches!(input.role, SegmentRole::Field)
                || input.field.as_deref() != Some(field)
                || !matches!(input.kind, SegmentKind::Delta)
                || input.local_rows.is_none()
            {
                bail!("partial compaction requires field delta inputs with row maps");
            }
        }
        let deltas: Vec<_> = field_segments
            .into_iter()
            .filter(|reference| matches!(reference.kind, SegmentKind::Delta))
            .collect();
        let first = inputs.first().expect("checked non-empty");
        let Some(start) = deltas
            .iter()
            .position(|candidate| same_reference(first, candidate).unwrap_or(false))
        else {
            bail!("partial compaction input is not an exact catalog reference");
        };
        let Some(window) = deltas.get(start..start + inputs.len()) else {
            bail!("partial compaction inputs run past the catalog delta window");
        };
        for (input, expected) in inputs.iter().zip(window) {
            if !same_reference(input, expected)? {
                bail!("partial compaction inputs must be one contiguous catalog window");
            }
        }
    }
    if !matches!(inputs[0].kind, SegmentKind::Base) && includes_base {
        bail!("base compaction must start with a base segment");
    }
    Ok(())
}

fn input_external_ids(
    root: &Path,
    collection: &CollectionCatalog,
    field: &str,
    input: &SegmentReference,
    bytes: &mut u64,
) -> Result<Vec<String>> {
    if let Some(local) = &input.local_rows {
        let path = confined(root, &local.path)?;
        *bytes = checked_bytes(*bytes, file_len(&path)?)?;
        let rows = segment::decode_sparse_local_rows(&path, local.count)?;
        return (0..local.count)
            .map(|row| {
                rows.external_id(row)
                    .map(str::to_owned)
                    .ok_or_else(|| anyhow!("local row map is incomplete"))
            })
            .collect();
    }
    if !matches!(input.kind, SegmentKind::Base) {
        bail!("delta input is missing local row map");
    }
    let role = if is_vector_field(collection, field)? {
        SegmentRole::VectorEids
    } else {
        SegmentRole::CollectionEids
    };
    let sidecar = collection
        .segments
        .iter()
        .find(|reference| {
            std::mem::discriminant(&reference.role) == std::mem::discriminant(&role)
                && matches!(reference.kind, SegmentKind::Base)
                && (matches!(&role, SegmentRole::CollectionEids)
                    || reference.field.as_deref() == Some(field))
        })
        .ok_or_else(|| anyhow!("base compaction has no required EID sidecar"))?;
    let path = confined(root, &sidecar.path)?;
    *bytes = checked_bytes(*bytes, file_len(&path)?)?;
    SegmentReader::open(&path)?
        .eids_all()
        .ok_or_else(|| anyhow!("base EID sidecar is invalid"))
}

fn is_vector_field(collection: &CollectionCatalog, field: &str) -> Result<bool> {
    let specs: BTreeMap<String, crate::types::FieldSpec> =
        serde_json::from_value(collection.schema.clone())?;
    Ok(specs
        .get(field)
        .is_some_and(|spec| spec.field_type == FieldType::Vector))
}

fn same_reference(left: &SegmentReference, right: &SegmentReference) -> Result<bool> {
    Ok(serde_json::to_value(left)? == serde_json::to_value(right)?)
}

pub(super) fn confined(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || relative
            .split('/')
            .any(|part| matches!(part, "" | "." | ".."))
    {
        bail!("compaction reference escapes root: {relative}");
    }
    Ok(root.join(path))
}

fn checked_bytes(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| anyhow!("compaction byte counter overflow"))
}

fn file_len(path: &Path) -> Result<u64> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!(
            "compaction input must be a regular file: {}",
            path.display()
        );
    }
    Ok(metadata.len())
}

fn unique_aux_temp(path: &Path) -> Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("output has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("output has no UTF-8 name"))?;
    for _ in 0..32 {
        let nonce = COMPACTION_TEMP_NONCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.compact-{}-{nonce}.tmp",
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                drop(file);
                return Ok(candidate);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| format!("create {}", candidate.display()))
            }
        }
    }
    bail!("could not allocate compaction temporary output")
}

fn write_sparse_rows_atomic(path: &Path, ids: &[String]) -> Result<()> {
    let temp = unique_aux_temp(path)?;
    let result =
        segment::encode_sparse_local_rows(&temp, ids).and_then(|_| sync_and_rename(&temp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn write_eids_atomic(path: &Path, sequence: u64, ids: &[String]) -> Result<()> {
    let temp = unique_aux_temp(path)?;
    let refs: Vec<_> = ids.iter().map(String::as_str).collect();
    let result = segment::write_eid_segment(&temp, sequence, &refs)
        .and_then(|_| sync_and_rename(&temp, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

fn sync_and_rename(temp: &Path, path: &Path) -> Result<()> {
    File::open(temp)?.sync_all()?;
    std::fs::rename(temp, path)?;
    File::open(
        path.parent()
            .ok_or_else(|| anyhow!("output has no parent"))?,
    )?
    .sync_all()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Postings;

    fn reference(
        role: SegmentRole,
        field: Option<&str>,
        kind: SegmentKind,
        ordinal: u32,
        path: &str,
        rows: Option<(&str, u32)>,
    ) -> SegmentReference {
        SegmentReference {
            role,
            field: field.map(str::to_owned),
            ordinal,
            kind,
            format: SegmentFormat::LsegV1,
            path: path.to_owned(),
            local_rows: rows.map(|(path, count)| LocalRowsReference {
                format: "lumen-local-eids-cbor-v1".to_owned(),
                path: path.to_owned(),
                count,
            }),
            applied_seq: Some(1),
            payload_sha256: None,
        }
    }

    fn fixture() -> (tempfile::TempDir, CollectionCatalog) {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("75");
        std::fs::create_dir_all(dir.join("__delta/k")).unwrap();
        std::fs::create_dir_all(dir.join("__delta/n")).unwrap();
        std::fs::create_dir_all(dir.join("__delta/v")).unwrap();
        std::fs::create_dir_all(dir.join("__delta/t")).unwrap();
        let fields = serde_json::json!({
            "k": {"type":"keyword"}, "n": {"type":"number"},
            "v": {"type":"vector", "dim":2, "metric":"l2", "backend":"flat-cpu"},
            "t": {"type":"text", "analyzer":"whitespace_lower"}
        });
        std::fs::write(
            dir.join("_schema.json"),
            serde_json::to_vec(&serde_json::json!({"version":1,"fields":fields})).unwrap(),
        )
        .unwrap();
        segment::write_eid_segment(&dir.join("_collection.lmeta.lseg"), 1, &["b"]).unwrap();
        segment::write_eid_segment(&dir.join("v.eids.lseg"), 1, &["b"]).unwrap();
        let mut postings = BTreeMap::new();
        postings.insert("old".to_owned(), roaring::RoaringBitmap::from_iter([0]));
        segment::write_keyword_segment(&dir.join("k.lseg"), 1, &[Some("old")], &postings).unwrap();
        segment::write_keyword_segment(
            &dir.join("__delta/k/1.lseg"),
            1,
            &[Some("new")],
            &BTreeMap::from([("new".to_owned(), roaring::RoaringBitmap::from_iter([0]))]),
        )
        .unwrap();
        segment::write_number_segment(&dir.join("n.lseg"), 1, &[Some(1.0)]).unwrap();
        segment::write_number_segment(&dir.join("__delta/n/1.lseg"), 1, &[Some(2.0)]).unwrap();
        segment::write_vector_segment(&dir.join("v.lseg"), 1, 2, &[Some(&[1.0, 1.0])]).unwrap();
        segment::write_vector_segment(&dir.join("__delta/v/1.lseg"), 1, 2, &[Some(&[2.0, 2.0])])
            .unwrap();
        let mut base_tokens = BTreeMap::new();
        base_tokens
            .entry("old".to_owned())
            .or_insert_with(Postings::default)
            .upsert(0, 1);
        segment::write_text_segment(&dir.join("t.lseg"), 1, &base_tokens, &[1], &[true], 1, 1)
            .unwrap();
        let mut delta_tokens = BTreeMap::new();
        delta_tokens
            .entry("new".to_owned())
            .or_insert_with(Postings::default)
            .upsert(0, 1);
        segment::write_text_segment(
            &dir.join("__delta/t/1.lseg"),
            1,
            &delta_tokens,
            &[1],
            &[true],
            1,
            1,
        )
        .unwrap();
        for field in ["k", "n", "v", "t"] {
            segment::encode_sparse_local_rows(
                &dir.join(format!("__delta/{field}/1.rows.cbor")),
                &["a".to_owned()],
            )
            .unwrap();
        }
        let mut segments = vec![reference(
            SegmentRole::CollectionEids,
            None,
            SegmentKind::Base,
            0,
            "75/_collection.lmeta.lseg",
            None,
        )];
        for field in ["k", "n", "v", "t"] {
            segments.push(reference(
                SegmentRole::Field,
                Some(field),
                SegmentKind::Base,
                0,
                &format!("75/{field}.lseg"),
                None,
            ));
            segments.push(reference(
                SegmentRole::Field,
                Some(field),
                SegmentKind::Delta,
                1,
                &format!("75/__delta/{field}/1.lseg"),
                Some((&format!("75/__delta/{field}/1.rows.cbor"), 1)),
            ));
        }
        segments.push(reference(
            SegmentRole::VectorEids,
            Some("v"),
            SegmentKind::Base,
            0,
            "75/v.eids.lseg",
            None,
        ));
        (
            root,
            CollectionCatalog {
                collection_id: "u".to_owned(),
                collection_generation: 1,
                schema_version: 1,
                data_version: 1,
                schema: serde_json::json!({"k":{"type":"keyword"},"n":{"type":"number"},"v":{"type":"vector","dim":2,"metric":"l2","backend":"flat-cpu"},"t":{"type":"text","analyzer":"whitespace_lower"}}),
                segments,
            },
        )
    }

    fn inputs(collection: &CollectionCatalog, field: &str) -> Vec<SegmentReference> {
        collection
            .segments
            .iter()
            .filter(|reference| {
                matches!(reference.role, SegmentRole::Field)
                    && reference.field.as_deref() == Some(field)
            })
            .cloned()
            .collect()
    }

    #[test]
    fn partial_window_uses_catalog_order_not_ordinal_arithmetic() {
        let (_root, mut collection) = fixture();
        collection.segments.retain(|reference| {
            !(matches!(reference.role, SegmentRole::Field)
                && reference.field.as_deref() == Some("k")
                && matches!(reference.kind, SegmentKind::Delta))
        });
        let mut deltas = Vec::new();
        for ordinal in [2, 4, 6] {
            let prefix = delta_path_prefix(&collection.collection_id, "k", ordinal);
            let segment = format!("{prefix}.lseg");
            let rows = format!("{prefix}.rows.cbor");
            let reference = reference(
                SegmentRole::Field,
                Some("k"),
                SegmentKind::Delta,
                ordinal,
                &segment,
                Some((&rows, 1)),
            );
            collection.segments.push(reference.clone());
            deltas.push(reference);
        }
        validate_compaction_inputs(&collection, "k", &deltas[..2], false)
            .expect("ordinal-2 then ordinal-4 is a contiguous catalog window");
        assert!(
            validate_compaction_inputs(
                &collection,
                "k",
                &[deltas[0].clone(), deltas[2].clone()],
                false
            )
            .is_err(),
            "selecting ordinal-2 then ordinal-6 skips the catalogued ordinal-4 delta"
        );
    }

    #[test]
    fn scalar_compaction_preserves_tombstones_empty_values_and_text_statistics() {
        for includes_base in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let dir = root.path().join("75");
            std::fs::create_dir_all(&dir).unwrap();
            let schema = serde_json::json!({
                "n": {"type":"number"}, "s": {"type":"set"},
                "h": {"type":"hash"}, "t": {"type":"text","analyzer":"whitespace_lower"}
            });
            std::fs::write(
                dir.join("_schema.json"),
                serde_json::to_vec(&serde_json::json!({"version":1,"fields":schema})).unwrap(),
            )
            .unwrap();
            segment::write_eid_segment(&dir.join("_collection.lmeta.lseg"), 1, &["m", "z"])
                .unwrap();
            let mut collection = CollectionCatalog {
                collection_id: "u".into(),
                collection_generation: 1,
                schema_version: 1,
                data_version: 3,
                schema,
                segments: vec![reference(
                    SegmentRole::CollectionEids,
                    None,
                    SegmentKind::Base,
                    0,
                    "75/_collection.lmeta.lseg",
                    None,
                )],
            };
            for field in ["n", "s", "h", "t"] {
                for ordinal in 0..=2 {
                    let (ids, numbers, members, hashes, texts) = match ordinal {
                        0 => (
                            ["m", "z"],
                            [Some(1.), Some(2.)],
                            [Some("old"), Some("old")],
                            [Some(22), Some(44)],
                            [Some(("old", 1)), Some(("old", 1))],
                        ),
                        1 => (
                            ["a", "z"],
                            [Some(7.), None],
                            [Some("first"), None],
                            [Some(11), None],
                            [Some(("first", 1)), None],
                        ),
                        _ => (
                            ["m", "a"],
                            [Some(3.), Some(7.)],
                            [Some("new"), Some("")],
                            [Some(33), Some(11)],
                            [Some(("new", 2)), Some(("", 0))],
                        ),
                    };
                    let path = if ordinal == 0 {
                        format!("75/{field}.lseg")
                    } else {
                        format!("75/__delta/{field}/{ordinal}.lseg")
                    };
                    let target = root.path().join(&path);
                    std::fs::create_dir_all(target.parent().unwrap()).unwrap();
                    match field {
                        "n" => segment::write_number_segment(&target, 1, &numbers).unwrap(),
                        "h" => segment::write_hash_segment(&target, 1, &hashes).unwrap(),
                        "s" => {
                            let values: Vec<Option<Vec<String>>> = members
                                .iter()
                                .map(|value| {
                                    value.map(|member| {
                                        if member.is_empty() {
                                            vec![]
                                        } else {
                                            vec![member.to_owned()]
                                        }
                                    })
                                })
                                .collect();
                            let mut postings = BTreeMap::<String, roaring::RoaringBitmap>::new();
                            for (row, members) in values.iter().enumerate() {
                                for member in members.iter().flatten() {
                                    postings
                                        .entry(member.clone())
                                        .or_default()
                                        .insert(row as u32);
                                }
                            }
                            let borrowed: Vec<_> =
                                values.iter().map(|value| value.as_deref()).collect();
                            segment::write_set_segment(&target, 1, &borrowed, &postings).unwrap();
                        }
                        "t" => {
                            let mut postings = BTreeMap::<String, Postings>::new();
                            let mut lens = [0; 2];
                            let present = texts.map(|text| text.is_some());
                            for (row, text) in texts.iter().enumerate() {
                                if let Some((token, count)) = text {
                                    lens[row] = *count;
                                    if *count != 0 {
                                        postings
                                            .entry((*token).to_owned())
                                            .or_default()
                                            .upsert(row as u32, *count);
                                    }
                                }
                            }
                            segment::write_text_segment(
                                &target,
                                1,
                                &postings,
                                &lens,
                                &present,
                                present.iter().filter(|value| **value).count() as u64,
                                lens.iter().map(|value| *value as u64).sum(),
                            )
                            .unwrap();
                        }
                        _ => unreachable!(),
                    }
                    let rows_path = format!("75/__delta/{field}/{ordinal}.rows.cbor");
                    if ordinal != 0 {
                        segment::encode_sparse_local_rows(
                            &root.path().join(&rows_path),
                            &ids.map(str::to_owned),
                        )
                        .unwrap();
                    }
                    collection.segments.push(reference(
                        SegmentRole::Field,
                        Some(field),
                        if ordinal == 0 {
                            SegmentKind::Base
                        } else {
                            SegmentKind::Delta
                        },
                        ordinal,
                        &path,
                        (ordinal != 0).then_some((rows_path.as_str(), 2)),
                    ));
                }
            }
            for field in ["n", "s", "h", "t"] {
                let selected: Vec<_> = inputs(&collection, field)
                    .into_iter()
                    .filter(|input| includes_base || matches!(input.kind, SegmentKind::Delta))
                    .collect();
                let output = write_compacted_field(
                    root.path(),
                    9,
                    &collection,
                    field,
                    &selected,
                    includes_base,
                )
                .unwrap();
                let local = output.output.local_rows.as_ref().unwrap();
                let rows =
                    segment::decode_sparse_local_rows(&root.path().join(&local.path), local.count)
                        .unwrap();
                assert_eq!(
                    (0..rows.len())
                        .map(|row| rows.external_id(row).unwrap())
                        .collect::<Vec<_>>(),
                    ["a", "m", "z"],
                    "compaction retains stable IDs including the deleted row"
                );
                let reader = SegmentReader::open(&root.path().join(output.output.path)).unwrap();
                match field {
                    "n" => {
                        assert_eq!(
                            (
                                reader.number_at(0),
                                reader.number_at(1),
                                reader.number_at(2)
                            ),
                            (Some(7.), Some(3.), None)
                        );
                    }
                    "h" => {
                        assert_eq!(
                            (reader.hash_at(0), reader.hash_at(1), reader.hash_at(2)),
                            (Some(11), Some(33), None)
                        );
                    }
                    "s" => {
                        assert_eq!(reader.set_at(0), Some(vec![]));
                        assert_eq!(reader.set_at(1), Some(vec!["new".into()]));
                        assert_eq!(reader.set_at(2), None);
                        assert_eq!(reader.set_postings("old"), None);
                        assert_eq!(reader.set_postings("first"), None);
                        assert_eq!(
                            reader.set_postings("new"),
                            Some(roaring::RoaringBitmap::from_iter([1]))
                        );
                    }
                    "t" => {
                        assert!(reader.text_is_present(0));
                        assert_eq!(reader.text_doc_len(0), 0);
                        assert!(!reader.text_is_present(2));
                        assert_eq!(reader.text_postings("old"), None);
                        assert_eq!(reader.text_postings("first"), None);
                        assert_eq!(reader.text_postings("new"), Some((vec![1], vec![2])));
                        assert_eq!(reader.text_doc_count(), 2);
                        assert_eq!(reader.text_total_doc_len(), 2);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn compacts_keyword_number_vector_and_text_with_real_readers() {
        let (root, collection) = fixture();
        let keyword = write_compacted_field(
            root.path(),
            9,
            &collection,
            "k",
            &inputs(&collection, "k"),
            true,
        )
        .unwrap();
        let number = write_compacted_field(
            root.path(),
            9,
            &collection,
            "n",
            &inputs(&collection, "n"),
            true,
        )
        .unwrap();
        let vector = write_compacted_field(
            root.path(),
            9,
            &collection,
            "v",
            &inputs(&collection, "v"),
            true,
        )
        .unwrap();
        let text = write_compacted_field(
            root.path(),
            9,
            &collection,
            "t",
            &inputs(&collection, "t"),
            true,
        )
        .unwrap();
        assert_eq!(
            SegmentReader::open(&root.path().join(&keyword.output.path))
                .unwrap()
                .keyword_at(0),
            Some("new".to_owned())
        );
        assert_eq!(
            SegmentReader::open(&root.path().join(&number.output.path))
                .unwrap()
                .number_at(0),
            Some(2.0)
        );
        assert_eq!(
            SegmentReader::open(&root.path().join(&vector.output.path))
                .unwrap()
                .vector_at(0, 2),
            Some(&[2.0, 2.0][..])
        );
        assert_eq!(
            SegmentReader::open(&root.path().join(&text.output.path))
                .unwrap()
                .text_postings("new"),
            Some((vec![0], vec![1]))
        );
        assert!(vector.vector_eids.is_some());
        assert!(keyword.logical_read_bytes > 0 && keyword.logical_write_bytes > 0);
    }
}
