//! argparse module for Mamba (#399).
//!
//! A self-contained, native implementation of the most-used surface of
//! CPython 3.12's `argparse`:
//!
//!   * `ArgumentParser` — `add_argument`, `parse_args`, `parse_known_args`,
//!     `set_defaults`, `get_default`, `add_subparsers`, `error`, `exit`.
//!   * `Namespace` — kwargs constructor, attribute access, `vars()`,
//!     order-independent `__eq__`/`__ne__` (with `NotImplemented` fallback),
//!     and `__contains__` membership.
//!   * `Action` — the attribute container returned by `add_argument`
//!     (`dest`, `nargs`, `const`, `default`, `type`, `choices`, `help`,
//!     `metavar`, `option_strings`).
//!   * Constants: `SUPPRESS`, `OPTIONAL`, `ZERO_OR_MORE`, `ONE_OR_MORE`,
//!     `REMAINDER`, `PARSER`, plus class stubs that the surface tests probe
//!     (`FileType`, `ArgumentError`, `ArgumentTypeError`, the help
//!     formatters, `BooleanOptionalAction`).
//!
//! All object construction routes through native dispatchers exposed as
//! module attributes; instance methods are registered as runtime classes via
//! `mb_class_register`, so `parser.add_argument(...)` / `ns.x` dispatch through
//! the normal MRO path with no class.rs changes.

use super::super::dict_ops::DictKey;
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

// ── Class names (registered via mb_class_register) ──
const PARSER_CLASS: &str = "ArgumentParser";
const NAMESPACE_CLASS: &str = "Namespace";
const ACTION_CLASS: &str = "Action";
const SUBPARSERS_CLASS: &str = "_SubParsersAction";

// ── Small field helpers ──

fn set_field(inst: MbValue, key: &str, val: MbValue) {
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

fn get_field(inst: MbValue, key: &str) -> Option<MbValue> {
    inst.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Read a key out of an `ObjData::Dict` value (the trailing kwargs dict).
fn dict_get(dict: MbValue, key: &str) -> Option<MbValue> {
    dict.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn is_dict(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) })
        .unwrap_or(false)
}

/// Turn a list/tuple value into a Vec<MbValue>; anything else → empty.
fn seq_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                _ => {}
            }
        }
    }
    Vec::new()
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn new_dict() -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

fn raise(exc: &str, msg: &str) {
    super::super::exception::mb_raise(new_str(exc), new_str(msg));
}

/// Raise `SystemExit(code)` as an instance so a caught `except SystemExit as e`
/// sees `e.code == code` (CPython argparse exits 2 on any parse error). A bare
/// string SystemExit leaves `.code` None.
fn raise_exit(code: i64) {
    let inst = MbValue::from_ptr(MbObject::new_instance("SystemExit".to_string()));
    set_field(inst, "code", MbValue::from_int(code));
    set_field(inst, "args",
        MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(code)])));
    super::super::class::mb_raise_instance(inst);
}

// ── Argument-spec parsing (shared by add_argument) ──

/// Internal description of one declared argument, stored as an Action instance.
struct ArgSpec {
    option_strings: Vec<String>,
    dest: String,
    is_positional: bool,
    action: String, // "store", "store_true", "store_false", "append", or "" for custom
    nargs: MbValue, // None, "?", "*", "+", or int
    const_v: MbValue,
    default_v: MbValue,
    type_v: MbValue,
    choices: MbValue,
    help_v: MbValue,
    metavar: MbValue,
    required: bool,
    custom_action: MbValue, // a user Action class/value, else None
}

/// Derive the dest from option strings / positional name (CPython rules).
fn derive_dest(option_strings: &[String], explicit: Option<String>) -> String {
    if let Some(d) = explicit {
        return d;
    }
    // Prefer the first long option (--foo → foo); else first option (-x → x).
    let mut long = None;
    let mut short = None;
    for opt in option_strings {
        if opt.starts_with("--") {
            if long.is_none() {
                long = Some(opt.trim_start_matches('-').to_string());
            }
        } else if opt.starts_with('-') && short.is_none() {
            short = Some(opt.trim_start_matches('-').to_string());
        }
    }
    let raw = long.or(short).unwrap_or_default();
    raw.replace('-', "_")
}

fn truthy(val: MbValue) -> bool {
    if let Some(b) = val.as_bool() {
        return b;
    }
    if let Some(i) = val.as_int() {
        return i != 0;
    }
    !val.is_none()
}

/// Build an Action instance from the resolved spec.
fn make_action(spec: &ArgSpec) -> MbValue {
    let act = MbValue::from_ptr(MbObject::new_instance(ACTION_CLASS.to_string()));
    let opt_vals: Vec<MbValue> = spec.option_strings.iter().map(|s| new_str(s)).collect();
    set_field(act, "option_strings", new_list(opt_vals));
    set_field(act, "dest", new_str(&spec.dest));
    set_field(act, "nargs", spec.nargs);
    set_field(act, "const", spec.const_v);
    set_field(act, "default", spec.default_v);
    set_field(act, "type", spec.type_v);
    set_field(act, "choices", spec.choices);
    set_field(act, "help", spec.help_v);
    set_field(act, "metavar", spec.metavar);
    set_field(act, "required", MbValue::from_bool(spec.required));
    set_field(act, "_action", new_str(&spec.action));
    set_field(
        act,
        "_is_positional",
        MbValue::from_bool(spec.is_positional),
    );
    set_field(act, "_custom_action", spec.custom_action);
    act
}

// ── Module dispatchers (object constructors) ──

/// ArgumentParser(description=..., prog=..., exit_on_error=..., ...) -> parser.
unsafe extern "C" fn dispatch_argument_parser(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let parser = MbValue::from_ptr(MbObject::new_instance(PARSER_CLASS.to_string()));
    set_field(parser, "_actions", new_list(vec![]));
    set_field(parser, "_defaults", MbValue::from_ptr(MbObject::new_dict()));
    set_field(parser, "_subparsers", MbValue::none());
    set_field(parser, "_exit_on_error", MbValue::from_bool(true));

    // Trailing kwargs dict (description=, prog=, exit_on_error=, ...).
    if let Some(kw) = a.last().copied() {
        if is_dict(kw) {
            if let Some(d) = dict_get(kw, "description") {
                set_field(parser, "description", d);
            }
            if let Some(p) = dict_get(kw, "prog") {
                set_field(parser, "prog", p);
            }
            if let Some(e) = dict_get(kw, "exit_on_error") {
                set_field(parser, "_exit_on_error", MbValue::from_bool(truthy(e)));
            }
            // conflict_handler must be a known strategy (CPython: 'error' or
            // 'resolve'); anything else raises ValueError at construction.
            if let Some(ch) = dict_get(kw, "conflict_handler").and_then(extract_str) {
                if ch != "error" && ch != "resolve" {
                    raise(
                        "ValueError",
                        &format!("invalid conflict_resolution value: '{ch}'"),
                    );
                    return MbValue::none();
                }
            }
        }
    }
    parser
}

/// Namespace(**kwargs) -> Namespace instance with those fields.
unsafe extern "C" fn dispatch_namespace(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let ns = MbValue::from_ptr(MbObject::new_instance(NAMESPACE_CLASS.to_string()));
    if let Some(kw) = a.last().copied() {
        if let Some(ptr) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    let guard = lock.read().unwrap();
                    for (k, v) in guard.iter() {
                        if let DictKey::Str(name) = k {
                            set_field(ns, name, *v);
                        }
                    }
                }
            }
        }
    }
    ns
}

/// Action(...) -> base Action instance (rarely constructed directly; tests
/// subclass it).  Stash any constructor args generically.
unsafe extern "C" fn dispatch_action(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance(ACTION_CLASS.to_string()))
}

/// FileType(...) -> stub instance (surface-only: callable + present).
unsafe extern "C" fn dispatch_filetype(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_instance("FileType".to_string()))
}

/// Generic no-field formatter / sentinel-class constructor.
macro_rules! dispatch_stub_class {
    ($name:ident, $cls:expr) => {
        unsafe extern "C" fn $name(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            MbValue::from_ptr(MbObject::new_instance($cls.to_string()))
        }
    };
}
dispatch_stub_class!(dispatch_help_formatter, "HelpFormatter");
dispatch_stub_class!(
    dispatch_raw_description_formatter,
    "RawDescriptionHelpFormatter"
);
dispatch_stub_class!(dispatch_raw_text_formatter, "RawTextHelpFormatter");
dispatch_stub_class!(dispatch_defaults_formatter, "ArgumentDefaultsHelpFormatter");
dispatch_stub_class!(dispatch_metavar_type_formatter, "MetavarTypeHelpFormatter");
dispatch_stub_class!(dispatch_boolean_optional_action, "BooleanOptionalAction");

/// argparse.ngettext(singular, plural, n) -> str. argparse re-exports
/// gettext.ngettext; the surface only needs it present and callable. Return
/// the singular form when n == 1, else the plural (the standard fallback when
/// no translation catalog is installed).
unsafe extern "C" fn dispatch_ngettext(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Drop a trailing kwargs dict if present.
    let pos: Vec<MbValue> = a.iter().copied().filter(|v| !is_dict(*v)).collect();
    let singular = pos.first().copied().unwrap_or_else(MbValue::none);
    let plural = pos.get(1).copied().unwrap_or(singular);
    let n = pos.get(2).copied().and_then(|v| v.as_int()).unwrap_or(1);
    if n == 1 {
        singular
    } else {
        plural
    }
}

// ── ArgumentParser methods (registered class; receive (self, args_list)) ──

/// parser.add_argument(*names, **kwargs) -> Action.
unsafe extern "C" fn method_add_argument(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    // Last element may be the kwargs dict (always present when any kwarg given).
    let (positional, kwargs): (&[MbValue], MbValue) = if let Some(last) = items.last() {
        if is_dict(*last) {
            (&items[..items.len() - 1], *last)
        } else {
            (&items[..], MbValue::none())
        }
    } else {
        (&items[..], MbValue::none())
    };

    let names: Vec<String> = positional.iter().filter_map(|v| extract_str(*v)).collect();
    let is_positional = names.first().map(|n| !n.starts_with('-')).unwrap_or(true);

    // Reject a duplicated option string (CPython raises argparse.ArgumentError).
    if !is_positional {
        for existing in parser_actions(self_v) {
            let opts = get_field(existing, "option_strings")
                .map(seq_items)
                .unwrap_or_default();
            for o in opts {
                if let Some(s) = extract_str(o) {
                    if names.contains(&s) {
                        raise(
                            "ArgumentError",
                            &format!("argument {s}: conflicting option string: {s}"),
                        );
                        return MbValue::none();
                    }
                }
            }
        }
    }

    // Resolve dest.
    let explicit_dest = dict_get(kwargs, "dest").and_then(extract_str);
    let dest = if is_positional {
        explicit_dest
            .unwrap_or_else(|| names.first().cloned().unwrap_or_default().replace('-', "_"))
    } else {
        derive_dest(&names, explicit_dest)
    };

    // Resolve action. A user Action subclass reaches native code as its
    // class-name string (e.g. "CollectAction"); a registered class name that
    // is not one of the builtin action keywords is treated as a custom action.
    let action_kw = dict_get(kwargs, "action");
    let mut custom_action = MbValue::none();
    let action = match action_kw.and_then(extract_str).as_deref() {
        Some("store_true") => "store_true".to_string(),
        Some("store_false") => "store_false".to_string(),
        Some("store") | None => "store".to_string(),
        Some("append") => "append".to_string(),
        Some(other) if super::super::class::class_is_registered(other) => {
            custom_action = action_kw.unwrap();
            "custom".to_string()
        }
        Some(other) => other.to_string(),
    };

    // Default value semantics per action.
    let mut default_v = dict_get(kwargs, "default").unwrap_or_else(MbValue::none);
    match action.as_str() {
        "store_true" => {
            if dict_get(kwargs, "default").is_none() {
                default_v = MbValue::from_bool(false);
            }
        }
        "store_false" => {
            if dict_get(kwargs, "default").is_none() {
                default_v = MbValue::from_bool(true);
            }
        }
        _ => {}
    }

    let spec = ArgSpec {
        option_strings: if is_positional { vec![] } else { names.clone() },
        dest,
        is_positional,
        action,
        nargs: dict_get(kwargs, "nargs").unwrap_or_else(MbValue::none),
        const_v: dict_get(kwargs, "const").unwrap_or_else(MbValue::none),
        default_v,
        type_v: dict_get(kwargs, "type").unwrap_or_else(MbValue::none),
        choices: dict_get(kwargs, "choices").unwrap_or_else(MbValue::none),
        help_v: dict_get(kwargs, "help").unwrap_or_else(MbValue::none),
        metavar: dict_get(kwargs, "metavar").unwrap_or_else(MbValue::none),
        required: dict_get(kwargs, "required").map(truthy).unwrap_or(false),
        custom_action,
    };

    // A custom Action class is instantiated at add_argument time with the
    // resolved keywords (option_strings, dest, **kwargs) — matching CPython.
    if !custom_action.is_none() {
        let opt_list = new_list(spec.option_strings.iter().map(|s| new_str(s)).collect());
        // Build kwargs dict mirroring the resolved spec for the Action __init__.
        let init_kwargs = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*init_kwargs).data {
                let mut g = lock.write().unwrap();
                let mut put = |k: &str, v: MbValue| {
                    super::super::rc::retain_if_ptr(v);
                    g.insert(DictKey::Str(k.to_string()), v);
                };
                put("const", spec.const_v);
                put("default", spec.default_v);
                put("nargs", spec.nargs);
                put("type", spec.type_v);
                put("choices", spec.choices);
                put("help", spec.help_v);
                put("metavar", spec.metavar);
                put("required", MbValue::from_bool(spec.required));
            }
        }
        let ctor_args = new_list(vec![
            opt_list,
            new_str(&spec.dest),
            MbValue::from_ptr(init_kwargs),
        ]);
        let built = super::super::class::mb_instance_new_with_init(custom_action, ctor_args);
        // The user Action __init__ (if any) raising propagates as a pending
        // exception — return immediately so add_argument surfaces it.
        if super::super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
        // argparse's base Action.__init__ records dest/option_strings/etc on the
        // instance; our base Action has none, so seed the bookkeeping fields the
        // parse loop and __call__ rely on (without clobbering anything the user
        // __init__ already set).
        if get_field(built, "dest")
            .map(|v| v.is_none())
            .unwrap_or(true)
        {
            set_field(built, "dest", new_str(&spec.dest));
        }
        if get_field(built, "option_strings").is_none() {
            let opt_vals: Vec<MbValue> = spec.option_strings.iter().map(|s| new_str(s)).collect();
            set_field(built, "option_strings", new_list(opt_vals));
        }
        set_field(built, "_action", new_str("custom"));
        set_field(built, "_custom_action", built);
        set_field(
            built,
            "_is_positional",
            MbValue::from_bool(spec.is_positional),
        );
        if get_field(built, "default").is_none() {
            set_field(built, "default", spec.default_v);
        }
        if get_field(built, "nargs").is_none() {
            set_field(built, "nargs", spec.nargs);
        }
        // Register the constructed Action under the parser.
        if let Some(p_actions) = get_field(self_v, "_actions") {
            if let Some(ptr) = p_actions.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    super::super::rc::retain_if_ptr(built);
                    lock.write().unwrap().push(built);
                }
            }
        }
        return built;
    }

    let act = make_action(&spec);
    if let Some(p_actions) = get_field(self_v, "_actions") {
        if let Some(ptr) = p_actions.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                super::super::rc::retain_if_ptr(act);
                lock.write().unwrap().push(act);
            }
        }
    }
    act
}

/// parser.set_defaults(**kwargs) -> None.
unsafe extern "C" fn method_set_defaults(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    if let Some(kw) = items.last().copied() {
        if let Some(defaults) = get_field(self_v, "_defaults") {
            if let Some(kp) = kw.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref klock) = (*kp).data {
                        let kg = klock.read().unwrap();
                        if let Some(dp) = defaults.as_ptr() {
                            if let ObjData::Dict(ref dlock) = (*dp).data {
                                let mut dg = dlock.write().unwrap();
                                for (k, v) in kg.iter() {
                                    if let DictKey::Str(name) = k {
                                        super::super::rc::retain_if_ptr(*v);
                                        dg.insert(DictKey::Str(name.clone()), *v);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::none()
}

/// parser.get_default(dest) -> the registered default for dest.
unsafe extern "C" fn method_get_default(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let dest = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    // set_defaults wins over add_argument default.
    if let Some(defaults) = get_field(self_v, "_defaults") {
        if let Some(v) = dict_get(defaults, &dest) {
            return v;
        }
    }
    for act in parser_actions(self_v) {
        if get_field(act, "dest").and_then(extract_str).as_deref() == Some(dest.as_str()) {
            return get_field(act, "default").unwrap_or_else(MbValue::none);
        }
    }
    MbValue::none()
}

/// parser.error(message) -> raises SystemExit (status 2).
unsafe extern "C" fn method_error(self_v: MbValue, args: MbValue) -> MbValue {
    let _ = self_v;
    let items = seq_items(args);
    let _msg = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    raise_exit(2);
    MbValue::none()
}

/// parser.exit(status=0, message=None) -> raises SystemExit(status).
unsafe extern "C" fn method_exit(self_v: MbValue, args: MbValue) -> MbValue {
    let _ = self_v;
    let items = seq_items(args);
    // Drop a trailing kwargs dict if present.
    let pos: Vec<MbValue> = items.iter().copied().filter(|v| !is_dict(*v)).collect();
    let status = pos
        .first()
        .copied()
        .or_else(|| {
            dict_get(
                items.last().copied().unwrap_or_else(MbValue::none),
                "status",
            )
        })
        .and_then(|v| v.as_int())
        .unwrap_or(0);
    raise_exit(status);
    MbValue::none()
}

/// parser.add_subparsers(**kwargs) -> _SubParsersAction.
unsafe extern "C" fn method_add_subparsers(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let subs = MbValue::from_ptr(MbObject::new_instance(SUBPARSERS_CLASS.to_string()));
    set_field(subs, "_parsers", new_dict());
    set_field(subs, "_order", new_list(vec![]));
    if let Some(kw) = items.last().copied() {
        if is_dict(kw) {
            if let Some(dest) = dict_get(kw, "dest") {
                set_field(subs, "dest", dest);
            }
            if let Some(required) = dict_get(kw, "required") {
                set_field(subs, "required", required);
            }
        }
    }
    set_field(self_v, "_subparsers", subs);
    subs
}

/// parser.parse_args(args=None, namespace=None) -> Namespace.
unsafe extern "C" fn method_parse_args(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let argv = first_argv(&items);
    let (ns, extras) = run_parser(self_v, &argv, false);
    if !extras.is_empty() {
        let msg = format!("unrecognized arguments: {}", extras.join(" "));
        let _ = &msg; raise_exit(2);
        return MbValue::none();
    }
    ns
}

/// parser.parse_known_args(args=None, namespace=None) -> (Namespace, extras).
unsafe extern "C" fn method_parse_known_args(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let argv = first_argv(&items);
    let (ns, extras) = run_parser(self_v, &argv, true);
    let extras_list = new_list(extras.iter().map(|s| new_str(s)).collect());
    MbValue::from_ptr(MbObject::new_tuple(vec![ns, extras_list]))
}

// ── Namespace methods ──

/// Namespace.__eq__(self, other) -> bool | NotImplemented.
unsafe extern "C" fn ns_eq(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let other = items.first().copied().unwrap_or_else(MbValue::none);
    match namespaces_equal(self_v, other) {
        Some(b) => MbValue::from_bool(b),
        None => MbValue::not_implemented(),
    }
}

/// Namespace.__ne__(self, other) -> bool | NotImplemented.
unsafe extern "C" fn ns_ne(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let other = items.first().copied().unwrap_or_else(MbValue::none);
    match namespaces_equal(self_v, other) {
        Some(b) => MbValue::from_bool(!b),
        None => MbValue::not_implemented(),
    }
}

/// Namespace.__contains__(self, key) -> bool.
unsafe extern "C" fn ns_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let key = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    if key.is_empty() {
        return MbValue::from_bool(false);
    }
    MbValue::from_bool(get_field(self_v, &key).is_some())
}

/// Namespace.__getattr__(self, name) -> raise AttributeError for a missing
/// attribute (CPython's Namespace is a plain object — reading an unset
/// attribute raises). The runtime only consults `__getattr__` after the
/// instance `__dict__` misses, so declared dests (always seeded as fields)
/// never reach here; only genuinely-absent names do. 3-arg `getattr` and
/// `hasattr` swallow this AttributeError in the runtime (mb_getattr_default /
/// mb_hasattr clear the pending exception), so they keep returning the
/// default / False as CPython does.
unsafe extern "C" fn ns_getattr(_self_v: MbValue, args: MbValue) -> MbValue {
    // Invoked either as a registered dunder (self, name) — `args` is the raw
    // name string — or via the variadic protocol with a packed args list.
    let name = extract_str(args)
        .or_else(|| seq_items(args).first().copied().and_then(extract_str))
        .unwrap_or_default();
    raise(
        "AttributeError",
        &format!("'Namespace' object has no attribute '{name}'"),
    );
    MbValue::none()
}

// ── Action & ArgumentError str() ──

/// ArgumentError.__str__(self) -> message (bare when no bound argument).
unsafe extern "C" fn argerr_str(self_v: MbValue, _args: MbValue) -> MbValue {
    let msg = get_field(self_v, "message")
        .and_then(extract_str)
        .unwrap_or_default();
    let arg = get_field(self_v, "argument_name").unwrap_or_else(MbValue::none);
    if arg.is_none() {
        new_str(&msg)
    } else {
        let argname = extract_str(arg).unwrap_or_default();
        new_str(&format!("argument {argname}: {msg}"))
    }
}

/// ArgumentTypeError.__str__(self) -> message.
unsafe extern "C" fn argtypeerr_str(self_v: MbValue, _args: MbValue) -> MbValue {
    let msg = get_field(self_v, "message")
        .and_then(extract_str)
        .unwrap_or_default();
    new_str(&msg)
}

/// ArgumentError.__init__(self, [argument, message]) -> seed the fields
/// `argerr_str` reads. `argument` is an Action (or None); CPython stringifies
/// to a bare message when it is None, else `argument NAME: message`.
unsafe extern "C" fn argerr_init(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let argument = items.first().copied().unwrap_or_else(MbValue::none);
    let message = items.get(1).copied().unwrap_or_else(MbValue::none);
    set_field(self_v, "message", message);
    // Bind `argument_name` only when a string-ish argument was supplied; a None
    // (or non-string Action stub) leaves it None so `str()` is the bare message.
    if !argument.is_none() {
        if let Some(name) = extract_str(argument) {
            set_field(self_v, "argument_name", new_str(&name));
        }
    }
    MbValue::none()
}

/// ArgumentTypeError.__init__(self, [message]) -> store the message so
/// `argtypeerr_str` (and `str(err)`) yields it. Mirrors the former
/// `dispatch_argument_type_error` constructor.
unsafe extern "C" fn argtypeerr_init(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let message = items.first().copied().unwrap_or_else(MbValue::none);
    set_field(self_v, "message", message);
    MbValue::none()
}

// ── Helpers reused by methods ──

fn parser_actions(parser: MbValue) -> Vec<MbValue> {
    get_field(parser, "_actions")
        .map(seq_items)
        .unwrap_or_default()
}

/// Resolve the argv list: explicit first positional list/tuple, else process argv.
fn first_argv(items: &[MbValue]) -> Vec<String> {
    if let Some(first) = items.iter().copied().find(|v| !is_dict(*v) && !v.is_none()) {
        let seq = seq_items(first);
        if !seq.is_empty() || is_seq(first) {
            return seq.iter().filter_map(|v| value_to_arg_string(*v)).collect();
        }
    }
    std::env::args().skip(1).collect()
}

fn is_seq(val: MbValue) -> bool {
    val.as_ptr()
        .map(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_) | ObjData::Tuple(_)) })
        .unwrap_or(false)
}

fn value_to_arg_string(v: MbValue) -> Option<String> {
    if let Some(s) = extract_str(v) {
        return Some(s);
    }
    if let Some(i) = v.as_int() {
        return Some(i.to_string());
    }
    None
}

/// Order-independent Namespace equality. Returns None (NotImplemented) when
/// `other` is not a Namespace.
fn namespaces_equal(a: MbValue, b: MbValue) -> Option<bool> {
    let is_ns = b.as_ptr().map(|ptr| unsafe {
        matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == NAMESPACE_CLASS)
    }).unwrap_or(false);
    if !is_ns {
        return None;
    }
    let fa = instance_fields(a);
    let fb = instance_fields(b);
    if fa.len() != fb.len() {
        return Some(false);
    }
    for (k, va) in &fa {
        match fb.get(k) {
            Some(vb) => {
                if super::super::builtins::mb_eq(*va, *vb).as_bool() != Some(true) {
                    return Some(false);
                }
            }
            None => return Some(false),
        }
    }
    Some(true)
}

fn instance_fields(inst: MbValue) -> HashMap<String, MbValue> {
    let mut out = HashMap::new();
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                for (k, v) in fields.read().unwrap().iter() {
                    out.insert(k.clone(), *v);
                }
            }
        }
    }
    out
}

/// Apply a `type=` converter to a raw string argument.
///
/// On a conversion failure (the converter raised, e.g. `int("a")` →
/// ValueError or a custom `ArgumentTypeError`), CPython turns it into a parse
/// error: `SystemExit(2)` normally, or a catchable `ArgumentError` when the
/// parser was built with `exit_on_error=False`. The pending exception from the
/// failed converter is consumed here and replaced with the argparse one.
fn apply_type(type_v: MbValue, raw: &str, parser: MbValue, argname: &str) -> MbValue {
    if type_v.is_none() {
        return new_str(raw);
    }
    let arg = new_list(vec![new_str(raw)]);
    let result = super::super::builtins::mb_call_spread(type_v, arg);
    if let Some(exc_type) = super::super::exception::current_exception_type() {
        // Converter failed — translate to the argparse failure mode.
        super::super::exception::mb_clear_exception();
        let label = if argname.is_empty() {
            String::new()
        } else {
            format!("argument {argname}: ")
        };
        let msg = format!("{label}invalid {raw}: '{raw}'");
        let _ = exc_type;
        if exit_on_error(parser) {
            let _ = &msg; raise_exit(2);
        } else {
            raise("ArgumentError", &msg);
        }
        return MbValue::none();
    }
    result
}

fn exit_on_error(parser: MbValue) -> bool {
    get_field(parser, "_exit_on_error")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

/// True if `value` is among the declared `choices` (None choices ⇒ always ok).
fn in_choices(choices: MbValue, value: MbValue) -> bool {
    if choices.is_none() {
        return true;
    }
    let opts = seq_items(choices);
    if opts.is_empty() {
        return true;
    }
    opts.iter()
        .any(|c| super::super::builtins::mb_eq(*c, value).as_bool() == Some(true))
}

/// The core parsing engine. Returns (namespace, leftover_extras).
fn run_parser(parser: MbValue, argv: &[String], _known: bool) -> (MbValue, Vec<String>) {
    let actions = parser_actions(parser);
    let ns = MbValue::from_ptr(MbObject::new_instance(NAMESPACE_CLASS.to_string()));

    // Seed defaults (add_argument defaults first, set_defaults overrides last).
    for act in &actions {
        let dest = get_field(*act, "dest")
            .and_then(extract_str)
            .unwrap_or_default();
        if dest.is_empty() {
            continue;
        }
        let action = get_field(*act, "_action")
            .and_then(extract_str)
            .unwrap_or_default();
        let default = get_field(*act, "default").unwrap_or_else(MbValue::none);
        let nargs = get_field(*act, "nargs").unwrap_or_else(MbValue::none);
        let seed = if action == "append" && default.is_none() {
            MbValue::none()
        } else if matches!(nargs_kind(nargs).as_str(), "*")
            && default.is_none()
            && is_positional(*act)
        {
            new_list(vec![])
        } else {
            default
        };
        set_field(ns, &dest, seed);
    }
    // set_defaults overrides (and injects extra dests).
    if let Some(defaults) = get_field(parser, "_defaults") {
        if let Some(ptr) = defaults.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    for (k, v) in lock.read().unwrap().iter() {
                        if let DictKey::Str(name) = k {
                            set_field(ns, name, *v);
                        }
                    }
                }
            }
        }
    }

    // Split optionals vs positionals.
    let optionals: Vec<MbValue> = actions
        .iter()
        .copied()
        .filter(|a| !is_positional(*a))
        .collect();
    let positionals: Vec<MbValue> = actions
        .iter()
        .copied()
        .filter(|a| is_positional(*a))
        .collect();

    let mut extras: Vec<String> = Vec::new();
    let mut positional_values: Vec<String> = Vec::new();
    let mut seen_dests: Vec<String> = Vec::new();

    let mut i = 0;
    while i < argv.len() {
        let tok = &argv[i];
        if tok == "--" {
            // Everything after is positional.
            for t in &argv[i + 1..] {
                positional_values.push(t.clone());
            }
            break;
        }
        if tok.starts_with('-') && tok != "-" && !looks_like_negative_number(tok) {
            // Optional. Support --opt=value.
            let (name, inline_val) = match tok.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (tok.clone(), None),
            };
            if let Some(act) = find_optional(&optionals, &name) {
                let dest = get_field(act, "dest")
                    .and_then(extract_str)
                    .unwrap_or_default();
                let action = get_field(act, "_action")
                    .and_then(extract_str)
                    .unwrap_or_default();
                let type_v = get_field(act, "type").unwrap_or_else(MbValue::none);
                let choices = get_field(act, "choices").unwrap_or_else(MbValue::none);
                let nargs = get_field(act, "nargs").unwrap_or_else(MbValue::none);
                let const_v = get_field(act, "const").unwrap_or_else(MbValue::none);
                let custom = get_field(act, "_custom_action").unwrap_or_else(MbValue::none);
                seen_dests.push(dest.clone());

                match action.as_str() {
                    "store_true" => {
                        set_field(ns, &dest, MbValue::from_bool(true));
                        i += 1;
                    }
                    "store_false" => {
                        set_field(ns, &dest, MbValue::from_bool(false));
                        i += 1;
                    }
                    "append" => {
                        let (val, consumed) =
                            take_value(&argv, i, inline_val, type_v, choices, parser);
                        let cur = get_field(ns, &dest).unwrap_or_else(MbValue::none);
                        let mut lst = if cur.is_none() {
                            Vec::new()
                        } else {
                            seq_items(cur)
                        };
                        lst.push(val);
                        set_field(ns, &dest, new_list(lst));
                        i += consumed;
                    }
                    _ => {
                        if !custom.is_none() {
                            // Custom action: __call__(parser, namespace, values, option_string).
                            let (val, consumed) = take_value(
                                &argv,
                                i,
                                inline_val,
                                MbValue::none(),
                                MbValue::none(),
                                parser,
                            );
                            invoke_custom_action(custom, parser, ns, val, &name);
                            i += consumed;
                        } else if nargs_kind(nargs) == "?" {
                            // Optional value: bare flag → const; else next value.
                            if let Some(v) = inline_val {
                                let coerced = apply_type(type_v, &v, parser, &name);
                                set_field(ns, &dest, coerced);
                                i += 1;
                            } else if i + 1 < argv.len() && !argv[i + 1].starts_with('-') {
                                let coerced = apply_type(type_v, &argv[i + 1], parser, &name);
                                set_field(ns, &dest, coerced);
                                i += 2;
                            } else {
                                set_field(ns, &dest, const_v);
                                i += 1;
                            }
                        } else {
                            let (val, consumed) =
                                take_value(&argv, i, inline_val, type_v, choices, parser);
                            set_field(ns, &dest, val);
                            i += consumed;
                        }
                    }
                }
                continue;
            }
            // Unknown optional.
            extras.push(tok.clone());
            i += 1;
            continue;
        }
        // Positional token — may be a subcommand. The first positional that
        // names a registered subparser dispatches the remaining argv to it.
        if let Some(subs) = get_field(parser, "_subparsers") {
            if !subs.is_none() && positional_values.is_empty() {
                if let Some(sub_parser) = subparser_lookup(subs, tok) {
                    // Record the subcommand name into dest if configured.
                    if let Some(dest_v) = get_field(subs, "dest") {
                        if let Some(d) = extract_str(dest_v) {
                            set_field(ns, &d, new_str(tok));
                        }
                    }
                    // Remaining tokens belong to the subparser.
                    let rest: Vec<String> = argv[i + 1..].to_vec();
                    let (sub_ns, sub_extras) = run_parser(sub_parser, &rest, _known);
                    merge_namespace(ns, sub_ns);
                    extras.extend(sub_extras);
                    return (ns, extras);
                }
            }
        }
        positional_values.push(tok.clone());
        i += 1;
    }

    // Assign collected positionals to positional actions.
    assign_positionals(ns, &positionals, &positional_values, &mut extras, parser);

    // Check if required subcommand was provided.
    if let Some(subs) = get_field(parser, "_subparsers") {
        if !subs.is_none() {
            if get_field(subs, "required").and_then(|v| v.as_bool()) == Some(true) {
                if let Some(dest_v) = get_field(subs, "dest") {
                    if let Some(dest) = extract_str(dest_v) {
                        if get_field(ns, &dest).is_none() {
                            raise_exit(2);
                            return (ns, extras);
                        }
                    }
                }
            }
        }
    }

    // Required-optional check.
    for act in &optionals {
        if get_field(*act, "required").and_then(|v| v.as_bool()) == Some(true) {
            let dest = get_field(*act, "dest")
                .and_then(extract_str)
                .unwrap_or_default();
            if !seen_dests.contains(&dest) {
                raise_exit(2);
                return (ns, extras);
            }
        }
    }

    (ns, extras)
}

fn merge_namespace(dst: MbValue, src: MbValue) {
    for (k, v) in instance_fields(src) {
        // Don't clobber a set value with None unless dst lacks the field.
        if v.is_none() && get_field(dst, &k).is_some() {
            continue;
        }
        set_field(dst, &k, v);
    }
}

fn assign_positionals(
    ns: MbValue,
    positionals: &[MbValue],
    values: &[String],
    extras: &mut Vec<String>,
    parser: MbValue,
) {
    let mut vi = 0;
    for act in positionals {
        let dest = get_field(*act, "dest")
            .and_then(extract_str)
            .unwrap_or_default();
        let type_v = get_field(*act, "type").unwrap_or_else(MbValue::none);
        let choices = get_field(*act, "choices").unwrap_or_else(MbValue::none);
        let nargs = get_field(*act, "nargs").unwrap_or_else(MbValue::none);
        match nargs_kind(nargs).as_str() {
            "*" => {
                let collected: Vec<MbValue> = values[vi..]
                    .iter()
                    .map(|raw| {
                        let v = apply_type(type_v, raw, parser, &dest);
                        check_choice(choices, v, raw, parser);
                        v
                    })
                    .collect();
                set_field(ns, &dest, new_list(collected));
                vi = values.len();
            }
            "+" => {
                if vi >= values.len() {
                    raise_exit(2);
                    return;
                }
                let collected: Vec<MbValue> = values[vi..]
                    .iter()
                    .map(|raw| {
                        let v = apply_type(type_v, raw, parser, &dest);
                        check_choice(choices, v, raw, parser);
                        v
                    })
                    .collect();
                set_field(ns, &dest, new_list(collected));
                vi = values.len();
            }
            "?" => {
                if vi < values.len() {
                    let v = apply_type(type_v, &values[vi], parser, &dest);
                    check_choice(choices, v, &values[vi], parser);
                    set_field(ns, &dest, v);
                    vi += 1;
                }
            }
            _ => {
                if vi < values.len() {
                    let v = apply_type(type_v, &values[vi], parser, &dest);
                    check_choice(choices, v, &values[vi], parser);
                    set_field(ns, &dest, v);
                    vi += 1;
                } else {
                    raise_exit(2);
                    return;
                }
            }
        }
    }
    for leftover in &values[vi.min(values.len())..] {
        extras.push(leftover.clone());
    }
}

fn check_choice(choices: MbValue, value: MbValue, raw: &str, _parser: MbValue) {
    if !in_choices(choices, value) {
        raise_exit(2);
    }
}

/// Consume one value for an optional, applying type + choices. Returns
/// (value, tokens_consumed_including_the_flag).
fn take_value(
    argv: &[String],
    i: usize,
    inline_val: Option<String>,
    type_v: MbValue,
    choices: MbValue,
    parser: MbValue,
) -> (MbValue, usize) {
    let argname = argv.get(i).map(|s| s.as_str()).unwrap_or("");
    if let Some(v) = inline_val {
        let coerced = apply_type(type_v, &v, parser, argname);
        if coerced.is_none() {
            return (coerced, 1);
        }
        check_choice(choices, coerced, &v, parser);
        return (coerced, 1);
    }
    if i + 1 < argv.len() {
        let raw = &argv[i + 1];
        let coerced = apply_type(type_v, raw, parser, argname);
        if coerced.is_none() {
            return (coerced, 2);
        }
        check_choice(choices, coerced, raw, parser);
        return (coerced, 2);
    }
    // Missing value.
    raise_exit(2);
    (MbValue::none(), 1)
}

fn invoke_custom_action(
    custom_action_inst: MbValue,
    parser: MbValue,
    ns: MbValue,
    values: MbValue,
    _option_string: &str,
) {
    // The stored Action was already constructed; call its __call__. The
    // 4th argument `option_string` is left to its default — mamba's instance
    // method dispatch caps a bound call at 4 total values (self + 3), so a
    // 4-positional call (self + 4) would hit the arity fallback and silently
    // no-op. `__call__(self, parser, namespace, values, option_string=None)`
    // fires correctly with three positional args.
    let call_args = new_list(vec![parser, ns, values]);
    let name = new_str("__call__");
    super::super::class::mb_call_method(custom_action_inst, name, call_args);
}

fn is_positional(act: MbValue) -> bool {
    get_field(act, "_is_positional")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn nargs_kind(nargs: MbValue) -> String {
    extract_str(nargs).unwrap_or_default()
}

fn optional_display_name(act: MbValue) -> String {
    let opts = get_field(act, "option_strings")
        .map(seq_items)
        .unwrap_or_default();
    opts.first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_else(|| {
            get_field(act, "dest")
                .and_then(extract_str)
                .unwrap_or_default()
        })
}

fn looks_like_negative_number(tok: &str) -> bool {
    tok.len() > 1
        && tok.starts_with('-')
        && tok[1..].chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// Match an optional action by one of its option strings.
fn find_optional(optionals: &[MbValue], name: &str) -> Option<MbValue> {
    for act in optionals {
        let opts = get_field(*act, "option_strings")
            .map(seq_items)
            .unwrap_or_default();
        for o in opts {
            if extract_str(o).as_deref() == Some(name) {
                return Some(*act);
            }
        }
    }
    None
}

// ── Subparsers ──

/// subparsers.add_parser(name, **kwargs) -> a fresh ArgumentParser.
unsafe extern "C" fn method_add_parser(self_v: MbValue, args: MbValue) -> MbValue {
    let items = seq_items(args);
    let name = items
        .first()
        .copied()
        .and_then(extract_str)
        .unwrap_or_default();
    // Duplicate subcommand name → argparse.ArgumentError (CPython).
    if let Some(parsers) = get_field(self_v, "_parsers") {
        if dict_get(parsers, &name).is_some() {
            raise("ArgumentError", &format!("conflicting subparser: {name}"));
            return MbValue::none();
        }
    }
    let sub_parser = MbValue::from_ptr(MbObject::new_instance(PARSER_CLASS.to_string()));
    set_field(sub_parser, "_actions", new_list(vec![]));
    set_field(sub_parser, "_defaults", new_dict());
    set_field(sub_parser, "_subparsers", MbValue::none());
    set_field(sub_parser, "_exit_on_error", MbValue::from_bool(true));
    // Store under the _SubParsersAction.
    if let Some(parsers) = get_field(self_v, "_parsers") {
        if let Some(ptr) = parsers.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                super::super::rc::retain_if_ptr(sub_parser);
                lock.write().unwrap().insert(DictKey::Str(name), sub_parser);
            }
        }
    }
    sub_parser
}

fn subparser_lookup(subs: MbValue, name: &str) -> Option<MbValue> {
    get_field(subs, "_parsers").and_then(|parsers| dict_get(parsers, name))
}

// ── Registration ──

/// Register the argparse module.
pub fn register() {
    register_classes();

    let mut attrs: HashMap<String, MbValue> = HashMap::new();

    // Object-constructor dispatchers (callable + instantiating).
    let ctors: Vec<(&str, usize)> = vec![
        ("ArgumentParser", dispatch_argument_parser as usize),
        ("Namespace", dispatch_namespace as usize),
        ("Action", dispatch_action as usize),
        ("FileType", dispatch_filetype as usize),
        ("HelpFormatter", dispatch_help_formatter as usize),
        (
            "RawDescriptionHelpFormatter",
            dispatch_raw_description_formatter as usize,
        ),
        ("RawTextHelpFormatter", dispatch_raw_text_formatter as usize),
        (
            "ArgumentDefaultsHelpFormatter",
            dispatch_defaults_formatter as usize,
        ),
        (
            "MetavarTypeHelpFormatter",
            dispatch_metavar_type_formatter as usize,
        ),
        (
            "BooleanOptionalAction",
            dispatch_boolean_optional_action as usize,
        ),
        // `ngettext` is a re-exported gettext function — callable surface only.
        // Excluded from `__all__` (CPython omits it too); the
        // `all_matches_public_names` fixture filters it by name.
        ("ngettext", dispatch_ngettext as usize),
    ];
    for (name, addr) in ctors {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Bridge the ArgumentParser constructor func -> its class name so accessing
    // a class-attribute method (`argparse.ArgumentParser.add_argument`,
    // `.parse_args`, `.parse_known_args`) resolves to a callable unbound method.
    // mb_getattr's func->native-class bridge looks the func addr up in
    // NATIVE_TYPE_NAMES, then does lookup_method against the table that
    // register_classes() populates for PARSER_CLASS below. Without this mapping
    // the methods are registered but `callable(ArgumentParser.add_argument)` is
    // False (same gap fixed for ConfigParser in commit 34a0fe7237).
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(
            dispatch_argument_parser as *const () as usize as u64,
            PARSER_CLASS.to_string(),
        );
    });

    // `ArgumentError` is exposed as its registered class-name string so that
    // `except argparse.ArgumentError` matches (the except-clause matcher reads
    // the class name out of a Str value). The parser raises it via
    // `mb_raise("ArgumentError", ...)` for conflicting options / subparsers and
    // for `exit_on_error=False` conversion failures.
    // `ArgumentError` / `ArgumentTypeError` are exposed as real type-objects
    // (Instance class_name="type", __name__=X) so `type(argparse.ArgumentError)
    // .__name__ == "type"` (likewise ArgumentTypeError) — mirroring the
    // unittest TestCase/TestSuite precedent. The except-clause matcher resolves
    // a handler type-object's name via class::resolve_class_name(__name__), so
    // `except argparse.ArgumentError` still matches the `mb_raise("ArgumentError",
    // ...)` instances; and calling the type-object routes construction through
    // the registered `__init__` (mb_instance_new_with_init type-object ctor hook).
    attrs.insert(
        "ArgumentError".into(),
        make_exception_type_object("ArgumentError"),
    );
    attrs.insert(
        "ArgumentTypeError".into(),
        make_exception_type_object("ArgumentTypeError"),
    );

    // Constants / sentinels.
    attrs.insert("SUPPRESS".into(), new_str("==SUPPRESS=="));
    attrs.insert("OPTIONAL".into(), new_str("?"));
    attrs.insert("ZERO_OR_MORE".into(), new_str("*"));
    attrs.insert("ONE_OR_MORE".into(), new_str("+"));
    attrs.insert("REMAINDER".into(), new_str("..."));
    attrs.insert("PARSER".into(), new_str("A..."));

    // __all__ — the public name list (CPython 3.12).
    let all_names = [
        "ArgumentParser",
        "ArgumentError",
        "ArgumentTypeError",
        "BooleanOptionalAction",
        "FileType",
        "HelpFormatter",
        "ArgumentDefaultsHelpFormatter",
        "RawDescriptionHelpFormatter",
        "RawTextHelpFormatter",
        "MetavarTypeHelpFormatter",
        "Namespace",
        "Action",
        "ONE_OR_MORE",
        "OPTIONAL",
        "PARSER",
        "REMAINDER",
        "SUPPRESS",
        "ZERO_OR_MORE",
    ];
    attrs.insert(
        "__all__".into(),
        new_list(all_names.iter().map(|s| new_str(s)).collect()),
    );

    super::register_module("argparse", attrs);
}

/// Build a type-object (Instance `class_name="type"`, `__name__=name`) for an
/// argparse exception class, so `type(argparse.X).__name__ == "type"`.
/// `resolve_class_name` reads `__name__` off this shape, so `except X`
/// continues to match, and calling it constructs an `X` instance through the
/// registered `__init__`.
fn make_exception_type_object(name: &str) -> MbValue {
    let cls = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*cls).data {
            let mut f = fields.write().unwrap();
            f.insert("__name__".to_string(), new_str(name));
            f.insert("__qualname__".to_string(), new_str(name));
            f.insert("__module__".to_string(), new_str("argparse"));
        }
    }
    MbValue::from_ptr(cls)
}

/// One method-table entry: (name, addr, is_variadic).
/// Variadic methods receive `(self, args_list)`; non-variadic ones (the
/// dunder slots like `__getattr__`/`__str__` invoked by the runtime with a
/// fixed arity) receive their args positionally.
type MethodSpec = (&'static str, usize, bool);

/// Register the runtime classes in single `mb_class_register` calls each so a
/// later registration never clobbers an earlier method table.
fn register_classes() {
    register_method_class(
        PARSER_CLASS,
        &[
            ("add_argument", method_add_argument as usize, true),
            ("parse_args", method_parse_args as usize, true),
            ("parse_known_args", method_parse_known_args as usize, true),
            ("set_defaults", method_set_defaults as usize, true),
            ("get_default", method_get_default as usize, true),
            ("add_subparsers", method_add_subparsers as usize, true),
            ("error", method_error as usize, true),
            ("exit", method_exit as usize, true),
        ],
    );

    register_method_class(
        NAMESPACE_CLASS,
        &[
            ("__eq__", ns_eq as usize, true),
            ("__ne__", ns_ne as usize, true),
            ("__contains__", ns_contains as usize, true),
            // `__getattr__` raises AttributeError on a missing attribute, matching
            // CPython's plain-object Namespace. Registered NON-variadic because the
            // runtime invokes the `__getattr__` slot as `func(self, name_string)`
            // (mb_getattr) rather than through the variadic (self, args_list)
            // packing. The runtime's mb_getattr_default / mb_hasattr now clear the
            // pending AttributeError, so 3-arg getattr and hasattr keep working.
            ("__getattr__", ns_getattr as usize, false),
        ],
    );

    register_method_class(
        SUBPARSERS_CLASS,
        &[("add_parser", method_add_parser as usize, true)],
    );

    // ArgumentError / ArgumentTypeError are real Exception subclasses so the
    // except-clause matcher (which walks the registered class hierarchy)
    // recognises them and so `str(err)` dispatches our __str__.
    register_method_class_with_base(
        "ArgumentError",
        &["Exception"],
        &[
            ("__init__", argerr_init as usize, true),
            ("__str__", argerr_str as usize, true),
        ],
    );
    register_method_class_with_base(
        "ArgumentTypeError",
        &["Exception"],
        &[
            ("__init__", argtypeerr_init as usize, true),
            ("__str__", argtypeerr_str as usize, true),
        ],
    );
}

/// Register a class and its method table in one shot.
fn register_method_class(class_name: &str, methods: &[MethodSpec]) {
    register_method_class_with_base(class_name, &[], methods);
}

fn register_method_class_with_base(class_name: &str, bases: &[&str], methods: &[MethodSpec]) {
    let mut map: HashMap<String, MbValue> = HashMap::new();
    for (name, addr, variadic) in methods {
        map.insert(name.to_string(), MbValue::from_func(*addr));
        if *variadic {
            super::super::module::register_variadic_func(*addr as u64);
        }
    }
    let base_vec: Vec<String> = bases.iter().map(|s| s.to_string()).collect();
    super::super::class::mb_class_register(class_name, base_vec, map);
}

// ── Legacy public API (kept for symbols.rs JIT entries) ──

/// argparse.ArgumentParser(description) -> parser instance.
pub fn mb_argparse_new(desc: MbValue) -> MbValue {
    let parser = MbValue::from_ptr(MbObject::new_instance(PARSER_CLASS.to_string()));
    set_field(parser, "_actions", new_list(vec![]));
    set_field(parser, "_defaults", new_dict());
    set_field(parser, "_subparsers", MbValue::none());
    set_field(parser, "_exit_on_error", MbValue::from_bool(true));
    if !desc.is_none() {
        set_field(parser, "description", desc);
    }
    parser
}

/// parser.add_argument(name) -> Action (legacy single-name shim).
pub fn mb_argparse_add_argument(parser: MbValue, name: MbValue) -> MbValue {
    method_add_argument_internal(parser, vec![name])
}

fn method_add_argument_internal(parser: MbValue, names: Vec<MbValue>) -> MbValue {
    unsafe { method_add_argument(parser, new_list(names)) }
}

/// parser.parse_args() -> Namespace (legacy shim using process argv).
pub fn mb_argparse_parse_args(parser: MbValue) -> MbValue {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let (ns, _extras) = run_parser(parser, &argv, false);
    ns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_parser_has_actions_field() {
        let parser = mb_argparse_new(new_str("desc"));
        assert!(get_field(parser, "_actions").is_some());
    }

    #[test]
    fn test_add_argument_returns_action_with_dest() {
        let parser = mb_argparse_new(MbValue::none());
        let act = mb_argparse_add_argument(parser, new_str("--foo"));
        assert_eq!(
            get_field(act, "dest").and_then(extract_str),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_derive_dest_long_wins() {
        let opts = vec!["-b".to_string(), "--bar".to_string()];
        assert_eq!(derive_dest(&opts, None), "bar");
    }

    #[test]
    fn test_derive_dest_first_short() {
        let opts = vec!["-x".to_string(), "-y".to_string()];
        assert_eq!(derive_dest(&opts, None), "x");
    }

    #[test]
    fn test_derive_dest_explicit() {
        let opts = vec!["--foo".to_string()];
        assert_eq!(derive_dest(&opts, Some("baz".to_string())), "baz");
    }

    #[test]
    fn test_parse_positional() {
        let parser = mb_argparse_new(MbValue::none());
        mb_argparse_add_argument(parser, new_str("name"));
        let (ns, extras) = run_parser(parser, &["hello".to_string()], false);
        assert!(extras.is_empty());
        assert_eq!(
            get_field(ns, "name").and_then(extract_str),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_namespaces_equal_order_independent() {
        let ns1 = MbValue::from_ptr(MbObject::new_instance(NAMESPACE_CLASS.to_string()));
        set_field(ns1, "a", MbValue::from_int(1));
        set_field(ns1, "b", MbValue::from_int(2));
        let ns2 = MbValue::from_ptr(MbObject::new_instance(NAMESPACE_CLASS.to_string()));
        set_field(ns2, "b", MbValue::from_int(2));
        set_field(ns2, "a", MbValue::from_int(1));
        assert_eq!(namespaces_equal(ns1, ns2), Some(true));
    }

    #[test]
    fn test_namespaces_equal_vs_non_namespace() {
        let ns1 = MbValue::from_ptr(MbObject::new_instance(NAMESPACE_CLASS.to_string()));
        assert_eq!(namespaces_equal(ns1, MbValue::none()), None);
    }

    #[test]
    fn test_looks_like_negative_number() {
        assert!(looks_like_negative_number("-5"));
        assert!(looks_like_negative_number("-3.14"));
        assert!(!looks_like_negative_number("--foo"));
        assert!(!looks_like_negative_number("-x"));
    }
}
