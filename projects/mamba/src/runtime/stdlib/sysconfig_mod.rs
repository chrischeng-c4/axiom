use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// sysconfig module for Mamba (#1261 long-tail).
///
/// Minimal surface used by pytest / pip / setuptools-shaped probes:
/// version + platform + path scheme dicts. Dispatchers use the native
/// extern "C" ABI (`args_ptr`, `nargs`) and register addresses in
/// NATIVE_FUNC_ADDRS.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_get_python_version(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("3.12".to_string()))
}

unsafe extern "C" fn dispatch_get_platform(_a: *const MbValue, _n: usize) -> MbValue {
    let plat = std::env::consts::OS;
    MbValue::from_ptr(MbObject::new_str(plat.to_string()))
}

unsafe extern "C" fn dispatch_get_default_scheme(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("posix_prefix".to_string()))
}

unsafe extern "C" fn dispatch_is_python_build(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_bool(false)
}

unsafe extern "C" fn dispatch_get_scheme_names(_a: *const MbValue, _n: usize) -> MbValue {
    let names: Vec<MbValue> = ["posix_prefix", "posix_home", "posix_user", "nt", "nt_user"]
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(names))
}

unsafe extern "C" fn dispatch_get_path_names(_a: *const MbValue, _n: usize) -> MbValue {
    let names: Vec<MbValue> = [
        "stdlib",
        "platstdlib",
        "purelib",
        "platlib",
        "include",
        "platinclude",
        "scripts",
        "data",
    ]
    .iter()
    .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
    .collect();
    MbValue::from_ptr(MbObject::new_list(names))
}

unsafe extern "C" fn dispatch_get_paths(_a: *const MbValue, _n: usize) -> MbValue {
    build_paths_dict()
}

unsafe extern "C" fn dispatch_get_path(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 {
        return MbValue::none();
    }
    let key = unsafe { *a };
    let dict = build_paths_dict();
    if let (Some(kp), Some(dp)) = (key.as_ptr(), dict.as_ptr()) {
        unsafe {
            if let (ObjData::Str(ref k), ObjData::Dict(ref lock)) = (&(*kp).data, &(*dp).data) {
                let map = lock.read().unwrap();
                if let Some(v) = map.get(k.as_str()) {
                    return *v;
                }
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_config_var(a: *const MbValue, n: usize) -> MbValue {
    if n == 0 {
        return MbValue::none();
    }
    let key = unsafe { *a };
    let dict = build_config_vars();
    if let (Some(kp), Some(dp)) = (key.as_ptr(), dict.as_ptr()) {
        unsafe {
            if let (ObjData::Str(ref k), ObjData::Dict(ref lock)) = (&(*kp).data, &(*dp).data) {
                let map = lock.read().unwrap();
                if let Some(v) = map.get(k.as_str()) {
                    return *v;
                }
            }
        }
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_get_config_vars(_a: *const MbValue, _n: usize) -> MbValue {
    build_config_vars()
}

unsafe extern "C" fn dispatch_get_makefile_filename(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("Makefile".to_string()))
}

unsafe extern "C" fn dispatch_get_config_h_filename(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("pyconfig.h".to_string()))
}

fn build_paths_dict() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for k in [
                "stdlib",
                "platstdlib",
                "purelib",
                "platlib",
                "include",
                "platinclude",
                "scripts",
                "data",
            ] {
                map.insert(
                    k.into(),
                    MbValue::from_ptr(MbObject::new_str("".to_string())),
                );
            }
        }
    }
    MbValue::from_ptr(dict)
}

fn build_config_vars() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            map.insert(
                "VERSION".into(),
                MbValue::from_ptr(MbObject::new_str("3.12".to_string())),
            );
            map.insert(
                "EXT_SUFFIX".into(),
                MbValue::from_ptr(MbObject::new_str(".so".to_string())),
            );
            map.insert(
                "SOABI".into(),
                MbValue::from_ptr(MbObject::new_str("mamba-312".to_string())),
            );
            map.insert(
                "py_version".into(),
                MbValue::from_ptr(MbObject::new_str("3.12.0".to_string())),
            );
            map.insert(
                "py_version_short".into(),
                MbValue::from_ptr(MbObject::new_str("3.12".to_string())),
            );
            map.insert(
                "py_version_nodot".into(),
                MbValue::from_ptr(MbObject::new_str("312".to_string())),
            );
            map.insert(
                "prefix".into(),
                MbValue::from_ptr(MbObject::new_str("".to_string())),
            );
            map.insert(
                "exec_prefix".into(),
                MbValue::from_ptr(MbObject::new_str("".to_string())),
            );
            map.insert(
                "platform".into(),
                MbValue::from_ptr(MbObject::new_str(std::env::consts::OS.to_string())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: &[(&str, usize)] = &[
        (
            "get_python_version",
            dispatch_get_python_version as *const () as usize,
        ),
        ("get_platform", dispatch_get_platform as *const () as usize),
        (
            "get_default_scheme",
            dispatch_get_default_scheme as *const () as usize,
        ),
        (
            "is_python_build",
            dispatch_is_python_build as *const () as usize,
        ),
        (
            "get_scheme_names",
            dispatch_get_scheme_names as *const () as usize,
        ),
        (
            "get_path_names",
            dispatch_get_path_names as *const () as usize,
        ),
        ("get_paths", dispatch_get_paths as *const () as usize),
        ("get_path", dispatch_get_path as *const () as usize),
        (
            "get_config_var",
            dispatch_get_config_var as *const () as usize,
        ),
        (
            "get_config_vars",
            dispatch_get_config_vars as *const () as usize,
        ),
        (
            "get_makefile_filename",
            dispatch_get_makefile_filename as *const () as usize,
        ),
        (
            "get_config_h_filename",
            dispatch_get_config_h_filename as *const () as usize,
        ),
    ];
    for (name, addr) in dispatchers {
        attrs.insert((*name).into(), MbValue::from_func(*addr));
    }
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        for (_, addr) in dispatchers {
            set.insert(*addr as u64);
        }
    });
    super::register_module("sysconfig", attrs);
}
