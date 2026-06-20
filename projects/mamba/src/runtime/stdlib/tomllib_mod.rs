use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// tomllib module for Mamba (#1261).
///
/// Backs `tomllib.loads` with the workspace `toml` crate so a TOML
/// string actually parses into Mamba dict/list/str/int/float/bool/None
/// values. `load(fp)` accepts a path-string fallback for sources that
/// already pass a filename through (CPython binary-file objects are
/// not yet wired). `TOMLDecodeError` remains a sentinel callable; on
/// malformed TOML, `loads` raises `ValueError` with the parser
/// diagnostic so a plain `except Exception:` catches it.
use std::collections::HashMap;

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_string(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    if let ObjData::Str(ref s) = (*ptr).data {
        Some(s.clone())
    } else {
        None
    }
}

fn toml_to_mbvalue(val: &toml::Value) -> MbValue {
    match val {
        toml::Value::String(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        toml::Value::Integer(i) => MbValue::from_int(*i),
        toml::Value::Float(f) => MbValue::from_float(*f),
        toml::Value::Boolean(b) => MbValue::from_bool(*b),
        toml::Value::Datetime(dt) => {
            // CPython tomllib returns datetime/date/time objects; Mamba has no
            // datetime runtime type wired here, so surface the ISO string and
            // let callers parse if they care.
            MbValue::from_ptr(MbObject::new_str(dt.to_string()))
        }
        toml::Value::Array(arr) => {
            let items: Vec<MbValue> = arr.iter().map(toml_to_mbvalue).collect();
            MbValue::from_ptr(MbObject::new_list(items))
        }
        toml::Value::Table(tbl) => {
            // Same pattern as json_mod::json_to_mbvalue: collect all child
            // values BEFORE acquiring the dict write lock to avoid a
            // gc_track ↔ RwLock self-deadlock on the in-flight outer dict
            // (see #2109 comment in json_mod).
            let pairs: Vec<(String, MbValue)> = tbl
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_mbvalue(v)))
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

fn parse_toml_string(source: &str) -> MbValue {
    match toml::from_str::<toml::Value>(source) {
        Ok(v) => toml_to_mbvalue(&v),
        Err(e) => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!("tomllib.loads: {}", e))),
            );
            MbValue::none()
        }
    }
}

/// `tomllib.loads(s)` — parse a TOML string into a dict tree.
unsafe extern "C" fn dispatch_loads(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return MbValue::from_ptr(MbObject::new_dict());
    };
    match as_string(arg) {
        Some(s) => parse_toml_string(&s),
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "tomllib.loads() argument must be str".to_string(),
                )),
            );
            MbValue::none()
        }
    }
}

/// `tomllib.load(fp)` — best-effort: if `fp` is a path string, read and parse
/// it; otherwise fall back to the previous empty-dict stub. CPython expects a
/// binary file object, which the runtime cannot yet introspect from here.
unsafe extern "C" fn dispatch_load(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let Some(arg) = args.first().copied() else {
        return MbValue::from_ptr(MbObject::new_dict());
    };
    // tomllib.load reads bytes; a text file object (io.StringIO) is a TypeError
    // in CPython — the source must be opened in binary mode.
    if let Some(p) = arg.as_ptr() {
        if let super::super::rc::ObjData::Instance { ref class_name, .. } = (*p).data {
            if class_name == "StringIO" {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "File must be opened in binary mode, e.g. use `open('foo.toml', 'rb')`"
                            .to_string(),
                    )),
                );
                return MbValue::none();
            }
        }
    }
    if let Some(path) = as_string(arg) {
        match std::fs::read_to_string(&path) {
            Ok(src) => parse_toml_string(&src),
            Err(e) => {
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!("tomllib.load: {}: {}", path, e))),
                );
                MbValue::none()
            }
        }
    } else {
        MbValue::from_ptr(MbObject::new_dict())
    }
}

unsafe extern "C" fn dispatch_toml_decode_error(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the tomllib module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_load = dispatch_load as *const () as usize;
    attrs.insert("load".into(), MbValue::from_func(addr_load));

    let addr_loads = dispatch_loads as *const () as usize;
    attrs.insert("loads".into(), MbValue::from_func(addr_loads));

    let addr_err = dispatch_toml_decode_error as *const () as usize;
    // Register TOMLDecodeError as an alias for ValueError (same pattern as
    // json.JSONDecodeError): CPython's TOMLDecodeError subclasses ValueError,
    // and `loads` raises ValueError on malformed TOML, so resolving the handler
    // class to "ValueError" lets `except tomllib.TOMLDecodeError:` catch it.
    // The func sentinel addr is still tracked below for NATIVE_FUNC_ADDRS.
    attrs.insert(
        "TOMLDecodeError".into(),
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_load as u64);
        set.insert(addr_loads as u64);
        set.insert(addr_err as u64);
    });

    super::register_module("tomllib", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn dict_get(d: MbValue, key: &str) -> MbValue {
        unsafe {
            let ptr = d.as_ptr().expect("dict ptr");
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                map.get(&super::super::super::dict_ops::DictKey::Str(
                    key.to_string(),
                ))
                .copied()
                .unwrap_or_else(MbValue::none)
            } else {
                MbValue::none()
            }
        }
    }

    #[test]
    fn loads_flat_kvs() {
        let arg = mk_str("title = \"TOML Example\"\nport = 8080\nactive = true\n");
        let v = unsafe { dispatch_loads(&arg, 1) };
        let title = dict_get(v, "title");
        let port = dict_get(v, "port");
        let active = dict_get(v, "active");
        unsafe {
            let p = title.as_ptr().unwrap();
            if let ObjData::Str(ref s) = (*p).data {
                assert_eq!(s, "TOML Example");
            } else {
                panic!("title not str");
            }
        }
        assert_eq!(port.as_int(), Some(8080));
        assert_eq!(active.as_bool(), Some(true));
    }

    #[test]
    fn loads_section_table() {
        let arg = mk_str("[server]\nhost = \"localhost\"\nport = 5432\n");
        let v = unsafe { dispatch_loads(&arg, 1) };
        let server = dict_get(v, "server");
        let host = dict_get(server, "host");
        unsafe {
            let p = host.as_ptr().unwrap();
            if let ObjData::Str(ref s) = (*p).data {
                assert_eq!(s, "localhost");
            } else {
                panic!("host not str");
            }
        }
        assert_eq!(dict_get(server, "port").as_int(), Some(5432));
    }

    #[test]
    fn loads_array() {
        let arg = mk_str("nums = [1, 2, 3]\n");
        let v = unsafe { dispatch_loads(&arg, 1) };
        let nums = dict_get(v, "nums");
        unsafe {
            let p = nums.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*p).data {
                let list = lock.read().unwrap();
                assert_eq!(list.len(), 3);
                assert_eq!(list[0].as_int(), Some(1));
                assert_eq!(list[2].as_int(), Some(3));
            } else {
                panic!("nums not list");
            }
        }
    }
}
