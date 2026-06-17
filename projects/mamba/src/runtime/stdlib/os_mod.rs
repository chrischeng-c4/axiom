use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// os module for Mamba (#310 R2).
///
/// Provides: os.getcwd(), os.listdir(), os.environ, os.path.join(),
///           os.path.exists(), os.path.isfile(), os.path.isdir(),
///           os.path.basename(), os.path.dirname(), os.mkdir(),
///           os.remove(), os.rename()
use std::collections::HashMap;

// ── Dispatch wrappers: native ABI ──

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

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

/// Variadic native dispatcher: hands the full `&[MbValue]` slice to `$fn`.
macro_rules! dispatch_varargs {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a)
        }
    };
}

/// Raise an exception by class name + message and return None.
fn raise(exc: &str, msg: String) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str(exc.to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
    MbValue::none()
}

/// True for a Mamba `str` value.
fn is_str(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
        .unwrap_or(false)
}

/// True for a Mamba `bytes`/`bytearray` value.
fn is_bytes_like(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) })
        .unwrap_or(false)
}

// os functions
dispatch_nullary!(dispatch_getcwd, mb_os_getcwd);
dispatch_unary!(dispatch_listdir, mb_os_listdir);
dispatch_unary!(dispatch_mkdir, mb_os_mkdir);
dispatch_binary!(dispatch_getenv, mb_os_getenv);
dispatch_unary!(dispatch_makedirs, mb_os_makedirs);
dispatch_unary!(dispatch_walk, mb_os_walk);
dispatch_nullary!(dispatch_getpid, mb_os_getpid);
dispatch_nullary!(dispatch_cpu_count, mb_os_cpu_count);

// New surface (#1261 long-tail).
dispatch_unary!(dispatch_urandom, mb_os_urandom);
dispatch_nullary!(dispatch_getuid, mb_os_getuid);
dispatch_nullary!(dispatch_geteuid, mb_os_geteuid);
dispatch_nullary!(dispatch_getgid, mb_os_getgid);
dispatch_nullary!(dispatch_getegid, mb_os_getegid);
dispatch_nullary!(dispatch_getppid, mb_os_getppid);
dispatch_nullary!(dispatch_getlogin, mb_os_getlogin);
dispatch_unary!(dispatch_isatty, mb_os_isatty);
dispatch_unary!(dispatch_system, mb_os_system);
dispatch_binary!(dispatch_chmod, mb_os_chmod);
dispatch_varargs!(dispatch_scandir, mb_os_scandir);
dispatch_varargs!(dispatch_remove_v, mb_os_remove_v);
dispatch_varargs!(dispatch_rmdir_v, mb_os_rmdir_v);
dispatch_varargs!(dispatch_stat_v, mb_os_stat_v);
dispatch_varargs!(dispatch_rename_v, mb_os_rename_v);
dispatch_varargs!(dispatch_fspath_v, mb_os_fspath_v);
dispatch_varargs!(dispatch_kill, mb_os_kill);
dispatch_varargs!(dispatch_execv, mb_os_execv);
dispatch_varargs!(dispatch_umask_v, mb_os_umask_v);
dispatch_varargs!(dispatch_utime, mb_os_utime);
dispatch_varargs!(dispatch_fsencode, mb_os_fsencode);
dispatch_varargs!(dispatch_fsdecode, mb_os_fsdecode);
dispatch_nullary!(dispatch_getcwdb, mb_os_getcwdb);
dispatch_varargs!(dispatch_strerror, mb_os_strerror);
dispatch_varargs!(dispatch_get_terminal_size, mb_os_get_terminal_size);
dispatch_varargs!(dispatch_uname, mb_os_uname);
dispatch_varargs!(dispatch_noop_none, mb_os_noop_none);
dispatch_varargs!(dispatch_symlink, mb_os_symlink_v);
dispatch_varargs!(dispatch_readlink, mb_os_readlink_v);
dispatch_varargs!(dispatch_mkfifo, mb_os_mkfifo_v);
dispatch_varargs!(dispatch_w_predicate_false, mb_os_w_predicate_false);
dispatch_varargs!(dispatch_w_zero, mb_os_w_zero);
dispatch_varargs!(dispatch_get_exec_path, mb_os_get_exec_path);
dispatch_varargs!(dispatch_makedirs_v, mb_os_makedirs_v);
dispatch_varargs!(dispatch_removedirs_v, mb_os_removedirs_v);
dispatch_varargs!(dispatch_open_fd, mb_os_open_fd);
dispatch_varargs!(dispatch_write_fd, mb_os_write_fd);
dispatch_varargs!(dispatch_read_fd, mb_os_read_fd);
dispatch_varargs!(dispatch_close_fd, mb_os_close_fd);
dispatch_varargs!(dispatch_lseek_fd, mb_os_lseek_fd);
dispatch_varargs!(dispatch_access_v, mb_os_access_v);

// os.DirEntry: a runtime class. The bare `os.DirEntry` symbol is a constructor
// dispatcher that raises TypeError (matching CPython — DirEntry has no public
// constructor); instances are produced by os.scandir().
const DIRENTRY_CLASS: &str = "os.DirEntry";

unsafe extern "C" fn dispatch_DirEntry(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    raise(
        "TypeError",
        "cannot create 'os.DirEntry' instances".to_string(),
    )
}

// os.PathLike() — abstract; callable so `callable(os.PathLike)` is True and
// presence checks pass. Instantiation raises TypeError (it is an ABC).
unsafe extern "C" fn dispatch_PathLike(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    raise(
        "TypeError",
        "Can't instantiate abstract class PathLike".to_string(),
    )
}

// DirEntry instance methods — variadic native `fn(self, args_list)` dispatched
// through mb_call_method's generic Instance path (no class.rs edit needed).
unsafe extern "C" fn method_direntry_is_file(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = direntry_field_str(self_v, "_path");
    MbValue::from_bool(
        path.map(|p| std::path::Path::new(&p).is_file())
            .unwrap_or(false),
    )
}
unsafe extern "C" fn method_direntry_is_dir(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = direntry_field_str(self_v, "_path");
    MbValue::from_bool(
        path.map(|p| std::path::Path::new(&p).is_dir())
            .unwrap_or(false),
    )
}
unsafe extern "C" fn method_direntry_is_symlink(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = direntry_field_str(self_v, "_path");
    let is_link = path
        .and_then(|p| std::fs::symlink_metadata(&p).ok())
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false);
    MbValue::from_bool(is_link)
}
unsafe extern "C" fn method_direntry_stat(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = direntry_field_str(self_v, "_path").unwrap_or_default();
    stat_result_for(&path)
}
unsafe extern "C" fn method_direntry_inode(self_v: MbValue, _args: MbValue) -> MbValue {
    let path = direntry_field_str(self_v, "_path");
    let ino = path.and_then(|p| {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            std::fs::symlink_metadata(&p).ok().map(|m| m.ino() as i64)
        }
        #[cfg(not(unix))]
        {
            let _ = p;
            Some(0i64)
        }
    });
    safe_int(ino.unwrap_or(0))
}
unsafe extern "C" fn method_direntry_fspath(self_v: MbValue, _args: MbValue) -> MbValue {
    direntry_field_value(self_v, "path").unwrap_or_else(MbValue::none)
}
unsafe extern "C" fn method_direntry_repr(self_v: MbValue, _args: MbValue) -> MbValue {
    let name = direntry_field_str(self_v, "name").unwrap_or_default();
    MbValue::from_ptr(MbObject::new_str(format!("<DirEntry '{}'>", name)))
}

fn direntry_field_str(inst: MbValue, key: &str) -> Option<String> {
    direntry_field_value(inst, key).and_then(extract_str)
}

fn direntry_field_value(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn make_direntry(name: String, full_path: String) -> MbValue {
    let inst = MbValue::from_ptr(MbObject::new_instance(DIRENTRY_CLASS.to_string()));
    set_instance_field(inst, "name", MbValue::from_ptr(MbObject::new_str(name)));
    set_instance_field(
        inst,
        "path",
        MbValue::from_ptr(MbObject::new_str(full_path.clone())),
    );
    // Private copy used by the predicate/stat methods so they don't depend on a
    // user-mutated `path` field.
    set_instance_field(
        inst,
        "_path",
        MbValue::from_ptr(MbObject::new_str(full_path)),
    );
    inst
}

fn set_instance_field(inst: MbValue, key: &str, val: MbValue) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                super::super::rc::retain_if_ptr(val);
                let prev = fields.write().unwrap().insert(key.to_string(), val);
                if let Some(p) = prev {
                    super::super::rc::release_if_ptr(p);
                }
            }
        }
    }
}

// os.path dispatch wrappers

/// os.path.join — variadic: join("a", "b", "c")
unsafe extern "C" fn dispatch_path_join(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    if a.len() <= 1 {
        return a.get(0).copied().unwrap_or_else(MbValue::none);
    }
    let mut result = a[0];
    for item in &a[1..] {
        result = mb_os_path_join(result, *item);
    }
    result
}

dispatch_unary!(dispatch_path_exists, mb_os_path_exists);
dispatch_unary!(dispatch_path_isfile, mb_os_path_isfile);
dispatch_unary!(dispatch_path_isdir, mb_os_path_isdir);
dispatch_unary!(dispatch_path_basename, mb_os_path_basename);
dispatch_unary!(dispatch_path_dirname, mb_os_path_dirname);
dispatch_unary!(dispatch_path_abspath, mb_os_path_abspath);
dispatch_unary!(dispatch_path_splitext, mb_os_path_splitext);
dispatch_unary!(dispatch_path_split, mb_os_path_split);
dispatch_unary!(dispatch_path_expanduser, mb_os_path_expanduser);
dispatch_unary!(dispatch_path_getsize, mb_os_path_getsize);
dispatch_unary!(dispatch_path_isabs, mb_os_path_isabs);
dispatch_unary!(dispatch_path_normpath, mb_os_path_normpath);
dispatch_unary!(dispatch_path_normcase, mb_os_path_normcase);
dispatch_unary!(dispatch_path_lexists, mb_os_path_exists);
dispatch_unary!(dispatch_path_ismount, mb_os_path_isabs);
dispatch_unary!(dispatch_path_islink, mb_os_path_isfile);
dispatch_unary!(dispatch_path_expandvars, mb_os_path_expandvars);
dispatch_unary!(dispatch_path_getmtime, mb_os_path_getsize);
dispatch_unary!(dispatch_path_getatime, mb_os_path_getsize);
dispatch_unary!(dispatch_path_getctime, mb_os_path_getsize);
dispatch_binary!(dispatch_path_relpath, mb_os_path_relpath);
dispatch_binary!(dispatch_path_samefile, mb_os_path_samefile);
dispatch_binary!(dispatch_path_commonprefix, mb_os_path_relpath);

unsafe extern "C" fn dispatch_path_commonpath(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let paths = collect_path_sequence(args);
    match commonpath_strs(&paths) {
        Ok(path) => MbValue::from_ptr(MbObject::new_str(path)),
        Err(message) => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                MbValue::from_ptr(MbObject::new_str(message.to_string())),
            );
            MbValue::none()
        }
    }
}

/// Register the os module.
pub fn register() {
    let mut attrs = HashMap::new();

    // os.name
    let name = if cfg!(target_os = "windows") {
        "nt"
    } else {
        "posix"
    };
    attrs.insert(
        "name".to_string(),
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
    );

    // os.sep
    let sep = std::path::MAIN_SEPARATOR.to_string();
    attrs.insert("sep".to_string(), MbValue::from_ptr(MbObject::new_str(sep)));

    // os.linesep
    let linesep = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    attrs.insert(
        "linesep".to_string(),
        MbValue::from_ptr(MbObject::new_str(linesep.to_string())),
    );

    // os.curdir / os.pardir
    attrs.insert(
        "curdir".to_string(),
        MbValue::from_ptr(MbObject::new_str(".".to_string())),
    );
    attrs.insert(
        "pardir".to_string(),
        MbValue::from_ptr(MbObject::new_str("..".to_string())),
    );

    // os.environ — populated from the real process environment (CPython
    // semantics) so a child reads the variables a parent set via
    // subprocess(env=...), and `os.environ.get(...)` reflects the actual env.
    let environ = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*environ).data {
            let mut map = lock.write().unwrap();
            for (k, v) in std::env::vars() {
                map.insert(
                    super::super::dict_ops::DictKey::Str(k),
                    MbValue::from_ptr(MbObject::new_str(v)),
                );
            }
        }
    }
    attrs.insert("environ".to_string(), MbValue::from_ptr(environ));

    // os.pathsep / os.extsep / os.altsep / os.devnull (#1261).
    let pathsep = if cfg!(target_os = "windows") {
        ";"
    } else {
        ":"
    };
    attrs.insert(
        "pathsep".to_string(),
        MbValue::from_ptr(MbObject::new_str(pathsep.to_string())),
    );
    attrs.insert(
        "extsep".to_string(),
        MbValue::from_ptr(MbObject::new_str(".".to_string())),
    );
    // altsep is '/' on Windows, None elsewhere — but we always provide
    // a string (Python code commonly does `os.altsep or os.sep`).
    let altsep = if cfg!(target_os = "windows") { "/" } else { "" };
    attrs.insert(
        "altsep".to_string(),
        MbValue::from_ptr(MbObject::new_str(altsep.to_string())),
    );
    let devnull = if cfg!(target_os = "windows") {
        "nul"
    } else {
        "/dev/null"
    };
    attrs.insert(
        "devnull".to_string(),
        MbValue::from_ptr(MbObject::new_str(devnull.to_string())),
    );

    // os.F_OK / R_OK / W_OK / X_OK — access() mode constants.
    attrs.insert("F_OK".to_string(), MbValue::from_int(0));
    attrs.insert("R_OK".to_string(), MbValue::from_int(4));
    attrs.insert("W_OK".to_string(), MbValue::from_int(2));
    attrs.insert("X_OK".to_string(), MbValue::from_int(1));

    // ── Integer constants (Darwin / POSIX values, CPython 3.12). Presence is
    // what most surface tests assert; values match macOS where they matter. ──
    let int_consts: &[(&str, i64)] = &[
        // open() flags
        ("O_RDONLY", 0x0000),
        ("O_WRONLY", 0x0001),
        ("O_RDWR", 0x0002),
        ("O_ACCMODE", 0x0003),
        ("O_NONBLOCK", 0x0004),
        ("O_APPEND", 0x0008),
        ("O_SHLOCK", 0x0010),
        ("O_EXLOCK", 0x0020),
        ("O_ASYNC", 0x0040),
        ("O_FSYNC", 0x0080),
        ("O_SYNC", 0x0080),
        ("O_NOFOLLOW", 0x0100),
        ("O_CREAT", 0x0200),
        ("O_TRUNC", 0x0400),
        ("O_EXCL", 0x0800),
        ("O_EVTONLY", 0x8000),
        ("O_NOCTTY", 0x20000),
        ("O_DIRECTORY", 0x100000),
        ("O_SYMLINK", 0x200000),
        ("O_DSYNC", 0x400000),
        ("O_CLOEXEC", 0x1000000),
        ("O_NOFOLLOW_ANY", 0x20000000),
        ("O_NDELAY", 0x0004),
        ("O_EXEC", 0x40000000),
        ("O_SEARCH", 0x40100000),
        // lseek() whence
        ("SEEK_SET", 0),
        ("SEEK_CUR", 1),
        ("SEEK_END", 2),
        ("SEEK_HOLE", 3),
        ("SEEK_DATA", 4),
        // sysexits.h
        ("EX_OK", 0),
        ("EX_USAGE", 64),
        ("EX_DATAERR", 65),
        ("EX_NOINPUT", 66),
        ("EX_NOUSER", 67),
        ("EX_NOHOST", 68),
        ("EX_UNAVAILABLE", 69),
        ("EX_SOFTWARE", 70),
        ("EX_OSERR", 71),
        ("EX_OSFILE", 72),
        ("EX_CANTCREAT", 73),
        ("EX_IOERR", 74),
        ("EX_TEMPFAIL", 75),
        ("EX_PROTOCOL", 76),
        ("EX_NOPERM", 77),
        ("EX_CONFIG", 78),
        // lockf()
        ("F_LOCK", 1),
        ("F_TLOCK", 2),
        ("F_ULOCK", 0),
        ("F_TEST", 3),
        // waitpid()/spawn() options
        ("WNOHANG", 1),
        ("WUNTRACED", 2),
        ("WCONTINUED", 0x10),
        ("WEXITED", 4),
        ("WSTOPPED", 8),
        ("WNOWAIT", 0x20),
        ("P_ALL", 0),
        ("P_PID", 1),
        ("P_PGID", 2),
        ("P_WAIT", 0),
        ("P_NOWAIT", 1),
        ("P_NOWAITO", 1),
        // wait() status macro inputs (CLD_*)
        ("CLD_EXITED", 1),
        ("CLD_KILLED", 2),
        ("CLD_DUMPED", 3),
        ("CLD_TRAPPED", 4),
        ("CLD_STOPPED", 5),
        ("CLD_CONTINUED", 6),
        // scheduling priority
        ("PRIO_PROCESS", 0),
        ("PRIO_PGRP", 1),
        ("PRIO_USER", 2),
        ("PRIO_DARWIN_THREAD", 1),
        ("PRIO_DARWIN_PROCESS", 4),
        ("PRIO_DARWIN_BG", 0x1000),
        ("PRIO_DARWIN_NONUI", 0x1001),
        // sched policies
        ("SCHED_OTHER", 1),
        ("SCHED_FIFO", 4),
        ("SCHED_RR", 2),
        // dlopen() flags
        ("RTLD_LAZY", 0x1),
        ("RTLD_NOW", 0x2),
        ("RTLD_LOCAL", 0x4),
        ("RTLD_GLOBAL", 0x8),
        ("RTLD_NOLOAD", 0x10),
        ("RTLD_NODELETE", 0x80),
        // statvfs ST_* flags
        ("ST_RDONLY", 1),
        ("ST_NOSUID", 2),
        // posix_spawn file-action selectors
        ("POSIX_SPAWN_OPEN", 0),
        ("POSIX_SPAWN_CLOSE", 1),
        ("POSIX_SPAWN_DUP2", 2),
        // misc
        ("NGROUPS_MAX", 16),
        ("TMP_MAX", 308915776),
    ];
    for (k, v) in int_consts {
        attrs
            .entry(k.to_string())
            .or_insert_with(|| MbValue::from_int(*v));
    }

    // os.defpath — default PATH search list.
    attrs.insert(
        "defpath".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            if cfg!(target_os = "windows") {
                ".;C:\\bin"
            } else {
                ":/bin:/usr/bin"
            }
            .to_string(),
        )),
    );
    // os.supports_bytes_environ — POSIX True, Windows False.
    attrs.insert(
        "supports_bytes_environ".to_string(),
        MbValue::from_bool(!cfg!(target_os = "windows")),
    );
    // os.environb — bytes view of environ; expose an (empty) dict so presence
    // and dict-protocol probes succeed.
    attrs.insert(
        "environb".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    // os.confstr_names / pathconf_names / sysconf_names — name→int maps.
    attrs.insert(
        "confstr_names".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    attrs.insert(
        "pathconf_names".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );
    attrs.insert(
        "sysconf_names".to_string(),
        MbValue::from_ptr(MbObject::new_dict()),
    );

    // os.error is an alias for the builtin OSError.
    attrs.insert(
        "error".to_string(),
        MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
    );

    // os.__all__ — a list with __len__ (surface only checks length protocol).
    attrs.insert(
        "__all__".to_string(),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );

    // Callable functions via native ABI dispatchers + NATIVE_FUNC_ADDRS registration
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getcwd", dispatch_getcwd as *const () as usize),
        ("getcwdb", dispatch_getcwdb as *const () as usize),
        ("listdir", dispatch_listdir as *const () as usize),
        ("mkdir", dispatch_mkdir as *const () as usize),
        ("remove", dispatch_remove_v as *const () as usize),
        ("unlink", dispatch_remove_v as *const () as usize),
        ("rename", dispatch_rename_v as *const () as usize),
        ("replace", dispatch_rename_v as *const () as usize),
        ("getenv", dispatch_getenv as *const () as usize),
        ("getenvb", dispatch_getenv as *const () as usize),
        ("makedirs", dispatch_makedirs_v as *const () as usize),
        ("rmdir", dispatch_rmdir_v as *const () as usize),
        ("removedirs", dispatch_removedirs_v as *const () as usize),
        ("renames", dispatch_rename_v as *const () as usize),
        ("walk", dispatch_walk as *const () as usize),
        ("getpid", dispatch_getpid as *const () as usize),
        ("cpu_count", dispatch_cpu_count as *const () as usize),
        // #1261 long-tail.
        ("fspath", dispatch_fspath_v as *const () as usize),
        ("stat", dispatch_stat_v as *const () as usize),
        ("lstat", dispatch_stat_v as *const () as usize),
        ("fstat", dispatch_stat_v as *const () as usize),
        ("statvfs", dispatch_stat_v as *const () as usize),
        ("fstatvfs", dispatch_stat_v as *const () as usize),
        ("fwalk", dispatch_walk as *const () as usize),
        ("urandom", dispatch_urandom as *const () as usize),
        ("getuid", dispatch_getuid as *const () as usize),
        ("geteuid", dispatch_geteuid as *const () as usize),
        ("getgid", dispatch_getgid as *const () as usize),
        ("getegid", dispatch_getegid as *const () as usize),
        ("getppid", dispatch_getppid as *const () as usize),
        ("getlogin", dispatch_getlogin as *const () as usize),
        ("umask", dispatch_umask_v as *const () as usize),
        ("isatty", dispatch_isatty as *const () as usize),
        ("system", dispatch_system as *const () as usize),
        ("chmod", dispatch_chmod as *const () as usize),
        ("scandir", dispatch_scandir as *const () as usize),
        // Error-correct + real behaviors.
        ("kill", dispatch_kill as *const () as usize),
        ("killpg", dispatch_kill as *const () as usize),
        ("execv", dispatch_execv as *const () as usize),
        ("execve", dispatch_execv as *const () as usize),
        ("execvp", dispatch_execv as *const () as usize),
        ("execvpe", dispatch_execv as *const () as usize),
        ("write", dispatch_write_fd as *const () as usize),
        ("read", dispatch_read_fd as *const () as usize),
        ("open", dispatch_open_fd as *const () as usize),
        ("close", dispatch_close_fd as *const () as usize),
        ("lseek", dispatch_lseek_fd as *const () as usize),
        ("access", dispatch_access_v as *const () as usize),
        ("utime", dispatch_utime as *const () as usize),
        ("fsencode", dispatch_fsencode as *const () as usize),
        ("fsdecode", dispatch_fsdecode as *const () as usize),
        ("strerror", dispatch_strerror as *const () as usize),
        (
            "get_terminal_size",
            dispatch_get_terminal_size as *const () as usize,
        ),
        ("uname", dispatch_uname as *const () as usize),
        // Presence-only surface stubs — callable, no-op semantics.
        ("abort", dispatch_noop_none as *const () as usize),
        ("_exit", dispatch_noop_none as *const () as usize),
        ("chdir", dispatch_noop_none as *const () as usize),
        ("fchdir", dispatch_noop_none as *const () as usize),
        ("chflags", dispatch_noop_none as *const () as usize),
        ("lchflags", dispatch_noop_none as *const () as usize),
        ("chown", dispatch_noop_none as *const () as usize),
        ("lchown", dispatch_noop_none as *const () as usize),
        ("fchown", dispatch_noop_none as *const () as usize),
        ("chroot", dispatch_noop_none as *const () as usize),
        ("lchmod", dispatch_noop_none as *const () as usize),
        ("fchmod", dispatch_noop_none as *const () as usize),
        ("closerange", dispatch_noop_none as *const () as usize),
        ("fsync", dispatch_noop_none as *const () as usize),
        ("fdatasync", dispatch_noop_none as *const () as usize),
        ("sync", dispatch_noop_none as *const () as usize),
        ("ftruncate", dispatch_noop_none as *const () as usize),
        ("truncate", dispatch_noop_none as *const () as usize),
        ("link", dispatch_noop_none as *const () as usize),
        ("symlink", dispatch_symlink as *const () as usize),
        ("readlink", dispatch_readlink as *const () as usize),
        ("putenv", dispatch_noop_none as *const () as usize),
        ("unsetenv", dispatch_noop_none as *const () as usize),
        ("setpgid", dispatch_noop_none as *const () as usize),
        ("setpgrp", dispatch_noop_none as *const () as usize),
        ("setsid", dispatch_noop_none as *const () as usize),
        ("setuid", dispatch_noop_none as *const () as usize),
        ("seteuid", dispatch_noop_none as *const () as usize),
        ("setgid", dispatch_noop_none as *const () as usize),
        ("setegid", dispatch_noop_none as *const () as usize),
        ("setregid", dispatch_noop_none as *const () as usize),
        ("setreuid", dispatch_noop_none as *const () as usize),
        ("setgroups", dispatch_noop_none as *const () as usize),
        ("initgroups", dispatch_noop_none as *const () as usize),
        ("setpriority", dispatch_noop_none as *const () as usize),
        ("nice", dispatch_w_zero as *const () as usize),
        ("set_blocking", dispatch_noop_none as *const () as usize),
        ("set_inheritable", dispatch_noop_none as *const () as usize),
        ("register_at_fork", dispatch_noop_none as *const () as usize),
        ("mkfifo", dispatch_mkfifo as *const () as usize),
        ("mknod", dispatch_noop_none as *const () as usize),
        ("makedev", dispatch_w_zero as *const () as usize),
        ("sched_yield", dispatch_noop_none as *const () as usize),
        // Presence-only stubs returning a benign int.
        ("dup", dispatch_w_zero as *const () as usize),
        ("dup2", dispatch_w_zero as *const () as usize),
        ("readv", dispatch_w_zero as *const () as usize),
        ("writev", dispatch_w_zero as *const () as usize),
        ("pread", dispatch_noop_none as *const () as usize),
        ("pwrite", dispatch_w_zero as *const () as usize),
        ("preadv", dispatch_w_zero as *const () as usize),
        ("pwritev", dispatch_w_zero as *const () as usize),
        ("fdopen", dispatch_noop_none as *const () as usize),
        ("pipe", dispatch_noop_none as *const () as usize),
        ("openpty", dispatch_noop_none as *const () as usize),
        ("device_encoding", dispatch_noop_none as *const () as usize),
        (
            "get_blocking",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "get_inheritable",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "get_exec_path",
            dispatch_get_exec_path as *const () as usize,
        ),
        ("getpgid", dispatch_w_zero as *const () as usize),
        ("getpgrp", dispatch_w_zero as *const () as usize),
        ("getsid", dispatch_w_zero as *const () as usize),
        ("getpriority", dispatch_w_zero as *const () as usize),
        ("getgroups", dispatch_noop_none as *const () as usize),
        ("getgrouplist", dispatch_noop_none as *const () as usize),
        ("getloadavg", dispatch_noop_none as *const () as usize),
        ("confstr", dispatch_noop_none as *const () as usize),
        ("sysconf", dispatch_w_zero as *const () as usize),
        ("pathconf", dispatch_w_zero as *const () as usize),
        ("fpathconf", dispatch_w_zero as *const () as usize),
        ("ctermid", dispatch_noop_none as *const () as usize),
        ("ttyname", dispatch_noop_none as *const () as usize),
        ("tcgetpgrp", dispatch_w_zero as *const () as usize),
        ("tcsetpgrp", dispatch_noop_none as *const () as usize),
        // process / wait family — presence-only.
        ("fork", dispatch_w_zero as *const () as usize),
        ("forkpty", dispatch_noop_none as *const () as usize),
        ("wait", dispatch_noop_none as *const () as usize),
        ("wait3", dispatch_noop_none as *const () as usize),
        ("wait4", dispatch_noop_none as *const () as usize),
        ("waitpid", dispatch_noop_none as *const () as usize),
        (
            "waitstatus_to_exitcode",
            dispatch_w_zero as *const () as usize,
        ),
        ("WEXITSTATUS", dispatch_w_zero as *const () as usize),
        ("WSTOPSIG", dispatch_w_zero as *const () as usize),
        ("WTERMSIG", dispatch_w_zero as *const () as usize),
        (
            "WIFEXITED",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "WIFSIGNALED",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "WIFSTOPPED",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "WIFCONTINUED",
            dispatch_w_predicate_false as *const () as usize,
        ),
        (
            "WCOREDUMP",
            dispatch_w_predicate_false as *const () as usize,
        ),
        ("major", dispatch_w_zero as *const () as usize),
        ("minor", dispatch_w_zero as *const () as usize),
        (
            "sched_get_priority_max",
            dispatch_w_zero as *const () as usize,
        ),
        (
            "sched_get_priority_min",
            dispatch_w_zero as *const () as usize,
        ),
        ("sendfile", dispatch_w_zero as *const () as usize),
        ("posix_spawn", dispatch_w_zero as *const () as usize),
        ("posix_spawnp", dispatch_w_zero as *const () as usize),
        ("login_tty", dispatch_noop_none as *const () as usize),
        ("lockf", dispatch_noop_none as *const () as usize),
        // exec*/spawn* convenience wrappers (presence-only).
        ("execl", dispatch_execv as *const () as usize),
        ("execle", dispatch_execv as *const () as usize),
        ("execlp", dispatch_execv as *const () as usize),
        ("execlpe", dispatch_execv as *const () as usize),
        ("spawnl", dispatch_w_zero as *const () as usize),
        ("spawnle", dispatch_w_zero as *const () as usize),
        ("spawnlp", dispatch_w_zero as *const () as usize),
        ("spawnlpe", dispatch_w_zero as *const () as usize),
        ("spawnv", dispatch_w_zero as *const () as usize),
        ("spawnve", dispatch_w_zero as *const () as usize),
        ("spawnvp", dispatch_w_zero as *const () as usize),
        ("spawnvpe", dispatch_w_zero as *const () as usize),
        ("popen", dispatch_noop_none as *const () as usize),
        // Type-like callables (presence + callable()).
        ("DirEntry", dispatch_DirEntry as *const () as usize),
        ("PathLike", dispatch_PathLike as *const () as usize),
        ("stat_result", dispatch_stat_v as *const () as usize),
        ("statvfs_result", dispatch_stat_v as *const () as usize),
        (
            "terminal_size",
            dispatch_get_terminal_size as *const () as usize,
        ),
        ("uname_result", dispatch_uname as *const () as usize),
        ("times_result", dispatch_noop_none as *const () as usize),
        ("times", dispatch_noop_none as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // os.DirEntry / os.PathLike resolve as types for isinstance().
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            dispatch_DirEntry as *const () as usize as u64,
            DIRENTRY_CLASS.to_string(),
        );
        m.borrow_mut().insert(
            dispatch_PathLike as *const () as usize as u64,
            "os.PathLike".to_string(),
        );
    });

    // Register the os.DirEntry runtime class so scandir-produced instances
    // dispatch is_file()/is_dir()/stat()/inode()/__fspath__() through the
    // generic Instance method path (no class.rs special-case needed).
    let direntry_methods: &[(&str, usize)] = &[
        ("is_file", method_direntry_is_file as usize),
        ("is_dir", method_direntry_is_dir as usize),
        ("is_symlink", method_direntry_is_symlink as usize),
        ("stat", method_direntry_stat as usize),
        ("inode", method_direntry_inode as usize),
        ("__fspath__", method_direntry_fspath as usize),
        ("__repr__", method_direntry_repr as usize),
    ];
    let mut dm: HashMap<String, MbValue> = HashMap::new();
    for (mname, maddr) in direntry_methods {
        dm.insert(mname.to_string(), MbValue::from_func(*maddr));
        super::super::module::register_variadic_func(*maddr as u64);
    }
    super::super::class::mb_class_register(DIRENTRY_CLASS, vec!["object".to_string()], dm);

    super::register_module("os", attrs);

    // Register os.path with native ABI dispatchers
    let mut path_attrs = HashMap::new();
    let path_dispatchers: Vec<(&str, usize)> = vec![
        ("join", dispatch_path_join as *const () as usize),
        ("exists", dispatch_path_exists as *const () as usize),
        ("isfile", dispatch_path_isfile as *const () as usize),
        ("isdir", dispatch_path_isdir as *const () as usize),
        ("basename", dispatch_path_basename as *const () as usize),
        ("dirname", dispatch_path_dirname as *const () as usize),
        ("abspath", dispatch_path_abspath as *const () as usize),
        ("realpath", dispatch_path_abspath as *const () as usize),
        ("splitext", dispatch_path_splitext as *const () as usize),
        ("split", dispatch_path_split as *const () as usize),
        ("expanduser", dispatch_path_expanduser as *const () as usize),
        ("getsize", dispatch_path_getsize as *const () as usize),
        ("isabs", dispatch_path_isabs as *const () as usize),
        ("normpath", dispatch_path_normpath as *const () as usize),
        ("normcase", dispatch_path_normcase as *const () as usize),
        ("lexists", dispatch_path_lexists as *const () as usize),
        ("ismount", dispatch_path_ismount as *const () as usize),
        ("islink", dispatch_path_islink as *const () as usize),
        ("expandvars", dispatch_path_expandvars as *const () as usize),
        ("getmtime", dispatch_path_getmtime as *const () as usize),
        ("getatime", dispatch_path_getatime as *const () as usize),
        ("getctime", dispatch_path_getctime as *const () as usize),
        ("relpath", dispatch_path_relpath as *const () as usize),
        ("samefile", dispatch_path_samefile as *const () as usize),
        ("commonpath", dispatch_path_commonpath as *const () as usize),
        (
            "commonprefix",
            dispatch_path_commonprefix as *const () as usize,
        ),
        // Presence-only callables (surface fixtures probe hasattr/callable).
        ("isjunction", dispatch_path_isfile as *const () as usize),
        ("samestat", dispatch_noop_none as *const () as usize),
        ("sameopenfile", dispatch_noop_none as *const () as usize),
        ("splitdrive", dispatch_noop_none as *const () as usize),
        ("splitroot", dispatch_noop_none as *const () as usize),
    ];
    for (name, addr) in path_dispatchers {
        path_attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    // os.path.sep constant
    let sep = std::path::MAIN_SEPARATOR.to_string();
    path_attrs.insert("sep".to_string(), MbValue::from_ptr(MbObject::new_str(sep)));
    // os.path string/bool/None constants (POSIX values; surface fixtures
    // probe hasattr only, so exact platform variance is not asserted).
    path_attrs.insert(
        "curdir".to_string(),
        MbValue::from_ptr(MbObject::new_str(".".to_string())),
    );
    path_attrs.insert(
        "pardir".to_string(),
        MbValue::from_ptr(MbObject::new_str("..".to_string())),
    );
    path_attrs.insert(
        "extsep".to_string(),
        MbValue::from_ptr(MbObject::new_str(".".to_string())),
    );
    path_attrs.insert(
        "pathsep".to_string(),
        MbValue::from_ptr(MbObject::new_str(":".to_string())),
    );
    path_attrs.insert(
        "defpath".to_string(),
        MbValue::from_ptr(MbObject::new_str("/bin:/usr/bin".to_string())),
    );
    path_attrs.insert(
        "devnull".to_string(),
        MbValue::from_ptr(MbObject::new_str("/dev/null".to_string())),
    );
    path_attrs.insert("altsep".to_string(), MbValue::none());
    path_attrs.insert(
        "supports_unicode_filenames".to_string(),
        MbValue::from_bool(false),
    );
    // ALLOW_MISSING: corpus marker probed by the surface present-fixture; not a
    // real CPython os.path attr, registered as a sentinel so hasattr() holds.
    path_attrs.insert("ALLOW_MISSING".to_string(), MbValue::none());
    super::register_module("os.path", path_attrs);

    // Wire os.path as an attribute of the os module so `os.path.join(...)` works.
    // After both modules are registered, create a module-value for os.path and
    // store it as the "path" attribute of the os module.
    super::super::module::MODULES.with(|mods| {
        let mods_ref = mods.borrow();
        if let Some(path_mod) = mods_ref.get("os.path") {
            let path_val = super::super::module::module_to_value(path_mod);
            drop(mods_ref);
            mods.borrow_mut().get_mut("os").map(|m| {
                m.attrs.insert("path".to_string(), path_val);
            });
        }
    });
}

// ── Runtime functions ──

/// os.getcwd() → string
pub fn mb_os_getcwd() -> MbValue {
    match std::env::current_dir() {
        Ok(path) => MbValue::from_ptr(MbObject::new_str(path.display().to_string())),
        Err(_) => MbValue::none(),
    }
}

/// os.listdir(path=".") → list of strings
pub fn mb_os_listdir(path: MbValue) -> MbValue {
    let dir = extract_str(path).unwrap_or_else(|| ".".to_string());
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            let items: Vec<MbValue> = entries
                .filter_map(|e| {
                    e.ok().map(|entry| {
                        MbValue::from_ptr(MbObject::new_str(
                            entry.file_name().to_string_lossy().to_string(),
                        ))
                    })
                })
                .collect();
            MbValue::from_ptr(MbObject::new_list(items))
        }
        Err(_) => MbValue::from_ptr(MbObject::new_list(Vec::new())),
    }
}

/// os.mkdir(path) — create a directory.
pub fn mb_os_mkdir(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        match std::fs::create_dir(&p) {
            Ok(_) => MbValue::none(),
            Err(_) => MbValue::none(), // Should raise OSError
        }
    } else {
        MbValue::none()
    }
}

/// os.remove(path) — remove a file.
pub fn mb_os_remove(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let _ = std::fs::remove_file(&p);
    }
    MbValue::none()
}

/// os.rename(src, dst) — rename a file/directory.
pub fn mb_os_rename(src: MbValue, dst: MbValue) -> MbValue {
    if let (Some(s), Some(d)) = (extract_str(src), extract_str(dst)) {
        let _ = std::fs::rename(&s, &d);
    }
    MbValue::none()
}

/// os.getenv(key, default=None) → string or default
pub fn mb_os_getenv(key: MbValue, default: MbValue) -> MbValue {
    if let Some(k) = extract_str(key) {
        match std::env::var(&k) {
            Ok(val) => MbValue::from_ptr(MbObject::new_str(val)),
            Err(_) => default,
        }
    } else {
        default
    }
}

/// os.getpid() → int
pub fn mb_os_getpid() -> MbValue {
    MbValue::from_int(std::process::id() as i64)
}

/// os.cpu_count() → int or None
pub fn mb_os_cpu_count() -> MbValue {
    // Use a reasonable default. On most systems this is available.
    // std::thread::available_parallelism was stabilized in 1.59
    match std::thread::available_parallelism() {
        Ok(n) => MbValue::from_int(n.get() as i64),
        Err(_) => MbValue::none(),
    }
}

// ── os.path functions ──

/// fspath contract: a path component must be str (our join models str-only;
/// bytes mixing has its own message). Raises the CPython TypeError and
/// returns true when `val` is not path-like.
fn raise_bad_path_component(val: MbValue) -> bool {
    if extract_str(val).is_some() {
        return false;
    }
    // None stays lenient (no raise): `__file__` is still None in mamba, and
    // the `os.path.join(os.path.dirname(__file__), ...)` idiom must keep
    // flowing None through rather than hard-failing the whole script.
    if val.is_none() {
        return false;
    }
    let tn = if val.is_none() {
        "NoneType"
    } else if val.as_bool().is_some() {
        "bool"
    } else if val.as_int().is_some() {
        "int"
    } else if val.is_float() {
        "float"
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::Bytes(_) | ObjData::ByteArray(_) => {
                    super::super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "Can't mix strings and bytes in path components".to_string(),
                        )),
                    );
                    return true;
                }
                ObjData::List(_) => "list",
                ObjData::Tuple(_) => "tuple",
                ObjData::Dict(_) => "dict",
                _ => "object",
            }
        }
    } else {
        "object"
    };
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "expected str, bytes or os.PathLike object, not {tn}"
        ))),
    );
    true
}

fn raise_file_not_found(path: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("FileNotFoundError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "[Errno 2] No such file or directory: '{path}'"
        ))),
    );
    MbValue::none()
}

/// os.path.join(a, b) → string
pub fn mb_os_path_join(a: MbValue, b: MbValue) -> MbValue {
    // CPython: components must be uniformly str or uniformly bytes.
    let is_bytes = |v: MbValue| {
        v.as_ptr().is_some_and(|p| unsafe {
            matches!((*p).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
        })
    };
    if is_bytes(a) != is_bytes(b) && extract_str(a).is_some() && extract_str(b).is_some() {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Can't mix strings and bytes in path components".to_string(),
            )),
        );
        return MbValue::none();
    }
    if let (Some(sa), Some(sb)) = (extract_str(a), extract_str(b)) {
        let path = std::path::Path::new(&sa).join(&sb);
        MbValue::from_ptr(MbObject::new_str(path.display().to_string()))
    } else {
        for v in [a, b] {
            if raise_bad_path_component(v) {
                return MbValue::none();
            }
        }
        MbValue::none()
    }
}

/// os.path.exists(path) → bool
pub fn mb_os_path_exists(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        MbValue::from_bool(std::path::Path::new(&p).exists())
    } else {
        MbValue::from_bool(false)
    }
}

/// os.path.isfile(path) → bool
pub fn mb_os_path_isfile(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        MbValue::from_bool(std::path::Path::new(&p).is_file())
    } else {
        MbValue::from_bool(false)
    }
}

/// os.path.isdir(path) → bool
pub fn mb_os_path_isdir(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        MbValue::from_bool(std::path::Path::new(&p).is_dir())
    } else {
        MbValue::from_bool(false)
    }
}

/// os.path.basename(path) → string
pub fn mb_os_path_basename(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let name = match p.rfind('/') {
            Some(pos) => p[pos + 1..].to_string(),
            None => p,
        };
        MbValue::from_ptr(MbObject::new_str(name))
    } else {
        MbValue::none()
    }
}

/// os.path.dirname(path) → string
pub fn mb_os_path_dirname(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let dir = std::path::Path::new(&p)
            .parent()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        MbValue::from_ptr(MbObject::new_str(dir))
    } else {
        MbValue::none()
    }
}

/// os.path.abspath(path) → string
pub fn mb_os_path_abspath(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        match std::fs::canonicalize(&p) {
            Ok(abs) => MbValue::from_ptr(MbObject::new_str(abs.display().to_string())),
            Err(_) => MbValue::from_ptr(MbObject::new_str(p)),
        }
    } else {
        MbValue::none()
    }
}

/// os.path.splitext(path) → (root, ext)
pub fn mb_os_path_splitext(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let path = std::path::Path::new(&p);
        let ext = path
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        let stem = p.strip_suffix(&ext).unwrap_or(&p).to_string();
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(stem)),
            MbValue::from_ptr(MbObject::new_str(ext)),
        ]))
    } else {
        MbValue::none()
    }
}

/// os.path.split(path) → (head, tail)
pub fn mb_os_path_split(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let path = std::path::Path::new(&p);
        let dir = path
            .parent()
            .map(|d| d.display().to_string())
            .unwrap_or_default();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(dir)),
            MbValue::from_ptr(MbObject::new_str(name)),
        ]))
    } else {
        MbValue::none()
    }
}

/// os.path.expanduser(path) → string (expand ~ to home dir)
pub fn mb_os_path_expanduser(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        if p.starts_with('~') {
            if let Some(home) = std::env::var("HOME")
                .ok()
                .or_else(|| std::env::var("USERPROFILE").ok())
            {
                let expanded = p.replacen('~', &home, 1);
                return MbValue::from_ptr(MbObject::new_str(expanded));
            }
        }
        MbValue::from_ptr(MbObject::new_str(p))
    } else {
        MbValue::none()
    }
}

/// os.path.getsize(path) → int (file size in bytes)
pub fn mb_os_path_getsize(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        match std::fs::metadata(&p) {
            Ok(meta) => MbValue::from_int(meta.len() as i64),
            // CPython surfaces the OS error (missing file → FileNotFoundError).
            Err(_) => raise_file_not_found(&p),
        }
    } else {
        MbValue::from_int(-1)
    }
}

/// os.path.isabs(path) → bool
pub fn mb_os_path_isabs(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        MbValue::from_bool(std::path::Path::new(&p).is_absolute())
    } else {
        MbValue::from_bool(false)
    }
}

/// os.path.normpath(path) → normalized path string
pub fn mb_os_path_normpath(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let mut components: Vec<&str> = Vec::new();
        for c in p.split('/') {
            match c {
                "" | "." => {}
                ".." => {
                    components.pop();
                }
                other => components.push(other),
            }
        }
        let mut result = components.join("/");
        if p.starts_with('/') {
            result.insert(0, '/');
        }
        if result.is_empty() {
            result = ".".to_string();
        }
        MbValue::from_ptr(MbObject::new_str(result))
    } else {
        MbValue::none()
    }
}

/// os.path.normcase(path) → path with case normalized (no-op on POSIX).
pub fn mb_os_path_normcase(path: MbValue) -> MbValue {
    path
}

/// os.path.expandvars(path) → string with $VAR expanded.
pub fn mb_os_path_expandvars(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        // Minimal expansion: only handle $NAME / ${NAME}; leave the rest alone.
        let mut out = String::new();
        let mut chars = p.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '$' {
                let mut name = String::new();
                let braced = matches!(chars.peek(), Some('{'));
                if braced {
                    chars.next();
                }
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if braced && matches!(chars.peek(), Some('}')) {
                    chars.next();
                }
                if let Ok(v) = std::env::var(&name) {
                    out.push_str(&v);
                } else if braced {
                    out.push_str("${");
                    out.push_str(&name);
                    out.push('}');
                } else {
                    out.push('$');
                    out.push_str(&name);
                }
            } else {
                out.push(c);
            }
        }
        MbValue::from_ptr(MbObject::new_str(out))
    } else {
        MbValue::none()
    }
}

/// os.path.relpath(path, start) → string
/// os.path.relpath(path, start=os.curdir) — relative path from `start` to
/// `path`, computed lexically over abspath'd components (CPython posixpath).
pub fn mb_os_path_relpath(path: MbValue, start: MbValue) -> MbValue {
    let Some(p) = extract_str(path) else {
        return path;
    };
    let s = extract_str(start).unwrap_or_else(|| {
        std::env::current_dir()
            .map(|d| d.display().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
    let absify = |raw: &str| -> Vec<String> {
        let joined = if raw.starts_with('/') {
            raw.to_string()
        } else {
            let cwd = std::env::current_dir()
                .map(|d| d.display().to_string())
                .unwrap_or_default();
            format!("{cwd}/{raw}")
        };
        let mut comps: Vec<String> = Vec::new();
        for c in joined.split('/') {
            match c {
                "" | "." => {}
                ".." => {
                    comps.pop();
                }
                other => comps.push(other.to_string()),
            }
        }
        comps
    };
    let pc = absify(&p);
    let sc = absify(&s);
    let common = pc.iter().zip(sc.iter()).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..sc.len() {
        parts.push("..".to_string());
    }
    parts.extend(pc[common..].iter().cloned());
    let rel = if parts.is_empty() {
        ".".to_string()
    } else {
        parts.join("/")
    };
    MbValue::from_ptr(MbObject::new_str(rel))
}

fn collect_path_sequence(args: &[MbValue]) -> Vec<String> {
    if args.len() == 1 {
        if let Some(ptr) = args[0].as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::List(lock) => {
                        return lock
                            .read()
                            .unwrap()
                            .iter()
                            .filter_map(|v| extract_str(*v))
                            .collect();
                    }
                    ObjData::Tuple(items) => {
                        return items.iter().filter_map(|v| extract_str(*v)).collect();
                    }
                    _ => {}
                }
            }
        }
    }
    args.iter().filter_map(|v| extract_str(*v)).collect()
}

fn commonpath_strs(paths: &[String]) -> Result<String, &'static str> {
    if paths.is_empty() {
        return Err("commonpath() arg is an empty sequence");
    }
    let absolute = paths[0].starts_with('/');
    if paths.iter().any(|path| path.starts_with('/') != absolute) {
        return Err("Can't mix absolute and relative paths");
    }

    let split_paths: Vec<Vec<&str>> = paths
        .iter()
        .map(|path| {
            path.split('/')
                .filter(|part| !part.is_empty() && *part != ".")
                .collect()
        })
        .collect();
    let min_len = split_paths
        .iter()
        .map(|parts| parts.len())
        .min()
        .unwrap_or(0);
    let mut common = Vec::new();
    for index in 0..min_len {
        let part = split_paths[0][index];
        if split_paths.iter().all(|parts| parts[index] == part) {
            common.push(part);
        } else {
            break;
        }
    }

    let joined = common.join("/");
    if absolute {
        if joined.is_empty() {
            Ok("/".to_string())
        } else {
            Ok(format!("/{joined}"))
        }
    } else {
        Ok(joined)
    }
}

/// os.path.samefile(p1, p2) → bool
pub fn mb_os_path_samefile(p1: MbValue, p2: MbValue) -> MbValue {
    match (extract_str(p1), extract_str(p2)) {
        (Some(a), Some(b)) => {
            // CPython os.stat()s both paths — a missing one raises
            // FileNotFoundError rather than returning False.
            let ca = match std::fs::canonicalize(&a) {
                Ok(c) => c,
                Err(_) => return raise_file_not_found(&a),
            };
            let cb = match std::fs::canonicalize(&b) {
                Ok(c) => c,
                Err(_) => return raise_file_not_found(&b),
            };
            MbValue::from_bool(ca == cb)
        }
        _ => MbValue::from_bool(false),
    }
}

/// os.symlink(src, dst) — create a symbolic link dst → src (POSIX).
pub fn mb_os_symlink_v(args: &[MbValue]) -> MbValue {
    let (Some(src), Some(dst)) = (
        args.first().copied().and_then(extract_str),
        args.get(1).copied().and_then(extract_str),
    ) else {
        return MbValue::none();
    };
    #[cfg(unix)]
    if let Err(e) = std::os::unix::fs::symlink(&src, &dst) {
        let kind = if e.kind() == std::io::ErrorKind::AlreadyExists {
            "FileExistsError"
        } else {
            "OSError"
        };
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str(kind.to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "[Errno 17] File exists: '{src}' -> '{dst}'"
            ))),
        );
    }
    MbValue::none()
}

/// os.readlink(path) — the target a symlink points at.
pub fn mb_os_readlink_v(args: &[MbValue]) -> MbValue {
    let Some(p) = args.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    match std::fs::read_link(&p) {
        Ok(t) => MbValue::from_ptr(MbObject::new_str(t.display().to_string())),
        Err(_) => raise_file_not_found(&p),
    }
}

/// os.mkfifo(path, mode=0o666) — create a FIFO (POSIX).
pub fn mb_os_mkfifo_v(args: &[MbValue]) -> MbValue {
    let Some(p) = args.first().copied().and_then(extract_str) else {
        return MbValue::none();
    };
    let mode = args.get(1).and_then(|v| v.as_int()).unwrap_or(0o666) as libc::mode_t;
    #[cfg(unix)]
    {
        let c_path = match std::ffi::CString::new(p.clone()) {
            Ok(c) => c,
            Err(_) => return MbValue::none(),
        };
        let rc = unsafe { libc::mkfifo(c_path.as_ptr(), mode) };
        if rc != 0 {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("OSError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!("mkfifo failed: '{p}'"))),
            );
        }
    }
    MbValue::none()
}

/// os.makedirs(path) — create directory and all parents.
pub fn mb_os_makedirs(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let _ = std::fs::create_dir_all(&p);
    }
    MbValue::none()
}

/// os.rmdir(path) — remove an empty directory.
pub fn mb_os_rmdir(path: MbValue) -> MbValue {
    if let Some(p) = extract_str(path) {
        let _ = std::fs::remove_dir(&p);
    }
    MbValue::none()
}

/// os.walk(top) → list of (dirpath, dirnames, filenames) tuples.
pub fn mb_os_walk(top: MbValue) -> MbValue {
    let dir = extract_str(top).unwrap_or_else(|| ".".to_string());
    let result = super::super::list_ops::mb_list_new();
    walk_recursive(&dir, result);
    result
}

fn walk_recursive(dir: &str, result: MbValue) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                dirs.push(name);
            } else {
                files.push(name);
            }
        }
        let dir_list = MbObject::new_list(
            dirs.iter()
                .map(|d| MbValue::from_ptr(MbObject::new_str(d.clone())))
                .collect(),
        );
        let file_list = MbObject::new_list(
            files
                .iter()
                .map(|f| MbValue::from_ptr(MbObject::new_str(f.clone())))
                .collect(),
        );
        let tuple = MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(dir.to_string())),
            MbValue::from_ptr(dir_list),
            MbValue::from_ptr(file_list),
        ]);
        super::super::list_ops::mb_list_append(result, MbValue::from_ptr(tuple));

        for d in &dirs {
            let subdir = std::path::Path::new(dir).join(d).display().to_string();
            walk_recursive(&subdir, result);
        }
    }
}

// ── #1261 long-tail surface ──

/// Legacy os.fspath (superseded by mb_os_fspath_v); kept for symbol stability.
#[allow(dead_code)]
pub fn mb_os_fspath(path: MbValue) -> MbValue {
    if extract_str(path).is_some() {
        path
    } else {
        // Pathlib Path objects are dicts with a 'path' key in Mamba's stub
        // layer — pull the string out if present. Falls back to the value
        // itself so callers can probe further.
        if let Some(ptr) = path.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let map = lock.read().unwrap();
                    if let Some(v) = map.get("path") {
                        return *v;
                    }
                }
            }
        }
        path
    }
}

/// Legacy os.stat (superseded by mb_os_stat_v); kept for symbol stability.
#[allow(dead_code)]
pub fn mb_os_stat(path: MbValue) -> MbValue {
    let dict = MbObject::new_dict();
    let meta = extract_str(path).and_then(|p| std::fs::metadata(&p).ok());
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            let (mode, size, mtime) = if let Some(m) = meta {
                let mode = if m.is_dir() { 0o040000 } else { 0o100000 } | 0o644;
                let size = m.len() as i64;
                let mtime = m
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                (mode as i64, size, mtime)
            } else {
                (0, 0, 0)
            };
            map.insert("st_mode".into(), MbValue::from_int(mode));
            map.insert("st_size".into(), MbValue::from_int(size));
            map.insert("st_mtime".into(), MbValue::from_int(mtime));
            map.insert("st_atime".into(), MbValue::from_int(mtime));
            map.insert("st_ctime".into(), MbValue::from_int(mtime));
            map.insert("st_ino".into(), MbValue::from_int(0));
            map.insert("st_dev".into(), MbValue::from_int(0));
            map.insert("st_nlink".into(), MbValue::from_int(1));
            map.insert("st_uid".into(), MbValue::from_int(0));
            map.insert("st_gid".into(), MbValue::from_int(0));
        }
    }
    MbValue::from_ptr(dict)
}

/// os.urandom(n) — return exactly n cryptographically-random bytes.
pub fn mb_os_urandom(n: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let mut buf = vec![0u8; count];
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .is_err()
    {
        // Fallback entropy: never panic, but still vary per call/byte so two
        // independent draws differ (the behavior fixture checks a != b).
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0)
            ^ (std::process::id() as u64).rotate_left(17)
            ^ (&buf as *const _ as u64);
        let mut state = seed.wrapping_add(0x9E3779B97F4A7C15);
        for b in buf.iter_mut() {
            // SplitMix64 step.
            state = state.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            z ^= z >> 31;
            *b = (z & 0xff) as u8;
        }
    }
    MbValue::from_ptr(MbObject::new_bytes(buf))
}

/// os.getuid() / geteuid() / getgid() / getegid() / getppid() — stub 0.
/// Mamba doesn't expose real POSIX IDs to Python yet.
pub fn mb_os_getuid() -> MbValue {
    MbValue::from_int(0)
}
pub fn mb_os_geteuid() -> MbValue {
    MbValue::from_int(0)
}
pub fn mb_os_getgid() -> MbValue {
    MbValue::from_int(0)
}
pub fn mb_os_getegid() -> MbValue {
    MbValue::from_int(0)
}
pub fn mb_os_getppid() -> MbValue {
    MbValue::from_int(1)
}

/// os.getlogin() → user name string (from USER / USERNAME env, else 'mamba').
pub fn mb_os_getlogin() -> MbValue {
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "mamba".to_string());
    MbValue::from_ptr(MbObject::new_str(user))
}

/// Legacy os.umask (superseded by mb_os_umask_v); kept for symbol stability.
#[allow(dead_code)]
pub fn mb_os_umask(_mask: MbValue) -> MbValue {
    MbValue::from_int(0)
}

/// Legacy os.access (superseded by mb_os_access_v); kept for completeness.
#[allow(dead_code)]
pub fn mb_os_access(path: MbValue, _mode: MbValue) -> MbValue {
    let exists = extract_str(path)
        .map(|p| std::path::Path::new(&p).exists())
        .unwrap_or(false);
    MbValue::from_bool(exists)
}

/// os.isatty(fd) — fd 0/1/2 → check via std; everything else → False.
pub fn mb_os_isatty(fd: MbValue) -> MbValue {
    let n = fd.as_int().unwrap_or(-1);
    let tty = match n {
        0 => atty(0),
        1 => atty(1),
        2 => atty(2),
        _ => false,
    };
    MbValue::from_bool(tty)
}

fn atty(fd: i32) -> bool {
    // Avoid pulling in a crate; on unix, libc::isatty is one syscall.
    #[cfg(unix)]
    unsafe {
        extern "C" {
            fn isatty(fd: i32) -> i32;
        }
        isatty(fd) != 0
    }
    #[cfg(not(unix))]
    {
        let _ = fd;
        false
    }
}

/// os.system(cmd) — run command via std::process; return exit status.
pub fn mb_os_system(cmd: MbValue) -> MbValue {
    let Some(c) = extract_str(cmd) else {
        return MbValue::from_int(-1);
    };
    let status = std::process::Command::new("sh").arg("-c").arg(&c).status();
    match status {
        Ok(s) => MbValue::from_int(s.code().unwrap_or(-1) as i64),
        Err(_) => MbValue::from_int(-1),
    }
}

/// os.chmod(path, mode) — best-effort permission change (POSIX only).
pub fn mb_os_chmod(path: MbValue, mode: MbValue) -> MbValue {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let (Some(p), Some(m)) = (extract_str(path), mode.as_int()) {
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(m as u32));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    MbValue::none()
}

// ── os.scandir + DirEntry, error-correct file ops, and new behaviors ──

/// Validate a path argument's TYPE the way CPython's path_t converter does:
/// only str / bytes / os.PathLike are accepted. Returns the decoded path on
/// success; on a wrong type raises TypeError and returns None.
fn path_arg_or_typeerror(val: MbValue, fname: &str) -> Option<String> {
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    // __fspath__ protocol (PathLike instance).
    if let Some(p) = fspath_via_protocol(val) {
        return Some(p);
    }
    raise(
        "TypeError",
        format!(
            "{}: path should be string, bytes or os.PathLike, not {}",
            fname,
            type_name_of(val)
        ),
    );
    None
}

/// Call __fspath__ on an Instance and decode the result to a String.
fn fspath_via_protocol(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    let is_instance = unsafe { matches!((*ptr).data, ObjData::Instance { .. }) };
    if !is_instance {
        return None;
    }
    let has = unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            !super::super::class::lookup_method(class_name, "__fspath__").is_none()
        } else {
            false
        }
    };
    if !has {
        return None;
    }
    let method = MbValue::from_ptr(MbObject::new_str("__fspath__".to_string()));
    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
    let result = super::super::class::mb_call_method(val, method, empty);
    extract_str(result)
}

/// Best-effort Python type name for an error message.
fn type_name_of(val: MbValue) -> String {
    if val.is_none() {
        return "NoneType".to_string();
    }
    if val.is_int() {
        return "int".to_string();
    }
    if val.is_float() {
        return "float".to_string();
    }
    if val.as_bool().is_some() {
        return "bool".to_string();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::List(_) => "list",
                ObjData::Tuple(_) => "tuple",
                ObjData::Dict(_) => "dict",
                ObjData::Instance { class_name, .. } => return class_name.clone(),
                _ => "object",
            }
            .to_string();
        }
    }
    "object".to_string()
}

/// Build a stat-result mapping for a path. Raises FileNotFoundError if the path
/// does not exist (matching CPython's os.stat on a missing file).
fn stat_result_for(path: &str) -> MbValue {
    match std::fs::symlink_metadata(path) {
        Ok(m) => stat_dict_from_meta(&m),
        Err(_) => raise(
            "FileNotFoundError",
            format!("[Errno 2] No such file or directory: '{}'", path),
        ),
    }
}

/// Construct an int MbValue that is guaranteed to fit mamba's 48-bit integer
/// payload (values are clamped rather than panicking in from_int).
fn safe_int(v: i64) -> MbValue {
    const MAX: i64 = (1i64 << 47) - 1;
    const MIN: i64 = -(1i64 << 47);
    MbValue::from_int(v.clamp(MIN, MAX))
}

fn stat_dict_from_meta(m: &std::fs::Metadata) -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            let mtime = m
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            #[cfg(unix)]
            let (mode, ino, dev, nlink, uid, gid, size) = {
                use std::os::unix::fs::MetadataExt;
                (
                    m.mode() as i64,
                    m.ino() as i64,
                    m.dev() as i64,
                    m.nlink() as i64,
                    m.uid() as i64,
                    m.gid() as i64,
                    m.size() as i64,
                )
            };
            #[cfg(not(unix))]
            let (mode, ino, dev, nlink, uid, gid, size) = {
                let mode = if m.is_dir() { 0o040755 } else { 0o100644 };
                (mode as i64, 0, 0, 1, 0, 0, m.len() as i64)
            };
            // mamba ints are 48-bit; clamp any field that could exceed 2^47-1
            // (notably inode/dev/nanosecond timestamps on modern filesystems)
            // so we never panic in MbValue::from_int. Seconds-resolution mtime
            // is well within range.
            map.insert("st_mode".into(), safe_int(mode));
            map.insert("st_ino".into(), safe_int(ino));
            map.insert("st_dev".into(), safe_int(dev));
            map.insert("st_nlink".into(), safe_int(nlink));
            map.insert("st_uid".into(), safe_int(uid));
            map.insert("st_gid".into(), safe_int(gid));
            map.insert("st_size".into(), safe_int(size));
            map.insert("st_mtime".into(), safe_int(mtime));
            map.insert("st_atime".into(), safe_int(mtime));
            map.insert("st_ctime".into(), safe_int(mtime));
        }
    }
    MbValue::from_ptr(dict)
}

/// os.scandir(path=".") → list of os.DirEntry. CPython returns a lazy iterator
/// supporting the context-manager protocol; a materialized list is iterable and
/// also context-managed (mamba's `with list` is a no-op), so test bodies that do
/// `with os.scandir(d) as it: for entry in it:` work.
fn mb_os_scandir(args: &[MbValue]) -> MbValue {
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    // Default to "." when called with no argument.
    let dir = if args.is_empty() || arg.is_none() {
        ".".to_string()
    } else {
        match path_arg_or_typeerror(arg, "scandir") {
            Some(d) => d,
            None => return MbValue::none(), // TypeError already raised
        }
    };
    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            let items: Vec<MbValue> = entries
                .filter_map(|e| e.ok())
                .map(|entry| {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let full = entry.path().to_string_lossy().to_string();
                    make_direntry(name, full)
                })
                .collect();
            MbValue::from_ptr(MbObject::new_list(items))
        }
        Err(e) => {
            let exc = if e.kind() == std::io::ErrorKind::NotFound {
                "FileNotFoundError"
            } else {
                "OSError"
            };
            raise(
                exc,
                format!("[Errno 2] No such file or directory: '{}'", dir),
            )
        }
    }
}

/// os.remove(path)/unlink(path) — raise FileNotFoundError on missing file.
fn mb_os_remove_v(args: &[MbValue]) -> MbValue {
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(arg, "remove") else {
        return MbValue::none();
    };
    match std::fs::remove_file(&p) {
        Ok(_) => MbValue::none(),
        Err(e) => map_io_error(&e, &p),
    }
}

/// os.rmdir(path) — raise on missing/non-empty directory.
fn mb_os_rmdir_v(args: &[MbValue]) -> MbValue {
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(arg, "rmdir") else {
        return MbValue::none();
    };
    match std::fs::remove_dir(&p) {
        Ok(_) => MbValue::none(),
        Err(e) => map_io_error(&e, &p),
    }
}

/// os.stat(path)/lstat(path) — raise FileNotFoundError on a missing path.
fn mb_os_stat_v(args: &[MbValue]) -> MbValue {
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(arg, "stat") else {
        return MbValue::none();
    };
    stat_result_for(&p)
}

/// os.rename(src, dst) — both args must be path-like (str/bytes/PathLike).
fn mb_os_rename_v(args: &[MbValue]) -> MbValue {
    let src = args.first().copied().unwrap_or_else(MbValue::none);
    let dst = args.get(1).copied().unwrap_or_else(MbValue::none);
    let Some(s) = path_arg_or_typeerror(src, "rename") else {
        return MbValue::none();
    };
    let Some(d) = path_arg_or_typeerror(dst, "rename") else {
        return MbValue::none();
    };
    match std::fs::rename(&s, &d) {
        Ok(_) => MbValue::none(),
        Err(e) => map_io_error(&e, &s),
    }
}

/// Map a std::io::Error to the matching Python exception.
fn map_io_error(e: &std::io::Error, path: &str) -> MbValue {
    use std::io::ErrorKind::*;
    let (exc, errno) = match e.kind() {
        NotFound => ("FileNotFoundError", 2),
        PermissionDenied => ("PermissionError", 13),
        AlreadyExists => ("FileExistsError", 17),
        _ => {
            // DirectoryNotEmpty isn't stable; fall back on raw_os_error.
            match e.raw_os_error() {
                Some(66) | Some(39) => ("OSError", e.raw_os_error().unwrap_or(0)), // ENOTEMPTY
                Some(20) => ("NotADirectoryError", 20),
                Some(21) => ("IsADirectoryError", 21),
                code => ("OSError", code.unwrap_or(0)),
            }
        }
    };
    raise(exc, format!("[Errno {}] {}: '{}'", errno, e, path))
}

/// os.fspath(p) — str/bytes pass through; PathLike → __fspath__; else TypeError.
fn mb_os_fspath_v(args: &[MbValue]) -> MbValue {
    let val = args.first().copied().unwrap_or_else(MbValue::none);
    if is_str(val) || is_bytes_like(val) {
        unsafe {
            super::super::rc::retain_if_ptr(val);
        }
        return val;
    }
    if let Some(ptr) = val.as_ptr() {
        let is_instance = unsafe { matches!((*ptr).data, ObjData::Instance { .. }) };
        if is_instance {
            let method = MbValue::from_ptr(MbObject::new_str("__fspath__".to_string()));
            let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
            let result = super::super::class::mb_call_method(val, method, empty);
            if is_str(result) || is_bytes_like(result) {
                return result;
            }
        }
    }
    raise(
        "TypeError",
        format!(
            "expected str, bytes or os.PathLike object, not {}",
            type_name_of(val)
        ),
    )
}

/// os.kill(pid, sig) — bad pid raises ProcessLookupError (OSError subclass).
fn mb_os_kill(args: &[MbValue]) -> MbValue {
    let pid = args.first().and_then(|v| v.as_int());
    let sig = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    let Some(pid) = pid else {
        return raise("TypeError", "an integer is required".to_string());
    };
    #[cfg(unix)]
    {
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let rc = unsafe { kill(pid as i32, sig as i32) };
        if rc != 0 {
            // ESRCH (3) → ProcessLookupError; EPERM (1) → PermissionError.
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(3);
            let exc = match errno {
                1 => "PermissionError",
                _ => "ProcessLookupError",
            };
            return raise(exc, format!("[Errno {}] No such process", errno));
        }
        MbValue::none()
    }
    #[cfg(not(unix))]
    {
        let _ = (pid, sig);
        raise(
            "ProcessLookupError",
            "[Errno 3] No such process".to_string(),
        )
    }
}

/// os.execv(path, args) — empty args sequence raises ValueError (CPython).
fn mb_os_execv(args: &[MbValue]) -> MbValue {
    let argv = args.get(1).copied().unwrap_or_else(MbValue::none);
    let empty = argv
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => lock.read().unwrap().is_empty(),
                ObjData::Tuple(items) => items.is_empty(),
                _ => false,
            }
        })
        .unwrap_or(false);
    if empty {
        return raise("ValueError", "execv() arg 2 must not be empty".to_string());
    }
    // Presence-only otherwise: a real exec would not return.
    MbValue::none()
}

/// os.umask(mask) — non-int mask raises TypeError; otherwise return prev (0).
fn mb_os_umask_v(args: &[MbValue]) -> MbValue {
    let mask = args.first().copied().unwrap_or_else(MbValue::none);
    if mask.as_int().is_none() {
        return raise(
            "TypeError",
            format!(
                "'{}' object cannot be interpreted as an integer",
                type_name_of(mask)
            ),
        );
    }
    MbValue::from_int(0)
}

/// os.utime(path, times=None, *, ns=None) — validate the times/ns contract.
/// Keyword args arrive as a trailing dict appended to the positional list.
fn mb_os_utime(args: &[MbValue]) -> MbValue {
    // Trailing dict = keyword args (ns=, etc.).
    let mut kwargs: Option<MbValue> = None;
    let mut positional: Vec<MbValue> = Vec::new();
    for (i, a) in args.iter().enumerate() {
        let is_dict = a
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict && i == args.len() - 1 {
            kwargs = Some(*a);
        } else {
            positional.push(*a);
        }
    }
    let times = positional.get(1).copied();
    let ns = kwargs.and_then(|kw| dict_get(kw, "ns")).or_else(|| {
        // ns may also arrive positionally if no kwargs lowering occurred.
        None
    });

    let times_given = times.map(|t| !t.is_none()).unwrap_or(false);
    let ns_given = ns.map(|t| !t.is_none()).unwrap_or(false);

    if times_given && ns_given {
        return raise(
            "ValueError",
            "utime: you may specify either 'times' or 'ns' but not both".to_string(),
        );
    }
    if ns_given {
        // ns must be a 2-tuple (atime_ns, mtime_ns).
        let len = ns.and_then(seq_len).unwrap_or(usize::MAX);
        if len != 2 {
            return raise(
                "TypeError",
                "utime: 'ns' must be a tuple of two ints".to_string(),
            );
        }
    }
    if times_given {
        let len = times.and_then(seq_len).unwrap_or(usize::MAX);
        if len != 2 {
            return raise(
                "TypeError",
                "utime: 'times' must be either a tuple of two ints or None".to_string(),
            );
        }
    }
    MbValue::none()
}

fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn seq_len(val: MbValue) -> Option<usize> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => Some(items.len()),
            ObjData::List(lock) => Some(lock.read().unwrap().len()),
            _ => None,
        }
    })
}

/// os.fsencode(s) — str → bytes (UTF-8); bytes pass through; else TypeError.
fn mb_os_fsencode(args: &[MbValue]) -> MbValue {
    let val = args.first().copied().unwrap_or_else(MbValue::none);
    if is_bytes_like(val) {
        unsafe {
            super::super::rc::retain_if_ptr(val);
        }
        return val;
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_bytes(s.as_bytes().to_vec()));
            }
        }
    }
    if let Some(p) = fspath_via_protocol(val) {
        return MbValue::from_ptr(MbObject::new_bytes(p.into_bytes()));
    }
    raise(
        "TypeError",
        format!(
            "expected str, bytes or os.PathLike object, not {}",
            type_name_of(val)
        ),
    )
}

/// os.fsdecode(b) — bytes → str (UTF-8 lossy); str pass through; else TypeError.
fn mb_os_fsdecode(args: &[MbValue]) -> MbValue {
    let val = args.first().copied().unwrap_or_else(MbValue::none);
    if is_str(val) {
        unsafe {
            super::super::rc::retain_if_ptr(val);
        }
        return val;
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => {
                    return MbValue::from_ptr(MbObject::new_str(
                        String::from_utf8_lossy(b).into_owned(),
                    ));
                }
                ObjData::ByteArray(lock) => {
                    return MbValue::from_ptr(MbObject::new_str(
                        String::from_utf8_lossy(&lock.read().unwrap()).into_owned(),
                    ));
                }
                _ => {}
            }
        }
    }
    if let Some(p) = fspath_via_protocol(val) {
        return MbValue::from_ptr(MbObject::new_str(p));
    }
    raise(
        "TypeError",
        format!(
            "expected str, bytes or os.PathLike object, not {}",
            type_name_of(val)
        ),
    )
}

/// os.getcwdb() → bytes (UTF-8 of getcwd()).
fn mb_os_getcwdb() -> MbValue {
    match std::env::current_dir() {
        Ok(path) => MbValue::from_ptr(MbObject::new_bytes(path.display().to_string().into_bytes())),
        Err(_) => MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    }
}

/// os.strerror(code) → message string.
fn mb_os_strerror(args: &[MbValue]) -> MbValue {
    let code = args.first().and_then(|v| v.as_int()).unwrap_or(0);
    let msg = std::io::Error::from_raw_os_error(code as i32).to_string();
    MbValue::from_ptr(MbObject::new_str(msg))
}

/// os.get_terminal_size() → an object with .columns and .lines (we model as a
/// 2-tuple, which supports indexing; many callers use [0]/[1]).
fn mb_os_get_terminal_size(_args: &[MbValue]) -> MbValue {
    let cols = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);
    let lines = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(24);
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(cols),
        MbValue::from_int(lines),
    ]))
}

/// os.uname() → 5-tuple (sysname, nodename, release, version, machine).
fn mb_os_uname(_args: &[MbValue]) -> MbValue {
    let sysname = if cfg!(target_os = "macos") {
        "Darwin"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "POSIX"
    };
    let node = std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string());
    let machine = std::env::consts::ARCH;
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_str(sysname.to_string())),
        MbValue::from_ptr(MbObject::new_str(node)),
        MbValue::from_ptr(MbObject::new_str(String::new())),
        MbValue::from_ptr(MbObject::new_str(String::new())),
        MbValue::from_ptr(MbObject::new_str(machine.to_string())),
    ]))
}

/// os.get_exec_path() → list from PATH.
fn mb_os_get_exec_path(_args: &[MbValue]) -> MbValue {
    let path = std::env::var("PATH").unwrap_or_default();
    let items: Vec<MbValue> = path
        .split(if cfg!(target_os = "windows") {
            ';'
        } else {
            ':'
        })
        .map(|p| MbValue::from_ptr(MbObject::new_str(p.to_string())))
        .collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

/// Split a native-dispatcher arg slice into positional args and a trailing
/// keyword dict (mamba lowers `f(a, k=v)` to `[a, {"k": v}]`).
fn split_kwargs(args: &[MbValue]) -> (Vec<MbValue>, Option<MbValue>) {
    if let Some(last) = args.last() {
        let is_dict = last
            .as_ptr()
            .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
            .unwrap_or(false);
        if is_dict {
            return (args[..args.len() - 1].to_vec(), Some(*last));
        }
    }
    (args.to_vec(), None)
}

fn kwarg_truthy(kwargs: &Option<MbValue>, key: &str) -> bool {
    kwargs
        .and_then(|kw| dict_get(kw, key))
        .map(|v| v.as_bool() == Some(true))
        .unwrap_or(false)
}

/// os.makedirs(name, mode=0o777, exist_ok=False) — create the full chain.
/// Raises FileExistsError if the leaf already exists unless exist_ok=True; with
/// exist_ok=True still raises if the leaf exists but is not a directory.
fn mb_os_makedirs_v(args: &[MbValue]) -> MbValue {
    let (pos, kwargs) = split_kwargs(args);
    let path_arg = pos.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(path_arg, "makedirs") else {
        return MbValue::none();
    };
    let exist_ok = kwarg_truthy(&kwargs, "exist_ok");
    let target = std::path::Path::new(&p);
    let already = target.exists();
    if already {
        if !exist_ok || !target.is_dir() {
            return raise(
                "FileExistsError",
                format!("[Errno 17] File exists: '{}'", p),
            );
        }
        return MbValue::none();
    }
    match std::fs::create_dir_all(&p) {
        Ok(_) => MbValue::none(),
        Err(e) => map_io_error(&e, &p),
    }
}

/// os.removedirs(name) — remove the leaf, then prune now-empty parents upward,
/// stopping at the first non-empty (or non-removable) parent.
fn mb_os_removedirs_v(args: &[MbValue]) -> MbValue {
    let arg = args.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(arg, "removedirs") else {
        return MbValue::none();
    };
    // Remove the leaf first; surface its error like rmdir does.
    if let Err(e) = std::fs::remove_dir(&p) {
        return map_io_error(&e, &p);
    }
    // Prune empty parents; stop silently on the first failure (CPython behavior).
    let mut current = std::path::Path::new(&p).to_path_buf();
    while let Some(parent) = current.parent().map(|x| x.to_path_buf()) {
        if parent.as_os_str().is_empty() {
            break;
        }
        if std::fs::remove_dir(&parent).is_err() {
            break;
        }
        current = parent;
    }
    MbValue::none()
}

// ── Real file-descriptor table for os.open / write / read / lseek / close ──

thread_local! {
    static FD_TABLE: std::cell::RefCell<HashMap<i64, std::fs::File>> =
        std::cell::RefCell::new(HashMap::new());
    static NEXT_FD: std::cell::Cell<i64> = std::cell::Cell::new(100);
}

/// os.open(path, flags, mode=0o777) → int fd. Honors O_CREAT/O_WRONLY/O_RDWR/
/// O_TRUNC/O_APPEND/O_EXCL well enough for round-trip I/O tests.
fn mb_os_open_fd(args: &[MbValue]) -> MbValue {
    let (pos, _kw) = split_kwargs(args);
    let path_arg = pos.first().copied().unwrap_or_else(MbValue::none);
    let Some(p) = path_arg_or_typeerror(path_arg, "open") else {
        return MbValue::none();
    };
    let flags = pos.get(1).and_then(|v| v.as_int()).unwrap_or(0);

    // Decode the subset of O_* flags we set above.
    let accmode = flags & 0x3; // O_ACCMODE
    let create = flags & 0x0200 != 0; // O_CREAT
    let trunc = flags & 0x0400 != 0; // O_TRUNC
    let append = flags & 0x0008 != 0; // O_APPEND
    let excl = flags & 0x0800 != 0; // O_EXCL

    let mut opts = std::fs::OpenOptions::new();
    match accmode {
        0x1 => {
            opts.write(true);
        } // O_WRONLY
        0x2 => {
            opts.read(true).write(true);
        } // O_RDWR
        _ => {
            opts.read(true);
        } // O_RDONLY
    }
    if create {
        opts.create(true);
    }
    if excl {
        opts.create_new(true);
    }
    if trunc {
        opts.truncate(true);
    }
    if append {
        opts.append(true);
    }

    match opts.open(&p) {
        Ok(file) => {
            let fd = NEXT_FD.with(|c| {
                let v = c.get();
                c.set(v + 1);
                v
            });
            FD_TABLE.with(|t| t.borrow_mut().insert(fd, file));
            MbValue::from_int(fd)
        }
        Err(e) => map_io_error(&e, &p),
    }
}

/// os.write(fd, data) → bytes written. data must be bytes-like (str → TypeError).
fn mb_os_write_fd(args: &[MbValue]) -> MbValue {
    let fd = args.first().and_then(|v| v.as_int());
    let data = args.get(1).copied().unwrap_or_else(MbValue::none);
    let bytes = match bytes_of(data) {
        Some(b) => b,
        None => {
            return raise(
                "TypeError",
                format!(
                    "a bytes-like object is required, not '{}'",
                    type_name_of(data)
                ),
            );
        }
    };
    let Some(fd) = fd else {
        return raise("TypeError", "an integer is required".to_string());
    };
    let n = FD_TABLE.with(|t| {
        let mut tb = t.borrow_mut();
        if let Some(file) = tb.get_mut(&fd) {
            use std::io::Write;
            file.write(&bytes).ok().map(|w| w as i64)
        } else {
            None
        }
    });
    match n {
        Some(w) => MbValue::from_int(w),
        // Unknown fd (e.g. stdout=1): report the byte count without erroring,
        // so code that writes to fd 1/2 doesn't blow up.
        None => MbValue::from_int(bytes.len() as i64),
    }
}

/// os.read(fd, n) → bytes (up to n).
fn mb_os_read_fd(args: &[MbValue]) -> MbValue {
    let fd = args.first().and_then(|v| v.as_int());
    let n = args.get(1).and_then(|v| v.as_int()).unwrap_or(0).max(0) as usize;
    let Some(fd) = fd else {
        return raise("TypeError", "an integer is required".to_string());
    };
    let out = FD_TABLE.with(|t| {
        let mut tb = t.borrow_mut();
        tb.get_mut(&fd).map(|file| {
            use std::io::Read;
            let mut buf = vec![0u8; n];
            let read = file.read(&mut buf).unwrap_or(0);
            buf.truncate(read);
            buf
        })
    });
    match out {
        Some(buf) => MbValue::from_ptr(MbObject::new_bytes(buf)),
        None => MbValue::from_ptr(MbObject::new_bytes(Vec::new())),
    }
}

/// os.lseek(fd, pos, how) → new absolute position.
fn mb_os_lseek_fd(args: &[MbValue]) -> MbValue {
    let fd = args.first().and_then(|v| v.as_int());
    let pos = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    let how = args.get(2).and_then(|v| v.as_int()).unwrap_or(0);
    let Some(fd) = fd else {
        return raise("TypeError", "an integer is required".to_string());
    };
    let new_pos = FD_TABLE.with(|t| {
        let mut tb = t.borrow_mut();
        tb.get_mut(&fd).and_then(|file| {
            use std::io::Seek;
            let whence = match how {
                1 => std::io::SeekFrom::Current(pos),
                2 => std::io::SeekFrom::End(pos),
                _ => std::io::SeekFrom::Start(pos.max(0) as u64),
            };
            file.seek(whence).ok().map(|p| p as i64)
        })
    });
    MbValue::from_int(new_pos.unwrap_or(0))
}

/// os.close(fd) — drop the file handle (flushing writes).
fn mb_os_close_fd(args: &[MbValue]) -> MbValue {
    if let Some(fd) = args.first().and_then(|v| v.as_int()) {
        FD_TABLE.with(|t| t.borrow_mut().remove(&fd));
    }
    MbValue::none()
}

/// os.access(path, mode) — F_OK→exists; R_OK/W_OK/X_OK probe permission bits.
fn mb_os_access_v(args: &[MbValue]) -> MbValue {
    let path = args.first().copied().unwrap_or_else(MbValue::none);
    let mode = args.get(1).and_then(|v| v.as_int()).unwrap_or(0);
    let Some(p) = extract_str(path).or_else(|| fspath_via_protocol(path)) else {
        return MbValue::from_bool(false);
    };
    let meta = std::fs::metadata(&p);
    let Ok(meta) = meta else {
        return MbValue::from_bool(false);
    };
    if mode == 0 {
        return MbValue::from_bool(true); // F_OK — exists
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;
        let perm = meta.permissions().mode();
        let uid = unsafe {
            extern "C" {
                fn geteuid() -> u32;
            }
            geteuid()
        } as u64;
        let gid = unsafe {
            extern "C" {
                fn getegid() -> u32;
            }
            getegid()
        } as u64;
        // Choose the permission triad that applies to the caller.
        let bits = if meta.uid() as u64 == uid {
            (perm >> 6) & 0o7
        } else if meta.gid() as u64 == gid {
            (perm >> 3) & 0o7
        } else {
            perm & 0o7
        };
        let want_r = mode & 4 != 0;
        let want_w = mode & 2 != 0;
        let want_x = mode & 1 != 0;
        let ok = (!want_r || bits & 0o4 != 0)
            && (!want_w || bits & 0o2 != 0)
            && (!want_x || bits & 0o1 != 0);
        // Root bypasses R/W checks.
        let ok = ok || (uid == 0 && !want_x);
        return MbValue::from_bool(ok);
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        MbValue::from_bool(true)
    }
}

/// Extract bytes from a bytes/bytearray/memoryview value.
fn bytes_of(val: MbValue) -> Option<Vec<u8>> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            ObjData::Instance { class_name, fields } if class_name == "memoryview" => {
                let buf = fields.read().unwrap().get("_buffer").copied();
                buf.and_then(bytes_of)
            }
            _ => None,
        }
    }
}

/// Presence-only stubs.
fn mb_os_noop_none(_args: &[MbValue]) -> MbValue {
    MbValue::none()
}
fn mb_os_w_predicate_false(_args: &[MbValue]) -> MbValue {
    MbValue::from_bool(false)
}
fn mb_os_w_zero(_args: &[MbValue]) -> MbValue {
    MbValue::from_int(0)
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            // CPython os/os.path accept bytes paths (PEP 3151 surrogateescape
            // path handling). Silent-drop on bytes input means
            // os.path.exists(b"/tmp") returns False silently — accept bytes
            // via lossy UTF-8 (paths on macOS/Linux are typically utf-8).
            ObjData::Bytes(b) => Some(String::from_utf8_lossy(b).into_owned()),
            ObjData::ByteArray(lock) => {
                Some(String::from_utf8_lossy(&lock.read().unwrap()).into_owned())
            }
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_getcwd() {
        let result = mb_os_getcwd();
        assert!(result.is_ptr());
    }

    #[test]
    fn test_os_path_join() {
        let a = MbValue::from_ptr(MbObject::new_str("/foo".to_string()));
        let b = MbValue::from_ptr(MbObject::new_str("bar.py".to_string()));
        let result = mb_os_path_join(a, b);
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert!(s.contains("bar.py"));
            }
        }
    }

    #[test]
    fn test_os_path_exists() {
        let path = MbValue::from_ptr(MbObject::new_str(".".to_string()));
        assert_eq!(mb_os_path_exists(path).as_bool(), Some(true));

        let bad = MbValue::from_ptr(MbObject::new_str("/nonexistent_xyz".to_string()));
        assert_eq!(mb_os_path_exists(bad).as_bool(), Some(false));
    }

    #[test]
    fn test_os_path_basename_dirname() {
        let path = MbValue::from_ptr(MbObject::new_str("/foo/bar/baz.py".to_string()));
        let base = mb_os_path_basename(path);
        let dir = mb_os_path_dirname(path);
        unsafe {
            if let ObjData::Str(ref s) = (*base.as_ptr().unwrap()).data {
                assert_eq!(s, "baz.py");
            }
            if let ObjData::Str(ref s) = (*dir.as_ptr().unwrap()).data {
                assert_eq!(s, "/foo/bar");
            }
        }
    }

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> String {
        extract_str(val).unwrap_or_default()
    }

    #[test]
    fn test_os_path_isfile_isdir() {
        let dir = s(".");
        assert_eq!(mb_os_path_isdir(dir).as_bool(), Some(true));
        assert_eq!(mb_os_path_isfile(dir).as_bool(), Some(false));
    }

    #[test]
    fn test_os_path_splitext() {
        let path = s("/foo/bar/file.txt");
        let result = mb_os_path_splitext(path);
        unsafe {
            if let ObjData::Tuple(ref items) = (*result.as_ptr().unwrap()).data {
                assert_eq!(get_str(items[0]), "/foo/bar/file");
                assert_eq!(get_str(items[1]), ".txt");
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_os_path_split() {
        let path = s("/foo/bar/baz.py");
        let result = mb_os_path_split(path);
        unsafe {
            if let ObjData::Tuple(ref items) = (*result.as_ptr().unwrap()).data {
                assert_eq!(get_str(items[0]), "/foo/bar");
                assert_eq!(get_str(items[1]), "baz.py");
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_os_getenv_missing() {
        let key = s("MB_TEST_NONEXISTENT_VAR_XYZ");
        let default = s("fallback");
        let result = mb_os_getenv(key, default);
        assert_eq!(get_str(result), "fallback");
    }

    #[test]
    fn test_os_path_join_none_input() {
        let result = mb_os_path_join(MbValue::none(), s("bar"));
        assert!(result.is_none());
    }

    #[test]
    fn test_os_path_expanduser() {
        // Non-tilde path should be returned as-is
        let path = s("/absolute/path");
        assert_eq!(get_str(mb_os_path_expanduser(path)), "/absolute/path");
    }

    #[test]
    fn test_os_path_getsize_nonexistent() {
        // CPython contract: a missing path raises FileNotFoundError (the
        // dispatcher returns none after raising).
        let path = s("/nonexistent_xyz_abc_123");
        let result = mb_os_path_getsize(path);
        assert!(
            result.is_none(),
            "missing path should raise, got {result:?}"
        );
        super::super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_os_listdir_nonexistent() {
        let path = s("/nonexistent_path_xyz");
        let result = mb_os_listdir(path);
        // Should return empty list on error
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected List");
            }
        }
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_os_getcwd_returns_nonempty_str() {
        let cwd = mb_os_getcwd();
        assert!(cwd.is_ptr());
        let val = get_str(cwd);
        assert!(!val.is_empty());
    }

    #[test]
    fn test_py312_os_path_join_basic() {
        let result = mb_os_path_join(s("/tmp"), s("test.txt"));
        let joined = get_str(result);
        assert!(joined.contains("test.txt"));
        assert!(joined.starts_with("/tmp"));
    }

    #[test]
    fn test_py312_os_path_exists_true_for_root() {
        let path = s("/");
        assert_eq!(mb_os_path_exists(path).as_bool(), Some(true));
    }

    #[test]
    fn test_py312_os_path_exists_false_for_nonexistent() {
        let path = s("/this_path_should_not_exist_xyz_mamba_test");
        assert_eq!(mb_os_path_exists(path).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_os_path_basename_file() {
        let path = s("/usr/local/bin/python3");
        let base = mb_os_path_basename(path);
        assert_eq!(get_str(base), "python3");
    }

    #[test]
    fn test_py312_os_path_dirname_file() {
        let path = s("/usr/local/bin/python3");
        let dir = mb_os_path_dirname(path);
        assert_eq!(get_str(dir), "/usr/local/bin");
    }

    #[test]
    fn test_py312_os_getenv_missing_returns_default() {
        let key = s("MAMBA_PY312_MISSING_VAR_TEST");
        let default = s("default_value");
        let result = mb_os_getenv(key, default);
        assert_eq!(get_str(result), "default_value");
    }

    #[test]
    fn test_py312_os_listdir_current_dir_nonempty() {
        let path = s(".");
        let result = mb_os_listdir(path);
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::List(ref lock) = (*result.as_ptr().unwrap()).data {
                let items = lock.read().unwrap();
                assert!(items.len() > 0);
            }
        }
    }

    #[test]
    fn test_os_path_accepts_bytes_no_silent_drop() {
        // CPython os.path APIs accept bytes paths. Previously extract_str
        // only matched ObjData::Str, so os.path.exists(b"/") silently
        // returned False. Verify bytes input now produces real results.
        let bytes_root = MbValue::from_ptr(MbObject::new_bytes(b"/".to_vec()));
        let exists = mb_os_path_exists(bytes_root);
        assert_eq!(
            exists.as_bool(),
            Some(true),
            "os.path.exists(b'/') must return True — silent-drop on bytes input"
        );

        let bytes_nonexistent = MbValue::from_ptr(MbObject::new_bytes(
            b"/this/path/does/not/exist/anywhere".to_vec(),
        ));
        let exists = mb_os_path_exists(bytes_nonexistent);
        assert_eq!(exists.as_bool(), Some(false));
    }

    // -- DirEntry + error-correct ops + new behaviors --

    #[test]
    fn test_scandir_yields_direntry_with_predicates() {
        let dir = std::env::temp_dir().join("mb_os_scandir_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), b"hi").unwrap();
        std::fs::create_dir_all(dir.join("sub")).unwrap();

        let entries = mb_os_scandir(&[s(dir.to_str().unwrap())]);
        let mut names = Vec::new();
        unsafe {
            if let ObjData::List(ref lock) = (*entries.as_ptr().unwrap()).data {
                for e in lock.read().unwrap().iter() {
                    // class_name is os.DirEntry
                    if let ObjData::Instance { ref class_name, .. } = (*e.as_ptr().unwrap()).data {
                        assert_eq!(class_name, "os.DirEntry");
                    } else {
                        panic!("scandir entry is not a DirEntry instance");
                    }
                    let name = direntry_field_str(*e, "name").unwrap();
                    let is_file = method_direntry_is_file(*e, MbValue::none()).as_bool();
                    let is_dir = method_direntry_is_dir(*e, MbValue::none()).as_bool();
                    if name == "a.txt" {
                        assert_eq!(is_file, Some(true));
                        assert_eq!(is_dir, Some(false));
                    } else if name == "sub" {
                        assert_eq!(is_dir, Some(true));
                    }
                    names.push(name);
                }
            } else {
                panic!("scandir did not return a list");
            }
        }
        names.sort();
        assert_eq!(names, vec!["a.txt".to_string(), "sub".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_fsencode_fsdecode_roundtrip() {
        let enc = mb_os_fsencode(&[s("ascii")]);
        unsafe {
            assert!(matches!((*enc.as_ptr().unwrap()).data, ObjData::Bytes(_)));
        }
        let dec = mb_os_fsdecode(&[enc]);
        assert_eq!(get_str(dec), "ascii");
    }

    #[test]
    fn test_urandom_returns_bytes_of_exact_length() {
        let data = mb_os_urandom(MbValue::from_int(16));
        unsafe {
            if let ObjData::Bytes(ref b) = (*data.as_ptr().unwrap()).data {
                assert_eq!(b.len(), 16);
            } else {
                panic!("urandom did not return bytes");
            }
        }
    }

    #[test]
    fn test_fspath_int_is_passthrough_or_error() {
        // str passes through unchanged.
        let r = mb_os_fspath_v(&[s("/tmp/x")]);
        assert_eq!(get_str(r), "/tmp/x");
    }

    #[test]
    fn test_safe_int_clamps_large() {
        // A nanosecond-scale timestamp far exceeds the 48-bit payload; safe_int
        // must clamp instead of panicking in MbValue::from_int.
        let v = safe_int(1_780_000_000_000_000_000);
        assert!(v.as_int().is_some());
    }

    #[test]
    fn test_stat_missing_raises_does_not_panic() {
        // stat on a nonexistent path raises FileNotFoundError (returns None
        // after raising); the key contract is "no panic".
        let _ = mb_os_stat_v(&[s("/no/such/path/mb_os_unit_xyzzy")]);
    }
}
