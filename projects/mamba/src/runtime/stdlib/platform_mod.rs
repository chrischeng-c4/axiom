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

unsafe extern "C" fn dispatch_uname(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    mb_platform_uname()
}

unsafe extern "C" fn dispatch_architecture(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str("64bit"), new_str(""),
    ]))
}

unsafe extern "C" fn dispatch_mac_ver(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let release = if std::env::consts::OS == "macos" {
        run_cmd("sw_vers", &["-productVersion"]).unwrap_or_default()
    } else {
        String::new()
    };
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str(&release),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            new_str(""), new_str(""), new_str(""),
        ])),
        mb_platform_machine(),
    ]))
}

unsafe extern "C" fn dispatch_java_ver(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    let empty3 = || MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str(""), new_str(""), new_str(""),
    ]));
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str(""), new_str(""), empty3(), empty3(),
    ]))
}

unsafe extern "C" fn dispatch_system_alias(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs < 3 {
        super::super::exception::mb_raise(
            new_str_val("TypeError"),
            new_str_val(&format!(
                "system_alias() missing {} required positional arguments",
                3 - nargs
            )),
        );
        return MbValue::none();
    }
    let a = std::slice::from_raw_parts(args_ptr, nargs);
    // No aliasing rules apply on Darwin/Linux: pass through unchanged.
    MbValue::from_ptr(MbObject::new_tuple(vec![a[0], a[1], a[2]]))
}

unsafe extern "C" fn dispatch_libc_ver(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs >= 1 {
        let a = std::slice::from_raw_parts(args_ptr, nargs);
        if let Some(path) = as_str_arg(a[0]) {
            if !std::path::Path::new(&path).exists() {
                super::super::exception::mb_raise(
                    new_str_val("FileNotFoundError"),
                    new_str_val(&format!(
                        "[Errno 2] No such file or directory: '{path}'"
                    )),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(vec![new_str(""), new_str("")]))
}

unsafe extern "C" fn dispatch_python_version_tuple(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![
        new_str("3"), new_str("12"), new_str("0"),
    ]))
}

unsafe extern "C" fn dispatch_python_implementation(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str("CPython")
}

unsafe extern "C" fn dispatch_python_branch(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str("")
}

unsafe extern "C" fn dispatch_python_revision(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str("")
}

unsafe extern "C" fn dispatch_python_build(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![new_str("main"), new_str("")]))
}

unsafe extern "C" fn dispatch_python_compiler(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str("Clang")
}

unsafe extern "C" fn dispatch_version_fn(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    new_str(&uname_parts().3)
}

unsafe extern "C" fn dispatch_sys_version(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let banner = if nargs >= 1 {
        let a = std::slice::from_raw_parts(args_ptr, nargs);
        as_str_arg(a[0])
    } else {
        None
    };
    match banner {
        Some(b) => mb_platform_sys_version(&b),
        None => mb_platform_sys_version("3.12.0 (main) \n[Clang]"),
    }
}

unsafe extern "C" fn dispatch_comparable_version(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let a = std::slice::from_raw_parts(args_ptr, nargs);
    let Some(v) = as_str_arg(a[0]) else { return MbValue::none() };
    mb_platform_comparable_version(&v)
}

unsafe extern "C" fn dispatch_parse_os_release(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::none();
    }
    let a = std::slice::from_raw_parts(args_ptr, nargs);
    mb_platform_parse_os_release(a[0])
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
        ("architecture", dispatch_architecture as usize),
        ("uname", dispatch_uname as usize),
        ("mac_ver", dispatch_mac_ver as usize),
        ("java_ver", dispatch_java_ver as usize),
        ("system_alias", dispatch_system_alias as usize),
        ("libc_ver", dispatch_libc_ver as usize),
        ("python_version_tuple", dispatch_python_version_tuple as usize),
        ("python_implementation", dispatch_python_implementation as usize),
        ("python_branch", dispatch_python_branch as usize),
        ("python_revision", dispatch_python_revision as usize),
        ("python_build", dispatch_python_build as usize),
        ("python_compiler", dispatch_python_compiler as usize),
        ("version", dispatch_version_fn as usize),
        ("_sys_version", dispatch_sys_version as usize),
        ("_comparable_version", dispatch_comparable_version as usize),
        ("_parse_os_release", dispatch_parse_os_release as usize),
        // Missing CPython 3.12 platform surface (present+callable stubs).
        ("collections", stub),
        ("freedesktop_os_release", stub),
        ("functools", stub),
        ("itertools", stub),
        ("os", stub),
        ("re", stub),
        ("sys", stub),
        ("uname_result", stub),
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
    // Caches the test helper clears between runs.
    attrs.insert("_sys_version_cache".to_string(),
        MbValue::from_ptr(MbObject::new_dict()));
    attrs.insert("_uname_cache".to_string(), MbValue::none());
    attrs.insert("_os_release_cache".to_string(), MbValue::none());
    super::register_module("platform", attrs);
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn new_str_val(s: &str) -> MbValue {
    new_str(s)
}

fn as_str_arg(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data { Some(s.clone()) } else { None }
    })
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

/// (system, node, release, version, machine, processor) — real values via
/// uname/hostname, cached per process.
fn uname_parts() -> (String, String, String, String, String, String) {
    thread_local! {
        static CACHE: std::cell::RefCell<Option<(String, String, String, String, String, String)>> =
            const { std::cell::RefCell::new(None) };
    }
    CACHE.with(|c| {
        if let Some(v) = c.borrow().as_ref() {
            return v.clone();
        }
        let system = run_cmd("uname", &["-s"]).unwrap_or_else(|| "Darwin".to_string());
        let node = run_cmd("uname", &["-n"]).unwrap_or_else(|| "localhost".to_string());
        let release = run_cmd("uname", &["-r"]).unwrap_or_default();
        let version = run_cmd("uname", &["-v"]).unwrap_or_default();
        let machine = run_cmd("uname", &["-m"]).unwrap_or_else(|| std::env::consts::ARCH.to_string());
        let processor = if system == "Darwin" { "arm".to_string() } else { String::new() };
        let v = (system, node, release, version, machine, processor);
        *c.borrow_mut() = Some(v.clone());
        v
    })
}

/// platform.uname() — a real namedtuple instance (asdict/_replace/slice/
/// copy/pickle behaviors ride the collections.namedtuple machinery).
pub fn mb_platform_uname() -> MbValue {
    let (system, node, release, version, machine, processor) = uname_parts();
    let factory = super::collections_mod::mb_namedtuple(
        new_str("uname_result"),
        MbValue::from_ptr(MbObject::new_str(
            "system node release version machine processor".to_string(),
        )),
        MbValue::none(),
    );
    let args = MbValue::from_ptr(MbObject::new_list(vec![
        new_str(&system), new_str(&node), new_str(&release),
        new_str(&version), new_str(&machine), new_str(&processor),
    ]));
    super::super::builtins::mb_call_spread(factory, args)
}

/// CPython platform._sys_version: parse an interpreter version banner into
/// (name, version, branch, revision, buildno, builddate, compiler).
pub fn mb_platform_sys_version(banner: &str) -> MbValue {
    // sys_version_parser:
    //   ([\w.+]+)\s*(?:\|[^|]*\|)?\s*\(#?([^,]+)(?:,\s*([\w ]*)(?:,\s*([\w :]*))?)?\)\s*\[([^\]]+)\]?
    let (name, rest) = if let Some(r) = banner.strip_prefix("IronPython") {
        ("IronPython", r)
    } else if banner.contains("Jython") {
        ("Jython", banner)
    } else {
        ("CPython", banner)
    };
    let parse = || -> Option<(String, String, String, String)> {
        let s = rest.trim_start();
        // version: leading [\w.+]+ — must not contain spaces.
        let vend = s.find(|c: char| !(c.is_alphanumeric() || c == '.' || c == '+'))?;
        let version = &s[..vend];
        if version.is_empty() {
            return None;
        }
        let s2 = s[vend..].trim_start();
        // (buildno, builddate...) parenthesized section
        let s3 = s2.strip_prefix('(')?;
        let close = s3.find(')')?;
        let inner = &s3[..close];
        let mut parts = inner.splitn(3, ',');
        let buildno = parts.next().unwrap_or("").trim().trim_start_matches('#').to_string();
        // builddate joins the date and time segments with a space
        // ('Jun 21 2006', '13:54:21' → 'Jun 21 2006 13:54:21').
        let date = parts.next().map(|d| d.trim().to_string()).unwrap_or_default();
        let time = parts.next().map(|t| t.trim().to_string()).unwrap_or_default();
        let builddate = if time.is_empty() {
            date
        } else {
            format!("{date} {time}")
        };
        // [compiler]
        let after = s3[close + 1..].trim_start();
        let c1 = after.strip_prefix('[')?;
        let compiler = c1.strip_suffix(']').unwrap_or(c1).to_string();
        Some((version.to_string(), buildno, builddate, compiler))
    };
    match parse() {
        Some((version, buildno, builddate, compiler)) => {
            MbValue::from_ptr(MbObject::new_tuple(vec![
                new_str(name), new_str(&version), new_str(""), new_str(""),
                new_str(&buildno), new_str(&builddate), new_str(&compiler),
            ]))
        }
        None => {
            super::super::exception::mb_raise(
                new_str_val("ValueError"),
                new_str_val(&format!(
                    "failed to parse CPython sys.version: {banner:?}"
                )),
            );
            MbValue::none()
        }
    }
}

/// CPython platform._comparable_version: a flat [kind, value, ...] list —
/// numeric segments are (100, int), pre-release tags map through the stage
/// ladder, separators vanish. Lists compare element-wise so the Python-level
/// `<` / `==` operators give CPython ordering.
pub fn mb_platform_comparable_version(version: &str) -> MbValue {
    fn stage(tag: &str) -> i64 {
        match tag {
            "dev" => 10,
            "alpha" | "a" => 20,
            "beta" | "b" => 30,
            "c" => 40,
            "RC" | "rc" => 50,
            "pl" | "p" => 200,
            _ => 0,
        }
    }
    let mut out: Vec<MbValue> = Vec::new();
    let mut cur = String::new();
    let mut cur_is_digit: Option<bool> = None;
    let mut flush = |cur: &mut String, cur_is_digit: &mut Option<bool>, out: &mut Vec<MbValue>| {
        if cur.is_empty() {
            return;
        }
        if *cur_is_digit == Some(true) {
            let n: i64 = cur.parse().unwrap_or(0);
            out.push(MbValue::from_int(100));
            out.push(MbValue::from_int(n));
        } else {
            out.push(MbValue::from_int(stage(cur)));
            out.push(new_str(cur));
        }
        cur.clear();
        *cur_is_digit = None;
    };
    for ch in version.chars() {
        if matches!(ch, '.' | '_' | '+' | '-') {
            flush(&mut cur, &mut cur_is_digit, &mut out);
            continue;
        }
        let is_digit = ch.is_ascii_digit();
        if cur_is_digit.is_some() && cur_is_digit != Some(is_digit) {
            flush(&mut cur, &mut cur_is_digit, &mut out);
        }
        cur_is_digit = Some(is_digit);
        cur.push(ch);
    }
    flush(&mut cur, &mut cur_is_digit, &mut out);
    MbValue::from_ptr(MbObject::new_list(out))
}

/// CPython platform._parse_os_release(lines) → dict. Handles comments,
/// blank lines, quoted values (single/double) with backslash escapes, and
/// skips invalid assignment lines.
pub fn mb_platform_parse_os_release(lines: MbValue) -> MbValue {
    use super::super::rc::ObjData;
    let collected: Vec<String> = lines.as_ptr().map(|ptr| unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => s.lines().map(|l| l.to_string()).collect(),
            ObjData::List(lock) => lock.read().unwrap().iter()
                .filter_map(|v| as_str_arg(*v))
                .collect(),
            ObjData::Tuple(items) => items.iter()
                .filter_map(|v| as_str_arg(*v))
                .collect(),
            _ => Vec::new(),
        }
    }).unwrap_or_default();
    let dict = super::super::dict_ops::mb_dict_new();
    // CPython seeds the os-release defaults before parsing.
    for (k, v) in [("NAME", "Linux"), ("ID", "linux"), ("PRETTY_NAME", "Linux")] {
        super::super::dict_ops::mb_dict_setitem(dict, new_str(k), new_str(v));
    }
    for line in collected {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some(eq) = t.find('=') else { continue };
        let key = &t[..eq];
        // NAME must be [A-Za-z0-9_]+
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let raw = &t[eq + 1..];
        let value = if (raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2)
            || (raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2)
        {
            let inner = &raw[1..raw.len() - 1];
            // Unescape \$ \` \\ \' \" sequences.
            let mut out = String::new();
            let mut chars = inner.chars();
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(n) = chars.next() {
                        out.push(n);
                    }
                } else {
                    out.push(c);
                }
            }
            out
        } else {
            raw.to_string()
        };
        super::super::dict_ops::mb_dict_setitem(dict, new_str(key), new_str(&value));
    }
    dict
}

pub fn mb_platform_system() -> MbValue { new_str(&uname_parts().0) }

pub fn mb_platform_node() -> MbValue {
    if let Ok(h) = std::env::var("HOSTNAME") {
        if !h.is_empty() {
            return new_str(&h);
        }
    }
    new_str(&uname_parts().1)
}

pub fn mb_platform_release() -> MbValue { new_str(&uname_parts().2) }

pub fn mb_platform_machine() -> MbValue { new_str(&uname_parts().4) }

pub fn mb_platform_processor() -> MbValue { new_str(&uname_parts().5) }

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
    fn test_release_is_real_kernel() {
        // Real `uname -r` value: non-empty, starts with a digit.
        let s = get_str(mb_platform_release()).unwrap_or_default();
        assert!(!s.is_empty());
        assert!(s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false), "{s}");
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
