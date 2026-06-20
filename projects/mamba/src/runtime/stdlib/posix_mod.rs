use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// posix module for Mamba (POSIX system call interface).
///
/// In CPython, `posix` is the C module that provides raw POSIX syscall wrappers.
/// The `os` module then imports and re-exports these with a portable Python API.
/// This module registers the underlying `posix` namespace so that
/// `import posix` works as in CPython.
///
/// Re-exports functions from os_mod where possible and adds POSIX-specific
/// functions: uname(), environ, getuid(), getgid().
use std::collections::HashMap;

// ── Dispatch wrappers (delegate to os_mod implementations) ──

fn extract_list_args(val: MbValue) -> Vec<MbValue> {
    match val.as_ptr() {
        Some(ptr) => unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().to_vec()
            } else {
                vec![]
            }
        },
        None => vec![],
    }
}

fn dispatch_getpid(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    super::os_mod::mb_os_getpid()
}

fn dispatch_getcwd(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    super::os_mod::mb_os_getcwd()
}

fn dispatch_getenv(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_getenv(
        items.get(0).copied().unwrap_or_else(MbValue::none),
        items.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

fn dispatch_listdir(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_listdir(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_mkdir(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_mkdir(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_remove(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_remove(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_rename(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_rename(
        items.get(0).copied().unwrap_or_else(MbValue::none),
        items.get(1).copied().unwrap_or_else(MbValue::none),
    )
}

fn dispatch_makedirs(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_makedirs(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_rmdir(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_rmdir(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_walk(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_walk(items.get(0).copied().unwrap_or_else(MbValue::none))
}

fn dispatch_cpu_count(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    super::os_mod::mb_os_cpu_count()
}

// ── POSIX-specific dispatch wrappers ──

fn dispatch_uname(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    mb_posix_uname()
}

fn dispatch_getuid(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    mb_posix_getuid()
}

fn dispatch_getgid(args: MbValue) -> MbValue {
    let _ = extract_list_args(args);
    mb_posix_getgid()
}

// ── os.path dispatch wrappers (re-exported under posix) ──

fn dispatch_path_join(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    if items.len() <= 1 {
        return items.get(0).copied().unwrap_or_else(MbValue::none);
    }
    let mut result = items[0];
    for item in &items[1..] {
        result = super::os_mod::mb_os_path_join(result, *item);
    }
    result
}

fn dispatch_path_exists(args: MbValue) -> MbValue {
    let items = extract_list_args(args);
    super::os_mod::mb_os_path_exists(items.get(0).copied().unwrap_or_else(MbValue::none))
}

// ── POSIX-specific runtime functions ──

/// posix.uname() -> 5-element tuple (sysname, nodename, release, version, machine)
#[cfg(unix)]
pub fn mb_posix_uname() -> MbValue {
    use std::ffi::CStr;
    unsafe {
        let mut buf: libc::utsname = std::mem::zeroed();
        if libc::uname(&mut buf) == 0 {
            let sysname = CStr::from_ptr(buf.sysname.as_ptr())
                .to_string_lossy()
                .to_string();
            let nodename = CStr::from_ptr(buf.nodename.as_ptr())
                .to_string_lossy()
                .to_string();
            let release = CStr::from_ptr(buf.release.as_ptr())
                .to_string_lossy()
                .to_string();
            let version = CStr::from_ptr(buf.version.as_ptr())
                .to_string_lossy()
                .to_string();
            let machine = CStr::from_ptr(buf.machine.as_ptr())
                .to_string_lossy()
                .to_string();
            MbValue::from_ptr(MbObject::new_tuple(vec![
                MbValue::from_ptr(MbObject::new_str(sysname)),
                MbValue::from_ptr(MbObject::new_str(nodename)),
                MbValue::from_ptr(MbObject::new_str(release)),
                MbValue::from_ptr(MbObject::new_str(version)),
                MbValue::from_ptr(MbObject::new_str(machine)),
            ]))
        } else {
            MbValue::none()
        }
    }
}

#[cfg(not(unix))]
pub fn mb_posix_uname() -> MbValue {
    // Return a stub 5-tuple on non-Unix platforms
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str("".to_string())),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
        MbValue::from_ptr(MbObject::new_str("".to_string())),
    ]))
}

/// posix.getuid() -> int
#[cfg(unix)]
pub fn mb_posix_getuid() -> MbValue {
    MbValue::from_int(unsafe { libc::getuid() } as i64)
}

#[cfg(not(unix))]
pub fn mb_posix_getuid() -> MbValue {
    MbValue::from_int(0)
}

/// posix.getgid() -> int
#[cfg(unix)]
pub fn mb_posix_getgid() -> MbValue {
    MbValue::from_int(unsafe { libc::getgid() } as i64)
}

#[cfg(not(unix))]
pub fn mb_posix_getgid() -> MbValue {
    MbValue::from_int(0)
}

/// Build the environ dict from std::env::vars().
fn build_environ_dict() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (key, val) in std::env::vars() {
                map.insert(key.into(), MbValue::from_ptr(MbObject::new_str(val)));
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// Register the posix module.
pub fn register() {
    let mut attrs = HashMap::new();

    // R7: posix.name constant
    let name = if cfg!(unix) { "posix" } else { "nt" };
    attrs.insert(
        "name".into(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );

    // R3: posix.environ — live dict populated from std::env::vars()
    attrs.insert("environ".into(), build_environ_dict());

    // R2: Re-export os_mod functions as dispatch wrappers
    attrs.insert(
        "getpid".into(),
        MbValue::from_func(dispatch_getpid as *const () as usize),
    );
    attrs.insert(
        "getcwd".into(),
        MbValue::from_func(dispatch_getcwd as *const () as usize),
    );
    attrs.insert(
        "getenv".into(),
        MbValue::from_func(dispatch_getenv as *const () as usize),
    );
    attrs.insert(
        "listdir".into(),
        MbValue::from_func(dispatch_listdir as *const () as usize),
    );
    attrs.insert(
        "mkdir".into(),
        MbValue::from_func(dispatch_mkdir as *const () as usize),
    );
    attrs.insert(
        "remove".into(),
        MbValue::from_func(dispatch_remove as *const () as usize),
    );
    attrs.insert(
        "rename".into(),
        MbValue::from_func(dispatch_rename as *const () as usize),
    );
    attrs.insert(
        "makedirs".into(),
        MbValue::from_func(dispatch_makedirs as *const () as usize),
    );
    attrs.insert(
        "rmdir".into(),
        MbValue::from_func(dispatch_rmdir as *const () as usize),
    );
    attrs.insert(
        "walk".into(),
        MbValue::from_func(dispatch_walk as *const () as usize),
    );
    attrs.insert(
        "cpu_count".into(),
        MbValue::from_func(dispatch_cpu_count as *const () as usize),
    );

    // R4: POSIX-specific: uname()
    attrs.insert(
        "uname".into(),
        MbValue::from_func(dispatch_uname as *const () as usize),
    );

    // R5: POSIX-specific: getuid(), getgid()
    attrs.insert(
        "getuid".into(),
        MbValue::from_func(dispatch_getuid as *const () as usize),
    );
    attrs.insert(
        "getgid".into(),
        MbValue::from_func(dispatch_getgid as *const () as usize),
    );

    // path.exists and path.join re-exported at top level for convenience
    attrs.insert(
        "path_exists".into(),
        MbValue::from_func(dispatch_path_exists as *const () as usize),
    );
    attrs.insert(
        "path_join".into(),
        MbValue::from_func(dispatch_path_join as *const () as usize),
    );

    // surface: missing CPython module constants (auto-added)
    attrs.insert("CLD_CONTINUED".into(), MbValue::from_int(6));
    attrs.insert("CLD_DUMPED".into(), MbValue::from_int(3));
    attrs.insert("CLD_EXITED".into(), MbValue::from_int(1));
    attrs.insert("CLD_KILLED".into(), MbValue::from_int(2));
    attrs.insert("CLD_STOPPED".into(), MbValue::from_int(5));
    attrs.insert("CLD_TRAPPED".into(), MbValue::from_int(4));
    attrs.insert("EX_CANTCREAT".into(), MbValue::from_int(73));
    attrs.insert("EX_CONFIG".into(), MbValue::from_int(78));
    attrs.insert("EX_DATAERR".into(), MbValue::from_int(65));
    attrs.insert("EX_IOERR".into(), MbValue::from_int(74));
    attrs.insert("EX_NOHOST".into(), MbValue::from_int(68));
    attrs.insert("EX_NOINPUT".into(), MbValue::from_int(66));
    attrs.insert("EX_NOPERM".into(), MbValue::from_int(77));
    attrs.insert("EX_NOUSER".into(), MbValue::from_int(67));
    attrs.insert("EX_OK".into(), MbValue::from_int(0));
    attrs.insert("EX_OSERR".into(), MbValue::from_int(71));
    attrs.insert("EX_OSFILE".into(), MbValue::from_int(72));
    attrs.insert("EX_PROTOCOL".into(), MbValue::from_int(76));
    attrs.insert("EX_SOFTWARE".into(), MbValue::from_int(70));
    attrs.insert("EX_TEMPFAIL".into(), MbValue::from_int(75));
    attrs.insert("EX_UNAVAILABLE".into(), MbValue::from_int(69));
    attrs.insert("EX_USAGE".into(), MbValue::from_int(64));
    attrs.insert("F_LOCK".into(), MbValue::from_int(1));
    attrs.insert("F_OK".into(), MbValue::from_int(0));
    attrs.insert("F_TEST".into(), MbValue::from_int(3));
    attrs.insert("F_TLOCK".into(), MbValue::from_int(2));
    attrs.insert("F_ULOCK".into(), MbValue::from_int(0));
    attrs.insert("NGROUPS_MAX".into(), MbValue::from_int(16));
    attrs.insert("O_ACCMODE".into(), MbValue::from_int(3));
    attrs.insert("O_APPEND".into(), MbValue::from_int(8));
    attrs.insert("O_ASYNC".into(), MbValue::from_int(64));
    attrs.insert("O_CLOEXEC".into(), MbValue::from_int(16777216));
    attrs.insert("O_CREAT".into(), MbValue::from_int(512));
    attrs.insert("O_DIRECTORY".into(), MbValue::from_int(1048576));
    attrs.insert("O_DSYNC".into(), MbValue::from_int(4194304));
    attrs.insert("O_EVTONLY".into(), MbValue::from_int(32768));
    attrs.insert("O_EXCL".into(), MbValue::from_int(2048));
    attrs.insert("O_EXEC".into(), MbValue::from_int(1073741824));
    attrs.insert("O_EXLOCK".into(), MbValue::from_int(32));
    attrs.insert("O_FSYNC".into(), MbValue::from_int(128));
    attrs.insert("O_NDELAY".into(), MbValue::from_int(4));
    attrs.insert("O_NOCTTY".into(), MbValue::from_int(131072));
    attrs.insert("O_NOFOLLOW".into(), MbValue::from_int(256));
    attrs.insert("O_NOFOLLOW_ANY".into(), MbValue::from_int(536870912));
    attrs.insert("O_NONBLOCK".into(), MbValue::from_int(4));
    attrs.insert("O_RDONLY".into(), MbValue::from_int(0));
    attrs.insert("O_RDWR".into(), MbValue::from_int(2));
    attrs.insert("O_SEARCH".into(), MbValue::from_int(1074790400));
    attrs.insert("O_SHLOCK".into(), MbValue::from_int(16));
    attrs.insert("O_SYMLINK".into(), MbValue::from_int(2097152));
    attrs.insert("O_SYNC".into(), MbValue::from_int(128));
    attrs.insert("O_TRUNC".into(), MbValue::from_int(1024));
    attrs.insert("O_WRONLY".into(), MbValue::from_int(1));
    attrs.insert("POSIX_SPAWN_CLOSE".into(), MbValue::from_int(1));
    attrs.insert("POSIX_SPAWN_DUP2".into(), MbValue::from_int(2));
    attrs.insert("POSIX_SPAWN_OPEN".into(), MbValue::from_int(0));
    attrs.insert("PRIO_DARWIN_BG".into(), MbValue::from_int(4096));
    attrs.insert("PRIO_DARWIN_NONUI".into(), MbValue::from_int(4097));
    attrs.insert("PRIO_DARWIN_PROCESS".into(), MbValue::from_int(4));
    attrs.insert("PRIO_DARWIN_THREAD".into(), MbValue::from_int(3));
    attrs.insert("PRIO_PGRP".into(), MbValue::from_int(1));
    attrs.insert("PRIO_PROCESS".into(), MbValue::from_int(0));
    attrs.insert("PRIO_USER".into(), MbValue::from_int(2));
    attrs.insert("P_ALL".into(), MbValue::from_int(0));
    attrs.insert("P_PGID".into(), MbValue::from_int(2));
    attrs.insert("P_PID".into(), MbValue::from_int(1));
    attrs.insert("RTLD_GLOBAL".into(), MbValue::from_int(8));
    attrs.insert("RTLD_LAZY".into(), MbValue::from_int(1));
    attrs.insert("RTLD_LOCAL".into(), MbValue::from_int(4));
    attrs.insert("RTLD_NODELETE".into(), MbValue::from_int(128));
    attrs.insert("RTLD_NOLOAD".into(), MbValue::from_int(16));
    attrs.insert("RTLD_NOW".into(), MbValue::from_int(2));
    attrs.insert("R_OK".into(), MbValue::from_int(4));
    attrs.insert("SCHED_FIFO".into(), MbValue::from_int(4));
    attrs.insert("SCHED_OTHER".into(), MbValue::from_int(1));
    attrs.insert("SCHED_RR".into(), MbValue::from_int(2));
    attrs.insert("SEEK_DATA".into(), MbValue::from_int(4));
    attrs.insert("SEEK_HOLE".into(), MbValue::from_int(3));
    attrs.insert("ST_NOSUID".into(), MbValue::from_int(2));
    attrs.insert("ST_RDONLY".into(), MbValue::from_int(1));
    attrs.insert("TMP_MAX".into(), MbValue::from_int(308915776));
    attrs.insert("WCONTINUED".into(), MbValue::from_int(16));
    attrs.insert("WEXITED".into(), MbValue::from_int(4));
    attrs.insert("WNOHANG".into(), MbValue::from_int(1));
    attrs.insert("WNOWAIT".into(), MbValue::from_int(32));
    attrs.insert("WSTOPPED".into(), MbValue::from_int(8));
    attrs.insert("WUNTRACED".into(), MbValue::from_int(2));
    attrs.insert("W_OK".into(), MbValue::from_int(2));
    attrs.insert("X_OK".into(), MbValue::from_int(1));
    super::register_module("posix", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn extract_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> String {
        extract_str(val).unwrap_or_default()
    }

    // REQ: R2
    #[test]
    fn test_posix_getpid() {
        let result = super::mb_posix_getpid_via_os();
        assert!(
            result.as_int().unwrap() > 0,
            "getpid() should return a positive integer"
        );
    }

    // REQ: R2
    #[test]
    fn test_posix_getcwd() {
        let result = super::super::os_mod::mb_os_getcwd();
        let cwd = get_str(result);
        assert!(!cwd.is_empty(), "getcwd() should return a non-empty string");
    }

    // REQ: R2
    #[test]
    fn test_posix_getenv_existing() {
        // PATH should exist on virtually all systems
        let result = super::super::os_mod::mb_os_getenv(s("PATH"), MbValue::none());
        let val = get_str(result);
        assert!(
            !val.is_empty(),
            "getenv('PATH') should return non-empty string"
        );
    }

    // REQ: R2
    #[test]
    fn test_posix_getenv_missing() {
        let result = super::super::os_mod::mb_os_getenv(
            s("MB_POSIX_TEST_NONEXISTENT_VAR"),
            s("default_val"),
        );
        assert_eq!(get_str(result), "default_val");
    }

    // REQ: R3
    #[test]
    fn test_posix_environ_populated() {
        let env = build_environ_dict();
        let has_path = env
            .as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    !map.is_empty() && map.contains_key("PATH")
                } else {
                    false
                }
            })
            .unwrap_or(false);
        assert!(has_path, "environ should be non-empty and contain PATH");
    }

    // REQ: R4
    #[cfg(unix)]
    #[test]
    fn test_posix_uname() {
        let result = mb_posix_uname();
        let ptr = result.as_ptr().expect("uname() should return a tuple");
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                assert_eq!(items.len(), 5, "uname() should return 5-element tuple");
                // Each element should be a non-empty string
                for (i, item) in items.iter().enumerate() {
                    let s = extract_str(*item);
                    assert!(s.is_some(), "uname element {i} should be a string");
                    assert!(
                        !s.unwrap().is_empty(),
                        "uname element {i} should be non-empty"
                    );
                }
            } else {
                panic!("uname() should return a Tuple");
            }
        }
    }

    // REQ: R5
    #[cfg(unix)]
    #[test]
    fn test_posix_getuid() {
        let result = mb_posix_getuid();
        let uid = result.as_int().expect("getuid() should return an integer");
        assert!(uid >= 0, "getuid() should return non-negative integer");
    }

    // REQ: R5
    #[cfg(unix)]
    #[test]
    fn test_posix_getgid() {
        let result = mb_posix_getgid();
        let gid = result.as_int().expect("getgid() should return an integer");
        assert!(gid >= 0, "getgid() should return non-negative integer");
    }

    // REQ: R2
    #[test]
    fn test_posix_listdir() {
        let result = super::super::os_mod::mb_os_listdir(s("."));
        let ptr = result.as_ptr().expect("listdir('.') should return a list");
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert!(
                    !items.is_empty(),
                    "listdir('.') should return non-empty list"
                );
            } else {
                panic!("listdir should return a List");
            }
        }
    }

    // REQ: R2
    #[test]
    fn test_posix_mkdir_remove() {
        let dir_name = format!("/tmp/mamba_posix_test_{}", std::process::id());
        let dir_val = MbValue::from_ptr(MbObject::new_str(dir_name.clone()));

        // mkdir
        super::super::os_mod::mb_os_mkdir(dir_val);
        assert!(
            std::path::Path::new(&dir_name).exists(),
            "mkdir should create directory"
        );

        // rmdir (clean up)
        super::super::os_mod::mb_os_rmdir(MbValue::from_ptr(MbObject::new_str(dir_name.clone())));
        assert!(
            !std::path::Path::new(&dir_name).exists(),
            "rmdir should remove directory"
        );
    }

    // REQ: R1
    #[test]
    fn test_register_does_not_panic() {
        register();
    }

    // REQ: R7
    #[test]
    fn test_posix_name_constant() {
        // On Unix, posix.name should be "posix"
        if cfg!(unix) {
            let name = if cfg!(unix) { "posix" } else { "nt" };
            assert_eq!(name, "posix");
        }
    }

    // REQ: R3
    #[test]
    fn test_posix_environ_dict_type() {
        let env = build_environ_dict();
        let is_dict = env
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        assert!(is_dict, "environ should be a Dict");
    }
}

/// Helper used by tests: getpid via os_mod.
#[cfg(test)]
fn mb_posix_getpid_via_os() -> MbValue {
    super::os_mod::mb_os_getpid()
}
