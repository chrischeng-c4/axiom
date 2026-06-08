use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// zipfile module for Mamba (#445).
///
/// Provides: ZipFile, is_zipfile, ZipInfo (stubs).
/// No external dependency — stores entries in-memory.
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

disp_binary!(d_zipfile_new, mb_zipfile_new);
disp_unary!(d_is_zipfile, mb_zipfile_is_zipfile);

/// Register the zipfile module.
pub fn register() {
    let mut attrs = HashMap::new();

    // Compression constants
    attrs.insert("ZIP_STORED".into(), MbValue::from_int(0));
    attrs.insert("ZIP_DEFLATED".into(), MbValue::from_int(8));

    let dispatchers: Vec<(&str, usize)> = vec![
        ("ZipFile", d_zipfile_new as *const () as usize),
        ("is_zipfile", d_is_zipfile as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    super::register_module("zipfile", attrs);
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

/// zipfile.ZipFile(path, mode) -> zip dict
pub fn mb_zipfile_new(path: MbValue, mode: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("ZipFile".to_string())),
            );
            map.insert("filename".into(), path);
            map.insert("mode".into(), mode);
            map.insert("_entries".into(), MbValue::from_ptr(MbObject::new_dict()));
        }
    }
    MbValue::from_ptr(dict)
}

/// zip.namelist() -> list of entry names
pub fn mb_zipfile_namelist(zf: MbValue) -> MbValue {
    if let Some(ptr) = zf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(entries) = map.get("_entries").copied() {
                    if let Some(e_ptr) = entries.as_ptr() {
                        if let ObjData::Dict(ref e_lock) = (*e_ptr).data {
                            let e_map = e_lock.read().unwrap();
                            let names: Vec<MbValue> = e_map
                                .keys()
                                .map(|k| MbValue::from_ptr(MbObject::new_str(k.to_string())))
                                .collect();
                            return MbValue::from_ptr(MbObject::new_list(names));
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// zip.writestr(name, data) -> None
pub fn mb_zipfile_writestr(zf: MbValue, name: MbValue, data: MbValue) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    if let Some(ptr) = zf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(entries) = map.get("_entries").copied() {
                    if let Some(e_ptr) = entries.as_ptr() {
                        if let ObjData::Dict(ref e_lock) = (*e_ptr).data {
                            let mut e_map = e_lock.write().unwrap();
                            e_map.insert(n.into(), data);
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// zip.read(name) -> data
pub fn mb_zipfile_read(zf: MbValue, name: MbValue) -> MbValue {
    let n = extract_str(name).unwrap_or_default();
    if let Some(ptr) = zf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(entries) = map.get("_entries").copied() {
                    if let Some(e_ptr) = entries.as_ptr() {
                        if let ObjData::Dict(ref e_lock) = (*e_ptr).data {
                            let e_map = e_lock.read().unwrap();
                            if let Some(val) = e_map.get(&n) {
                                return *val;
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// zip.close() -> None
pub fn mb_zipfile_close(_zf: MbValue) -> MbValue {
    MbValue::none()
}

/// zipfile.is_zipfile(path) -> bool (stub: always false)
pub fn mb_zipfile_is_zipfile(_path: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_writestr_read() {
        let zf = mb_zipfile_new(s("test.zip"), s("w"));
        mb_zipfile_writestr(zf, s("hello.txt"), s("hello world"));
        let data = mb_zipfile_read(zf, s("hello.txt"));
        assert_eq!(extract_str(data).unwrap(), "hello world");
    }

    #[test]
    fn test_namelist() {
        let zf = mb_zipfile_new(s("test.zip"), s("w"));
        mb_zipfile_writestr(zf, s("a.txt"), s("a"));
        let names = mb_zipfile_namelist(zf);
        unsafe {
            if let ObjData::List(ref lock) = (*names.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
            }
        }
    }

    #[test]
    fn test_close_is_zipfile_and_read_missing() {
        let zf = mb_zipfile_new(s("test.zip"), s("w"));
        assert!(mb_zipfile_close(zf).is_none());
        assert_eq!(
            mb_zipfile_is_zipfile(s("whatever.zip")).as_bool(),
            Some(false)
        );
        assert!(mb_zipfile_read(zf, s("missing.txt")).is_none());
    }
}
