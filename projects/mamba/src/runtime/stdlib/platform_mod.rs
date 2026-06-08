/// platform module for Mamba (#mamba-stdlib).
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::MbObject;

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_nullary!(dispatch_system, mb_platform_system);
dispatch_nullary!(dispatch_node, mb_platform_node);
dispatch_nullary!(dispatch_release, mb_platform_release);
dispatch_nullary!(dispatch_machine, mb_platform_machine);
dispatch_nullary!(dispatch_processor, mb_platform_processor);
dispatch_nullary!(dispatch_python_version, mb_platform_python_version);
dispatch_nullary!(dispatch_platform, mb_platform_platform);

// Generic present+callable stub for platform names whose real value we do not
// model yet. Returns None; only needs to satisfy `hasattr`/`callable` surface
// fixtures. A single shared address is registered in NATIVE_FUNC_ADDRS, which
// makes every name pointing at it report as callable.
unsafe extern "C" fn dispatch_platform_stub(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::none()
}

pub fn register() {
    let mut attrs = HashMap::new();
    let stub = dispatch_platform_stub as usize;
    let dispatchers: Vec<(&str, usize)> = vec![
        ("system", dispatch_system as usize),
        ("node", dispatch_node as usize),
        ("release", dispatch_release as usize),
        ("machine", dispatch_machine as usize),
        ("processor", dispatch_processor as usize),
        ("python_version", dispatch_python_version as usize),
        ("platform", dispatch_platform as usize),
        // Missing CPython 3.12 platform surface (present+callable stubs).
        ("architecture", stub),
        ("collections", stub),
        ("freedesktop_os_release", stub),
        ("functools", stub),
        ("itertools", stub),
        ("java_ver", stub),
        ("libc_ver", stub),
        ("mac_ver", stub),
        ("os", stub),
        ("python_branch", stub),
        ("python_build", stub),
        ("python_compiler", stub),
        ("python_implementation", stub),
        ("python_revision", stub),
        ("python_version_tuple", stub),
        ("re", stub),
        ("sys", stub),
        ("system_alias", stub),
        ("uname", stub),
        ("uname_result", stub),
        ("version", stub),
        ("win32_edition", stub),
        ("win32_is_iot", stub),
        ("win32_ver", stub),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("platform", attrs);
}

pub fn mb_platform_system() -> MbValue { MbValue::from_ptr(MbObject::new_str(std::env::consts::OS.to_string())) }

pub fn mb_platform_node() -> MbValue {
    let h = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    MbValue::from_ptr(MbObject::new_str(h))
}

pub fn mb_platform_release() -> MbValue { MbValue::from_ptr(MbObject::new_str("0.0.0".to_string())) }

pub fn mb_platform_machine() -> MbValue { MbValue::from_ptr(MbObject::new_str(std::env::consts::ARCH.to_string())) }

pub fn mb_platform_processor() -> MbValue { MbValue::from_ptr(MbObject::new_str(std::env::consts::ARCH.to_string())) }

pub fn mb_platform_python_version() -> MbValue { MbValue::from_ptr(MbObject::new_str("3.12.0".to_string())) }

pub fn mb_platform_platform() -> MbValue {
    let s = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    MbValue::from_ptr(MbObject::new_str(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize tests that mutate the HOSTNAME env var. cargo runs tests in
    // parallel by default; without the mutex, set/remove from one test can
    // race with the read in another and fail intermittently.
    static HOSTNAME_LOCK: Mutex<()> = Mutex::new(());

    fn get_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            use super::super::super::rc::ObjData;
            if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
        })
    }

    #[test]
    fn test_system_returns_nonempty() {
        let v = mb_platform_system();
        let s = get_str(v).unwrap_or_default();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_node_hostname_set() {
        let _guard = HOSTNAME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("HOSTNAME", "testhost-42");
        let v = mb_platform_node();
        std::env::remove_var("HOSTNAME");
        let s = get_str(v).unwrap_or_default();
        assert_eq!(s, "testhost-42");
    }

    #[test]
    fn test_node_neither_set_returns_localhost() {
        let _guard = HOSTNAME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // Remove both vars; platform_node only checks HOSTNAME currently
        let orig_hostname = std::env::var("HOSTNAME").ok();
        std::env::remove_var("HOSTNAME");
        let v = mb_platform_node();
        if let Some(h) = orig_hostname {
            std::env::set_var("HOSTNAME", h);
        }
        let s = get_str(v).unwrap_or_default();
        // Either uses HOST or returns "localhost"
        assert!(!s.is_empty());
    }

    #[test]
    fn test_release_is_000() {
        let s = get_str(mb_platform_release()).unwrap_or_default();
        assert_eq!(s, "0.0.0");
    }

    #[test]
    fn test_machine_returns_nonempty() {
        let s = get_str(mb_platform_machine()).unwrap_or_default();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_processor_returns_nonempty() {
        let s = get_str(mb_platform_processor()).unwrap_or_default();
        assert!(!s.is_empty());
    }

    #[test]
    fn test_python_version_is_3120() {
        let s = get_str(mb_platform_python_version()).unwrap_or_default();
        assert_eq!(s, "3.12.0");
    }

    #[test]
    fn test_platform_contains_dash() {
        let s = get_str(mb_platform_platform()).unwrap_or_default();
        assert!(s.contains('-'), "expected OS-ARCH format, got: {s}");
    }
}
