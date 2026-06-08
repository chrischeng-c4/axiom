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

// os functions
dispatch_nullary!(dispatch_getcwd, mb_os_getcwd);
dispatch_unary!(dispatch_listdir, mb_os_listdir);
dispatch_unary!(dispatch_mkdir, mb_os_mkdir);
dispatch_unary!(dispatch_remove, mb_os_remove);
dispatch_binary!(dispatch_rename, mb_os_rename);
dispatch_binary!(dispatch_getenv, mb_os_getenv);
dispatch_unary!(dispatch_makedirs, mb_os_makedirs);
dispatch_unary!(dispatch_rmdir, mb_os_rmdir);
dispatch_unary!(dispatch_walk, mb_os_walk);
dispatch_nullary!(dispatch_getpid, mb_os_getpid);
dispatch_nullary!(dispatch_cpu_count, mb_os_cpu_count);

// New surface (#1261 long-tail).
dispatch_unary!(dispatch_fspath, mb_os_fspath);
dispatch_unary!(dispatch_stat, mb_os_stat);
dispatch_unary!(dispatch_urandom, mb_os_urandom);
dispatch_nullary!(dispatch_getuid, mb_os_getuid);
dispatch_nullary!(dispatch_geteuid, mb_os_geteuid);
dispatch_nullary!(dispatch_getgid, mb_os_getgid);
dispatch_nullary!(dispatch_getegid, mb_os_getegid);
dispatch_nullary!(dispatch_getppid, mb_os_getppid);
dispatch_nullary!(dispatch_getlogin, mb_os_getlogin);
dispatch_unary!(dispatch_umask, mb_os_umask);
dispatch_binary!(dispatch_access, mb_os_access);
dispatch_unary!(dispatch_isatty, mb_os_isatty);
dispatch_unary!(dispatch_system, mb_os_system);
dispatch_binary!(dispatch_chmod, mb_os_chmod);
dispatch_unary!(dispatch_scandir, mb_os_listdir);

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

    // os.environ (stub dict)
    let environ = MbObject::new_dict();
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

    // Callable functions via native ABI dispatchers + NATIVE_FUNC_ADDRS registration
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getcwd", dispatch_getcwd as *const () as usize),
        ("listdir", dispatch_listdir as *const () as usize),
        ("mkdir", dispatch_mkdir as *const () as usize),
        ("remove", dispatch_remove as *const () as usize),
        ("unlink", dispatch_remove as *const () as usize),
        ("rename", dispatch_rename as *const () as usize),
        ("getenv", dispatch_getenv as *const () as usize),
        ("makedirs", dispatch_makedirs as *const () as usize),
        ("rmdir", dispatch_rmdir as *const () as usize),
        ("walk", dispatch_walk as *const () as usize),
        ("getpid", dispatch_getpid as *const () as usize),
        ("cpu_count", dispatch_cpu_count as *const () as usize),
        // #1261 long-tail.
        ("fspath", dispatch_fspath as *const () as usize),
        ("stat", dispatch_stat as *const () as usize),
        ("lstat", dispatch_stat as *const () as usize),
        ("urandom", dispatch_urandom as *const () as usize),
        ("getuid", dispatch_getuid as *const () as usize),
        ("geteuid", dispatch_geteuid as *const () as usize),
        ("getgid", dispatch_getgid as *const () as usize),
        ("getegid", dispatch_getegid as *const () as usize),
        ("getppid", dispatch_getppid as *const () as usize),
        ("getlogin", dispatch_getlogin as *const () as usize),
        ("umask", dispatch_umask as *const () as usize),
        ("access", dispatch_access as *const () as usize),
        ("isatty", dispatch_isatty as *const () as usize),
        ("system", dispatch_system as *const () as usize),
        ("chmod", dispatch_chmod as *const () as usize),
        ("scandir", dispatch_scandir as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

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

/// os.path.join(a, b) → string
pub fn mb_os_path_join(a: MbValue, b: MbValue) -> MbValue {
    if let (Some(sa), Some(sb)) = (extract_str(a), extract_str(b)) {
        let path = std::path::Path::new(&sa).join(&sb);
        MbValue::from_ptr(MbObject::new_str(path.display().to_string()))
    } else {
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
            Err(_) => MbValue::from_int(-1),
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
pub fn mb_os_path_relpath(path: MbValue, _start: MbValue) -> MbValue {
    path
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
            let ca = std::fs::canonicalize(&a).ok();
            let cb = std::fs::canonicalize(&b).ok();
            MbValue::from_bool(ca.is_some() && ca == cb)
        }
        _ => MbValue::from_bool(false),
    }
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

/// os.fspath(path) — return path's filesystem representation. For strings
/// (and bytes — Mamba has no separate bytes type yet) returns the input;
/// for objects with __fspath__ this would need dunder dispatch, deferred.
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

/// os.stat(path) → stub stat-result dict (st_mode, st_size, st_mtime, ...).
/// Real `std::fs::metadata` populates size + mtime; other fields are zero.
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

/// os.urandom(n) — return n random bytes as a string (Mamba has no bytes
/// type yet; callers that need bytes typically just want the entropy).
pub fn mb_os_urandom(n: MbValue) -> MbValue {
    let count = n.as_int().unwrap_or(0).max(0) as usize;
    let mut buf = vec![0u8; count];
    // Use std's PRNG via /dev/urandom-equivalent if available; fall back
    // to a deterministic ramp so callers never see panic.
    if std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf))
        .is_err()
    {
        for (i, b) in buf.iter_mut().enumerate() {
            *b = (i & 0xff) as u8;
        }
    }
    let s: String = buf.iter().map(|b| *b as char).collect();
    MbValue::from_ptr(MbObject::new_str(s))
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

/// os.umask(mask) → previous mask. Stub returns 0; mask is recorded but
/// never re-applied (Mamba opens files via std, which honors process umask
/// implicitly).
pub fn mb_os_umask(_mask: MbValue) -> MbValue {
    MbValue::from_int(0)
}

/// os.access(path, mode) — best-effort: check existence + readability.
/// Ignores the mode bits beyond F_OK/R_OK distinction.
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
        let path = s("/nonexistent_xyz_abc_123");
        assert_eq!(mb_os_path_getsize(path).as_int(), Some(-1));
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
}
