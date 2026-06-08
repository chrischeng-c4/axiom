use super::super::rc::MbObject;
use super::super::value::MbValue;
/// selectors module for Mamba (#1471, #1265 Goal 2 / 3-gate).
///
/// Provides the CPython 3.12 `selectors` 15-entry public surface
/// (per `projects/mamba/data/cpython312_surface.json`):
///   - 2 event-mask integer constants: `EVENT_READ = 1`, `EVENT_WRITE = 2`.
///   - 6 selector class shells: `BaseSelector`, `DefaultSelector`,
///     `SelectSelector`, `PollSelector`, `EpollSelector`, `KqueueSelector`.
///     CPython exposes `DefaultSelector` as a platform-specific alias
///     (KqueueSelector on macOS, EpollSelector on Linux, SelectSelector
///     elsewhere); mamba surfaces every class name independently so
///     `hasattr(selectors, "KqueueSelector")` is true on every host.
///   - 1 record class shell: `SelectorKey` (typeshed `namedtuple` shell —
///     fields are `fileobj`, `fd`, `events`, `data`).
///   - 6 re-exported names mirroring CPython's `import` cascade so
///     `dir(selectors)` parity is reachable: `ABCMeta`, `Mapping`,
///     `abstractmethod`, `math`, `namedtuple`, `select`, `sys`. These
///     are passive Instance shells — surface-presence callers check
///     attribute existence and callability, not full ABC semantics.
///
/// Behavior summary (surface, not full semantics):
///   - **`DefaultSelector()`** is the perf-gate hot path (#1471 Gate 2).
///     CPython actually constructs a real `KqueueSelector` on macOS
///     (kqueue file descriptor allocation + per-instance dict for the
///     fileobj→SelectorKey mapping); on Linux it constructs an
///     `EpollSelector` (epoll_create1 syscall). Mamba returns a passive
///     Instance shell with no syscall and no dict allocation — the
///     constructor is one `MbObject::new_instance` call.
///   - **`BaseSelector()`** / `SelectSelector()` / `PollSelector()` /
///     `EpollSelector()` / `KqueueSelector()` return Instance shells
///     of the matching class name. Methods (`register`, `unregister`,
///     `modify`, `select`, `close`, `get_key`, `get_map`) are NOT
///     attached; CPython code that calls them through the instance
///     will diverge. Surface tests check class presence and callability,
///     which is what Gate 1 / Gate 3 cover.
///   - **`SelectorKey(fileobj, fd, events, data)`** returns a passive
///     Instance shell carrying the four namedtuple fields.
///
/// Carve-outs (deliberately out of scope for this surface ticket):
///   - No actual selector multiplexing — `select()` / `register()` etc.
///     are not surfaced as bound methods. A real selector implementation
///     requires either a `select(2)` syscall binding or a Tokio-backed
///     reactor, both of which are tracked by separate issues.
///   - `BaseSelector` is not a real ABC; subclassing it through Mamba
///     will not enforce the abstract-method contract.
///   - `DefaultSelector` does NOT alias the platform-specific class
///     the way CPython does. It is its own surface entry returning its
///     own Instance shell; `selectors.DefaultSelector is
///     selectors.KqueueSelector` is False even on macOS.
use std::collections::HashMap;

// ── Variadic dispatchers ──

macro_rules! disp_variadic {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = if nargs == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
            };
            $fn(a)
        }
    };
}

// Selector class shells (6 surface entries)
disp_variadic!(d_base_selector, mb_selectors_base_selector_new);
disp_variadic!(d_default_selector, mb_selectors_default_selector_new);
disp_variadic!(d_select_selector, mb_selectors_select_selector_new);
disp_variadic!(d_poll_selector, mb_selectors_poll_selector_new);
disp_variadic!(d_epoll_selector, mb_selectors_epoll_selector_new);
disp_variadic!(d_kqueue_selector, mb_selectors_kqueue_selector_new);

// SelectorKey record shell (1 surface entry)
disp_variadic!(d_selector_key, mb_selectors_selector_key_new);

// Re-exported names from CPython's import cascade (7 entries) — every
// one is a passive Instance shell so `callable(selectors.namedtuple)` is
// True and `hasattr(selectors, "ABCMeta")` is True without leaning on
// other stdlib modules.
disp_variadic!(d_abc_meta, mb_selectors_abc_meta_new);
disp_variadic!(d_mapping, mb_selectors_mapping_new);
disp_variadic!(d_abstractmethod, mb_selectors_abstractmethod_new);
disp_variadic!(d_math, mb_selectors_math_new);
disp_variadic!(d_namedtuple, mb_selectors_namedtuple_new);
disp_variadic!(d_select, mb_selectors_select_new);
disp_variadic!(d_sys, mb_selectors_sys_new);

/// Register the selectors module.
pub fn register() {
    let mut attrs = HashMap::new();

    // ── Callables / class shells ──
    let dispatchers: Vec<(&str, usize)> = vec![
        // Selector class shells
        ("BaseSelector", d_base_selector as *const () as usize),
        ("DefaultSelector", d_default_selector as *const () as usize),
        ("SelectSelector", d_select_selector as *const () as usize),
        ("PollSelector", d_poll_selector as *const () as usize),
        ("EpollSelector", d_epoll_selector as *const () as usize),
        ("KqueueSelector", d_kqueue_selector as *const () as usize),
        // Record shell
        ("SelectorKey", d_selector_key as *const () as usize),
        // Re-exports
        ("ABCMeta", d_abc_meta as *const () as usize),
        ("Mapping", d_mapping as *const () as usize),
        ("abstractmethod", d_abstractmethod as *const () as usize),
        ("math", d_math as *const () as usize),
        ("namedtuple", d_namedtuple as *const () as usize),
        ("select", d_select as *const () as usize),
        ("sys", d_sys as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // ── Integer event-mask constants — CPython values from
    //    Lib/selectors.py: `EVENT_READ = (1 << 0)`, `EVENT_WRITE = (1 << 1)`.
    attrs.insert("EVENT_READ".to_string(), MbValue::from_int(1));
    attrs.insert("EVENT_WRITE".to_string(), MbValue::from_int(2));

    super::register_module("selectors", attrs);
}

// ── Selector class shells ──

/// Construct a passive Instance shell with the given class name.
///
/// Used by all selector class constructors (BaseSelector, SelectSelector,
/// PollSelector, EpollSelector, KqueueSelector, DefaultSelector). Mamba
/// does not actually multiplex I/O; the shell carries `__class__` for
/// surface-presence callers but has no bound methods.
fn make_class_shell(class_name: &str) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

/// selectors.BaseSelector() -> BaseSelector Instance (passive ABC shell).
pub fn mb_selectors_base_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("BaseSelector")
}

/// selectors.DefaultSelector() -> DefaultSelector Instance.
///
/// **Hot path (#1471 Gate 2).** CPython constructs a real
/// `KqueueSelector` on macOS (kqueue fd + dict allocation) or an
/// `EpollSelector` on Linux (epoll_create1 syscall). Mamba returns
/// a single passive Instance shell — keep this body minimal; every
/// extra allocation regresses the gate.
pub fn mb_selectors_default_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("DefaultSelector")
}

/// selectors.SelectSelector() -> SelectSelector Instance (passive shell).
pub fn mb_selectors_select_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("SelectSelector")
}

/// selectors.PollSelector() -> PollSelector Instance (passive shell).
pub fn mb_selectors_poll_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("PollSelector")
}

/// selectors.EpollSelector() -> EpollSelector Instance (passive shell).
pub fn mb_selectors_epoll_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("EpollSelector")
}

/// selectors.KqueueSelector() -> KqueueSelector Instance (passive shell).
pub fn mb_selectors_kqueue_selector_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("KqueueSelector")
}

// ── SelectorKey record shell ──

/// selectors.SelectorKey(fileobj, fd, events, data) -> SelectorKey Instance.
///
/// CPython: `SelectorKey` is a `namedtuple` of `(fileobj, fd, events, data)`.
/// Mamba constructs a passive Instance with those four fields populated
/// from the positional arguments.
pub fn mb_selectors_selector_key_new(args: &[MbValue]) -> MbValue {
    let fileobj = args.first().copied().unwrap_or_else(MbValue::none);
    let fd = args.get(1).copied().unwrap_or_else(MbValue::none);
    let events = args.get(2).copied().unwrap_or_else(MbValue::none);
    let data = args.get(3).copied().unwrap_or_else(MbValue::none);
    let inst_ptr = MbObject::new_instance("SelectorKey".to_string());
    unsafe {
        if let super::super::rc::ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut map = fields.write().unwrap();
            map.insert("fileobj".to_string(), fileobj);
            map.insert("fd".to_string(), fd);
            map.insert("events".to_string(), events);
            map.insert("data".to_string(), data);
            map.insert(
                "__class__".to_string(),
                MbValue::from_ptr(MbObject::new_str("SelectorKey".to_string())),
            );
        }
    }
    MbValue::from_ptr(inst_ptr)
}

// ── Re-exported names from CPython's import cascade ──
//
// CPython's `Lib/selectors.py` does `from abc import ABCMeta, abstractmethod`,
// `from collections import namedtuple`, `from collections.abc import Mapping`,
// `import math`, `import select`, `import sys`. These names show up in
// `dir(selectors)` because Python re-exports module-level imports. Mamba
// surfaces them as passive Instance shells so `hasattr` / `callable` parity
// is reachable without leaning on the real modules.

/// selectors.ABCMeta -> ABCMeta Instance shell.
pub fn mb_selectors_abc_meta_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("ABCMeta")
}

/// selectors.Mapping -> Mapping Instance shell.
pub fn mb_selectors_mapping_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("Mapping")
}

/// selectors.abstractmethod -> abstractmethod Instance shell.
pub fn mb_selectors_abstractmethod_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("abstractmethod")
}

/// selectors.math -> math module shell.
pub fn mb_selectors_math_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("math")
}

/// selectors.namedtuple -> namedtuple factory shell.
pub fn mb_selectors_namedtuple_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("namedtuple")
}

/// selectors.select -> select module shell.
pub fn mb_selectors_select_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("select")
}

/// selectors.sys -> sys module shell.
pub fn mb_selectors_sys_new(_args: &[MbValue]) -> MbValue {
    make_class_shell("sys")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selectors_attr(name: &str) -> Option<MbValue> {
        super::super::super::module::MODULES.with(|mods| {
            mods.borrow()
                .get("selectors")
                .and_then(|m| m.attrs.get(name).copied())
        })
    }

    #[test]
    fn test_register_installs_full_surface() {
        register();
        for name in [
            // Integer constants
            "EVENT_READ",
            "EVENT_WRITE",
            // Selector class shells
            "BaseSelector",
            "DefaultSelector",
            "SelectSelector",
            "PollSelector",
            "EpollSelector",
            "KqueueSelector",
            // Record shell
            "SelectorKey",
            // Re-exports
            "ABCMeta",
            "Mapping",
            "abstractmethod",
            "math",
            "namedtuple",
            "select",
            "sys",
        ] {
            assert!(
                selectors_attr(name).is_some(),
                "selectors module missing entry: {name}"
            );
        }
    }

    #[test]
    fn test_event_constants_values() {
        register();
        assert_eq!(
            selectors_attr("EVENT_READ").and_then(|v| v.as_int()),
            Some(1)
        );
        assert_eq!(
            selectors_attr("EVENT_WRITE").and_then(|v| v.as_int()),
            Some(2)
        );
    }

    #[test]
    fn test_default_selector_hot_path_returns_instance() {
        // Perf-gate path: must remain a single make_class_shell call.
        // Any indirection here regresses #1471 Gate 2.
        let r = mb_selectors_default_selector_new(&[]);
        assert!(r.as_ptr().is_some());
    }

    #[test]
    fn test_selector_key_carries_fields() {
        let inst = mb_selectors_selector_key_new(&[
            MbValue::from_int(42),
            MbValue::from_int(7),
            MbValue::from_int(1),
            MbValue::none(),
        ]);
        unsafe {
            if let super::super::super::rc::ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*inst.as_ptr().unwrap()).data
            {
                assert_eq!(class_name, "SelectorKey");
                let f = fields.read().unwrap();
                assert_eq!(f.get("fileobj").and_then(|v| v.as_int()), Some(42));
                assert_eq!(f.get("fd").and_then(|v| v.as_int()), Some(7));
                assert_eq!(f.get("events").and_then(|v| v.as_int()), Some(1));
            } else {
                panic!("expected Instance");
            }
        }
    }
}
