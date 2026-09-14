//! Rebase a completed field merge onto the latest complete catalog.
//! Input identity includes the row map, checksum, format, sequence, and order.
//! A full-base merge must cover the entire older prefix. Later layers survive.

use super::{SegmentGenerationManifest, SegmentKind, SegmentReference, SegmentRole};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(super) struct MergeSelection {
    pub collection_id: String,
    pub collection_generation: u64,
    pub schema_version: u32,
    pub schema: serde_json::Value,
    pub field: String,
    /// The selected deltas in their original order; the optional base is separate.
    pub inputs: Vec<SegmentReference>,
    pub base: Option<SegmentReference>,
    pub vector_sidecar: Option<SegmentReference>,
}

#[derive(Clone, Debug)]
pub(super) struct VerifiedOutput {
    pub field: SegmentReference,
    pub vector_sidecar: Option<SegmentReference>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RebaseRefusal {
    MissingCollection,
    CollectionIdentityChanged,
    EmptyInputs,
    InputIdentityChanged,
    OutputShape,
}

pub(super) fn rebase_manifest(
    latest: &SegmentGenerationManifest,
    selection: &MergeSelection,
    output: VerifiedOutput,
) -> Result<SegmentGenerationManifest, RebaseRefusal> {
    use RebaseRefusal::*;
    if selection.inputs.is_empty() {
        return Err(EmptyInputs);
    }
    let collection_index = latest
        .collections
        .iter()
        .position(|collection| collection.collection_id == selection.collection_id)
        .ok_or(MissingCollection)?;
    let collection = &latest.collections[collection_index];
    if collection.collection_generation != selection.collection_generation
        || collection.schema_version != selection.schema_version
        || collection.schema != selection.schema
    {
        return Err(CollectionIdentityChanged);
    }
    let field_matches = |reference: &SegmentReference| {
        reference.role == SegmentRole::Field
            && reference.field.as_deref() == Some(selection.field.as_str())
    };
    if selection
        .inputs
        .iter()
        .any(|input| !field_matches(input) || input.kind != SegmentKind::Delta)
        || selection
            .inputs
            .windows(2)
            .any(|pair| pair[0].ordinal >= pair[1].ordinal)
    {
        return Err(InputIdentityChanged);
    }
    let positions: Vec<_> = collection
        .segments
        .iter()
        .enumerate()
        .filter(|(_, reference)| field_matches(reference) && reference.kind == SegmentKind::Delta)
        .map(|(index, _)| index)
        .collect();
    let current: Vec<_> = positions
        .iter()
        .map(|index| &collection.segments[*index])
        .collect();
    let selected: Vec<_> = selection.inputs.iter().collect();
    let start = current
        .windows(selected.len())
        .position(|window| window == selected.as_slice())
        .ok_or(InputIdentityChanged)?;
    let base_merge = selection.base.is_some();
    if base_merge && start != 0 {
        // Omitting an older delta would let a discarded deletion revive its value.
        return Err(InputIdentityChanged);
    }
    let mut removals: BTreeSet<usize> = positions[start..start + selected.len()]
        .iter()
        .copied()
        .collect();
    if let Some(base) = &selection.base {
        if !field_matches(base) || base.kind != SegmentKind::Base || base.ordinal != 0 {
            return Err(InputIdentityChanged);
        }
        let bases: Vec<_> = collection
            .segments
            .iter()
            .enumerate()
            .filter(|(_, reference)| {
                field_matches(reference) && reference.kind == SegmentKind::Base
            })
            .collect();
        if bases.len() != 1 || bases[0].1 != base {
            return Err(InputIdentityChanged);
        }
        removals.insert(bases[0].0);
    }
    let sidecars: Vec<_> = collection
        .segments
        .iter()
        .enumerate()
        .filter(|(_, reference)| {
            reference.role == SegmentRole::VectorEids
                && reference.field.as_deref() == Some(selection.field.as_str())
        })
        .collect();
    if base_merge {
        match (&selection.vector_sidecar, sidecars.as_slice()) {
            (None, []) => {}
            (Some(expected), [(index, actual)]) if expected == *actual => {
                removals.insert(*index);
            }
            _ => return Err(InputIdentityChanged),
        }
    } else if selection.vector_sidecar.is_some() {
        return Err(InputIdentityChanged);
    }
    let expected_ordinal = if base_merge {
        0
    } else {
        selection.inputs.last().unwrap().ordinal
    };
    let expected_kind = if base_merge {
        SegmentKind::Base
    } else {
        SegmentKind::Delta
    };
    if !field_matches(&output.field)
        || output.field.kind != expected_kind
        || output.field.ordinal != expected_ordinal
        || output.field.local_rows.is_none()
        || output.field.payload_sha256.is_none()
        || output.vector_sidecar.is_some() != selection.vector_sidecar.is_some()
    {
        return Err(OutputShape);
    }
    let newest_input = selection
        .inputs
        .iter()
        .filter_map(|input| input.applied_seq)
        .max()
        .unwrap_or(0);
    if !output
        .field
        .applied_seq
        .is_some_and(|sequence| sequence >= newest_input && sequence <= latest.checkpoint_sequence)
    {
        return Err(OutputShape);
    }
    if let Some(sidecar) = &output.vector_sidecar {
        if !base_merge
            || sidecar.role != SegmentRole::VectorEids
            || sidecar.field != output.field.field
            || sidecar.kind != SegmentKind::Base
            || sidecar.ordinal != 0
            || sidecar.local_rows.is_some()
            || sidecar.payload_sha256.is_none()
            || sidecar.applied_seq != output.field.applied_seq
        {
            return Err(OutputShape);
        }
    }
    // An output can replace the selected input's pathname. It cannot alias any
    // payload or row map that remains referenced by another field or collection.
    let mut occupied = BTreeSet::new();
    for (ci, catalog) in latest.collections.iter().enumerate() {
        for (index, reference) in catalog.segments.iter().enumerate() {
            if ci == collection_index && removals.contains(&index) {
                continue;
            }
            occupied.insert(reference.path.as_str());
            if let Some(rows) = &reference.local_rows {
                occupied.insert(rows.path.as_str());
            }
        }
    }
    for reference in std::iter::once(&output.field).chain(output.vector_sidecar.iter()) {
        if reference.path.is_empty() || !occupied.insert(reference.path.as_str()) {
            return Err(OutputShape);
        }
        if let Some(rows) = &reference.local_rows {
            if rows.path.is_empty() || !occupied.insert(rows.path.as_str()) {
                return Err(OutputShape);
            }
        }
    }
    let insert_at = *removals.first().expect("nonempty selected inputs");
    let mut next = latest.clone();
    let references = &mut next.collections[collection_index].segments;
    let mut replacement = Some(output.field);
    let mut sidecar = output.vector_sidecar;
    *references = collection
        .segments
        .iter()
        .enumerate()
        .flat_map(|(index, reference)| {
            if index == insert_at {
                let mut result = vec![replacement.take().expect("one insertion position")];
                result.extend(sidecar.take());
                result
            } else if removals.contains(&index) {
                Vec::new()
            } else {
                vec![reference.clone()]
            }
        })
        .collect();
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::super::{CollectionCatalog, LocalRowsReference, SegmentFormat};
    use super::*;
    fn r(field: &str, kind: SegmentKind, path: &str, ordinal: u32) -> SegmentReference {
        SegmentReference {
            role: SegmentRole::Field,
            field: Some(field.into()),
            ordinal,
            kind,
            format: SegmentFormat::LsegV1,
            path: path.into(),
            local_rows: Some(LocalRowsReference {
                format: "rows".into(),
                path: format!("{path}.rows"),
                count: 1,
            }),
            applied_seq: Some(7),
            payload_sha256: Some(path.into()),
        }
    }
    fn manifest() -> SegmentGenerationManifest {
        SegmentGenerationManifest {
            schema_version: 2,
            checkpoint_sequence: 99,
            revision: 4,
            previous: Some("old".into()),
            next_collection_generation: 9,
            collections: vec![CollectionCatalog {
                collection_id: "c".into(),
                collection_generation: 7,
                schema_version: 3,
                data_version: 41,
                schema: serde_json::json!({"x":1}),
                segments: vec![
                    r("f", SegmentKind::Base, "base", 0),
                    r("other", SegmentKind::Delta, "other", 1),
                    r("f", SegmentKind::Delta, "d1", 1),
                    r("f", SegmentKind::Delta, "d2", 2),
                    r("f", SegmentKind::Delta, "later", 3),
                ],
            }],
        }
    }
    fn select(m: &SegmentGenerationManifest) -> MergeSelection {
        MergeSelection {
            collection_id: "c".into(),
            collection_generation: 7,
            schema_version: 3,
            schema: serde_json::json!({"x":1}),
            field: "f".into(),
            inputs: vec![
                m.collections[0].segments[2].clone(),
                m.collections[0].segments[3].clone(),
            ],
            base: None,
            vector_sidecar: None,
        }
    }
    #[test]
    fn interleaved_preserves_newer_and_data_version() {
        let m = manifest();
        let n = rebase_manifest(
            &m,
            &select(&m),
            VerifiedOutput {
                field: r("f", SegmentKind::Delta, "merged", 2),
                vector_sidecar: None,
            },
        )
        .unwrap();
        assert_eq!(n.checkpoint_sequence, 99);
        assert_eq!(n.revision, 4);
        assert_eq!(n.collections[0].data_version, 41);
        assert_eq!(
            n.collections[0]
                .segments
                .iter()
                .map(|r| r.path.as_str())
                .collect::<Vec<_>>(),
            vec!["base", "other", "merged", "later"]
        );
    }
    #[test]
    fn strict_actual_identity_and_output_kind_refuse() {
        let m = manifest();
        let mut s = select(&m);
        s.inputs[1].local_rows.as_mut().unwrap().count = 2;
        assert_eq!(
            rebase_manifest(
                &m,
                &s,
                VerifiedOutput {
                    field: r("f", SegmentKind::Delta, "m", 1),
                    vector_sidecar: None
                }
            ),
            Err(RebaseRefusal::InputIdentityChanged)
        );
        assert_eq!(
            rebase_manifest(
                &m,
                &select(&m),
                VerifiedOutput {
                    field: r("f", SegmentKind::Base, "m", 0),
                    vector_sidecar: None
                }
            ),
            Err(RebaseRefusal::OutputShape)
        );
    }

    #[test]
    fn output_ordinal_and_paths_cannot_replace_unselected_data() {
        let m = manifest();
        let s = select(&m);
        let wrong_ordinal = VerifiedOutput {
            field: r("f", SegmentKind::Delta, "merged", 9),
            vector_sidecar: None,
        };
        assert_eq!(
            rebase_manifest(&m, &s, wrong_ordinal),
            Err(RebaseRefusal::OutputShape)
        );
        let colliding = VerifiedOutput {
            field: r("f", SegmentKind::Delta, "other", 2),
            vector_sidecar: None,
        };
        assert_eq!(
            rebase_manifest(&m, &s, colliding),
            Err(RebaseRefusal::OutputShape)
        );
        let mut colliding_rows = r("f", SegmentKind::Delta, "merged", 2);
        colliding_rows.local_rows.as_mut().unwrap().path = "later".into();
        assert_eq!(
            rebase_manifest(
                &m,
                &s,
                VerifiedOutput {
                    field: colliding_rows,
                    vector_sidecar: None
                }
            ),
            Err(RebaseRefusal::OutputShape)
        );
    }

    #[test]
    fn actual_input_identity_includes_sequence_checksum_and_row_path() {
        let m = manifest();
        for altered in 0..3 {
            let mut s = select(&m);
            match altered {
                0 => s.inputs[0].applied_seq = Some(6),
                1 => s.inputs[0].payload_sha256 = Some("another-checksum".into()),
                _ => s.inputs[0].local_rows.as_mut().unwrap().path = "another.rows".into(),
            }
            assert_eq!(
                rebase_manifest(
                    &m,
                    &s,
                    VerifiedOutput {
                        field: r("f", SegmentKind::Delta, "merged", 2),
                        vector_sidecar: None
                    }
                ),
                Err(RebaseRefusal::InputIdentityChanged)
            );
        }
    }

    #[test]
    fn field_interleaving_is_valid_but_an_inserted_input_layer_is_stale() {
        let mut m = manifest();
        let s = select(&m);
        m.collections[0]
            .segments
            .insert(3, r("g", SegmentKind::Delta, "g", 1));
        let output = || VerifiedOutput {
            field: r("f", SegmentKind::Delta, "merged", 2),
            vector_sidecar: None,
        };
        let n = rebase_manifest(&m, &s, output()).unwrap();
        assert!(n.collections[0]
            .segments
            .contains(&m.collections[0].segments[3]));
        assert!(n.collections[0]
            .segments
            .contains(m.collections[0].segments.last().unwrap()));
        m.collections[0]
            .segments
            .insert(3, r("f", SegmentKind::Delta, "inserted", 1));
        assert_eq!(
            rebase_manifest(&m, &s, output()),
            Err(RebaseRefusal::InputIdentityChanged)
        );
    }

    #[test]
    fn base_merge_requires_complete_older_prefix_and_exact_vector_sidecar() {
        let mut m = manifest();
        let mut sidecar = r("f", SegmentKind::Base, "vector-eids", 0);
        sidecar.role = SegmentRole::VectorEids;
        sidecar.local_rows = None;
        m.collections[0].segments.push(sidecar.clone());
        let mut s = select(&m);
        s.base = Some(m.collections[0].segments[0].clone());
        s.vector_sidecar = Some(sidecar.clone());
        let mut new_sidecar = sidecar.clone();
        new_sidecar.path = "merged-eids".into();
        new_sidecar.payload_sha256 = Some("merged-eids-checksum".into());
        let output = || VerifiedOutput {
            field: r("f", SegmentKind::Base, "merged-base", 0),
            vector_sidecar: Some(new_sidecar.clone()),
        };
        m.collections[0].data_version = 123;
        m.checkpoint_sequence = 120;
        let n = rebase_manifest(&m, &s, output()).unwrap();
        assert_eq!(n.checkpoint_sequence, 120);
        assert_eq!(n.collections[0].data_version, 123);
        assert!(n.collections[0].segments.contains(&new_sidecar));
        assert!(n.collections[0]
            .segments
            .contains(&m.collections[0].segments[4]));
        assert!(!n.collections[0].segments.contains(&sidecar));
        assert_eq!(
            rebase_manifest(
                &m,
                &s,
                VerifiedOutput {
                    field: r("f", SegmentKind::Base, "merged-base", 0),
                    vector_sidecar: None
                }
            ),
            Err(RebaseRefusal::OutputShape)
        );
        s.inputs.remove(0);
        assert_eq!(
            rebase_manifest(&m, &s, output()),
            Err(RebaseRefusal::InputIdentityChanged)
        );
    }

    #[test]
    fn replacement_refuses_changed_schema_generation_and_missing_collection() {
        let m = manifest();
        let output = || VerifiedOutput {
            field: r("f", SegmentKind::Delta, "merged", 2),
            vector_sidecar: None,
        };
        for altered in 0..3 {
            let mut s = select(&m);
            match altered {
                0 => s.collection_generation += 1,
                1 => s.schema_version += 1,
                _ => s.schema = serde_json::json!({"x": 2}),
            }
            assert_eq!(
                rebase_manifest(&m, &s, output()),
                Err(RebaseRefusal::CollectionIdentityChanged)
            );
        }
        let mut missing = select(&m);
        missing.collection_id = "gone".into();
        assert_eq!(
            rebase_manifest(&m, &missing, output()),
            Err(RebaseRefusal::MissingCollection)
        );
    }
}
