use super::super::rc::{MbObject, ObjData};
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

fn raise_type_error(msg: String) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

fn known_paths_is_setlike(value: MbValue) -> bool {
    if super::super::dict_ops::dict_view_is_setlike(value) {
        return true;
    }
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Set(_) | ObjData::FrozenSet(_))
    })
}

unsafe fn dispatch_known_paths_passthrough(a: *const MbValue, n: usize) -> MbValue {
    let args: &[MbValue] = if n == 0 || a.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(a, n) }
    };
    let known_paths = args.first().copied().unwrap_or(MbValue::none());
    if !known_paths_is_setlike(known_paths) {
        return raise_type_error(format!(
            "known_paths must be set-like, not {}",
            super::super::builtins::value_type_name(known_paths)
        ));
    }
    known_paths
}

unsafe extern "C" fn dispatch_addsitepackages(a: *const MbValue, n: usize) -> MbValue {
    unsafe { dispatch_known_paths_passthrough(a, n) }
}

unsafe extern "C" fn dispatch_addusersitepackages(a: *const MbValue, n: usize) -> MbValue {
    unsafe { dispatch_known_paths_passthrough(a, n) }
}

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
        (
            "addsitepackages",
            dispatch_addsitepackages as *const () as usize,
        ),
        (
            "addusersitepackages",
            dispatch_addusersitepackages as *const () as usize,
        ),
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::exception::{current_exception_type, mb_clear_exception};

    #[test]
    fn addsitepackages_rejects_none() {
        mb_clear_exception();
        let args = [MbValue::none()];
        let result = unsafe { dispatch_addsitepackages(args.as_ptr(), args.len()) };
        assert!(result.is_none());
        assert_eq!(current_exception_type().as_deref(), Some("TypeError"));
        mb_clear_exception();
    }

    #[test]
    fn addsitepackages_rejects_missing_known_paths() {
        mb_clear_exception();
        let result = unsafe { dispatch_addsitepackages(std::ptr::null(), 0) };
        assert!(result.is_none());
        assert_eq!(current_exception_type().as_deref(), Some("TypeError"));
        mb_clear_exception();
    }

    #[test]
    fn addsitepackages_accepts_set() {
        mb_clear_exception();
        let set = MbValue::from_ptr(MbObject::new_set(Vec::new()));
        let args = [set];
        let result = unsafe { dispatch_addsitepackages(args.as_ptr(), args.len()) };
        assert_eq!(result.to_bits(), set.to_bits());
        assert_eq!(current_exception_type(), None);
    }

    #[test]
    fn addsitepackages_rejects_wrong_known_paths_type() {
        mb_clear_exception();
        let args = [MbValue::from_int(7)];
        let result = unsafe { dispatch_addsitepackages(args.as_ptr(), args.len()) };
        assert!(result.is_none());
        assert_eq!(current_exception_type().as_deref(), Some("TypeError"));
        mb_clear_exception();
    }

    #[test]
    fn addusersitepackages_accepts_frozenset() {
        mb_clear_exception();
        let set = MbValue::from_ptr(MbObject::new_frozenset(Vec::new()));
        let args = [set];
        let result = unsafe { dispatch_addusersitepackages(args.as_ptr(), args.len()) };
        assert_eq!(result.to_bits(), set.to_bits());
        assert_eq!(current_exception_type(), None);
    }

    #[test]
    fn addusersitepackages_rejects_wrong_known_paths_type() {
        mb_clear_exception();
        let args = [MbValue::from_int(7)];
        let result = unsafe { dispatch_addusersitepackages(args.as_ptr(), args.len()) };
        assert!(result.is_none());
        assert_eq!(current_exception_type().as_deref(), Some("TypeError"));
        mb_clear_exception();
    }
}
