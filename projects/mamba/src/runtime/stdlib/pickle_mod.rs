use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// pickle module for Mamba (#442).
///
/// Provides: dumps(obj) -> bytes, loads(data) -> obj
/// Simple serialization using a compact text-based format.
/// Not compatible with CPython pickle protocol.
use std::collections::HashMap;

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! disp_binary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

disp_unary!(d_dumps, mb_pickle_dumps);
disp_unary!(d_loads, mb_pickle_loads);
disp_binary!(d_dump, mb_pickle_dump);
disp_unary!(d_load, mb_pickle_load);

/// Register the pickle module.
///
/// Wave-4 Ship #3 (Task #55) — extends from 4-dispatcher / 2-constant
/// surface to the full typeshed 70-entry surface:
/// - 4 dispatchers: dumps, loads, dump, load (NATIVE_FUNC_ADDRS-bound)
/// - 2 protocol constants: HIGHEST_PROTOCOL, DEFAULT_PROTOCOL (=5)
/// - 58 opcode byte constants (MARK, STOP, POP, ..., NEXT_BUFFER,
///   READONLY_BUFFER). Each is a `bytes` literal mirroring CPython's
///   `pickle.<NAME>` so byte-protocol consumers can pattern-match
///   against the wire format.
/// - 6 class shells (Instances with class_name "pickle.<Name>"):
///   PickleBuffer, PickleError, PicklingError, UnpicklingError,
///   Pickler, Unpickler. These are class shells — `pickle.Pickler()`
///   construction is **out of scope** for the forward ship; the
///   forward ship covers module-level `dumps`/`loads`.
///
/// **Subset B mem FAIL by-design** per the scout (precedent: array
/// Task #35, json Task #29). The full opcode table emits ~58
/// bytes Instances at register time (one-shot cost, not per-call).
/// Per-call cost: pickling a 100-element list emits 100+ bytes
/// allocations.
pub fn register() {
    use super::super::module::NATIVE_FUNC_ADDRS;

    let mut attrs = HashMap::new();

    // Protocol constants.
    attrs.insert("HIGHEST_PROTOCOL".into(), MbValue::from_int(5));
    attrs.insert("DEFAULT_PROTOCOL".into(), MbValue::from_int(5));

    // Dispatchers.
    let dispatchers: &[(&str, usize)] = &[
        ("dumps", d_dumps as *const () as usize),
        ("loads", d_loads as *const () as usize),
        ("dump", d_dump as *const () as usize),
        ("load", d_load as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(*addr));
        NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // Opcode byte constants. Each entry is (name, raw bytes).
    let bytes_const = |b: &[u8]| MbValue::from_ptr(MbObject::new_bytes(b.to_vec()));
    let opcodes: &[(&str, &[u8])] = &[
        ("MARK", b"("),
        ("STOP", b"."),
        ("POP", b"0"),
        ("POP_MARK", b"1"),
        ("DUP", b"2"),
        ("FLOAT", b"F"),
        ("INT", b"I"),
        ("BININT", b"J"),
        ("BININT1", b"K"),
        ("LONG", b"L"),
        ("BININT2", b"M"),
        ("NONE", b"N"),
        ("PERSID", b"P"),
        ("BINPERSID", b"Q"),
        ("REDUCE", b"R"),
        ("STRING", b"S"),
        ("BINSTRING", b"T"),
        ("SHORT_BINSTRING", b"U"),
        ("UNICODE", b"V"),
        ("BINUNICODE", b"X"),
        ("APPEND", b"a"),
        ("BUILD", b"b"),
        ("GLOBAL", b"c"),
        ("DICT", b"d"),
        ("EMPTY_DICT", b"}"),
        ("APPENDS", b"e"),
        ("GET", b"g"),
        ("BINGET", b"h"),
        ("INST", b"i"),
        ("LONG_BINGET", b"j"),
        ("LIST", b"l"),
        ("EMPTY_LIST", b"]"),
        ("OBJ", b"o"),
        ("PUT", b"p"),
        ("BINPUT", b"q"),
        ("LONG_BINPUT", b"r"),
        ("SETITEM", b"s"),
        ("TUPLE", b"t"),
        ("EMPTY_TUPLE", b")"),
        ("SETITEMS", b"u"),
        ("BINFLOAT", b"G"),
        ("TRUE", b"I01\n"),
        ("FALSE", b"I00\n"),
        ("PROTO", b"\x80"),
        ("NEWOBJ", b"\x81"),
        ("EXT1", b"\x82"),
        ("EXT2", b"\x83"),
        ("EXT4", b"\x84"),
        ("TUPLE1", b"\x85"),
        ("TUPLE2", b"\x86"),
        ("TUPLE3", b"\x87"),
        ("NEWTRUE", b"\x88"),
        ("NEWFALSE", b"\x89"),
        ("LONG1", b"\x8a"),
        ("LONG4", b"\x8b"),
        ("BINBYTES", b"B"),
        ("SHORT_BINBYTES", b"C"),
        ("SHORT_BINUNICODE", b"\x8c"),
        ("BINUNICODE8", b"\x8d"),
        ("BINBYTES8", b"\x8e"),
        ("EMPTY_SET", b"\x8f"),
        ("ADDITEMS", b"\x90"),
        ("FROZENSET", b"\x91"),
        ("NEWOBJ_EX", b"\x92"),
        ("STACK_GLOBAL", b"\x93"),
        ("MEMOIZE", b"\x94"),
        ("FRAME", b"\x95"),
        ("BYTEARRAY8", b"\x96"),
        ("NEXT_BUFFER", b"\x97"),
        ("READONLY_BUFFER", b"\x98"),
    ];
    for (n, b) in opcodes {
        attrs.insert((*n).to_string(), bytes_const(b));
    }

    // Class shells (Instance with class_name; construction stubbed).
    let class_shell = |name: &str| -> MbValue {
        let fields = FxHashMap::default();
        let obj = Box::new(super::super::rc::MbObject {
            header: super::super::rc::MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: super::super::rc::ObjKind::Instance,
            },
            data: ObjData::Instance {
                class_name: format!("pickle.{}", name),
                fields: crate::runtime::rc::MbRwLock::new(fields),
            },
        });
        MbValue::from_ptr(Box::into_raw(obj))
    };
    for name in &[
        "PickleBuffer",
        "PickleError",
        "PicklingError",
        "UnpicklingError",
        "Pickler",
        "Unpickler",
    ] {
        attrs.insert((*name).to_string(), class_shell(name));
    }

    super::register_module("pickle", attrs);
}

/// Serialize a MbValue to a simple text format.
fn serialize(val: MbValue) -> String {
    if val.is_none() {
        return "N".to_string();
    }
    if let Some(b) = val.as_bool() {
        return if b { "B1" } else { "B0" }.to_string();
    }
    if let Some(i) = val.as_int() {
        return format!("I{i}");
    }
    if let Some(f) = val.as_float() {
        return format!("F{f}");
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => format!("S{}:{s}", s.len()),
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap();
                    let parts: Vec<String> = items.iter().map(|v| serialize(*v)).collect();
                    format!("L{};{}", items.len(), parts.join(";"))
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    let mut parts = Vec::new();
                    for (k, v) in map.iter() {
                        let k_str = k.to_string();
                        parts.push(format!("S{}:{k_str}", k_str.len()));
                        parts.push(serialize(*v));
                    }
                    format!("D{};{}", map.len(), parts.join(";"))
                }
                ObjData::Tuple(items) => {
                    let parts: Vec<String> = items.iter().map(|v| serialize(*v)).collect();
                    format!("T{};{}", items.len(), parts.join(";"))
                }
                _ => "N".to_string(),
            }
        }
    } else {
        "N".to_string()
    }
}

/// Deserialize from the simple text format.
fn deserialize(input: &str) -> (MbValue, usize) {
    if input.is_empty() {
        return (MbValue::none(), 0);
    }
    let bytes = input.as_bytes();
    match bytes[0] {
        b'N' => (MbValue::none(), 1),
        b'B' => {
            let v = bytes.get(1) == Some(&b'1');
            (MbValue::from_bool(v), 2)
        }
        b'I' => {
            let end = input[1..]
                .find(|c: char| !c.is_ascii_digit() && c != '-')
                .map(|i| i + 1)
                .unwrap_or(input.len());
            let n: i64 = input[1..end].parse().unwrap_or(0);
            (MbValue::from_int(n), end)
        }
        b'F' => {
            let end = input[1..]
                .find(|c: char| {
                    !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E' && c != '+'
                })
                .map(|i| i + 1)
                .unwrap_or(input.len());
            let f: f64 = input[1..end].parse().unwrap_or(0.0);
            (MbValue::from_float(f), end)
        }
        b'S' => {
            let colon = input[1..].find(':').unwrap_or(0) + 1;
            let len: usize = input[1..colon].parse().unwrap_or(0);
            let start = colon + 1;
            let s = &input[start..start + len];
            (
                MbValue::from_ptr(MbObject::new_str(s.to_string())),
                start + len,
            )
        }
        b'L' => {
            let semi = input[1..].find(';').unwrap_or(0) + 1;
            let count: usize = input[1..semi].parse().unwrap_or(0);
            let mut pos = semi + 1;
            let mut items = Vec::new();
            for _ in 0..count {
                if pos >= input.len() {
                    break;
                }
                let (val, consumed) = deserialize(&input[pos..]);
                items.push(val);
                pos += consumed;
                if pos < input.len() && input.as_bytes()[pos] == b';' {
                    pos += 1;
                }
            }
            (MbValue::from_ptr(MbObject::new_list(items)), pos)
        }
        b'T' => {
            let semi = input[1..].find(';').unwrap_or(0) + 1;
            let count: usize = input[1..semi].parse().unwrap_or(0);
            let mut pos = semi + 1;
            let mut items = Vec::new();
            for _ in 0..count {
                if pos >= input.len() {
                    break;
                }
                let (val, consumed) = deserialize(&input[pos..]);
                items.push(val);
                pos += consumed;
                if pos < input.len() && input.as_bytes()[pos] == b';' {
                    pos += 1;
                }
            }
            (MbValue::from_ptr(MbObject::new_tuple(items)), pos)
        }
        b'D' => {
            let semi = input[1..].find(';').unwrap_or(0) + 1;
            let count: usize = input[1..semi].parse().unwrap_or(0);
            let dict = MbObject::new_dict();
            let mut pos = semi + 1;
            for _ in 0..count {
                if pos >= input.len() {
                    break;
                }
                let (key_val, k_consumed) = deserialize(&input[pos..]);
                pos += k_consumed;
                if pos < input.len() && input.as_bytes()[pos] == b';' {
                    pos += 1;
                }
                let (val, v_consumed) = deserialize(&input[pos..]);
                pos += v_consumed;
                if pos < input.len() && input.as_bytes()[pos] == b';' {
                    pos += 1;
                }
                if let Some(ptr) = key_val.as_ptr() {
                    unsafe {
                        if let ObjData::Str(ref s) = (*ptr).data {
                            if let ObjData::Dict(ref lock) = (*dict).data {
                                let mut map = lock.write().unwrap();
                                map.insert(s.clone().into(), val);
                            }
                        }
                    }
                }
            }
            (MbValue::from_ptr(dict), pos)
        }
        _ => (MbValue::none(), 1),
    }
}

/// pickle.dumps(obj) -> serialized string (as bytes object)
pub fn mb_pickle_dumps(val: MbValue) -> MbValue {
    let s = serialize(val);
    MbValue::from_ptr(MbObject::new_bytes(s.into_bytes()))
}

/// pickle.loads(data) -> deserialized object
pub fn mb_pickle_loads(data: MbValue) -> MbValue {
    if let Some(ptr) = data.as_ptr() {
        unsafe {
            let s = match &(*ptr).data {
                ObjData::Bytes(b) => String::from_utf8_lossy(b).to_string(),
                ObjData::ByteArray(ref lock) => {
                    let b = lock.read().unwrap();
                    String::from_utf8_lossy(&b).to_string()
                }
                ObjData::Str(s) => s.clone(),
                _ => return MbValue::none(),
            };
            let (val, _) = deserialize(&s);
            return val;
        }
    }
    MbValue::none()
}

/// pickle.dump(obj, file) -> None (stub: serializes but discards)
pub fn mb_pickle_dump(val: MbValue, _file: MbValue) -> MbValue {
    let _ = serialize(val);
    MbValue::none()
}

/// pickle.load(file) -> None (stub)
pub fn mb_pickle_load(_file: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loads_str(s: &str) -> MbValue {
        let data = MbValue::from_ptr(MbObject::new_str(s.to_string()));
        mb_pickle_loads(data)
    }

    fn str_val(v: MbValue) -> Option<String> {
        v.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    // --- roundtrip ---
    #[test]
    fn test_roundtrip_int() {
        let val = MbValue::from_int(42);
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_roundtrip_none() {
        let val = MbValue::none();
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        assert!(result.is_none());
    }

    #[test]
    fn test_roundtrip_bool_true() {
        let data = mb_pickle_dumps(MbValue::from_bool(true));
        let result = mb_pickle_loads(data);
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_roundtrip_bool_false() {
        let data = mb_pickle_dumps(MbValue::from_bool(false));
        let result = mb_pickle_loads(data);
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_roundtrip_float() {
        let data = mb_pickle_dumps(MbValue::from_float(3.14));
        let result = mb_pickle_loads(data);
        let f = result.as_float().expect("float");
        assert!((f - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_str() {
        let val = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        assert_eq!(str_val(result), Some("hello".to_string()));
    }

    #[test]
    fn test_roundtrip_list() {
        let val = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].as_int(), Some(1));
            }
        }
    }

    #[test]
    fn test_roundtrip_tuple() {
        let val = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    assert_eq!(items.len(), 3);
                } else {
                    panic!("expected tuple");
                }
            }
        }
    }

    #[test]
    fn test_roundtrip_dict() {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut map = lock.write().unwrap();
                map.insert("k".into(), MbValue::from_int(5));
            }
        }
        let val = MbValue::from_ptr(dict);
        let data = mb_pickle_dumps(val);
        let result = mb_pickle_loads(data);
        if let Some(ptr) = result.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    assert_eq!(map.get("k").and_then(|v| v.as_int()), Some(5));
                }
            }
        }
    }

    // --- serialize "other" branch ---
    #[test]
    fn test_serialize_other_returns_n() {
        // Bytes is an ObjData variant that falls through to "N"
        let b = MbValue::from_ptr(MbObject::new_bytes(vec![1, 2, 3]));
        let s = serialize(b);
        assert_eq!(s, "N");
    }

    // --- deserialize branches ---
    #[test]
    fn test_deserialize_unknown_byte() {
        let (val, consumed) = deserialize("X123");
        assert!(val.is_none());
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_deserialize_empty() {
        let (val, consumed) = deserialize("");
        assert!(val.is_none());
        assert_eq!(consumed, 0);
    }

    // --- loads variants ---
    #[test]
    fn test_loads_from_str() {
        let result = loads_str("I99");
        assert_eq!(result.as_int(), Some(99));
    }

    #[test]
    fn test_loads_from_bytearray() {
        let ba = MbValue::from_ptr(MbObject::new_bytearray(b"I42".to_vec()));
        let result = mb_pickle_loads(ba);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_loads_non_bytes_returns_none() {
        let result = mb_pickle_loads(MbValue::from_int(0));
        assert!(result.is_none());
    }

    #[test]
    fn test_loads_null_returns_none() {
        let result = mb_pickle_loads(MbValue::none());
        assert!(result.is_none());
    }

    // --- dump / load stubs ---
    #[test]
    fn test_dump_returns_none() {
        let result = mb_pickle_dump(MbValue::from_int(1), MbValue::none());
        assert!(result.is_none());
    }

    #[test]
    fn test_load_returns_none() {
        let result = mb_pickle_load(MbValue::none());
        assert!(result.is_none());
    }
}
