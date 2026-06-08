use super::super::rc::MbObject;
use super::super::value::MbValue;
/// linecache module for Mamba (#672).
///
/// Implements Python 3.12 `linecache` stdlib: random access to text lines
/// from files, with a cache keyed by filename. Functions match CPython 3.12
/// signatures.
///
/// Functions:
///   getline(filename, lineno) — return the line at 1-based lineno (or "")
///   getlines(filename)        — return all lines as a list
///   clearcache()              — clear the entire line cache
///   checkcache()              — re-validate all cached files from disk
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

macro_rules! dispatch_nullary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

dispatch_binary!(dispatch_getline, mb_linecache_getline);
dispatch_unary!(dispatch_getlines, mb_linecache_getlines);
dispatch_nullary!(dispatch_clearcache, mb_linecache_clearcache);
dispatch_nullary!(dispatch_checkcache, mb_linecache_checkcache);

// ── Internal line cache ──

thread_local! {
    static LINE_CACHE: std::cell::RefCell<HashMap<String, Vec<String>>> =
        std::cell::RefCell::new(HashMap::new());
}

// ── Helpers ──

/// Extract a Rust String from an MbValue that holds a heap string object.
/// Returns None if the value is not a string.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Extract an i64 from an MbValue (integer).
fn extract_int(val: MbValue) -> Option<i64> {
    val.as_int()
}

/// Read the file at `filename` into LINE_CACHE if not already present.
/// Returns true if the file was found (either already cached or just read).
fn ensure_cached(filename: &str) -> bool {
    LINE_CACHE.with(|c| {
        {
            let cache = c.borrow();
            if cache.contains_key(filename) {
                return true;
            }
        }
        match std::fs::read_to_string(filename) {
            Ok(contents) => {
                let lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
                c.borrow_mut().insert(filename.to_string(), lines);
                true
            }
            Err(_) => false,
        }
    })
}

// ── Module registration ──

/// Register the linecache module in the stdlib registry.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("getline", dispatch_getline as usize),
        ("getlines", dispatch_getlines as usize),
        ("clearcache", dispatch_clearcache as usize),
        ("checkcache", dispatch_checkcache as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }
    super::register_module("linecache", attrs);
}

// ── Public runtime functions ──

/// linecache.getline(filename, lineno) -> str
///
/// Return the line at 1-based lineno from filename (cached). Returns an
/// empty string if the file does not exist or lineno is out of range.
pub fn mb_linecache_getline(filename: MbValue, lineno: MbValue) -> MbValue {
    let filename_s = match extract_str(filename) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_str(String::new())),
    };
    let lineno_i = match extract_int(lineno) {
        Some(n) => n,
        None => return MbValue::from_ptr(MbObject::new_str(String::new())),
    };

    // 1-based index; lineno <= 0 is out of range.
    if lineno_i <= 0 {
        return MbValue::from_ptr(MbObject::new_str(String::new()));
    }

    if !ensure_cached(&filename_s) {
        return MbValue::from_ptr(MbObject::new_str(String::new()));
    }

    let line = LINE_CACHE.with(|c| {
        let cache = c.borrow();
        cache
            .get(&filename_s)
            .and_then(|lines| lines.get((lineno_i - 1) as usize))
            .cloned()
    });

    MbValue::from_ptr(MbObject::new_str(line.unwrap_or_default()))
}

/// linecache.getlines(filename) -> list[str]
///
/// Return all lines from filename as a list (cached). Returns an empty
/// list if the file does not exist.
pub fn mb_linecache_getlines(filename: MbValue) -> MbValue {
    let filename_s = match extract_str(filename) {
        Some(s) => s,
        None => return MbValue::from_ptr(MbObject::new_list(vec![])),
    };

    if !ensure_cached(&filename_s) {
        return MbValue::from_ptr(MbObject::new_list(vec![]));
    }

    let lines: Vec<MbValue> = LINE_CACHE.with(|c| {
        let cache = c.borrow();
        cache
            .get(&filename_s)
            .map(|lines| {
                lines
                    .iter()
                    .map(|l| MbValue::from_ptr(MbObject::new_str(l.clone())))
                    .collect()
            })
            .unwrap_or_default()
    });

    MbValue::from_ptr(MbObject::new_list(lines))
}

/// linecache.clearcache() -> None
///
/// Clear the entire line cache. Equivalent to CPython's linecache.clearcache().
pub fn mb_linecache_clearcache() -> MbValue {
    LINE_CACHE.with(|c| {
        c.borrow_mut().clear();
    });
    MbValue::none()
}

/// linecache.checkcache() -> None
///
/// Re-validate cached files against disk. Files that have disappeared are
/// removed from the cache; files still present are re-read to pick up changes.
/// Equivalent to CPython's linecache.checkcache().
pub fn mb_linecache_checkcache() -> MbValue {
    // Collect all currently cached filenames.
    let filenames: Vec<String> = LINE_CACHE.with(|c| c.borrow().keys().cloned().collect());

    for filename in &filenames {
        match std::fs::read_to_string(filename) {
            Ok(contents) => {
                let lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
                LINE_CACHE.with(|c| {
                    c.borrow_mut().insert(filename.clone(), lines);
                });
            }
            Err(_) => {
                // File no longer accessible — evict from cache.
                LINE_CACHE.with(|c| {
                    c.borrow_mut().remove(filename);
                });
            }
        }
    }

    MbValue::none()
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::super::super::rc::ObjData;
    use super::*;

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    fn make_int(i: i64) -> MbValue {
        MbValue::from_int(i)
    }

    fn get_str(v: MbValue) -> String {
        extract_str(v).unwrap_or_default()
    }

    fn get_list_len(val: MbValue) -> usize {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::List(ref rw) = (*ptr).data {
                    rw.read().ok().map(|g| g.len())
                } else {
                    None
                }
            })
            .unwrap_or(0)
    }

    fn get_list_strings(val: MbValue) -> Vec<String> {
        val.as_ptr()
            .and_then(|ptr| unsafe {
                if let ObjData::List(ref rw) = (*ptr).data {
                    let guard = rw.read().ok()?;
                    let results: Vec<String> = guard
                        .iter()
                        .map(|v| extract_str(*v).unwrap_or_default())
                        .collect();
                    Some(results)
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    fn clear() {
        mb_linecache_clearcache();
    }

    fn write_temp_file(name: &str, content: &str) -> String {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content).expect("failed to write temp file");
        path.to_str().unwrap().to_string()
    }

    // REQ: R2, R4
    #[test]
    fn test_clearcache_returns_none() {
        clear();
        let result = mb_linecache_clearcache();
        assert!(result.is_none(), "clearcache() must return None");
    }

    // REQ: R2, R4
    #[test]
    fn test_getline_nonexistent_file() {
        clear();
        let result = mb_linecache_getline(
            make_str("/nonexistent/path/that/does/not/exist.txt"),
            make_int(1),
        );
        assert_eq!(
            get_str(result),
            "",
            "getline on missing file must return empty string"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_getlines_nonexistent_file() {
        clear();
        let result = mb_linecache_getlines(make_str("/nonexistent/path/that/does/not/exist.txt"));
        assert_eq!(
            get_list_len(result),
            0,
            "getlines on missing file must return empty list"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_getline_with_real_file() {
        clear();
        let path = write_temp_file("mb_linecache_test_getline.txt", "alpha\nbeta\ngamma\n");
        let result = mb_linecache_getline(make_str(&path), make_int(2));
        assert_eq!(
            get_str(result),
            "beta",
            "getline(file, 2) must return second line"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_getlines_with_real_file() {
        clear();
        let path = write_temp_file("mb_linecache_test_getlines.txt", "line1\nline2\nline3\n");
        let result = mb_linecache_getlines(make_str(&path));
        let lines = get_list_strings(result);
        assert_eq!(
            lines,
            vec!["line1", "line2", "line3"],
            "getlines must return all lines from file"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_getline_out_of_range() {
        clear();
        let path = write_temp_file("mb_linecache_test_oor.txt", "only_line\n");
        // File has 1 line; lineno 99 is out of range.
        let result = mb_linecache_getline(make_str(&path), make_int(99));
        assert_eq!(
            get_str(result),
            "",
            "getline with lineno beyond file length must return empty string"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_getline_zero_lineno() {
        clear();
        let path = write_temp_file("mb_linecache_test_zero.txt", "first_line\n");
        // lineno 0 is invalid (1-based indexing).
        let result = mb_linecache_getline(make_str(&path), make_int(0));
        assert_eq!(
            get_str(result),
            "",
            "getline with lineno=0 must return empty string (1-based indexing)"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_checkcache_after_file_change() {
        clear();
        let path = write_temp_file("mb_linecache_test_checkcache.txt", "original\n");
        // Prime the cache.
        let r1 = mb_linecache_getline(make_str(&path), make_int(1));
        assert_eq!(
            get_str(r1),
            "original",
            "initial getline must return 'original'"
        );

        // Overwrite the file with new content.
        std::fs::write(&path, "updated\n").expect("failed to update temp file");

        // checkcache() should refresh the cache.
        mb_linecache_checkcache();

        let r2 = mb_linecache_getline(make_str(&path), make_int(1));
        assert_eq!(
            get_str(r2),
            "updated",
            "getline after checkcache must return updated content"
        );
    }

    // REQ: R2, R4
    #[test]
    fn test_checkcache_returns_none() {
        clear();
        let result = mb_linecache_checkcache();
        assert!(result.is_none(), "checkcache() must return None");
    }
}
