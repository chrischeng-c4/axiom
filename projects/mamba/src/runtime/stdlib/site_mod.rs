use super::super::rc::MbObject;
use super::super::value::MbValue;
/// site module for Mamba (#1261 long-tail).
///
/// Surface-only shim: `addsitedir`, `main`, `getsitepackages`,
/// `getuserbase`, `getusersitepackages`, plus the documented module
/// attributes `ENABLE_USER_SITE`, `PREFIXES`, `USER_BASE`, `USER_SITE`.
/// Mamba doesn't honor `.pth` files or per-user site-packages discovery
/// yet — the shim's job is to make `import site` resolve cleanly so
/// pytest / setuptools probes don't bail.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_addsitedir(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_main(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_getsitepackages(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

unsafe extern "C" fn dispatch_getuserbase(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("".to_string()))
}

unsafe extern "C" fn dispatch_getusersitepackages(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str("".to_string()))
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Module-level constants.
    attrs.insert("ENABLE_USER_SITE".into(), MbValue::from_bool(false));
    attrs.insert(
        "USER_BASE".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );
    attrs.insert(
        "USER_SITE".into(),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    );
    attrs.insert(
        "PREFIXES".into(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );

    let dispatchers: &[(&str, usize)] = &[
        ("addsitedir", dispatch_addsitedir as *const () as usize),
        ("main", dispatch_main as *const () as usize),
        (
            "getsitepackages",
            dispatch_getsitepackages as *const () as usize,
        ),
        ("getuserbase", dispatch_getuserbase as *const () as usize),
        (
            "getusersitepackages",
            dispatch_getusersitepackages as *const () as usize,
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
    super::register_module("site", attrs);
}
