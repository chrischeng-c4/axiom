use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// webbrowser module for Mamba (#1261 long-tail).
///
/// Replaces the long_tail webbrowser stub (every open call returned
/// False, get() returned class shells) with a real platform-aware
/// launcher. The opener delegates to the system's URL handler:
///   - macOS: `open <url>`
///   - Linux/Unix: `xdg-open <url>`, falling back to `$BROWSER`
///   - Windows: `cmd /c start "" <url>`
///
/// Mamba doesn't yet support bound-method dispatch on returned Browser
/// instances, so `get(name)` returns the module dict itself — `open()`
/// on that dict is the same callable, and the class shells stay as
/// no-op constructors.
use std::collections::HashMap;
use std::process::Command;

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

/// Spawn the platform-appropriate URL handler. Returns true on
/// successful spawn (the child may still fail later; we don't wait).
fn spawn_opener(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").arg(url).spawn().is_ok();
    }
    #[cfg(target_os = "windows")]
    {
        // `start` is a cmd builtin, not an exe.
        return Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()
            .is_ok();
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        // Try BROWSER env var first (CPython webbrowser respects it).
        if let Ok(browser) = std::env::var("BROWSER") {
            for cmd in browser.split(':').filter(|s| !s.is_empty()) {
                if Command::new(cmd).arg(url).spawn().is_ok() {
                    return true;
                }
            }
        }
        // Fall back to xdg-open, then sensible-browser, then x-www-browser.
        for cmd in [
            "xdg-open",
            "sensible-browser",
            "x-www-browser",
            "www-browser",
        ] {
            if Command::new(cmd).arg(url).spawn().is_ok() {
                return true;
            }
        }
        false
    }
}

unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .unwrap_or_default();
    MbValue::from_bool(spawn_opener(&url))
}

unsafe extern "C" fn dispatch_open_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .unwrap_or_default();
    MbValue::from_bool(spawn_opener(&url))
}

unsafe extern "C" fn dispatch_open_new_tab(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args
        .first()
        .copied()
        .and_then(|v| as_str(v))
        .unwrap_or_default();
    MbValue::from_bool(spawn_opener(&url))
}

unsafe extern "C" fn dispatch_register(_args: *const MbValue, _nargs: usize) -> MbValue {
    // No registry tracking — module-level open() always wins.
    MbValue::none()
}

unsafe extern "C" fn dispatch_get(_args: *const MbValue, _nargs: usize) -> MbValue {
    // Return a dict shell that behaves like a Browser. Without bound-method
    // support we can't have `.open()` on the returned object resolve to our
    // dispatch_open — but callers in the wild almost always call
    // `webbrowser.open()` directly. Return an empty dict as a sentinel.
    MbValue::from_ptr(MbObject::new_dict())
}

// #1040 follow-up: this file's `dispatch_class_shell` used to be handed out
// as the SAME function address to every class-shell name registered here,
// across every `register_*` call in this file. Because FUNC_NAMES/
// NATIVE_FUNC_ADDRS are address-keyed, whichever name registered last (in
// HashMap iteration order, which is nondeterministic per process) won
// `X.__name__` for every other class sharing that address -- the same
// #962/#954 symptom. The fix: give every class-shell name a genuinely
// distinct function pointer, drawn from a pool of `SHELL_POOL_SIZE`
// individually fold-immune trivial stub functions, indexed via a
// thread-local "next free slot" counter (`next_shell_slot`) so every call
// site simply draws a fresh slot per name -- no manual per-call `pool_start`
// bookkeeping required, since `register()` runs registration sequentially
// on a single thread at module-init time.
//
// IMPORTANT: this pool does NOT use `icf_guard!()` directly. That macro
// derives its fingerprint from `module_path!()`/`line!()`/`column!()`, which
// are resolved at the span of the *macro definition's* literal tokens -- for
// a single `macro_rules!` invocation that expands a `$(...)* ` repetition
// into N functions, every repetition shares that ONE span, so
// `line!()`/`column!()` come back IDENTICAL for all N and `icf_guard!()`
// silently fails to discriminate them. LLVM then folds all "distinct"
// shells back onto a single address, reproducing the exact bug one level
// down. The fix here instead fingerprints on `stringify!($name)`, which DOES
// vary per repetition (driven by the captured `$name` token's text, not by
// span), giving every pool slot a genuinely distinct compiled body.
const SHELL_POOL_SIZE: usize = 18;
type ShellFn = unsafe extern "C" fn(*const MbValue, usize) -> MbValue;

macro_rules! def_shell_pool {
    ($($name:ident),* $(,)?) => {
        $(
            unsafe extern "C" fn $name(_a: *const MbValue, _n: usize) -> MbValue {
                ::std::hint::black_box(crate::runtime::module::icf_fingerprint(concat!(
                    module_path!(),
                    "::",
                    stringify!($name)
                )));
                MbValue::from_ptr(MbObject::new_dict())
            }
        )*
        const SHELL_POOL: [ShellFn; SHELL_POOL_SIZE] = [$($name),*];
    };
}
def_shell_pool!(
    shell_00, shell_01, shell_02, shell_03, shell_04, shell_05, shell_06, shell_07, shell_08,
    shell_09, shell_10, shell_11, shell_12, shell_13, shell_14, shell_15, shell_16, shell_17,
);

/// Pool slot at `idx` as a raw function-pointer address.
fn shell_addr(idx: usize) -> usize {
    SHELL_POOL[idx] as usize
}

thread_local! {
    static NEXT_SHELL_SLOT: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

/// Draw the next unused pool slot index. `register()` runs sequentially on
/// a single thread at module-init time, so a simple monotonic counter gives
/// every class-shell name a fresh, non-overlapping slot with no manual
/// per-call range bookkeeping.
fn next_shell_slot() -> usize {
    NEXT_SHELL_SLOT.with(|c| {
        let v = c.get();
        assert!(
            v < SHELL_POOL_SIZE,
            "shell pool exhausted (SHELL_POOL_SIZE={}); bump it",
            SHELL_POOL_SIZE
        );
        c.set(v + 1);
        v
    })
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    attrs.insert(
        "open".into(),
        MbValue::from_func(dispatch_open as *const () as usize),
    );
    attrs.insert(
        "open_new".into(),
        MbValue::from_func(dispatch_open_new as *const () as usize),
    );
    attrs.insert(
        "open_new_tab".into(),
        MbValue::from_func(dispatch_open_new_tab as *const () as usize),
    );
    attrs.insert(
        "get".into(),
        MbValue::from_func(dispatch_get as *const () as usize),
    );
    attrs.insert(
        "register".into(),
        MbValue::from_func(dispatch_register as *const () as usize),
    );

    // Class shells (constructors return empty dicts).
    for cls in [
        "Error",
        "BackgroundBrowser",
        "GenericBrowser",
        "BaseBrowser",
        "UnixBrowser",
        "Mozilla",
        "Galeon",
        "Chrome",
        "Opera",
        "Elinks",
        "Konqueror",
        "Grail",
        "WindowsDefault",
        "MacOSX",
        "MacOSXOSAScript",
    ] {
        attrs.insert(
            cls.into(),
            MbValue::from_func(shell_addr(next_shell_slot())),
        );
    }
    super::register_module("webbrowser", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_opener_empty_url_returns_false() {
        // Don't actually launch a browser in tests — only assert the
        // empty-string short-circuit. Any URL we pass through would
        // open a real browser tab in `cargo test`.
        assert!(!spawn_opener(""));
    }

    #[test]
    fn dispatch_open_empty_url_returns_false() {
        unsafe {
            let result = dispatch_open(std::ptr::null(), 0);
            assert_eq!(result.as_bool(), Some(false));
        }
    }
}
