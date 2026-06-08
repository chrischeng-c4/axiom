use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// tarfile module for Mamba (#445).
///
/// Provides: open, is_tarfile, TarInfo (stubs).
/// No external dependency — in-memory entry storage.
use std::collections::HashMap;

macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_binary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(
                a.get(0).copied().unwrap_or_else(MbValue::none),
                a.get(1).copied().unwrap_or_else(MbValue::none),
            )
        }
    };
}

dispatch_binary!(dispatch_open, mb_tarfile_open);
dispatch_unary!(dispatch_is_tarfile, mb_tarfile_is_tarfile);

/// Register the tarfile module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("open", dispatch_open as usize),
        ("is_tarfile", dispatch_is_tarfile as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("tarfile", attrs);
}

/// tarfile.open(path, mode) -> tar dict
pub fn mb_tarfile_open(path: MbValue, mode: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "__class__".into(),
                MbValue::from_ptr(MbObject::new_str("TarFile".to_string())),
            );
            map.insert("name".into(), path);
            map.insert("mode".into(), mode);
            map.insert(
                "_members".into(),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            );
        }
    }
    MbValue::from_ptr(dict)
}

/// tar.getnames() -> list of member names
pub fn mb_tarfile_getnames(tf: MbValue) -> MbValue {
    if let Some(ptr) = tf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(members) = map.get("_members").copied() {
                    return members;
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// tar.add(name) -> None
pub fn mb_tarfile_add(tf: MbValue, name: MbValue) -> MbValue {
    if let Some(ptr) = tf.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                if let Some(members) = map.get("_members").copied() {
                    if let Some(m_ptr) = members.as_ptr() {
                        if let ObjData::List(ref list_lock) = (*m_ptr).data {
                            let mut items = list_lock.write().unwrap();
                            items.push(name);
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// tar.close() -> None
pub fn mb_tarfile_close(_tf: MbValue) -> MbValue {
    MbValue::none()
}

/// tarfile.is_tarfile(path) -> bool (stub: false)
pub fn mb_tarfile_is_tarfile(_path: MbValue) -> MbValue {
    MbValue::from_bool(false)
}

/// tar.extractall(path) -> None (stub)
pub fn mb_tarfile_extractall(_tf: MbValue, _path: MbValue) -> MbValue {
    MbValue::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    #[test]
    fn test_tarfile_add_getnames() {
        let tf = mb_tarfile_open(s("test.tar"), s("w"));
        mb_tarfile_add(tf, s("file1.txt"));
        let names = mb_tarfile_getnames(tf);
        unsafe {
            if let ObjData::List(ref lock) = (*names.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert_eq!(items.len(), 1);
            }
        }
    }

    #[test]
    fn test_tarfile_is_tarfile_close_extractall() {
        assert_eq!(mb_tarfile_is_tarfile(s("/nope")).as_bool(), Some(false));
        let tf = mb_tarfile_open(s("t.tar"), s("r"));
        assert!(mb_tarfile_close(tf).is_none());
        assert!(mb_tarfile_extractall(tf, s("/tmp")).is_none());
    }
}
