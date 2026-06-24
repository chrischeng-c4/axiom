use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// dbm module for Mamba — functional `dbm.dumb` backend (#1261 Gate 1).
///
/// `dbm.open` / `dbm.dumb.open` return a real database handle (Instance
/// "dbm.dumb._Database") backed by an in-memory map persisted to two files,
/// `<path>.dir` (hex-encoded `key value` lines) and `<path>.dat` (raw value
/// bytes), so data survives close/reopen and `whichdb` recognizes the pair
/// as "dbm.dumb".
///
/// Semantics covered: flags r/w/c/n (ValueError otherwise; r/w require the
/// files to exist, n truncates), str keys/values encode to bytes and read
/// back AS bytes, KeyError on missing subscript, read-only handles reject
/// writes/deletes ("The database is opened for reading only"), closed
/// handles reject all access ("DBM object has already been closed", double
/// close is a no-op), keys/values/items/get/setdefault/len/contains, and
/// context-manager close.
///
/// `dbm.error` mirrors CPython's tuple-of-exception-classes shape with the
/// lead element subclassing OSError; `dbm.dumb.error` is `class
/// error(OSError)`.
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Path argument: str or bytes.
fn extract_path(val: MbValue) -> Option<String> {
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Bytes(ref b) = (*ptr).data {
            Some(String::from_utf8_lossy(b).to_string())
        } else {
            None
        }
    })
}

/// Key/value coercion: str encodes to UTF-8 bytes; bytes pass through.
fn as_bytes(val: MbValue) -> Option<Vec<u8>> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.as_bytes().to_vec()),
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            _ => None,
        }
    })
}

fn hex_encode(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

fn raise(exc: &str, msg: &str) -> MbValue {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
    MbValue::none()
}

const CLOSED_MSG: &str = "DBM object has already been closed";
const READONLY_MSG: &str = "The database is opened for reading only";

// ── per-handle store ──────────────────────────────────────────────────────────

struct DbState {
    path: String,
    entries: Vec<(Vec<u8>, Vec<u8>)>,
    readonly: bool,
    closed: bool,
}

thread_local! {
    static DBM_STORES: std::cell::RefCell<FxHashMap<u64, DbState>> =
        std::cell::RefCell::new(FxHashMap::default());
    static DBM_NEXT_ID: std::cell::Cell<u64> = const { std::cell::Cell::new(1) };
}

fn handle_id(inst: MbValue) -> Option<u64> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields
                .read()
                .ok()?
                .get("_id")
                .copied()
                .and_then(|v| v.as_int())
                .map(|i| i as u64)
        } else {
            None
        }
    })
}

fn load_entries(path: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string(format!("{path}.dir")) {
        for line in text.lines() {
            let mut it = line.splitn(2, ' ');
            if let (Some(k), Some(v)) = (it.next(), it.next()) {
                if let (Some(kb), Some(vb)) = (hex_decode(k), hex_decode(v)) {
                    out.push((kb, vb));
                }
            }
        }
    }
    out
}

fn persist(state: &DbState) {
    let mut dir = String::new();
    let mut dat: Vec<u8> = Vec::new();
    for (k, v) in &state.entries {
        dir.push_str(&format!("{} {}\n", hex_encode(k), hex_encode(v)));
        dat.extend_from_slice(v);
    }
    let _ = std::fs::write(format!("{}.dir", state.path), dir);
    let _ = std::fs::write(format!("{}.dat", state.path), dat);
}

/// Run `f` with the open handle's state; raises the closed-handle error
/// (using the backend error class) when the handle is closed/invalid.
fn with_state<R>(inst: MbValue, f: impl FnOnce(&mut DbState) -> R) -> Option<R> {
    let id = handle_id(inst)?;
    DBM_STORES.with(|stores| {
        let mut stores = stores.borrow_mut();
        match stores.get_mut(&id) {
            Some(st) if !st.closed => Some(f(st)),
            _ => {
                raise("dbm.dumb.error", CLOSED_MSG);
                None
            }
        }
    })
}

fn args_of(args: MbValue) -> Vec<MbValue> {
    args.as_ptr()
        .and_then(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().ok().map(|g| g.to_vec()),
                ObjData::Tuple(items) => Some(items.clone()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

// ── open / whichdb ────────────────────────────────────────────────────────────

fn db_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let Some(path) = a.first().copied().and_then(extract_path) else {
        return raise("TypeError", "open() requires a filename");
    };
    let flag = a
        .get(1)
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| "r".to_string());
    if !matches!(flag.as_str(), "r" | "w" | "c" | "n") {
        return raise(
            "ValueError",
            &format!("Flag must be one of 'r', 'w', 'c', or 'n', not {flag:?}"),
        );
    }
    let dir_exists = std::path::Path::new(&format!("{path}.dir")).exists();
    if matches!(flag.as_str(), "r" | "w") && !dir_exists {
        return raise(
            "dbm.error",
            &format!("need 'c' or 'n' flag to open new db: {path:?}"),
        );
    }
    let entries = if flag == "n" {
        Vec::new()
    } else {
        load_entries(&path)
    };
    let state = DbState {
        path: path.clone(),
        entries,
        readonly: flag == "r",
        closed: false,
    };
    // Create/truncate the on-disk pair immediately so whichdb sees it.
    persist(&state);
    let id = DBM_NEXT_ID.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    });
    DBM_STORES.with(|stores| {
        stores.borrow_mut().insert(id, state);
    });
    let mut fields = FxHashMap::default();
    fields.insert("_id".to_string(), MbValue::from_int(id as i64));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "dbm.dumb._Database".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    db_open(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_dumb_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    db_open(args_ptr, nargs)
}

unsafe extern "C" fn dispatch_whichdb(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    let Some(path) = a.first().copied().and_then(extract_path) else {
        return MbValue::none();
    };
    let dir = std::path::Path::new(&format!("{path}.dir")).exists();
    let dat = std::path::Path::new(&format!("{path}.dat")).exists();
    if dir && dat {
        return new_str("dbm.dumb");
    }
    // An empty bare `.db` file is not identifiable (issue 17198) → None.
    MbValue::none()
}

// ── handle methods (variadic (self, args_list) ABI) ──────────────────────────

fn key_arg(args: MbValue) -> Option<Vec<u8>> {
    args_of(args).first().copied().and_then(as_bytes)
}

unsafe extern "C" fn db_setitem(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_of(args);
    let (Some(k), Some(v)) = (
        a.first().copied().and_then(as_bytes),
        a.get(1).copied().and_then(as_bytes),
    ) else {
        return raise("TypeError", "keys and values must be bytes or strings");
    };
    with_state(self_v, |st| {
        if st.readonly {
            raise("dbm.dumb.error", READONLY_MSG);
            return;
        }
        if let Some(slot) = st.entries.iter_mut().find(|(ek, _)| *ek == k) {
            slot.1 = v;
        } else {
            st.entries.push((k, v));
        }
        persist(st);
    });
    MbValue::none()
}

unsafe extern "C" fn db_getitem(self_v: MbValue, args: MbValue) -> MbValue {
    let Some(k) = key_arg(args) else {
        return raise("TypeError", "keys must be bytes or strings");
    };
    with_state(self_v, |st| {
        st.entries
            .iter()
            .find(|(ek, _)| *ek == k)
            .map(|(_, v)| v.clone())
    })
    .map(|found| match found {
        Some(v) => MbValue::from_ptr(MbObject::new_bytes(v)),
        None => raise("KeyError", &String::from_utf8_lossy(&k)),
    })
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_delitem(self_v: MbValue, args: MbValue) -> MbValue {
    let Some(k) = key_arg(args) else {
        return raise("TypeError", "keys must be bytes or strings");
    };
    with_state(self_v, |st| {
        if st.readonly {
            raise("dbm.dumb.error", READONLY_MSG);
            return false;
        }
        let before = st.entries.len();
        st.entries.retain(|(ek, _)| *ek != k);
        let removed = st.entries.len() != before;
        if removed {
            persist(st);
        } else {
            raise("KeyError", &String::from_utf8_lossy(&k));
        }
        removed
    });
    MbValue::none()
}

unsafe extern "C" fn db_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let Some(k) = key_arg(args) else {
        return MbValue::from_bool(false);
    };
    with_state(self_v, |st| st.entries.iter().any(|(ek, _)| *ek == k))
        .map(MbValue::from_bool)
        .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_len(self_v: MbValue, _args: MbValue) -> MbValue {
    with_state(self_v, |st| st.entries.len() as i64)
        .map(MbValue::from_int)
        .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_keys(self_v: MbValue, _args: MbValue) -> MbValue {
    with_state(self_v, |st| {
        st.entries
            .iter()
            .map(|(k, _)| MbValue::from_ptr(MbObject::new_bytes(k.clone())))
            .collect::<Vec<_>>()
    })
    .map(|items| MbValue::from_ptr(MbObject::new_list(items)))
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_values(self_v: MbValue, _args: MbValue) -> MbValue {
    with_state(self_v, |st| {
        st.entries
            .iter()
            .map(|(_, v)| MbValue::from_ptr(MbObject::new_bytes(v.clone())))
            .collect::<Vec<_>>()
    })
    .map(|items| MbValue::from_ptr(MbObject::new_list(items)))
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_items(self_v: MbValue, _args: MbValue) -> MbValue {
    with_state(self_v, |st| {
        st.entries
            .iter()
            .map(|(k, v)| {
                MbValue::from_ptr(MbObject::new_tuple(vec![
                    MbValue::from_ptr(MbObject::new_bytes(k.clone())),
                    MbValue::from_ptr(MbObject::new_bytes(v.clone())),
                ]))
            })
            .collect::<Vec<_>>()
    })
    .map(|items| MbValue::from_ptr(MbObject::new_list(items)))
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_get(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_of(args);
    let Some(k) = a.first().copied().and_then(as_bytes) else {
        return MbValue::none();
    };
    let default = a.get(1).copied().unwrap_or_else(MbValue::none);
    with_state(self_v, |st| {
        st.entries
            .iter()
            .find(|(ek, _)| *ek == k)
            .map(|(_, v)| v.clone())
    })
    .map(|found| match found {
        Some(v) => MbValue::from_ptr(MbObject::new_bytes(v)),
        None => default,
    })
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_setdefault(self_v: MbValue, args: MbValue) -> MbValue {
    let a = args_of(args);
    let (Some(k), Some(d)) = (
        a.first().copied().and_then(as_bytes),
        a.get(1).copied().and_then(as_bytes),
    ) else {
        return raise("TypeError", "keys and values must be bytes or strings");
    };
    with_state(self_v, |st| {
        if let Some((_, v)) = st.entries.iter().find(|(ek, _)| *ek == k) {
            v.clone()
        } else {
            if st.readonly {
                raise("dbm.dumb.error", READONLY_MSG);
                return Vec::new();
            }
            st.entries.push((k, d.clone()));
            persist(st);
            d
        }
    })
    .map(|v| MbValue::from_ptr(MbObject::new_bytes(v)))
    .unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn db_close(self_v: MbValue, _args: MbValue) -> MbValue {
    if let Some(id) = handle_id(self_v) {
        DBM_STORES.with(|stores| {
            let mut stores = stores.borrow_mut();
            if let Some(st) = stores.get_mut(&id) {
                // Double close is a no-op.
                if !st.closed {
                    persist(st);
                    st.closed = true;
                }
            }
        });
    }
    MbValue::none()
}

unsafe extern "C" fn db_sync(self_v: MbValue, _args: MbValue) -> MbValue {
    with_state(self_v, |st| persist(st));
    MbValue::none()
}

unsafe extern "C" fn db_enter(self_v: MbValue, _args: MbValue) -> MbValue {
    self_v
}

unsafe extern "C" fn db_exit(self_v: MbValue, args: MbValue) -> MbValue {
    unsafe { db_close(self_v, args) };
    MbValue::from_bool(false)
}

// ── Registration ──────────────────────────────────────────────────────────────

/// Register the dbm module (and the `dbm.dumb` submodule).
pub fn register() {
    register_db_class();

    let mut attrs = HashMap::new();

    let addr_o = dispatch_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_o));

    let addr_w = dispatch_whichdb as *const () as usize;
    attrs.insert("whichdb".into(), MbValue::from_func(addr_w));

    // `dbm.error` is a tuple of exception classes on CPython, with the lead
    // element being a real `OSError` subclass.
    super::super::class::mb_class_register(
        "dbm.error",
        vec!["OSError".to_string()],
        HashMap::new(),
    );
    let error_tuple = MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str("dbm.error".to_string())),
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
    ]));
    attrs.insert("error".into(), error_tuple);

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_o as u64);
        set.insert(addr_w as u64);
    });

    super::register_module("dbm", attrs);

    register_dumb();
}

fn register_db_class() {
    use std::collections::HashMap as Map;
    let var = |addr: usize| {
        super::super::module::register_variadic_func(addr as u64);
        MbValue::from_func(addr)
    };
    let mut m: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("__setitem__", db_setitem as *const () as usize),
        ("__getitem__", db_getitem as *const () as usize),
        ("__delitem__", db_delitem as *const () as usize),
        ("__contains__", db_contains as *const () as usize),
        ("__len__", db_len as *const () as usize),
        ("__enter__", db_enter as *const () as usize),
        ("__exit__", db_exit as *const () as usize),
        ("keys", db_keys as *const () as usize),
        ("values", db_values as *const () as usize),
        ("items", db_items as *const () as usize),
        ("get", db_get as *const () as usize),
        ("setdefault", db_setdefault as *const () as usize),
        ("close", db_close as *const () as usize),
        ("sync", db_sync as *const () as usize),
    ] {
        m.insert(name.to_string(), var(addr));
    }
    super::super::class::mb_class_register("dbm.dumb._Database", vec![], m);
}

/// Register the `dbm.dumb` pure-Python backend submodule.
fn register_dumb() {
    let mut attrs = HashMap::new();

    let addr_d = dispatch_dumb_open as *const () as usize;
    attrs.insert("open".into(), MbValue::from_func(addr_d));

    // `dbm.dumb.error` is `class error(OSError)`.
    super::super::class::mb_class_register(
        "dbm.dumb.error",
        vec!["OSError".to_string()],
        HashMap::new(),
    );
    attrs.insert(
        "error".into(),
        MbValue::from_ptr(MbObject::new_str("dbm.dumb.error".to_string())),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr_d as u64);
    });

    super::register_module("dbm.dumb", attrs);
}
