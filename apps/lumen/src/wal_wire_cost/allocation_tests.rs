//! Actual heap measurements for the WAL preflight and decoder.
//! The input buffers exist before each
//! measurement. The decoded record remains alive until accounting is disabled.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::collections::BTreeMap;

use crate::log_entry::RaftLogEntry;
use crate::types::{FieldValue, IndexItem, IndexRequest, ReplaceDocItem, ReplaceDocsRequest};
use crate::wal::WalRecord;

#[derive(Clone, Copy, Default, Debug)]
struct AllocState {
    enabled: bool,
    current: usize,
    peak: usize,
    largest: usize,
}

thread_local! {
    static ALLOC_STATE: Cell<AllocState> = const { Cell::new(AllocState {
        enabled: false,
        current: 0,
        peak: 0,
        largest: 0,
    }) };
}

struct CountingSystem;

#[global_allocator]
static ALLOCATOR: CountingSystem = CountingSystem;

fn allocated(size: usize) {
    let _ = ALLOC_STATE.try_with(|cell| {
        let mut state = cell.get();
        if !state.enabled {
            return;
        }
        state.current = state.current.saturating_add(size);
        state.peak = state.peak.max(state.current);
        state.largest = state.largest.max(size);
        cell.set(state);
    });
}

fn deallocated(size: usize) {
    let _ = ALLOC_STATE.try_with(|cell| {
        let mut state = cell.get();
        if state.enabled {
            state.current = state.current.saturating_sub(size);
            cell.set(state);
        }
    });
}

fn reallocated(old: usize, new: usize) {
    let _ = ALLOC_STATE.try_with(|cell| {
        let mut state = cell.get();
        if !state.enabled {
            return;
        }
        // The allocator may hold both blocks during realloc. Count that
        // overlap, then replace the old live allocation with the new one.
        state.peak = state.peak.max(state.current.saturating_add(new));
        state.current = state.current.saturating_sub(old).saturating_add(new);
        state.largest = state.largest.max(old).max(new);
        cell.set(state);
    });
}

unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        deallocated(layout.size());
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new: usize) -> *mut u8 {
        let next = unsafe { System.realloc(pointer, layout, new) };
        if !next.is_null() {
            reallocated(layout.size(), new);
        }
        next
    }
}

struct Restore(AllocState);
impl Drop for Restore {
    fn drop(&mut self) {
        ALLOC_STATE.with(|state| state.set(self.0));
    }
}

fn measure<T>(f: impl FnOnce() -> T) -> (T, AllocState) {
    let old = ALLOC_STATE.with(|state| {
        let old = state.get();
        state.set(AllocState {
            enabled: true,
            ..AllocState::default()
        });
        old
    });
    let restore = Restore(old);
    let output = f();
    let observed = ALLOC_STATE.with(Cell::get);
    drop(restore);
    (output, observed)
}

fn generic_cbor(record: &WalRecord) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(record, &mut bytes).unwrap();
    bytes
}

fn generic_index() -> WalRecord {
    WalRecord {
        version: 1,
        entry: RaftLogEntry::Index {
            collection_id: "docs".into(),
            req: IndexRequest {
                request_id: Some("request-雪".into()),
                items: vec![
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "title".into(),
                        value: FieldValue::String("escaped \\ \" snow 雪".into()),
                        version: Some(7),
                    },
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "tags".into(),
                        value: FieldValue::StringList(vec!["one".into(), "é".into(), "雪".into()]),
                        version: None,
                    },
                    IndexItem {
                        external_id: "doc-1".into(),
                        field: "embedding".into(),
                        value: FieldValue::Vector(vec![0.25, 0.5, 0.75]),
                        version: None,
                    },
                ],
            },
        },
    }
}

fn generic_replace() -> WalRecord {
    WalRecord {
        version: 1,
        entry: RaftLogEntry::ReplaceDocs {
            collection_id: "docs".into(),
            req: ReplaceDocsRequest {
                docs: vec![ReplaceDocItem {
                    external_id: "doc-2".into(),
                    version: Some(11),
                    fields: BTreeMap::from([
                        ("title".into(), FieldValue::String("多字節".into())),
                        (
                            "tags".into(),
                            FieldValue::StringList(vec!["a".into(), "b".into()]),
                        ),
                        ("embedding".into(), FieldValue::Vector(vec![1.0, 2.0, 3.0])),
                    ]),
                }],
            },
        },
    }
}

fn legacy_json_with_ignored_text(bytes: usize) -> Vec<u8> {
    let mut json = br#"{"version":1,"entry":{"Delete":{"collection_id":"docs","external_id":"doc-3","field":null}},"ignored":""#.to_vec();
    json.extend(std::iter::repeat_n(b'x', bytes));
    json.extend_from_slice(br#""}"#);
    json
}

fn indefinite_cbor_index() -> Vec<u8> {
    let mut bytes = generic_cbor(&WalRecord {
        version: 1,
        entry: RaftLogEntry::Index {
            collection_id: "c".into(),
            req: IndexRequest {
                request_id: None,
                items: vec![IndexItem {
                    external_id: "id".into(),
                    field: "field".into(),
                    value: FieldValue::String("value".into()),
                    version: None,
                }],
            },
        },
    });
    let offset = bytes
        .windows(3)
        .position(|window| window == [0x62, b'i', b'd'])
        .expect("fixture must contain external_id as a definite CBOR string");
    bytes.splice(offset..offset + 3, [0x7f, 0x61, b'i', 0x61, b'd', 0xff]);
    bytes
}

fn assert_bounds(bytes: &[u8]) {
    let (workspace, token_preflight) = measure(|| super::scan_workspace_bound(bytes).unwrap());
    assert_eq!(
        token_preflight.peak, 0,
        "token preflight allocated payload memory"
    );

    let (decoded_bound, cost_only) = measure(|| super::decoded_peak_bound(bytes).unwrap());
    assert!(
        cost_only.peak <= workspace,
        "cost-only scanner peak {} exceeded workspace bound {} (largest {})",
        cost_only.peak,
        workspace,
        cost_only.largest,
    );

    let (record, decode) = measure(|| WalRecord::decode(bytes).unwrap());
    assert!(
        decode.peak <= decoded_bound,
        "decoder peak {} exceeded decoded bound {} (largest {})",
        decode.peak,
        decoded_bound,
        decode.largest,
    );
    // Keep every decoded String/Vec/field map alive while its allocation is
    // charged. Dropping happens only after `measure` restores the old state.
    assert_eq!(record.version, 1);
    drop(record);
}

#[test]
fn generic_cbor_preflight_and_decode_heap_stay_within_production_bounds() {
    let bytes = generic_cbor(&generic_index());
    assert_bounds(&bytes);
}

#[test]
fn legacy_json_unknown_two_mib_text_is_preflighted_and_ignored_without_underpricing() {
    let bytes = legacy_json_with_ignored_text(2 * 1024 * 1024);
    assert_bounds(&bytes);
}

#[test]
fn legacy_json_escaped_multibyte_lists_and_vectors_stay_within_bounds() {
    let bytes = serde_json::to_vec(&generic_index()).unwrap();
    assert_bounds(&bytes);
}

#[test]
fn replace_docs_maps_and_indefinite_cbor_text_stay_within_bounds() {
    let replace = generic_cbor(&generic_replace());
    assert_bounds(&replace);
    let indefinite = indefinite_cbor_index();
    assert_bounds(&indefinite);
}

#[test]
fn large_owned_and_escaped_tokens_use_measured_bounds() {
    let mut record = generic_index();
    if let RaftLogEntry::Index { req, .. } = &mut record.entry {
        req.items[0].value = FieldValue::String("雪\n\"".repeat(400_000));
    }
    assert_bounds(&generic_cbor(&record));
    assert_bounds(&serde_json::to_vec(&record).unwrap());
}

#[test]
fn malformed_large_variant_error_stays_within_reserved_scanner_workspace() {
    let variant = "bad-variant".repeat(200_000);
    let value = serde_json::json!({"version": 1, "entry": {variant: {}}});
    let mut cbor = Vec::new();
    ciborium::ser::into_writer(&value, &mut cbor).unwrap();
    for bytes in [cbor, serde_json::to_vec(&value).unwrap()] {
        let workspace = super::scan_workspace_bound(&bytes).unwrap();
        let (result, observed) = measure(|| super::decoded_peak_bound(&bytes));
        assert!(
            result.is_err(),
            "malformed unknown WAL variant must be refused"
        );
        assert!(
            observed.peak <= workspace,
            "error construction peak {} exceeded workspace {}",
            observed.peak,
            workspace
        );
        drop(result);
    }
}

#[test]
fn malformed_escaped_scalar_error_stays_within_reserved_scanner_workspace() {
    let value = serde_json::json!({"version": "\u{0}".repeat(400_000), "entry": {"DropCollection": {"collection_id":"c", "force":true}}});
    let bytes = serde_json::to_vec(&value).unwrap();
    let workspace = super::scan_workspace_bound(&bytes).unwrap();
    let (result, observed) = measure(|| super::decoded_peak_bound(&bytes));
    assert!(result.is_err(), "a string is not a WAL version");
    assert!(
        observed.peak <= workspace,
        "escaped error construction peak {} exceeded workspace {}",
        observed.peak,
        workspace
    );
    drop(result);
}

#[test]
fn malformed_enum_payloads_keep_diagnostics_bounded() {
    let large = "\u{0}".repeat(200_000);
    let values = [
        (
            serde_json::json!({"version":1,"entry":{"Index":large}}),
            true,
        ),
        (
            serde_json::json!({"version":1,"entry":{"AddField":{"collection_id":"c","field_name":"f","spec":{"type":{"keyword":large}}}}}),
            false,
        ),
    ];
    for (value, reject_cbor) in values {
        let mut cbor = Vec::new();
        ciborium::ser::into_writer(&value, &mut cbor).unwrap();
        // Ciborium's legacy unit_variant ignores its payload. Keep that
        // accepted input as a compatibility control, not a malformed oracle.
        if !reject_cbor {
            assert_bounds(&cbor);
        }
        let inputs = reject_cbor
            .then_some(cbor)
            .into_iter()
            .chain(std::iter::once(serde_json::to_vec(&value).unwrap()));
        for bytes in inputs {
            let workspace = super::scan_workspace_bound(&bytes).unwrap();
            let (result, observed) = measure(|| super::decoded_peak_bound(&bytes));
            assert!(result.is_err());
            assert!(
                observed.peak <= workspace,
                "enum diagnostic peak {} exceeded {}",
                observed.peak,
                workspace
            );
            drop(result);
        }
    }
}

#[test]
fn malformed_cbor_byte_value_keeps_diagnostics_bounded() {
    use ciborium::value::Value;
    fn replace(v: &mut Value) -> bool {
        match v {
            Value::Map(entries) => {
                for (key, value) in entries {
                    if key.as_text() == Some("value") {
                        *value = Value::Bytes(vec![255; 200_000]);
                        return true;
                    }
                    if replace(value) {
                        return true;
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    if replace(value) {
                        return true;
                    }
                }
            }
            _ => (),
        }
        false
    }
    let mut value: Value =
        ciborium::de::from_reader(generic_cbor(&generic_index()).as_slice()).unwrap();
    assert!(replace(&mut value));
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&value, &mut bytes).unwrap();
    let workspace = super::scan_workspace_bound(&bytes).unwrap();
    let (result, observed) = measure(|| super::decoded_peak_bound(&bytes));
    assert!(result.is_err());
    assert!(
        observed.peak <= workspace,
        "byte diagnostic peak {} exceeded {}",
        observed.peak,
        workspace
    );
}
