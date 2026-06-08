/// filecmp module for Mamba (#1261 long-tail).
///
/// Replaces the long_tail stub which returned False for every `cmp()`
/// call (totally broken for anyone diffing files). Implements CPython's
/// `cmp` / `cmpfiles` semantics on top of `std::fs`:
///
///   filecmp.cmp(f1, f2, shallow=True) -> bool
///     - Both files must be regular files; non-files return False.
///     - shallow=True: equal iff (mode, size, mtime) signatures match.
///     - shallow=False (or signatures differ): size mismatch -> False;
///       otherwise byte-by-byte compare in BUFSIZE chunks.
///
///   filecmp.cmpfiles(a, b, common, shallow=True) -> (match, mismatch, errors)
///     For each name in `common`, run cmp(a/name, b/name, shallow) and
///     bucket into one of the three return lists. Any error (stat /
///     read failure) goes into `errors`.
///
///   filecmp.clear_cache() -> None
///     We don't cache (CPython caches stat signatures per-call); the
///     surface stays for API parity.
///
/// `dircmp` stays as a callable class shell — CPython's dircmp does
/// recursive directory traversal with cached attributes (left/right/common/
/// diff_files/funny_files/...); not worth porting until a consumer needs it.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

const BUFSIZE: usize = 8192;
const DEFAULT_IGNORES: &[&str] = &[
    "RCS", "CVS", "tags", ".git", ".hg", ".bzr", "_darcs", "__pycache__",
];

unsafe fn args_slice<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(args_ptr, nargs)
    }
}

unsafe fn as_str(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    match &(*ptr).data {
        ObjData::Str(s) => Some(s.clone()),
        ObjData::Bytes(b) => std::str::from_utf8(b).ok().map(str::to_string),
        _ => None,
    }
}

fn arg_bool(val: Option<MbValue>, default: bool) -> bool {
    let Some(v) = val else { return default; };
    if let Some(b) = v.as_bool() {
        return b;
    }
    if let Some(i) = v.as_int() {
        return i != 0;
    }
    default
}

unsafe fn collect_strings(val: MbValue) -> Vec<String> {
    let Some(ptr) = val.as_ptr() else { return Vec::new(); };
    match &(*ptr).data {
        ObjData::List(lock) => lock.read().unwrap().iter().filter_map(|v| as_str(*v)).collect(),
        ObjData::Tuple(items) => items.iter().filter_map(|v| as_str(*v)).collect(),
        _ => Vec::new(),
    }
}

#[derive(Clone, Copy)]
struct Sig {
    is_file: bool,
    size: u64,
    mtime_secs: i64,
    mtime_nanos: u32,
}

fn signature(path: &Path) -> Option<Sig> {
    let md = fs::metadata(path).ok()?;
    let is_file = md.is_file();
    let size = md.len();
    let (mtime_secs, mtime_nanos) = match md.modified() {
        Ok(t) => match t.duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => (d.as_secs() as i64, d.subsec_nanos()),
            Err(e) => (-(e.duration().as_secs() as i64), 0),
        },
        Err(_) => (0, 0),
    };
    Some(Sig { is_file, size, mtime_secs, mtime_nanos })
}

fn sigs_match(a: &Sig, b: &Sig) -> bool {
    a.is_file && b.is_file
        && a.size == b.size
        && a.mtime_secs == b.mtime_secs
        && a.mtime_nanos == b.mtime_nanos
}

fn byte_compare(p1: &Path, p2: &Path) -> std::io::Result<bool> {
    let mut f1 = fs::File::open(p1)?;
    let mut f2 = fs::File::open(p2)?;
    let mut b1 = vec![0u8; BUFSIZE];
    let mut b2 = vec![0u8; BUFSIZE];
    loop {
        let n1 = f1.read(&mut b1)?;
        let n2 = f2.read(&mut b2)?;
        if n1 != n2 {
            return Ok(false);
        }
        if n1 == 0 {
            return Ok(true);
        }
        if b1[..n1] != b2[..n2] {
            return Ok(false);
        }
    }
}

fn cmp_files(p1: &Path, p2: &Path, shallow: bool) -> Result<bool, std::io::Error> {
    let s1 = signature(p1).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "stat f1"))?;
    let s2 = signature(p2).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "stat f2"))?;
    if !s1.is_file || !s2.is_file {
        return Ok(false);
    }
    if shallow && sigs_match(&s1, &s2) {
        return Ok(true);
    }
    if s1.size != s2.size {
        return Ok(false);
    }
    byte_compare(p1, p2)
}

unsafe extern "C" fn dispatch_cmp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let (Some(a), Some(b)) = (args.first().copied(), args.get(1).copied()) else {
        return MbValue::from_bool(false);
    };
    let (Some(p1), Some(p2)) = (as_str(a), as_str(b)) else {
        return MbValue::from_bool(false);
    };
    let shallow = arg_bool(args.get(2).copied(), true);
    match cmp_files(Path::new(&p1), Path::new(&p2), shallow) {
        Ok(eq) => MbValue::from_bool(eq),
        Err(_) => MbValue::from_bool(false),
    }
}

unsafe extern "C" fn dispatch_cmpfiles(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let empty = || {
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_list(Vec::new())),
            MbValue::from_ptr(MbObject::new_list(Vec::new())),
            MbValue::from_ptr(MbObject::new_list(Vec::new())),
        ]))
    };
    let (Some(a), Some(b), Some(c)) = (
        args.first().copied(),
        args.get(1).copied(),
        args.get(2).copied(),
    ) else {
        return empty();
    };
    let (Some(da), Some(db)) = (as_str(a), as_str(b)) else {
        return empty();
    };
    let common = collect_strings(c);
    let shallow = arg_bool(args.get(3).copied(), true);

    let dir_a = PathBuf::from(da);
    let dir_b = PathBuf::from(db);
    let mut matches = Vec::new();
    let mut mismatches = Vec::new();
    let mut errors = Vec::new();
    for name in common {
        let pa = dir_a.join(&name);
        let pb = dir_b.join(&name);
        match cmp_files(&pa, &pb, shallow) {
            Ok(true) => matches.push(MbValue::from_ptr(MbObject::new_str(name))),
            Ok(false) => mismatches.push(MbValue::from_ptr(MbObject::new_str(name))),
            Err(_) => errors.push(MbValue::from_ptr(MbObject::new_str(name))),
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_ptr(MbObject::new_list(matches)),
        MbValue::from_ptr(MbObject::new_list(mismatches)),
        MbValue::from_ptr(MbObject::new_list(errors)),
    ]))
}

unsafe extern "C" fn dispatch_clear_cache(_a: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

unsafe extern "C" fn dispatch_dircmp_shell(_a: *const MbValue, _n: usize) -> MbValue {
    // CPython's dircmp is a complex recursive class. Hand back an empty
    // dict so `filecmp.dircmp(a, b)` doesn't crash; callers that probe
    // for real attributes will get None/false rather than wrong data.
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs = HashMap::new();

    let addr_cmp = dispatch_cmp as *const () as usize;
    let addr_cmpfiles = dispatch_cmpfiles as *const () as usize;
    let addr_clear = dispatch_clear_cache as *const () as usize;
    let addr_dircmp = dispatch_dircmp_shell as *const () as usize;

    attrs.insert("cmp".into(), MbValue::from_func(addr_cmp));
    attrs.insert("cmpfiles".into(), MbValue::from_func(addr_cmpfiles));
    attrs.insert("clear_cache".into(), MbValue::from_func(addr_clear));
    attrs.insert("dircmp".into(), MbValue::from_func(addr_dircmp));

    attrs.insert("BUFSIZE".into(), MbValue::from_int(BUFSIZE as i64));
    let ignores: Vec<MbValue> = DEFAULT_IGNORES
        .iter()
        .map(|s| MbValue::from_ptr(MbObject::new_str((*s).to_string())))
        .collect();
    attrs.insert(
        "DEFAULT_IGNORES".into(),
        MbValue::from_ptr(MbObject::new_list(ignores)),
    );

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_cmp as u64);
        set.insert(addr_cmpfiles as u64);
        set.insert(addr_clear as u64);
        set.insert(addr_dircmp as u64);
    });

    super::register_module("filecmp", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("mamba_filecmp_{}_{}", std::process::id(), name));
        p
    }

    fn write_bytes(path: &Path, data: &[u8]) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(data).unwrap();
    }

    #[test]
    fn cmp_returns_true_for_identical_files() {
        let a = tmp_path("ia.bin");
        let b = tmp_path("ib.bin");
        write_bytes(&a, b"hello world");
        write_bytes(&b, b"hello world");
        let result = cmp_files(&a, &b, false).unwrap();
        assert!(result, "identical content must compare equal");
        let _ = fs::remove_file(&a);
        let _ = fs::remove_file(&b);
    }

    #[test]
    fn cmp_returns_false_for_different_sizes() {
        let a = tmp_path("sa.bin");
        let b = tmp_path("sb.bin");
        write_bytes(&a, b"short");
        write_bytes(&b, b"a much longer payload");
        let result = cmp_files(&a, &b, false).unwrap();
        assert!(!result);
        let _ = fs::remove_file(&a);
        let _ = fs::remove_file(&b);
    }

    #[test]
    fn cmp_returns_false_for_same_size_different_content() {
        let a = tmp_path("da.bin");
        let b = tmp_path("db.bin");
        write_bytes(&a, b"aaaaa");
        write_bytes(&b, b"bbbbb");
        let result = cmp_files(&a, &b, false).unwrap();
        assert!(!result);
        let _ = fs::remove_file(&a);
        let _ = fs::remove_file(&b);
    }

    #[test]
    fn cmp_missing_file_returns_err() {
        let a = tmp_path("present.bin");
        let b = tmp_path("does_not_exist.bin");
        write_bytes(&a, b"x");
        let result = cmp_files(&a, &b, false);
        assert!(result.is_err());
        let _ = fs::remove_file(&a);
    }

    #[test]
    fn cmp_directory_inputs_false() {
        let d = tmp_path("dir_in");
        fs::create_dir_all(&d).unwrap();
        let a = tmp_path("file_in");
        write_bytes(&a, b"x");
        let result = cmp_files(&d, &a, false).unwrap();
        assert!(!result, "non-regular file inputs must be false");
        let _ = fs::remove_dir_all(&d);
        let _ = fs::remove_file(&a);
    }

    #[test]
    fn shallow_match_on_identical_stat_returns_true() {
        let a = tmp_path("shal_a.bin");
        write_bytes(&a, b"same");
        // Compare the file with itself: signatures match exactly.
        let result = cmp_files(&a, &a, true).unwrap();
        assert!(result);
        let _ = fs::remove_file(&a);
    }

    #[test]
    fn dispatch_cmp_via_str_args() {
        let a = tmp_path("dca.bin");
        let b = tmp_path("dcb.bin");
        write_bytes(&a, b"payload");
        write_bytes(&b, b"payload");
        let av = MbValue::from_ptr(MbObject::new_str(a.to_string_lossy().into_owned()));
        let bv = MbValue::from_ptr(MbObject::new_str(b.to_string_lossy().into_owned()));
        let argv = [av, bv];
        let r = unsafe { dispatch_cmp(argv.as_ptr(), argv.len()) };
        assert_eq!(r.as_bool(), Some(true));
        let _ = fs::remove_file(&a);
        let _ = fs::remove_file(&b);
    }

    #[test]
    fn cmpfiles_buckets_three_categories() {
        let dir_a = tmp_path("ca_dir");
        let dir_b = tmp_path("cb_dir");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();
        write_bytes(&dir_a.join("same.txt"), b"hello");
        write_bytes(&dir_b.join("same.txt"), b"hello");
        write_bytes(&dir_a.join("diff.txt"), b"foo");
        write_bytes(&dir_b.join("diff.txt"), b"bar");
        write_bytes(&dir_a.join("only_a.txt"), b"x");
        // common includes a name only present in `a` to exercise the error path.
        let common = vec![
            MbValue::from_ptr(MbObject::new_str("same.txt".into())),
            MbValue::from_ptr(MbObject::new_str("diff.txt".into())),
            MbValue::from_ptr(MbObject::new_str("only_a.txt".into())),
        ];
        let common_list = MbValue::from_ptr(MbObject::new_list(common));
        let av = MbValue::from_ptr(MbObject::new_str(dir_a.to_string_lossy().into_owned()));
        let bv = MbValue::from_ptr(MbObject::new_str(dir_b.to_string_lossy().into_owned()));
        let argv = [av, bv, common_list];
        let r = unsafe { dispatch_cmpfiles(argv.as_ptr(), argv.len()) };
        unsafe {
            let p = r.as_ptr().expect("ptr");
            if let ObjData::Tuple(items) = &(*p).data {
                assert_eq!(items.len(), 3);
                let read_list = |v: MbValue| -> Vec<String> {
                    let ptr = v.as_ptr().unwrap();
                    if let ObjData::List(lock) = &(*ptr).data {
                        lock.read().unwrap().iter().filter_map(|x| {
                            let xp = x.as_ptr()?;
                            if let ObjData::Str(s) = &(*xp).data {
                                Some(s.clone())
                            } else {
                                None
                            }
                        }).collect()
                    } else {
                        Vec::new()
                    }
                };
                let matches = read_list(items[0]);
                let mismatches = read_list(items[1]);
                let errors = read_list(items[2]);
                assert_eq!(matches, vec!["same.txt"]);
                assert_eq!(mismatches, vec!["diff.txt"]);
                assert_eq!(errors, vec!["only_a.txt"]);
            } else {
                panic!("expected tuple");
            }
        }
        let _ = fs::remove_dir_all(&dir_a);
        let _ = fs::remove_dir_all(&dir_b);
    }

    #[test]
    fn default_ignores_registered() {
        register();
        // Just ensure register() doesn't panic; full attr lookup is exercised
        // via the integration surface.
    }
}
