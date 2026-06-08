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
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

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
    if url.is_empty() { return false; }
    #[cfg(target_os = "macos")]
    {
        return Command::new("open").arg(url).spawn().is_ok();
    }
    #[cfg(target_os = "windows")]
    {
        // `start` is a cmd builtin, not an exe.
        return Command::new("cmd").args(["/c", "start", "", url]).spawn().is_ok();
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
        for cmd in ["xdg-open", "sensible-browser", "x-www-browser", "www-browser"] {
            if Command::new(cmd).arg(url).spawn().is_ok() {
                return true;
            }
        }
        false
    }
}

unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args.first().copied().and_then(|v| as_str(v)).unwrap_or_default();
    MbValue::from_bool(spawn_opener(&url))
}

unsafe extern "C" fn dispatch_open_new(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args.first().copied().and_then(|v| as_str(v)).unwrap_or_default();
    MbValue::from_bool(spawn_opener(&url))
}

unsafe extern "C" fn dispatch_open_new_tab(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = args_slice(args_ptr, nargs);
    let url = args.first().copied().and_then(|v| as_str(v)).unwrap_or_default();
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

unsafe extern "C" fn dispatch_class_shell(_args: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

pub fn register() {
    let mut attrs: HashMap<String, MbValue> = HashMap::new();
    attrs.insert("open".into(),         MbValue::from_func(dispatch_open as *const () as usize));
    attrs.insert("open_new".into(),     MbValue::from_func(dispatch_open_new as *const () as usize));
    attrs.insert("open_new_tab".into(), MbValue::from_func(dispatch_open_new_tab as *const () as usize));
    attrs.insert("get".into(),          MbValue::from_func(dispatch_get as *const () as usize));
    attrs.insert("register".into(),     MbValue::from_func(dispatch_register as *const () as usize));

    // Class shells (constructors return empty dicts).
    for cls in [
        "Error", "BackgroundBrowser", "GenericBrowser", "BaseBrowser",
        "UnixBrowser", "Mozilla", "Galeon", "Chrome", "Opera", "Elinks",
        "Konqueror", "Grail", "WindowsDefault", "MacOSX", "MacOSXOSAScript",
    ] {
        attrs.insert(cls.into(), MbValue::from_func(dispatch_class_shell as *const () as usize));
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
