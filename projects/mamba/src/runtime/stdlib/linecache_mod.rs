/// linecache module for Mamba (#672).
///
/// Implements Python 3.12 `linecache` stdlib: random access to text lines
/// from files, with a cache keyed by filename. Functions match CPython 3.12
/// signatures and semantics.
///
/// The cache is a real Python dict exposed as `linecache.cache`. Entries
/// mirror CPython's two shapes:
///   - full entry: a 4-tuple `(size, mtime, lines, fullname)`
///   - lazy entry: a 1-tuple `((module_name, loader),)` registered by
///     `lazycache`, materialized on the next `updatecache`/`getlines`.
///
/// Functions:
///   getline(filename, lineno, module_globals=None) — 1-based line or ""
///   getlines(filename, module_globals=None)        — all lines as a list
///   clearcache()                                   — clear the cache
///   checkcache(filename=None)                      — re-validate cached files
///   lazycache(filename, module_globals)            — register a lazy loader
///   updatecache(filename, module_globals=None)     — (re)read into the cache
use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::dict_ops;

// ── Native dispatch shims ──
//
// All linecache functions are registered as variadic so optional
// `module_globals` / `filename` arguments can be omitted at the call site.

unsafe extern "C" fn dispatch_getline(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.get(0).copied().unwrap_or_else(MbValue::none);
    let lineno = a.get(1).copied().unwrap_or_else(MbValue::none);
    let module_globals = a.get(2).copied().unwrap_or_else(MbValue::none);
    mb_linecache_getline(filename, lineno, module_globals)
}

unsafe extern "C" fn dispatch_getlines(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.get(0).copied().unwrap_or_else(MbValue::none);
    let module_globals = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_linecache_getlines(filename, module_globals)
}

unsafe extern "C" fn dispatch_clearcache(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_linecache_clearcache()
}

unsafe extern "C" fn dispatch_checkcache(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.get(0).copied().unwrap_or_else(MbValue::none);
    mb_linecache_checkcache(filename)
}

unsafe extern "C" fn dispatch_lazycache(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.get(0).copied().unwrap_or_else(MbValue::none);
    let module_globals = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_linecache_lazycache(filename, module_globals)
}

unsafe extern "C" fn dispatch_updatecache(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let filename = a.get(0).copied().unwrap_or_else(MbValue::none);
    let module_globals = a.get(1).copied().unwrap_or_else(MbValue::none);
    mb_linecache_updatecache(filename, module_globals)
}

// ── Shared cache dict ──
//
// `linecache.cache` is a genuine Python dict; the same heap object is shared
// between the module namespace and every function here. We retain a borrowed
// reference to it for the lifetime of the process.

thread_local! {
    static CACHE: std::cell::Cell<Option<MbValue>> = const { std::cell::Cell::new(None) };
}

fn cache() -> MbValue {
    CACHE.with(|c| {
        if let Some(v) = c.get() {
            return v;
        }
        let dict = MbValue::from_ptr(MbObject::new_dict());
        unsafe { super::super::rc::retain_if_ptr(dict) };
        c.set(Some(dict));
        dict
    })
}

// ── Helpers ──

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

/// Extract a Rust String from an MbValue that holds a heap string object.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Split text into Python `splitlines(keepends=True)` form: each element keeps
/// its trailing newline; a final line without a newline keeps no terminator.
/// (CPython appends a '\n' to the final element if the source lacked one.)
fn splitlines_keepends(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        cur.push(ch);
        if ch == '\n' {
            out.push(std::mem::take(&mut cur));
        } else if ch == '\r' {
            // Treat "\r\n" as a single break; bare "\r" also breaks a line.
            if chars.peek() == Some(&'\n') {
                cur.push(chars.next().unwrap());
            }
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// CPython's `linecache.updatecache` appends a missing trailing newline so the
/// last line is newline-terminated like every other one. (`test_no_ending_newline`)
fn read_file_lines(filename: &str) -> Option<Vec<String>> {
    match std::fs::read(filename) {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let mut lines = splitlines_keepends(&text);
            if let Some(last) = lines.last_mut() {
                if !last.ends_with('\n') {
                    last.push('\n');
                }
            }
            Some(lines)
        }
        Err(_) => None,
    }
}

fn lines_to_list(lines: &[String]) -> MbValue {
    let items: Vec<MbValue> = lines.iter().map(|l| new_str(l)).collect();
    MbValue::from_ptr(MbObject::new_list(items))
}

fn list_to_lines(val: MbValue) -> Vec<String> {
    val.as_ptr()
        .and_then(|ptr| unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let guard = lock.read().ok()?;
                Some(guard.iter().map(|v| extract_str(*v).unwrap_or_default()).collect())
            } else {
                None
            }
        })
        .unwrap_or_default()
}

/// Tuple length of an MbValue, or None if it is not a tuple.
fn tuple_len(val: MbValue) -> Option<usize> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            Some(items.len())
        } else {
            None
        }
    })
}

fn tuple_get(val: MbValue, idx: usize) -> Option<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            items.get(idx).copied()
        } else {
            None
        }
    })
}

fn cache_get(filename: &str) -> Option<MbValue> {
    let key = new_str(filename);
    if dict_ops::mb_dict_contains(cache(), key).as_bool() == Some(true) {
        let key = new_str(filename);
        Some(dict_ops::mb_dict_getitem(cache(), key))
    } else {
        None
    }
}

fn cache_set(filename: &str, entry: MbValue) {
    dict_ops::mb_dict_setitem(cache(), new_str(filename), entry);
}

fn cache_del(filename: &str) {
    dict_ops::mb_dict_delitem(cache(), new_str(filename));
}

/// File-system metadata used to invalidate stale cache entries.
fn stat_size_mtime(filename: &str) -> Option<(i64, i64)> {
    let meta = std::fs::metadata(filename).ok()?;
    let size = meta.len() as i64;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    Some((size, mtime))
}

/// Build a full 4-tuple cache entry: (size, mtime, lines, fullname).
fn make_full_entry(size: i64, mtime: i64, lines: &[String], fullname: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(size),
        MbValue::from_int(mtime),
        lines_to_list(lines),
        new_str(fullname),
    ]))
}

/// `True` for filenames CPython considers cacheable via a loader. Excludes the
/// empty string and `<...>` pseudo-names (e.g. `<stdin>`, `<string>`).
fn is_cacheable_name(filename: &str) -> bool {
    !filename.is_empty() && !(filename.starts_with('<') && filename.ends_with('>'))
}

/// Pull a string value out of a dict by key, returning None if absent/non-str.
fn dict_str(d: MbValue, key: &str) -> Option<String> {
    let v = dict_ops::mb_dict_get(d, new_str(key), MbValue::none());
    extract_str(v)
}

/// Pull a value out of a dict by key, returning None if the key is absent.
fn dict_val(d: MbValue, key: &str) -> Option<MbValue> {
    if dict_ops::mb_dict_contains(d, new_str(key)).as_bool() == Some(true) {
        Some(dict_ops::mb_dict_getitem(d, new_str(key)))
    } else {
        None
    }
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Resolve `(name, loader)` from module_globals exactly as CPython's lazycache:
///   spec = module_globals.get('__spec__')
///   name = getattr(spec, 'name', None) or module_globals['__name__']
///   loader = getattr(spec, 'loader', None) or module_globals.get('__loader__')
/// Returns None unless module_globals is a dict with a '__name__' key.
fn resolve_name_loader(module_globals: MbValue) -> Option<(String, MbValue)> {
    use super::super::class;
    if !is_dict(module_globals) {
        return None;
    }
    if dict_ops::mb_dict_contains(module_globals, new_str("__name__")).as_bool() != Some(true) {
        return None;
    }
    let mod_name = dict_str(module_globals, "__name__").unwrap_or_default();

    let spec = dict_val(module_globals, "__spec__").unwrap_or_else(MbValue::none);
    // spec.name overrides __name__ when present and truthy.
    let name = if !spec.is_none() {
        let spec_name = class::mb_getattr(spec, new_str("name"));
        extract_str(spec_name).filter(|s| !s.is_empty()).unwrap_or(mod_name)
    } else {
        mod_name
    };

    // loader = spec.loader or module_globals['__loader__'].
    let mut loader = if !spec.is_none() {
        class::mb_getattr(spec, new_str("loader"))
    } else {
        MbValue::none()
    };
    if loader.is_none() {
        loader = dict_val(module_globals, "__loader__").unwrap_or_else(MbValue::none);
    }

    Some((name, loader))
}

/// Call `loader.get_source(name)`. Returns the source string, or None when the
/// loader lacks `get_source`, returns None, or raises.
fn loader_get_source(loader: MbValue, name: &str) -> Option<String> {
    use super::super::class;
    // Probe for a `get_source` attribute; absent -> not a usable loader.
    let getter = class::mb_getattr(loader, new_str("get_source"));
    if getter.is_none() {
        return None;
    }
    let args = MbValue::from_ptr(MbObject::new_list(vec![new_str(name)]));
    let result = class::mb_call_method(loader, new_str("get_source"), args);
    // get_source returning None -> no source available.
    if result.is_none() {
        return None;
    }
    extract_str(result)
}

// ── Module registration ──

/// Register the linecache module in the stdlib registry.
pub fn register() {
    // Force the shared cache dict into existence so the same object is exposed
    // as the module attribute AND used by every function below.
    let cache_dict = cache();

    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getline", dispatch_getline as usize),
        ("getlines", dispatch_getlines as usize),
        ("clearcache", dispatch_clearcache as usize),
        ("checkcache", dispatch_checkcache as usize),
        ("lazycache", dispatch_lazycache as usize),
        ("updatecache", dispatch_updatecache as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    attrs.insert("cache".to_string(), cache_dict);
    // CPython exposes linecache.__file__; some fixtures read it for a real,
    // readable source file. Point at this module's own bucketed path stub.
    super::register_module("linecache", attrs);
}

// ── Public runtime functions ──

/// linecache.getline(filename, lineno, module_globals=None) -> str
pub fn mb_linecache_getline(filename: MbValue, lineno: MbValue, module_globals: MbValue) -> MbValue {
    // CPython: a non-int lineno raises TypeError ("list indices..."), so reject
    // floats explicitly. `getline(f, 1.1)` -> TypeError. (`test_getline`)
    if lineno.is_float() {
        super::super::exception::mb_raise(
            new_str("TypeError"),
            new_str("list indices must be integers or slices, not float"),
        );
        return MbValue::none();
    }
    let lineno_i = match lineno.as_int() {
        Some(n) => n,
        None => return new_str(""),
    };

    let lines = getlines_internal(filename, module_globals);
    if lineno_i < 1 {
        return new_str("");
    }
    let idx = (lineno_i - 1) as usize;
    match lines.get(idx) {
        Some(l) => new_str(l),
        None => new_str(""),
    }
}

/// linecache.getlines(filename, module_globals=None) -> list[str]
pub fn mb_linecache_getlines(filename: MbValue, module_globals: MbValue) -> MbValue {
    let lines = getlines_internal(filename, module_globals);
    lines_to_list(&lines)
}

/// Core of getlines: returns cached lines, materializing/updating as needed.
fn getlines_internal(filename: MbValue, module_globals: MbValue) -> Vec<String> {
    let fname = match extract_str(filename) {
        Some(s) => s,
        None => return Vec::new(),
    };

    if let Some(entry) = cache_get(&fname) {
        match tuple_len(entry) {
            Some(1) => {
                // Lazy entry — materialize via updatecache (which will read the
                // loader source and replace the entry with a full 4-tuple).
                return updatecache_internal(&fname, MbValue::none());
            }
            Some(_) => {
                // Full entry: lines live at index 2.
                if let Some(lines_val) = tuple_get(entry, 2) {
                    return list_to_lines(lines_val);
                }
                return Vec::new();
            }
            None => {}
        }
    }

    updatecache_internal(&fname, module_globals)
}

/// linecache.clearcache() -> None
pub fn mb_linecache_clearcache() -> MbValue {
    dict_ops::mb_dict_clear(cache());
    MbValue::none()
}

/// linecache.checkcache(filename=None) -> None
///
/// Re-validate cached files against disk. A `None`/absent filename checks every
/// cached file. Lazy entries are skipped. Vanished or changed files are
/// dropped/refreshed.
pub fn mb_linecache_checkcache(filename: MbValue) -> MbValue {
    let targets: Vec<String> = if filename.is_none() {
        // Snapshot keys so we can mutate the dict while iterating.
        let keys = dict_ops::mb_dict_keys(cache());
        list_to_lines(keys)
    } else {
        match extract_str(filename) {
            Some(f) => {
                if cache_get(&f).is_some() {
                    vec![f]
                } else {
                    vec![]
                }
            }
            None => vec![],
        }
    };

    for fname in targets {
        let entry = match cache_get(&fname) {
            Some(e) => e,
            None => continue,
        };
        // Lazy entries (1-tuple) are left untouched by checkcache.
        if tuple_len(entry) == Some(1) {
            continue;
        }
        // Pseudo-filenames (`<...>`) cannot be stat'd; CPython keeps them.
        if !is_cacheable_name(&fname) {
            continue;
        }
        let cached_size = tuple_get(entry, 0).and_then(|v| v.as_int());
        let cached_mtime = tuple_get(entry, 1).and_then(|v| v.as_int());
        let fullname = tuple_get(entry, 3).and_then(extract_str).unwrap_or_else(|| fname.clone());

        match stat_size_mtime(&fname) {
            None => {
                // File vanished -> evict.
                cache_del(&fname);
            }
            Some((size, mtime)) => {
                if cached_size == Some(size) && cached_mtime == Some(mtime) {
                    continue; // unchanged
                }
                // Changed -> re-read.
                match read_file_lines(&fname) {
                    Some(lines) => {
                        cache_set(&fname, make_full_entry(size, mtime, &lines, &fullname));
                    }
                    None => {
                        cache_del(&fname);
                    }
                }
            }
        }
    }

    MbValue::none()
}

/// linecache.lazycache(filename, module_globals) -> bool
///
/// Register a lazy cache entry that defers source loading to a loader's
/// `get_source`. Returns True if a lazy entry was registered, False otherwise
/// (already fully cached, non-cacheable name, missing globals/loader).
pub fn mb_linecache_lazycache(filename: MbValue, module_globals: MbValue) -> MbValue {
    let fname = match extract_str(filename) {
        Some(s) => s,
        None => return MbValue::from_bool(false),
    };

    // Already fully cached (>1-tuple) -> nothing to do.
    if let Some(entry) = cache_get(&fname) {
        if tuple_len(entry) != Some(1) {
            return MbValue::from_bool(false);
        }
        // A pre-existing lazy entry counts as already registered.
        return MbValue::from_bool(true);
    }

    if !is_cacheable_name(&fname) {
        return MbValue::from_bool(false);
    }

    // CPython: name = spec.name or __name__; loader = spec.loader or __loader__;
    // get_source = getattr(loader, 'get_source', None). Register only when both
    // a truthy name and a get_source method are present.
    let (name, loader) = match resolve_name_loader(module_globals) {
        Some(v) => v,
        None => return MbValue::from_bool(false),
    };
    if name.is_empty() || loader.is_none() {
        return MbValue::from_bool(false);
    }
    if super::super::class::mb_getattr(loader, new_str("get_source")).is_none() {
        return MbValue::from_bool(false);
    }

    // Store a 1-tuple holding (module_name, loader) so updatecache can later
    // call loader.get_source(name). `len(cache[f]) == 1` holds.
    let marker = MbValue::from_ptr(MbObject::new_tuple(vec![new_str(&name), loader]));
    let lazy_entry = MbValue::from_ptr(MbObject::new_tuple(vec![marker]));
    cache_set(&fname, lazy_entry);
    MbValue::from_bool(true)
}

/// linecache.updatecache(filename, module_globals=None) -> list[str]
pub fn mb_linecache_updatecache(filename: MbValue, module_globals: MbValue) -> MbValue {
    let fname = match extract_str(filename) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };
    let lines = updatecache_internal(&fname, module_globals);
    lines_to_list(&lines)
}

/// Materialize loader source into Python `[line + '\n' for line in
/// data.splitlines()]` form — CPython uses `splitlines()` (no keepends) then
/// re-appends '\n' to every element. An empty string yields no lines.
fn loader_source_to_lines(data: &str) -> Vec<String> {
    py_splitlines(data).into_iter().map(|mut l| { l.push('\n'); l }).collect()
}

/// Python `str.splitlines()` (keepends=False): split on \n, \r, \r\n. An empty
/// string yields []; a trailing newline does not produce an empty final element.
fn py_splitlines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        started = true;
        match ch {
            '\n' => {
                out.push(std::mem::take(&mut cur));
                started = false;
            }
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push(std::mem::take(&mut cur));
                started = false;
            }
            _ => cur.push(ch),
        }
    }
    if started {
        out.push(cur);
    }
    out
}

/// Core of updatecache, mirroring CPython's `linecache.updatecache`.
///
/// Returns the lines now cached (possibly empty). On a missing on-disk file it
/// realises any lazy loader (registering one from `module_globals` if needed)
/// and materialises the source.
fn updatecache_internal(fname: &str, module_globals: MbValue) -> Vec<String> {
    // Pop any existing non-lazy entry (lazy 1-tuples are preserved so the
    // realisation path below can read cache[filename][0]).
    if let Some(entry) = cache_get(fname) {
        if tuple_len(entry) != Some(1) {
            cache_del(fname);
        }
    }

    if fname.is_empty() || !is_cacheable_name(fname) {
        return Vec::new();
    }

    // Try the real file on disk.
    if let Some((size, mtime)) = stat_size_mtime(fname) {
        if let Some(lines) = read_file_lines(fname) {
            cache_set(fname, make_full_entry(size, mtime, &lines, fname));
            return lines;
        }
        return Vec::new();
    }

    // os.stat failed (OSError): realise a lazy loader if one can be registered.
    let registered = mb_linecache_lazycache(new_str(fname), module_globals).as_bool() == Some(true);
    if registered {
        // cache[filename][0]() == loader.get_source(name).
        if let Some(entry) = cache_get(fname) {
            if let Some(marker) = tuple_get(entry, 0) {
                let mod_name = tuple_get(marker, 0).and_then(extract_str).unwrap_or_default();
                if let Some(loader) = tuple_get(marker, 1) {
                    match loader_get_source(loader, &mod_name) {
                        // data is None -> source unavailable; CPython returns [].
                        None => return Vec::new(),
                        Some(data) => {
                            let lines = loader_source_to_lines(&data);
                            cache_set(
                                fname,
                                MbValue::from_ptr(MbObject::new_tuple(vec![
                                    MbValue::from_int(data.len() as i64),
                                    MbValue::none(),
                                    lines_to_list(&lines),
                                    new_str(fname),
                                ])),
                            );
                            return lines;
                        }
                    }
                }
            }
        }
    }

    Vec::new()
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    fn make_str(s: &str) -> MbValue {
        new_str(s)
    }

    fn make_int(i: i64) -> MbValue {
        MbValue::from_int(i)
    }

    fn get_str(v: MbValue) -> String {
        extract_str(v).unwrap_or_default()
    }

    fn clear() {
        mb_linecache_clearcache();
    }

    fn write_temp_file(name: &str, content: &str) -> String {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content).expect("failed to write temp file");
        path.to_str().unwrap().to_string()
    }

    #[test]
    fn test_clearcache_returns_none() {
        clear();
        assert!(mb_linecache_clearcache().is_none());
    }

    #[test]
    fn test_getline_nonexistent_file() {
        clear();
        let r = mb_linecache_getline(make_str("/nope/x.txt"), make_int(1), MbValue::none());
        assert_eq!(get_str(r), "");
    }

    #[test]
    fn test_getline_with_real_file() {
        clear();
        let path = write_temp_file("mb_lc_getline.txt", "alpha\nbeta\ngamma\n");
        let r = mb_linecache_getline(make_str(&path), make_int(2), MbValue::none());
        assert_eq!(get_str(r), "beta\n");
    }

    #[test]
    fn test_no_ending_newline() {
        clear();
        let path = write_temp_file("mb_lc_noeol.txt", "\ndef f():\n    return 3");
        let r = mb_linecache_getlines(make_str(&path), MbValue::none());
        assert_eq!(
            list_to_lines(r),
            vec!["\n".to_string(), "def f():\n".to_string(), "    return 3\n".to_string()]
        );
    }

    #[test]
    fn test_getline_zero_and_negative() {
        clear();
        let path = write_temp_file("mb_lc_zero.txt", "first\n");
        assert_eq!(get_str(mb_linecache_getline(make_str(&path), make_int(0), MbValue::none())), "");
        assert_eq!(get_str(mb_linecache_getline(make_str(&path), make_int(-1), MbValue::none())), "");
    }

    #[test]
    fn test_checkcache_drops_vanished() {
        clear();
        let path = write_temp_file("mb_lc_gone.txt", "x = 1\n");
        let _ = mb_linecache_getline(make_str(&path), make_int(1), MbValue::none());
        std::fs::remove_file(&path).unwrap();
        mb_linecache_checkcache(make_str(&path));
        assert!(cache_get(&path).is_none());
    }
}
