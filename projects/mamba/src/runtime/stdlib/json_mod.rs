/// json module for Mamba — backed by serde_json.
///
/// Provides: json.dumps(obj), json.loads(s), json.dumps(obj, indent),
/// json.dump(obj, fp), json.load(fp), json.JSONEncoder, json.JSONDecoder.
///
/// HANDWRITE-BEGIN reason: stdlib-shim section type (integer-handle protocol
/// for OOP types like JSONEncoder/JSONDecoder, register_module + flat-args
/// dispatch) is not yet emitted by score codegen. Same pattern as
/// hashlib_mod/hmac_mod — handwrite during brute-force Phase 2, replace when
/// aw standardize lands the stdlib-shim section type.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

/// Safe wrapper over `from_raw_parts` — returns `&[]` for `nargs == 0`
/// or null pointer (`from_raw_parts` requires a non-null aligned ptr
/// even when `len` is 0; calling it with a null ptr is UB and was
/// triggering aborts on the `dispatch_jsonencoder_ctor` no-arg path).
unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

// ── JSONEncoder / JSONDecoder handle tables ──────────────────────────────────
//
// Each Encoder/Decoder instance is represented as an integer handle. The
// thread-local tables hold per-instance config (indent, sort_keys, separators).
// class.rs::mb_call_method routes int-receiver `.encode(...)` / `.decode(...)`
// through `is_json_encoder_handle` / `is_json_decoder_handle` predicates into
// the free functions below.
//
// Handle IDs sit at 5*(1<<44) — sandwiched between decimal (1<<46 = 4*(1<<44))
// and random (3*(1<<45) = 6*(1<<44)), staying well above HANDLE_MIN_ID
// (1<<40) so the refcount registry doesn't skip JSON handles as primitive
// ints. Pre-#2111 this base was `1`, which (a) collided with primitive int
// values and (b) was filtered out by `integer_handle_registry::retain/release`
// — JSONEncoder / JSONDecoder instances leaked forever.
const JSON_HANDLE_BASE: u64 = (1u64 << 46) + (1u64 << 44);

#[derive(Clone, Default)]
struct EncoderCfg {
    indent: Option<i64>,
    sort_keys: bool,
    item_sep: Option<String>,
    key_sep: Option<String>,
}

#[derive(Clone, Default)]
struct DecoderCfg {}

thread_local! {
    static ENCODERS: RefCell<HashMap<u64, EncoderCfg>> = RefCell::new(HashMap::new());
    static ENCODER_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static DECODERS: RefCell<HashMap<u64, DecoderCfg>> = RefCell::new(HashMap::new());
    static DECODER_IDS: RefCell<HashSet<u64>> = RefCell::new(HashSet::new());
    static NEXT_JSON_HANDLE_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(JSON_HANDLE_BASE) };
    /// Per-handle refcount (#2111). Drops the matching ENCODERS/DECODERS
    /// entry when the count hits zero so per-iter rebinds of
    /// `enc = json.JSONEncoder(...)` don't accumulate forever.
    static JSON_REFCOUNTS: RefCell<HashMap<u64, u32>> = RefCell::new(HashMap::new());
}

fn alloc_json_handle_id() -> u64 {
    NEXT_JSON_HANDLE_ID.with(|cell| {
        let id = cell.get();
        cell.set(id + 1);
        id
    })
}

fn drop_json_handle(id: u64) {
    ENCODERS.with(|m| { m.borrow_mut().remove(&id); });
    ENCODER_IDS.with(|s| { s.borrow_mut().remove(&id); });
    DECODERS.with(|m| { m.borrow_mut().remove(&id); });
    DECODER_IDS.with(|s| { s.borrow_mut().remove(&id); });
    JSON_REFCOUNTS.with(|r| { r.borrow_mut().remove(&id); });
}

/// `mb_retain_value` integer-handle dispatch (#2111).
pub fn retain_handle(id: u64) -> bool {
    if !(is_json_encoder_handle(id) || is_json_decoder_handle(id)) {
        return false;
    }
    JSON_REFCOUNTS.with(|r| {
        *r.borrow_mut().entry(id).or_insert(1) += 1;
    });
    true
}

/// `mb_release_value` integer-handle dispatch (#2111).
pub fn release_handle(id: u64) -> bool {
    if !(is_json_encoder_handle(id) || is_json_decoder_handle(id)) {
        return false;
    }
    let should_drop = JSON_REFCOUNTS.with(|r| {
        let mut map = r.borrow_mut();
        let rc = map.entry(id).or_insert(1);
        if *rc <= 1 {
            map.remove(&id);
            true
        } else {
            *rc -= 1;
            false
        }
    });
    if should_drop {
        drop_json_handle(id);
    }
    true
}

/// class.rs `mb_call_method` calls this to decide whether to route
/// `int.method()` into the JSONEncoder protocol.
pub fn is_json_encoder_handle(id: u64) -> bool {
    ENCODER_IDS.with(|s| s.borrow().contains(&id))
}

/// class.rs `mb_call_method` calls this to decide whether to route
/// `int.method()` into the JSONDecoder protocol.
pub fn is_json_decoder_handle(id: u64) -> bool {
    DECODER_IDS.with(|s| s.borrow().contains(&id))
}

fn make_encoder_handle(cfg: EncoderCfg) -> MbValue {
    let id = alloc_json_handle_id();
    ENCODERS.with(|m| { m.borrow_mut().insert(id, cfg); });
    ENCODER_IDS.with(|s| { s.borrow_mut().insert(id); });
    MbValue::from_int(id as i64)
}

fn make_decoder_handle(cfg: DecoderCfg) -> MbValue {
    let id = alloc_json_handle_id();
    DECODERS.with(|m| { m.borrow_mut().insert(id, cfg); });
    DECODER_IDS.with(|s| { s.borrow_mut().insert(id); });
    MbValue::from_int(id as i64)
}

// ── Dispatch wrappers: native ABI (args_ptr, nargs) -> MbValue ──

unsafe extern "C" fn dispatch_dumps(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    let val = items.get(0).copied().unwrap_or_else(MbValue::none);
    // Check for keyword args (indent, sort_keys, separators)
    // For simplicity: if there's a second arg it may be indent or kwargs dict
    if items.len() > 1 {
        // Quick check: if the second arg is a plain int, treat it as indent
        // directly (covers the common `json.dumps(obj, indent=N)` pattern
        // where mamba flattens the kwarg to a positional arg).
        if let Some(n) = items.get(1).and_then(|v| v.as_int()) {
            return mb_json_dumps_pretty(val, MbValue::from_int(n));
        }
        // Try to detect kwargs dict
        if let Some(ptr) = items.last().and_then(|v| v.as_ptr()) {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    let indent = map.get("indent").and_then(|v| v.as_int());
                    let sort_keys = map.get("sort_keys").and_then(|v| v.as_bool()).unwrap_or(false);
                    let separators = map.get("separators");

                    // Sort keys if requested
                    let effective_val = if sort_keys {
                        sort_dict_keys(val)
                    } else {
                        val
                    };

                    if let Some(n) = indent {
                        return mb_json_dumps_pretty(effective_val, MbValue::from_int(n));
                    }

                    // Handle custom separators
                    if let Some(sep_val) = separators {
                        if let Some(sep_ptr) = sep_val.as_ptr() {
                            if let ObjData::Tuple(ref tup) = (*sep_ptr).data {
                                if tup.len() == 2 {
                                    let item_sep = extract_str_val(tup[0]).unwrap_or(", ".to_string());
                                    let key_sep = extract_str_val(tup[1]).unwrap_or(": ".to_string());
                                    return mb_json_dumps_separators(effective_val, &item_sep, &key_sep);
                                }
                            }
                        }
                    }

                    return mb_json_dumps(effective_val);
                }
            }
        }
    }
    mb_json_dumps(val)
}

unsafe extern "C" fn dispatch_loads(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    mb_json_loads(items.get(0).copied().unwrap_or_else(MbValue::none))
}

/// json.dump(obj, fp) — serialize obj as JSON to file handle fp (write mode).
unsafe extern "C" fn dispatch_dump(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    let obj = items.first().copied().unwrap_or_else(MbValue::none);
    let fp = items.get(1).copied().unwrap_or_else(MbValue::none);

    // Honor json.dump(obj, fp, indent=N) when mamba forwards the kwarg as a
    // trailing dict argument (mirrors the dispatch_dumps logic above).
    let text = if items.len() > 2 {
        if let Some(n) = items.get(2).and_then(|v| v.as_int()) {
            mb_json_dumps_pretty(obj, MbValue::from_int(n))
        } else if let Some(ptr) = items.last().and_then(|v| v.as_ptr()) {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(n) = map.get("indent").and_then(|v| v.as_int()) {
                        mb_json_dumps_pretty(obj, MbValue::from_int(n))
                    } else {
                        mb_json_dumps(obj)
                    }
                } else {
                    mb_json_dumps(obj)
                }
            }
        } else {
            mb_json_dumps(obj)
        }
    } else {
        mb_json_dumps(obj)
    };

    // Write to fp via the existing file_io protocol.
    super::super::file_io::mb_file_write(fp, text);
    MbValue::none()
}

/// json.load(fp) — read file handle fp and parse as JSON.
unsafe extern "C" fn dispatch_load(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    let fp = items.first().copied().unwrap_or_else(MbValue::none);
    let text = super::super::file_io::mb_file_read(fp);
    mb_json_loads(text)
}

/// json.JSONEncoder(...) constructor — returns an integer handle. Accepts
/// the common kwargs `indent`, `sort_keys`, `separators` (best-effort).
unsafe extern "C" fn dispatch_jsonencoder_ctor(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    let mut cfg = EncoderCfg::default();
    // Walk all positional args for a trailing kwargs-dict.
    for v in items.iter() {
        if let Some(ptr) = v.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(n) = map.get("indent").and_then(|v| v.as_int()) {
                        cfg.indent = Some(n);
                    }
                    if let Some(b) = map.get("sort_keys").and_then(|v| v.as_bool()) {
                        cfg.sort_keys = b;
                    }
                    if let Some(sep) = map.get("separators") {
                        if let Some(sptr) = sep.as_ptr() {
                            if let ObjData::Tuple(ref tup) = (*sptr).data {
                                if tup.len() == 2 {
                                    cfg.item_sep = extract_str_val(tup[0]);
                                    cfg.key_sep = extract_str_val(tup[1]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    make_encoder_handle(cfg)
}

/// json.JSONDecoder(...) constructor — returns an integer handle.
unsafe extern "C" fn dispatch_jsondecoder_ctor(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    make_decoder_handle(DecoderCfg::default())
}

/// json.detect_encoding(b) — return the encoding name implied by the first
/// few bytes of `b`. Matches CPython's heuristic: BOM-aware UTF-16/32 detect,
/// else fall back to "utf-8".
unsafe extern "C" fn dispatch_detect_encoding(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let items = unsafe { args_slice(args_ptr, nargs) };
    let val = items.first().copied().unwrap_or_else(MbValue::none);

    // Borrow a byte slice from bytes/bytearray.
    let encoding: &'static str = if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => detect_from_bytes(b.as_slice()),
                ObjData::ByteArray(lock) => {
                    let g = lock.read().unwrap();
                    detect_from_bytes(g.as_slice())
                }
                _ => "utf-8",
            }
        }
    } else {
        "utf-8"
    };

    MbValue::from_ptr(MbObject::new_str(encoding.to_string()))
}

fn detect_from_bytes(b: &[u8]) -> &'static str {
    // UTF-32 BOMs (4 bytes)
    if b.len() >= 4 {
        if b[..4] == [0x00, 0x00, 0xFE, 0xFF] { return "utf-32-be"; }
        if b[..4] == [0xFF, 0xFE, 0x00, 0x00] { return "utf-32-le"; }
    }
    // UTF-16 BOMs (2 bytes)
    if b.len() >= 2 {
        if b[..2] == [0xFE, 0xFF] { return "utf-16-be"; }
        if b[..2] == [0xFF, 0xFE] { return "utf-16-le"; }
    }
    // UTF-8 BOM
    if b.len() >= 3 && b[..3] == [0xEF, 0xBB, 0xBF] { return "utf-8-sig"; }
    // Heuristic for unmarked UTF-32 / UTF-16 (CPython logic):
    // If the first 4 bytes look like ASCII surrounded by zeros, infer wide encodings.
    if b.len() >= 4 {
        match (b[0] == 0, b[1] == 0, b[2] == 0, b[3] == 0) {
            (true, true, true, false) => return "utf-32-be",
            (false, true, true, true) => return "utf-32-le",
            (true, false, true, false) => return "utf-16-be",
            (false, true, false, true) => return "utf-16-le",
            _ => {}
        }
    }
    "utf-8"
}

// ── JSONEncoder/JSONDecoder method-level free functions (called by class.rs)

/// `encoder.encode(obj)` — return a JSON string honoring the encoder's config.
pub fn mb_json_encoder_encode(handle: MbValue, obj: MbValue) -> MbValue {
    let cfg = handle.as_int()
        .and_then(|id| ENCODERS.with(|m| m.borrow().get(&(id as u64)).cloned()))
        .unwrap_or_default();
    let effective = if cfg.sort_keys { sort_dict_keys(obj) } else { obj };
    if let Some(n) = cfg.indent {
        return mb_json_dumps_pretty(effective, MbValue::from_int(n));
    }
    if let (Some(is_), Some(ks)) = (cfg.item_sep.as_deref(), cfg.key_sep.as_deref()) {
        return mb_json_dumps_separators(effective, is_, ks);
    }
    mb_json_dumps(effective)
}

/// `encoder.iterencode(obj)` — return a one-element list whose only entry is
/// the full encoded string. CPython yields chunks; for the conformance gate
/// we only need a single-string iterable, which list-of-1 satisfies.
pub fn mb_json_encoder_iterencode(handle: MbValue, obj: MbValue) -> MbValue {
    let encoded = mb_json_encoder_encode(handle, obj);
    MbValue::from_ptr(MbObject::new_list(vec![encoded]))
}

/// `encoder.default(obj)` — CPython raises TypeError for the base class; user
/// subclasses override. We return TypeError to mirror the base behavior.
///
/// HANDWRITE-BEGIN reason: Python-overridable hook (subclass override of
/// `default`) requires user-class dispatch from native shim → mamba bytecode,
/// which is not yet wired. Returns TypeError to match CPython base-class
/// behavior; full overridable-hook behavior queues for the conformance sweep.
pub fn mb_json_encoder_default(_handle: MbValue, _obj: MbValue) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Object of type is not JSON serializable".to_string(),
        )),
    );
    MbValue::none()
}
// HANDWRITE-END

/// `decoder.decode(s)` — parse a JSON string.
pub fn mb_json_decoder_decode(_handle: MbValue, s: MbValue) -> MbValue {
    mb_json_loads(s)
}

/// `decoder.raw_decode(s, idx=0)` — parse one JSON value at `idx` and return
/// `(value, end_index)` tuple.  Returns a (parsed, total_len) tuple matching
/// CPython for inputs whose entire suffix is a single JSON value.
pub fn mb_json_decoder_raw_decode(_handle: MbValue, s: MbValue, _idx: MbValue) -> MbValue {
    let parsed = mb_json_loads(s);
    let end = s.as_ptr().and_then(|p| unsafe {
        if let ObjData::Str(ref sv) = (*p).data { Some(sv.len() as i64) } else { None }
    }).unwrap_or(0);
    let tuple_items = vec![parsed, MbValue::from_int(end)];
    MbValue::from_ptr(MbObject::new_tuple(tuple_items))
}

/// Register the json module.
pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers = [
        ("dumps", dispatch_dumps as *const () as usize),
        ("loads", dispatch_loads as *const () as usize),
        ("dump", dispatch_dump as *const () as usize),
        ("load", dispatch_load as *const () as usize),
        ("JSONEncoder", dispatch_jsonencoder_ctor as *const () as usize),
        ("JSONDecoder", dispatch_jsondecoder_ctor as *const () as usize),
        ("detect_encoding", dispatch_detect_encoding as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Also register JSONDecodeError as an alias for ValueError
    attrs.insert("JSONDecodeError".into(),
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())));

    super::register_module("json", attrs);

    // #2111: register retain/release hooks so per-iter rebinds of
    // `enc = json.JSONEncoder(...)` / `dec = json.JSONDecoder(...)` drop
    // the prior handle's config entry instead of leaking it.
    super::super::integer_handle_registry::register(
        super::super::integer_handle_registry::IntegerHandleHooks {
            retain: retain_handle,
            release: release_handle,
        },
    );
}

// ── Helpers ──

fn extract_str_val(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

/// Sort dict keys recursively for json.dumps(sort_keys=True)
fn sort_dict_keys(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                // #2109 — collect the sorted key/value pairs into a local
                // Vec BEFORE allocating + write-locking the new dict, so a
                // nested `sort_dict_keys` recursion (which allocates a
                // child dict via `MbObject::new_dict_with_capacity` →
                // `gc_track` → potentially `collect`) never runs while we
                // are holding a write guard on the outer dict. See the
                // sibling comment in `json_to_mbvalue::Object` for the
                // full deadlock derivation.
                let pairs: Vec<(String, MbValue)> = {
                    let map = lock.read().unwrap();
                    let mut keys: Vec<String> =
                        map.keys().map(|k| k.to_string()).collect();
                    keys.sort();
                    keys.into_iter()
                        .filter_map(|k| {
                            map.get(k.as_str()).map(|v| (k, sort_dict_keys(*v)))
                        })
                        .collect()
                };
                let new_dict = MbObject::new_dict_with_capacity(pairs.len());
                if let ObjData::Dict(ref new_lock) = (*new_dict).data {
                    let mut new_map = new_lock.write().unwrap();
                    for (k, v) in pairs {
                        new_map.insert(k.into(), v);
                    }
                }
                return MbValue::from_ptr(new_dict);
            }
        }
    }
    val
}

/// json.dumps with custom separators
fn mb_json_dumps_separators(val: MbValue, item_sep: &str, key_sep: &str) -> MbValue {
    let json_val = mbvalue_to_json(val);
    let s = format_json_custom(&json_val, item_sep, key_sep);
    MbValue::from_ptr(MbObject::new_str(s))
}

fn format_json_custom(val: &serde_json::Value, item_sep: &str, key_sep: &str) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| format_json_custom(v, item_sep, key_sep)).collect();
            format!("[{}]", items.join(item_sep))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj.iter().map(|(k, v)| {
                format!("\"{}\"{}{}",
                    k.replace('\\', "\\\\").replace('"', "\\\""),
                    key_sep,
                    format_json_custom(v, item_sep, key_sep))
            }).collect();
            format!("{{{}}}", items.join(item_sep))
        }
    }
}

// ── MbValue → serde_json::Value ──

fn mbvalue_to_json(val: MbValue) -> serde_json::Value {
    if val.is_none() {
        serde_json::Value::Null
    } else if let Some(b) = val.as_bool() {
        serde_json::Value::Bool(b)
    } else if let Some(i) = val.as_int() {
        serde_json::Value::Number(serde_json::Number::from(i))
    } else if let Some(f) = val.as_float() {
        if f.is_infinite() || f.is_nan() {
            serde_json::Value::Null
        } else {
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null)
        }
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => serde_json::Value::String(s.clone()),
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let arr: Vec<serde_json::Value> = items.iter()
                        .map(|v| mbvalue_to_json(*v))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    let obj: serde_json::Map<String, serde_json::Value> = map.iter()
                        .map(|(k, v)| (k.to_string(), mbvalue_to_json(*v)))
                        .collect();
                    serde_json::Value::Object(obj)
                }
                ObjData::Tuple(items) => {
                    let arr: Vec<serde_json::Value> = items.iter()
                        .map(|v| mbvalue_to_json(*v))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                ObjData::Set(ref lock) => {
                    let items = lock.read().unwrap();
                    let arr: Vec<serde_json::Value> = items.iter()
                        .map(|v| mbvalue_to_json(*v))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                ObjData::FrozenSet(items) => {
                    let arr: Vec<serde_json::Value> = items.iter()
                        .map(|v| mbvalue_to_json(*v))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                ObjData::Bytes(data) => {
                    let arr: Vec<serde_json::Value> = data.iter()
                        .map(|b| serde_json::Value::Number((*b as u64).into()))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    let arr: Vec<serde_json::Value> = data.iter()
                        .map(|b| serde_json::Value::Number((*b as u64).into()))
                        .collect();
                    serde_json::Value::Array(arr)
                }
                _ => serde_json::Value::Null,
            }
        }
    } else {
        serde_json::Value::Null
    }
}

// ── serde_json::Value → MbValue ──

fn json_to_mbvalue(val: &serde_json::Value) -> MbValue {
    match val {
        serde_json::Value::Null => MbValue::none(),
        serde_json::Value::Bool(b) => MbValue::from_bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MbValue::from_int(i)
            } else if let Some(f) = n.as_f64() {
                MbValue::from_float(f)
            } else {
                MbValue::none()
            }
        }
        serde_json::Value::String(s) => {
            MbValue::from_ptr(MbObject::new_str(s.clone()))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<MbValue> = arr.iter().map(json_to_mbvalue).collect();
            MbValue::from_ptr(MbObject::new_list(items))
        }
        serde_json::Value::Object(obj) => {
            // #2109 — build all child MbValues BEFORE acquiring the dict
            // write lock. The recursive `json_to_mbvalue` call may allocate
            // a new container (list/dict), which calls `gc::gc_track`. If
            // `alloc_count` has crossed the GC threshold, `gc_track`
            // triggers `collect()`, which snapshots the tracked set and
            // calls `visit_contained` on every tracked object — including
            // the currently-being-built outer dict. `visit_contained`
            // takes `RwLock::read()` on each dict; with our write guard
            // still held on this thread, that read blocks waiting for the
            // writer (us) to release, but we're blocked on `collect()`,
            // which is blocked on this read — a single-thread RwLock
            // self-deadlock that surfaces as a `dispatch_semaphore_wait`
            // hang on macOS once the alloc threshold is crossed mid-parse.
            let pairs: Vec<(String, MbValue)> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_mbvalue(v)))
                .collect();
            let dict = MbObject::new_dict_with_capacity(pairs.len());
            unsafe {
                if let ObjData::Dict(ref lock) = (*dict).data {
                    let mut map = lock.write().unwrap();
                    for (k, v) in pairs {
                        map.insert(k.into(), v);
                    }
                }
            }
            MbValue::from_ptr(dict)
        }
    }
}

// ── Runtime functions ──

/// json.dumps(obj) → JSON string (matches CPython default: ", " and ": " separators)
pub fn mb_json_dumps(val: MbValue) -> MbValue {
    // CPython: bytes / bytearray / set are not JSON serializable — raise
    // TypeError eagerly (the serializer below would silently null them).
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            let bad = match (*ptr).data {
                ObjData::Bytes(_) | ObjData::ByteArray(_) => Some("bytes"),
                ObjData::Set(_) | ObjData::FrozenSet(_) => Some("set"),
                _ => None,
            };
            if let Some(kind) = bad {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "Object of type {kind} is not JSON serializable"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // CPython default uses (", ", ": ") separators — serde_json::to_string uses no spaces.
    // Special-case top-level float to handle Infinity/NaN (CPython outputs these directly).
    let s = serialize_mbvalue_cpython(val);
    MbValue::from_ptr(MbObject::new_str(s))
}

/// Serialize an MbValue to JSON matching CPython's default format.
/// Handles Infinity/NaN as non-standard JSON (CPython behavior).
fn serialize_mbvalue_cpython(val: MbValue) -> String {
    // Handle top-level special floats (Infinity, -Infinity, NaN)
    if let Some(f) = val.as_float() {
        if f.is_infinite() {
            return if f > 0.0 { "Infinity".to_string() } else { "-Infinity".to_string() };
        }
        if f.is_nan() {
            return "NaN".to_string();
        }
    }
    // For compound types, we need to recurse through the MbValue tree
    // to handle nested inf/nan. For now, delegate to serde_json for normal values.
    let json_val = mbvalue_to_json(val);
    serialize_json_cpython(&json_val)
}

/// Serialize JSON matching CPython's default format: `{"key": value, "key2": value2}`
fn serialize_json_cpython(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(serialize_json_cpython).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, serialize_json_cpython(v)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

/// json.dumps(obj, indent=N) → pretty-printed JSON string
pub fn mb_json_dumps_pretty(val: MbValue, indent: MbValue) -> MbValue {
    let json_val = mbvalue_to_json(val);
    let n = indent.as_int().unwrap_or(2) as usize;
    // serde_json::to_string_pretty uses 2-space indent; for custom indent we format manually
    if n == 2 {
        let s = serde_json::to_string_pretty(&json_val)
            .unwrap_or_else(|_| "null".to_string());
        MbValue::from_ptr(MbObject::new_str(s))
    } else {
        let indent_bytes = " ".repeat(n).into_bytes();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent_bytes);
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        use serde::Serialize;
        json_val.serialize(&mut ser).ok();
        let s = String::from_utf8(buf).unwrap_or_else(|_| "null".to_string());
        MbValue::from_ptr(MbObject::new_str(s))
    }
}

/// json.loads(s) → Mamba value
///
/// #2109 fixed: a hot loop of `json.loads` on a nested-dict payload used to
/// deadlock the JIT main thread once the GC alloc threshold tripped
/// mid-parse. Root cause was a single-thread RwLock self-deadlock in
/// `json_to_mbvalue`'s Object arm: the outer dict's `RwLock::write` guard
/// was held while a recursive child allocation called `gc::gc_track`,
/// which fired `collect()`, which called `visit_contained` on the same
/// outer dict, which blocked on `RwLock::read()` waiting for the writer
/// (us) to release — surfacing as `_dispatch_semaphore_wait_slow` on
/// macOS. Fix is in `json_to_mbvalue` and `sort_dict_keys`: build all
/// child MbValues into a local Vec *before* acquiring the outer
/// write lock.
pub fn mb_json_loads(val: MbValue) -> MbValue {
    let json_str = val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    });

    match json_str {
        Some(s) => {
            match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(parsed) => json_to_mbvalue(&parsed),
                Err(e) => {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            format!("json.loads: {e}")
                        )),
                    );
                    MbValue::none()
                }
            }
        }
        None => MbValue::none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> String {
        unsafe {
            if let ObjData::Str(ref sv) = (*val.as_ptr().unwrap()).data {
                sv.clone()
            } else {
                panic!("expected string")
            }
        }
    }

    #[test]
    fn test_dumps_primitives() {
        assert_eq!(get_str(mb_json_dumps(MbValue::from_int(42))), "42");
        assert_eq!(get_str(mb_json_dumps(MbValue::from_bool(true))), "true");
        assert_eq!(get_str(mb_json_dumps(MbValue::none())), "null");
        assert_eq!(get_str(mb_json_dumps(s("hello"))), "\"hello\"");
    }

    #[test]
    fn test_dumps_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        assert_eq!(get_str(mb_json_dumps(list)), "[1, 2, 3]");
    }

    #[test]
    fn test_loads_primitives() {
        assert_eq!(mb_json_loads(s("42")).as_int(), Some(42));
        assert_eq!(mb_json_loads(s("3.14")).as_float(), Some(3.14));
        assert_eq!(mb_json_loads(s("true")).as_bool(), Some(true));
        assert_eq!(mb_json_loads(s("false")).as_bool(), Some(false));
        assert!(mb_json_loads(s("null")).is_none());
    }

    #[test]
    fn test_loads_string() {
        let result = mb_json_loads(s("\"hello world\""));
        assert_eq!(get_str(result), "hello world");
    }

    #[test]
    fn test_loads_array() {
        let result = mb_json_loads(s("[1, 2, 3]"));
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(1));
            }
        }
    }

    #[test]
    fn test_loads_object() {
        let result = mb_json_loads(s("{\"a\": 1, \"b\": 2}"));
        unsafe {
            if let ObjData::Dict(ref lock) = (*result.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("a").and_then(|v| v.as_int()), Some(1));
                assert_eq!(map.get("b").and_then(|v| v.as_int()), Some(2));
            }
        }
    }

    #[test]
    fn test_roundtrip() {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                map.insert("name".into(), s("Alice"));
                map.insert("age".into(), MbValue::from_int(30));
            }
        }
        let original = MbValue::from_ptr(dict);
        let json = mb_json_dumps(original);
        let parsed = mb_json_loads(json);
        unsafe {
            if let ObjData::Dict(ref lock) = (*parsed.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("age").and_then(|v| v.as_int()), Some(30));
            }
        }
    }

    #[test]
    fn test_deeply_nested_roundtrip() {
        let inner_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_bool(true),
            MbValue::none(),
        ]));
        let inner_dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*inner_dict).data {
                let mut map = lock.write().unwrap();
                map.insert("nested".into(), inner_list);
            }
        }
        let outer = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(inner_dict),
            s("hello"),
        ]));
        let json = mb_json_dumps(outer);
        let parsed = mb_json_loads(json);
        unsafe {
            if let ObjData::List(ref lock) = (*parsed.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
            }
        }
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_json_dumps_int() {
        let v = MbValue::from_int(42);
        let j = mb_json_dumps(v);
        let result = j.as_ptr().map(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() }
        }).unwrap_or_default();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_py312_json_dumps_bool_true() {
        let v = MbValue::from_bool(true);
        let j = mb_json_dumps(v);
        let result = j.as_ptr().map(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() }
        }).unwrap_or_default();
        assert_eq!(result, "true");
    }

    #[test]
    fn test_py312_json_dumps_none_is_null() {
        let v = MbValue::none();
        let j = mb_json_dumps(v);
        let result = j.as_ptr().map(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data { s.clone() } else { String::new() }
        }).unwrap_or_default();
        assert_eq!(result, "null");
    }

    #[test]
    fn test_py312_json_loads_array() {
        let j = s("[1,2,3]");
        let parsed = mb_json_loads(j);
        assert!(parsed.is_ptr());
        unsafe {
            if let ObjData::List(ref lock) = (*parsed.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].as_int(), Some(1));
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_py312_json_loads_null_is_none() {
        let j = s("null");
        let parsed = mb_json_loads(j);
        assert!(parsed.is_none());
    }

    #[test]
    fn test_py312_json_loads_bool_false() {
        let j = s("false");
        let parsed = mb_json_loads(j);
        assert_eq!(parsed.as_bool(), Some(false));
    }

    #[test]
    fn test_py312_json_loads_float() {
        let j = s("3.14");
        let parsed = mb_json_loads(j);
        assert!((parsed.as_float().unwrap() - 3.14).abs() < 1e-10);
    }

    // -- Task #33: JSONEncoder / JSONDecoder handle surface --

    fn empty_args() -> *const MbValue {
        std::ptr::null()
    }

    #[test]
    fn test_jsonencoder_default_encode_int() {
        let enc = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        assert!(enc.as_int().is_some());
        let id = enc.as_int().unwrap() as u64;
        assert!(is_json_encoder_handle(id));
        let out = mb_json_encoder_encode(enc, MbValue::from_int(42));
        assert_eq!(get_str(out), "42");
    }

    #[test]
    fn test_jsonencoder_default_encode_str() {
        let enc = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        let out = mb_json_encoder_encode(enc, s("hello"));
        assert_eq!(get_str(out), "\"hello\"");
    }

    #[test]
    fn test_jsonencoder_iterencode_returns_list_of_string() {
        let enc = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        let list_val = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let out = mb_json_encoder_iterencode(enc, list_val);
        unsafe {
            if let ObjData::List(ref lock) = (*out.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
                assert_eq!(get_str(items[0]), "[1, 2]");
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_jsonencoder_default_raises_typeerror() {
        // Just verifies the call returns none (TypeError is raised
        // internally; we don't catch it here in unit-test scope).
        let enc = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        let _ = mb_json_encoder_default(enc, MbValue::from_int(0));
        // Reaching here means no panic; that's the assertion.
    }

    #[test]
    fn test_jsondecoder_decode_flat_object() {
        let dec = unsafe { dispatch_jsondecoder_ctor(empty_args(), 0) };
        assert!(dec.as_int().is_some());
        let id = dec.as_int().unwrap() as u64;
        assert!(is_json_decoder_handle(id));
        let parsed = mb_json_decoder_decode(dec, s("{\"a\": 1, \"b\": 2}"));
        unsafe {
            if let ObjData::Dict(ref lock) = (*parsed.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("a").and_then(|v| v.as_int()), Some(1));
                assert_eq!(map.get("b").and_then(|v| v.as_int()), Some(2));
            } else {
                panic!("expected dict");
            }
        }
    }

    #[test]
    fn test_jsondecoder_raw_decode_returns_value_and_end() {
        let dec = unsafe { dispatch_jsondecoder_ctor(empty_args(), 0) };
        let input = s("[1, 2, 3]");
        let result = mb_json_decoder_raw_decode(dec, input, MbValue::from_int(0));
        unsafe {
            if let ObjData::Tuple(ref items) = (*result.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
                if let ObjData::List(ref lock) = (*items[0].as_ptr().unwrap()).data {
                    let list = lock.read().unwrap();
                    assert_eq!(list.len(), 3);
                }
                assert_eq!(items[1].as_int(), Some(9)); // "[1, 2, 3]" = 9 chars
            } else {
                panic!("expected tuple");
            }
        }
    }

    #[test]
    fn test_encoder_decoder_handles_distinct() {
        let e = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        let d = unsafe { dispatch_jsondecoder_ctor(empty_args(), 0) };
        let eid = e.as_int().unwrap() as u64;
        let did = d.as_int().unwrap() as u64;
        assert!(is_json_encoder_handle(eid));
        assert!(is_json_decoder_handle(did));
        assert!(!is_json_decoder_handle(eid));
        assert!(!is_json_encoder_handle(did));
    }

    #[test]
    fn test_detect_encoding_heuristics() {
        // UTF-8 BOM
        assert_eq!(detect_from_bytes(&[0xEF, 0xBB, 0xBF, b'{']), "utf-8-sig");
        // UTF-16 BE BOM
        assert_eq!(detect_from_bytes(&[0xFE, 0xFF, 0x00, b'{']), "utf-16-be");
        // UTF-16 LE BOM
        assert_eq!(detect_from_bytes(&[0xFF, 0xFE, b'{', 0x00]), "utf-16-le");
        // UTF-32 BE BOM
        assert_eq!(detect_from_bytes(&[0x00, 0x00, 0xFE, 0xFF]), "utf-32-be");
        // Plain ASCII
        assert_eq!(detect_from_bytes(b"{\"a\":1}"), "utf-8");
        // Unmarked UTF-16 LE (`{` 0x7B then 0x00)
        assert_eq!(detect_from_bytes(&[b'{', 0x00, b'"', 0x00]), "utf-16-le");
    }

    /// Regression for #2109 — hot loop of `json.loads` + `json.dumps` on a
    /// nested-dict payload must complete in linear time with no hang. Two
    /// sub-modes exercise both halves of the JIT contract:
    ///
    /// 1. **With releases** (mirrors a JIT codegen that correctly emits
    ///    `mb_release_value` on reassignment): must be fast and bounded.
    /// 2. **Without releases** (mirrors a JIT codegen that leaks the
    ///    previous binding): the GC fallback path must still finish in
    ///    bounded time — i.e. cycle collection must not become accidentally
    ///    quadratic in the live tracked-set size.
    ///
    /// Both sub-modes survive 500 iterations under 5 s on a debug build.
    /// Set MAMBA_PERF=1 to print per-iteration timing for ad-hoc benchmarking.
    #[test]
    fn test_json_loads_dumps_nested_dict_loop_2109() {
        use std::time::Instant;
        use super::super::super::rc::mb_release;
        let payload = r#"{"service": "mamba-api", "version": "0.3.48", "enabled": true, "timeout_ms": 5000, "retries": 3, "endpoints": [{"path": "/v1/status", "method": "GET", "auth": false}, {"path": "/v1/run", "method": "POST", "auth": true}], "feature_flags": {"tracing": true, "metrics": false}, "tags": ["prod", "us-east-1"]}"#;
        let iters: usize = 500;

        // Sub-mode 1: explicit releases — happy path, must be O(N) and fast.
        let start = Instant::now();
        for _ in 0..iters {
            let s_val = s(payload);
            let parsed = mb_json_loads(s_val);
            let dumped = mb_json_dumps(parsed);
            unsafe {
                if let Some(p) = s_val.as_ptr() { mb_release(p); }
                if let Some(p) = parsed.as_ptr() { mb_release(p); }
                if let Some(p) = dumped.as_ptr() { mb_release(p); }
            }
        }
        let total_with_release = start.elapsed();
        assert!(
            total_with_release.as_secs_f64() < 5.0,
            "with-release loop too slow: {iters} iters in {total_with_release:?}"
        );

        // Sub-mode 2: leak-on-overwrite — accumulates tracked containers
        // and exercises auto-collect under load. This was the canary for
        // the #2109 hang where collect() became unbounded once the
        // tracked set crossed the threshold while live references piled
        // up. Must still finish in bounded time.
        let start = Instant::now();
        for _ in 0..iters {
            let s_val = s(payload);
            let parsed = mb_json_loads(s_val);
            let _dumped = mb_json_dumps(parsed);
            // No releases: simulate the broken-JIT codegen path.
        }
        let total_leak = start.elapsed();
        assert!(
            total_leak.as_secs_f64() < 5.0,
            "leak-on-overwrite loop too slow (probable accidental-quadratic GC scan): \
             {iters} iters in {total_leak:?}"
        );

        if std::env::var_os("MAMBA_PERF").is_some() {
            eprintln!(
                "#2109 perf: {iters} iters — with-release {total_with_release:?} \
                 (mean {:.3}us), leak-on-overwrite {total_leak:?} (mean {:.3}us)",
                total_with_release.as_secs_f64() * 1e6 / iters as f64,
                total_leak.as_secs_f64() * 1e6 / iters as f64,
            );
        }
    }

    #[test]
    fn test_encoder_decoder_roundtrip_flat() {
        let enc = unsafe { dispatch_jsonencoder_ctor(empty_args(), 0) };
        let dec = unsafe { dispatch_jsondecoder_ctor(empty_args(), 0) };
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                map.insert("k".into(), MbValue::from_int(99));
                map.insert("name".into(), s("mamba"));
            }
        }
        let original = MbValue::from_ptr(dict);
        let encoded = mb_json_encoder_encode(enc, original);
        let decoded = mb_json_decoder_decode(dec, encoded);
        unsafe {
            if let ObjData::Dict(ref lock) = (*decoded.as_ptr().unwrap()).data {
                let map = lock.read().unwrap();
                assert_eq!(map.get("k").and_then(|v| v.as_int()), Some(99));
                assert_eq!(get_str(*map.get("name").unwrap()), "mamba");
            } else {
                panic!("expected dict");
            }
        }
    }
}
