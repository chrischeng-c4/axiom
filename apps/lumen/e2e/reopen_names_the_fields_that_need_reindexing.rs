//! A reopen that restores a field empty has to say which field, and a tool has
//! to be able to ask the same question of a snapshot document offline.
//!
//! The damage is real and already shipped. Before the segment-aware forward
//! gather landed, sealing a `Set` field emptied BOTH of its halves — the
//! inverted `elements` map and the `forward` column — and the snapshot writer
//! read them in that state. Every `Set` field sealed by such a build is written
//! to the snapshot with an empty `forward`, and `FieldIndex::from_snapshot`
//! restores it as an empty index. The `Keyword` arm rebuilds its inverted map
//! from `forward` and therefore self-heals; the `Set` arm has nothing left to
//! rebuild from. `storage.rs`'s own comment on that arm says as much and stops
//! there: *"Nothing at this layer can invent the data back; finding which
//! fields are in that state is a separate audit."* This file is that audit's
//! oracle.
//!
//! What makes it invisible today is that the reopen SUCCEEDS. The collection is
//! there, the schema still declares the field, `stats` reports it, and every
//! query against it matches nothing — which is indistinguishable from a
//! collection where nobody happened to write that field. The operator learns
//! about it from a user, months later.
//!
//! The signal the audit reads is the collection's own document census.
//! `eid_fields` maps each live document to the set of fields it carries, is
//! written by the same snapshot, and is untouched by the seal bug. A field the
//! census says N documents carry, whose index holds a value for none of them,
//! is not an empty field — it is a field whose contents were dropped. That
//! disagreement is what gets reported, and it is why an empty collection and a
//! never-written field are both silent here.
//!
//! Two observation points, because the two audiences are different:
//!
//! | observation | who reads it |
//! |---|---|
//! | `Engine::reindex_needed` after a reopen | the running node, and its logs |
//! | `lumen inspect --file <snapshot>` | an operator holding a backup, offline |
//!
//! The damaged snapshot below is not hand-written. It is produced by taking a
//! healthy snapshot and emptying exactly the one column the seal bug emptied,
//! so the census, the schema, the version and every other field arrive exactly
//! as the writer wrote them and the only difference is the defect itself.

use std::collections::BTreeMap;
use std::process::Command;

use serde_json::json;

use lumen::storage::{Engine, SnapshotV1};
use lumen::types::{CreateCollectionRequest, IndexRequest};

const COLLECTION: &str = "papers";
/// The `Set` field the seal bug empties.
const DAMAGED: &str = "tags";
/// A `keyword` field beside it, written for the same documents. It is the
/// control that says the audit reads a per-field fact rather than "this
/// collection came back thin".
const INTACT: &str = "title";
/// A field the schema declares and no document ever carries. It is the control
/// that says the census — not the emptiness — is what makes a field damaged.
const NEVER_WRITTEN: &str = "retracted";

const DOCS: [(&str, &str, [&str; 2]); 3] = [
    ("p-1", "attention is all you need", ["ml", "nlp"]),
    ("p-2", "a calculus of communicating systems", ["pl", "theory"]),
    ("p-3", "time clocks and the ordering of events", ["dist", "theory"]),
];

/// A collection with three documents, each carrying `title` and `tags`, and a
/// `retracted` field nobody writes.
fn healthy_engine() -> Engine {
    let engine = Engine::new();
    let schema: CreateCollectionRequest = serde_json::from_value(json!({
        "fields": {
            INTACT: { "type": "keyword" },
            DAMAGED: { "type": "set" },
            NEVER_WRITTEN: { "type": "keyword" },
        }
    }))
    .expect("schema");
    engine
        .create_collection(COLLECTION, schema)
        .expect("create collection");

    let items: Vec<serde_json::Value> = DOCS
        .iter()
        .flat_map(|(id, title, tags)| {
            [
                json!({ "external_id": id, "field": INTACT, "value": title }),
                json!({ "external_id": id, "field": DAMAGED, "value": tags }),
            ]
        })
        .collect();
    let req: IndexRequest = serde_json::from_value(json!({ "items": items })).expect("index body");
    engine.index(COLLECTION, req).expect("index");
    engine
}

/// Empty exactly the column the 0.4.28 seal emptied, and nothing else.
fn as_written_by_the_broken_seal(mut snap: SnapshotV1) -> SnapshotV1 {
    let coll = snap
        .collections
        .get_mut(COLLECTION)
        .expect("the snapshot has the collection");
    let field = coll
        .fields
        .get_mut(DAMAGED)
        .expect("the snapshot has the damaged field");
    let mut raw = serde_json::to_value(&*field).expect("field to json");
    assert_eq!(raw["type"], "Set", "the damaged arm is the Set arm: {raw}");
    assert!(
        raw["forward"].as_object().is_some_and(|f| !f.is_empty()),
        "the healthy snapshot has to carry a forward column for this to be a \
         reproduction of the defect rather than a construction of it: {raw}"
    );
    raw["forward"] = json!({});
    *field = serde_json::from_value(raw).expect("damaged field from json");
    snap
}

/// The reopen names the field, the collection, and how many documents the
/// census says are missing from it.
#[test]
fn a_reopen_names_the_field_whose_contents_did_not_survive() {
    let damaged = as_written_by_the_broken_seal(healthy_engine().snapshot().expect("snapshot"));

    let reopened = Engine::new();
    // Tolerant: the reopen still succeeds. Refusing it would strand the
    // `title` field, which is intact, and every other collection in the same
    // document — the operator needs the node up to re-index at all.
    reopened
        .restore(damaged)
        .expect("a damaged field must not cost the whole restore");

    let needed = reopened.reindex_needed().expect("audit");
    let named: Vec<(String, String, u64)> = needed
        .iter()
        .map(|r| (r.collection.clone(), r.field.clone(), r.documents_covered))
        .collect();
    assert_eq!(
        named,
        vec![(COLLECTION.to_string(), DAMAGED.to_string(), 3)],
        "the audit must name the collection, the field, and the census count \
         that disagrees with it — and nothing else"
    );
}

/// The other half of the differential. The same reopen, the same collection,
/// without the mutation: the audit reports nothing at all. Without this the
/// case above passes against an audit that reports every `Set` field, or every
/// field, or a constant.
#[test]
fn a_healthy_round_trip_names_nothing() {
    let reopened = Engine::new();
    reopened
        .restore(healthy_engine().snapshot().expect("snapshot"))
        .expect("restore");
    assert!(
        reopened.reindex_needed().expect("audit").is_empty(),
        "a snapshot that round-trips intact has nothing to re-index: {:?}",
        reopened.reindex_needed().expect("audit")
    );
}

/// A field no document carries is empty on purpose. `retracted` is declared by
/// the schema, restores with an empty index, and is not damage — the census
/// covers it for nobody. This is what stops the audit from reporting every
/// sparsely-used field on every node in the fleet.
#[test]
fn a_field_no_document_carries_is_not_damage() {
    let damaged = as_written_by_the_broken_seal(healthy_engine().snapshot().expect("snapshot"));
    let reopened = Engine::new();
    reopened.restore(damaged).expect("restore");

    let reported: Vec<String> = reopened
        .reindex_needed()
        .expect("audit")
        .into_iter()
        .map(|r| r.field)
        .collect();
    assert!(
        !reported.iter().any(|f| f == NEVER_WRITTEN),
        "a declared field the census covers for nobody is empty, not dropped: \
         {reported:?}"
    );
    assert!(
        !reported.iter().any(|f| f == INTACT),
        "a field that restored with its contents is not damaged: {reported:?}"
    );
}

/// The operator's half: the same question, asked of a snapshot document on
/// disk, by the shipped binary, with no server running. An operator holding a
/// backup taken by the broken build has to be able to find out what it costs
/// them BEFORE they restore it into production.
#[test]
fn the_inspect_command_reports_the_same_damage_offline() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("backup.json");
    let damaged = as_written_by_the_broken_seal(healthy_engine().snapshot().expect("snapshot"));
    std::fs::write(&path, serde_json::to_vec(&damaged).expect("encode")).expect("write snapshot");

    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["inspect", "--file"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run lumen inspect");
    assert!(
        out.status.success(),
        "inspect exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );

    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
            panic!(
                "inspect --format json must print a JSON document ({e}): {}",
                String::from_utf8_lossy(&out.stdout)
            )
        });
    let rows = report["reindex_needed"]
        .as_array()
        .unwrap_or_else(|| panic!("no reindex_needed array in {report}"));
    let named: Vec<(&str, &str, u64)> = rows
        .iter()
        .map(|r| {
            (
                r["collection"].as_str().unwrap_or_default(),
                r["field"].as_str().unwrap_or_default(),
                r["documents_covered"].as_u64().unwrap_or_default(),
            )
        })
        .collect();
    assert_eq!(
        named,
        vec![(COLLECTION, DAMAGED, 3)],
        "the offline report has to name exactly what the reopen names: {report}"
    );
}

/// …and it says so on a clean backup too, rather than only ever printing a
/// problem. An inspector that cannot report "nothing to do" is one an operator
/// cannot use to clear a volume.
#[test]
fn the_inspect_command_clears_a_clean_snapshot() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("clean.json");
    let snap = healthy_engine().snapshot().expect("snapshot");
    std::fs::write(&path, serde_json::to_vec(&snap).expect("encode")).expect("write snapshot");

    let out = Command::new(env!("CARGO_BIN_EXE_lumen"))
        .args(["inspect", "--file"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run lumen inspect");
    assert!(out.status.success(), "inspect exited {}", out.status);

    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let rows = report["reindex_needed"]
        .as_array()
        .unwrap_or_else(|| panic!("no reindex_needed array in {report}"));
    assert!(
        rows.is_empty(),
        "a clean snapshot must clear: {report}"
    );
    // The census the report is measured against travels with it, so the
    // operator can tell "nothing is damaged" from "nothing was read".
    let counted: BTreeMap<String, u64> =
        serde_json::from_value(report["documents_scanned"].clone())
            .unwrap_or_else(|e| panic!("no per-collection scan count ({e}) in {report}"));
    assert_eq!(
        counted.get(COLLECTION).copied(),
        Some(3),
        "a clean verdict has to say what it read: {report}"
    );
}
