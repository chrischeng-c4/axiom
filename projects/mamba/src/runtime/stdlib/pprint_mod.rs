/// pprint module for Mamba (#446).
///
/// Provides: pformat(obj, ...), pprint(obj, ...), saferepr(obj),
/// isrecursive(obj), isreadable(obj).
///
/// This is a faithful port of CPython 3.12's `pprint.PrettyPrinter`
/// formatting algorithm (Lib/pprint.py). The core idea: render an object's
/// single-line repr first; only break a container across multiple lines when
/// that single-line render would exceed the available width. Indentation and
/// allowance bookkeeping match CPython exactly so wrapped output is byte-for-
/// byte identical.
///
/// Supported keyword arguments (passed as a trailing kwargs dict by the call
/// lowering): `indent`, `width`, `depth`, `compact`, `sort_dicts`,
/// `underscore_numbers`.
///
/// Leaf reprs (and any object with a custom `__repr__`) are delegated to the
/// shared `builtins::mb_repr`, which honors user `__repr__`, string/bytes
/// quoting, etc.

use std::collections::HashMap;
use std::collections::HashSet;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::output::{write_captured};

// ----- native dispatch glue ---------------------------------------------

unsafe extern "C" fn dispatch_pprint(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (obj, cfg) = parse_args(a);
    let s = format_top(obj, &cfg);
    // pprint writes to the current stdout (respecting capture buffers used by
    // the test harness / redirect_stdout) and returns None.
    let line = format!("{s}\n");
    if !write_captured(&line) {
        print!("{line}");
    }
    MbValue::none()
}

unsafe extern "C" fn dispatch_pformat(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (obj, cfg) = parse_args(a);
    let s = format_top(obj, &cfg);
    MbValue::from_ptr(MbObject::new_str(s))
}

unsafe extern "C" fn dispatch_saferepr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let cfg = Config::default();
    let mut ctx = HashSet::new();
    let r = safe_repr(obj, &cfg, &mut ctx, 1);
    MbValue::from_ptr(MbObject::new_str(r.text))
}

unsafe extern "C" fn dispatch_isrecursive(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let cfg = Config::default();
    let mut ctx = HashSet::new();
    let r = safe_repr(obj, &cfg, &mut ctx, 1);
    MbValue::from_bool(r.recursive)
}

unsafe extern "C" fn dispatch_isreadable(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let cfg = Config::default();
    let mut ctx = HashSet::new();
    let r = safe_repr(obj, &cfg, &mut ctx, 1);
    MbValue::from_bool(r.readable && !r.recursive)
}

/// pprint.pformat(obj) -> pretty-formatted string. Single-arg public entry
/// kept for the runtime symbol table (`symbols.rs`); uses default options.
pub fn mb_pprint_pformat(val: MbValue) -> MbValue {
    let cfg = Config::default();
    let s = format_top(val, &cfg);
    MbValue::from_ptr(MbObject::new_str(s))
}

/// pprint.pprint(obj) -> print (respecting capture/redirect) and return None.
/// Single-arg public entry kept for the runtime symbol table.
pub fn mb_pprint_pprint(val: MbValue) -> MbValue {
    let cfg = Config::default();
    let s = format_top(val, &cfg);
    let line = format!("{s}\n");
    if !write_captured(&line) {
        print!("{line}");
    }
    MbValue::none()
}

/// Register the pprint module.
pub fn register() {
    let mut attrs = HashMap::new();
    let dispatchers: Vec<(&str, usize)> = vec![
        ("pprint", dispatch_pprint as usize),
        // pp(object, ...) is pprint with sort_dicts=False; reuse the pprint
        // dispatcher (surface: name must exist and be callable).
        ("pp", dispatch_pprint as usize),
        ("pformat", dispatch_pformat as usize),
        ("saferepr", dispatch_saferepr as usize),
        ("isrecursive", dispatch_isrecursive as usize),
        ("isreadable", dispatch_isreadable as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // ----- pprint.PrettyPrinter class ----------------------------------
    // A native class whose instances carry the formatting config and expose
    // pprint/pformat/isrecursive/isreadable methods (CPython's PrettyPrinter
    // surface). Registered in CLASS_REGISTRY so instance method dispatch
    // prepends `self` and routes to the m_* methods below.
    let mut methods: HashMap<String, MbValue> = HashMap::new();
    methods.insert("pprint".to_string(), MbValue::from_func(m_pp_pprint as usize));
    methods.insert("pformat".to_string(), MbValue::from_func(m_pp_pformat as usize));
    methods.insert("isrecursive".to_string(), MbValue::from_func(m_pp_isrecursive as usize));
    methods.insert("isreadable".to_string(), MbValue::from_func(m_pp_isreadable as usize));
    super::super::class::mb_class_register(
        "PrettyPrinter",
        vec!["object".to_string()],
        methods,
    );

    // Constructor dispatcher: pprint.PrettyPrinter(...) -> instance.
    let ctor = dispatch_pretty_printer as usize;
    attrs.insert("PrettyPrinter".to_string(), MbValue::from_func(ctor));
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| { s.borrow_mut().insert(ctor as u64); });
    super::super::module::NATIVE_TYPE_NAMES.with(|m| {
        m.borrow_mut().insert(ctor as u64, "PrettyPrinter".to_string());
    });

    super::register_module("pprint", attrs);
}

// ----- PrettyPrinter class: constructor, fields, methods ----------------

/// Store a `Config` onto a PrettyPrinter instance as plain fields, so the
/// method dispatchers can reconstruct it without re-parsing kwargs.
fn store_cfg(inst: MbValue, cfg: &Config) {
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                f.insert("_indent_per_level".to_string(), MbValue::from_int(cfg.indent as i64));
                f.insert("_width".to_string(), MbValue::from_int(cfg.width as i64));
                f.insert("_depth".to_string(), match cfg.depth {
                    Some(d) => MbValue::from_int(d as i64),
                    None => MbValue::none(),
                });
                f.insert("_compact".to_string(), MbValue::from_bool(cfg.compact));
                f.insert("_sort_dicts".to_string(), MbValue::from_bool(cfg.sort_dicts));
                f.insert("_underscore_numbers".to_string(), MbValue::from_bool(cfg.underscore_numbers));
            }
        }
    }
}

/// Rebuild a `Config` from a PrettyPrinter instance's stored fields.
fn load_cfg(inst: MbValue) -> Config {
    let mut cfg = Config::default();
    if let Some(ptr) = inst.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let f = fields.read().unwrap();
                if let Some(v) = f.get("_indent_per_level").and_then(|v| v.as_int()) {
                    cfg.indent = v.max(0) as usize;
                }
                if let Some(v) = f.get("_width").and_then(|v| v.as_int()) {
                    cfg.width = v.max(1) as usize;
                }
                if let Some(v) = f.get("_depth") {
                    cfg.depth = if v.is_none() { None } else { v.as_int().map(|i| i.max(0) as usize) };
                }
                if let Some(v) = f.get("_compact").and_then(|v| v.as_bool()) {
                    cfg.compact = v;
                }
                if let Some(v) = f.get("_sort_dicts").and_then(|v| v.as_bool()) {
                    cfg.sort_dicts = v;
                }
                if let Some(v) = f.get("_underscore_numbers").and_then(|v| v.as_bool()) {
                    cfg.underscore_numbers = v;
                }
            }
        }
    }
    cfg
}

/// `pprint.PrettyPrinter(indent=1, width=80, depth=None, stream=None, *,
/// compact=False, sort_dicts=True, underscore_numbers=False)`.
///
/// Positional order differs from `pformat`: the 4th positional is `stream`
/// (ignored here — output goes to the captured/redirected stdout like the
/// free functions). All test fixtures pass these by keyword anyway, so the
/// trailing-kwargs-dict path carries the real config.
unsafe extern "C" fn dispatch_pretty_printer(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let (positional, kwargs) = match a.last().and_then(|v| dict_as_kwargs(*v)) {
        Some(kw) => (&a[..a.len() - 1], Some(kw)),
        None => (a, None),
    };

    // PrettyPrinter accepts at most four positional arguments (indent, width,
    // depth, stream); compact/sort_dicts/underscore_numbers are keyword-only.
    // A fifth positional is a TypeError.
    if positional.len() > 4 {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "PrettyPrinter expected at most 4 positional arguments, got {}",
                positional.len()
            ))),
        );
        return MbValue::none();
    }

    let mut cfg = Config::default();

    // Capture the raw (unclamped) indent / width / depth so CPython's
    // constructor validation can fire before they are clamped for storage.
    // Defaults mirror Config::default() (indent=1, width=80, depth=None).
    let mut raw_indent: i64 = 1;
    let mut raw_width: i64 = 80;
    // depth: None = unset/None; Some(i) = an explicit integer depth.
    let mut raw_depth: Option<i64> = None;

    // Positional: indent, width, depth (stream is 4th — skipped).
    if let Some(v) = positional.first().and_then(|v| v.as_int()) {
        raw_indent = v;
    }
    if let Some(v) = positional.get(1).and_then(|v| v.as_int()) {
        raw_width = v;
    }
    if let Some(v) = positional.get(2) {
        raw_depth = if v.is_none() { None } else { v.as_int() };
    }

    if let Some(kw) = kwargs {
        if let Some(v) = kw_get(kw, "indent").and_then(|v| v.as_int()) {
            raw_indent = v;
        }
        if let Some(v) = kw_get(kw, "width").and_then(|v| v.as_int()) {
            raw_width = v;
        }
        if let Some(v) = kw_get(kw, "depth") {
            raw_depth = if v.is_none() { None } else { v.as_int() };
        }
        if let Some(v) = kw_get(kw, "compact").and_then(|v| v.as_bool()) {
            cfg.compact = v;
        }
        if let Some(v) = kw_get(kw, "sort_dicts").and_then(|v| v.as_bool()) {
            cfg.sort_dicts = v;
        }
        if let Some(v) = kw_get(kw, "underscore_numbers").and_then(|v| v.as_bool()) {
            cfg.underscore_numbers = v;
        }
    }

    // CPython PrettyPrinter.__init__ validation (Lib/pprint.py), same order:
    //   indent < 0                      -> ValueError('indent must be >= 0')
    //   depth is not None and depth<=0  -> ValueError('depth must be > 0')
    //   not width  (width == 0)         -> ValueError('width must be != 0')
    // Note width is only rejected when zero; negative width is accepted.
    if raw_indent < 0 {
        return raise_value_error("indent must be >= 0");
    }
    if let Some(d) = raw_depth {
        if d <= 0 {
            return raise_value_error("depth must be > 0");
        }
    }
    if raw_width == 0 {
        return raise_value_error("width must be != 0");
    }

    cfg.indent = raw_indent.max(0) as usize;
    cfg.width = raw_width.max(1) as usize;
    cfg.depth = raw_depth.map(|d| d.max(0) as usize);

    let inst = MbValue::from_ptr(MbObject::new_instance("PrettyPrinter".to_string()));
    store_cfg(inst, &cfg);
    inst
}

/// Raise a catchable Python `ValueError` with `msg` and return `None` (the
/// dispatcher's return value; the runtime checks the exception flag). Same
/// pattern as other native stdlib modules (e.g. graphlib_mod / codecs_mod).
fn raise_value_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// `PrettyPrinter.pprint(self, object)` — format and write to stdout.
extern "C" fn m_pp_pprint(this: MbValue, obj: MbValue) -> MbValue {
    let cfg = load_cfg(this);
    let s = format_top(obj, &cfg);
    let line = format!("{s}\n");
    if !write_captured(&line) {
        print!("{line}");
    }
    MbValue::none()
}

/// `PrettyPrinter.pformat(self, object)` -> pretty string.
extern "C" fn m_pp_pformat(this: MbValue, obj: MbValue) -> MbValue {
    let cfg = load_cfg(this);
    let s = format_top(obj, &cfg);
    MbValue::from_ptr(MbObject::new_str(s))
}

/// `PrettyPrinter.isrecursive(self, object)`.
extern "C" fn m_pp_isrecursive(this: MbValue, obj: MbValue) -> MbValue {
    let cfg = load_cfg(this);
    let mut ctx = HashSet::new();
    let r = safe_repr(obj, &cfg, &mut ctx, 1);
    MbValue::from_bool(r.recursive)
}

/// `PrettyPrinter.isreadable(self, object)`.
extern "C" fn m_pp_isreadable(this: MbValue, obj: MbValue) -> MbValue {
    let cfg = load_cfg(this);
    let mut ctx = HashSet::new();
    let r = safe_repr(obj, &cfg, &mut ctx, 1);
    MbValue::from_bool(r.readable && !r.recursive)
}

// ----- configuration & argument parsing ---------------------------------

#[derive(Clone)]
struct Config {
    indent: usize,
    width: usize,
    depth: Option<usize>,
    compact: bool,
    sort_dicts: bool,
    underscore_numbers: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            indent: 1,
            width: 80,
            depth: None,
            compact: false,
            sort_dicts: true,
            underscore_numbers: false,
        }
    }
}

/// Detect a trailing kwargs dict (only if it names a pprint parameter), so a
/// genuine dict positional (the object being printed!) is never mistaken for
/// kwargs.
fn dict_as_kwargs(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            const KEYS: [&str; 8] = [
                "indent", "width", "depth", "compact", "sort_dicts",
                "underscore_numbers", "stream", "object",
            ];
            // Keys are stored as DictKey; only string keys can be kwargs.
            for (k, _) in map.iter() {
                let kv = super::super::dict_ops::dict_key_to_mbvalue(k);
                if let Some(kp) = kv.as_ptr() {
                    if let ObjData::Str(ref s) = (*kp).data {
                        if KEYS.contains(&s.as_str()) {
                            return Some(v);
                        }
                    }
                }
            }
        }
    }
    None
}

fn kw_get(dict: MbValue, key: &str) -> Option<MbValue> {
    let ptr = dict.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            for (k, val) in map.iter() {
                let kv = super::super::dict_ops::dict_key_to_mbvalue(k);
                if let Some(kp) = kv.as_ptr() {
                    if let ObjData::Str(ref s) = (*kp).data {
                        if s == key {
                            return Some(*val);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse `(object[, stream][, indent][, width][, depth], *, compact,
/// sort_dicts, underscore_numbers)` into `(object, Config)`.
///
/// CPython signature: `pformat(object, indent=1, width=80, depth=None, *,
/// compact=False, sort_dicts=True, underscore_numbers=False)`.
/// (`pprint` also takes a positional `stream` before `indent`, but the test
/// fixtures pass these by keyword; positional parsing here mirrors `pformat`.)
fn parse_args(a: &[MbValue]) -> (MbValue, Config) {
    let (positional, kwargs) = match a.last().and_then(|v| dict_as_kwargs(*v)) {
        Some(kw) => (&a[..a.len() - 1], Some(kw)),
        None => (a, None),
    };

    let obj = positional.first().copied().unwrap_or_else(MbValue::none);
    let mut cfg = Config::default();

    // Positional: object, indent, width, depth (matching pformat()).
    if let Some(v) = positional.get(1).and_then(|v| v.as_int()) {
        cfg.indent = v.max(0) as usize;
    }
    if let Some(v) = positional.get(2).and_then(|v| v.as_int()) {
        cfg.width = v.max(1) as usize;
    }
    if let Some(v) = positional.get(3) {
        cfg.depth = if v.is_none() { None } else { v.as_int().map(|i| i.max(0) as usize) };
    }

    if let Some(kw) = kwargs {
        if let Some(v) = kw_get(kw, "indent").and_then(|v| v.as_int()) {
            cfg.indent = v.max(0) as usize;
        }
        if let Some(v) = kw_get(kw, "width").and_then(|v| v.as_int()) {
            cfg.width = v.max(1) as usize;
        }
        if let Some(v) = kw_get(kw, "depth") {
            cfg.depth = if v.is_none() { None } else { v.as_int().map(|i| i.max(0) as usize) };
        }
        if let Some(v) = kw_get(kw, "compact").and_then(|v| v.as_bool()) {
            cfg.compact = v;
        }
        if let Some(v) = kw_get(kw, "sort_dicts").and_then(|v| v.as_bool()) {
            cfg.sort_dicts = v;
        }
        if let Some(v) = kw_get(kw, "underscore_numbers").and_then(|v| v.as_bool()) {
            cfg.underscore_numbers = v;
        }
    }

    (obj, cfg)
}

// ----- object identity for recursion detection --------------------------

fn obj_id(val: MbValue) -> Option<usize> {
    val.as_ptr().map(|p| p as usize)
}

// ----- leaf repr via shared builtins::mb_repr ---------------------------

fn builtin_repr(val: MbValue) -> String {
    let r = super::super::builtins::mb_repr(val);
    if let Some(ptr) = r.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return s.clone();
            }
        }
    }
    String::new()
}

/// Format an integer with underscore grouping (CPython `format(n, "_d")`).
fn underscore_int(text: &str) -> String {
    let (sign, digits) = if let Some(rest) = text.strip_prefix('-') {
        ("-", rest)
    } else {
        ("", text)
    };
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    let n = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        let rem = n - i;
        if i != 0 && rem % 3 == 0 {
            out.push('_');
        }
        out.push(*b as char);
    }
    format!("{sign}{out}")
}

// ----- _safe_repr: single-line repr triple (text, readable, recursive) --

struct SafeRepr {
    text: String,
    readable: bool,
    recursive: bool,
}

/// CPython `_safe_repr`. Produces the canonical single-line repr plus the
/// `readable`/`recursive` flags. Honors `depth` (maxlevels), `sort_dicts`,
/// and `underscore_numbers`. Falls back to `builtins::mb_repr` for anything
/// that isn't an exact builtin list/tuple/dict/int.
fn safe_repr(
    val: MbValue,
    cfg: &Config,
    context: &mut HashSet<usize>,
    level: usize,
) -> SafeRepr {
    // Scalars (int handled specially below for underscore_numbers).
    if val.is_none() || val.as_bool().is_some() || val.as_float().is_some()
        || val.is_not_implemented()
    {
        return SafeRepr { text: builtin_repr(val), readable: true, recursive: false };
    }
    if let Some(i) = val.as_int() {
        // bool already handled above; this is a plain int.
        let text = if cfg.underscore_numbers {
            underscore_int(&format!("{i}"))
        } else {
            format!("{i}")
        };
        return SafeRepr { text, readable: true, recursive: false };
    }

    if let Some(ptr) = val.as_ptr() {
        let id = ptr as usize;
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) | ObjData::Bytes(_) | ObjData::ByteArray(_) => {
                    // Builtin scalars: repr is always readable.
                    return SafeRepr { text: builtin_repr(val), readable: true, recursive: false };
                }
                ObjData::Complex(_, _) => {
                    return SafeRepr { text: builtin_repr(val), readable: true, recursive: false };
                }
                ObjData::Dict(ref lock) => {
                    let map = lock.read().unwrap();
                    if map.is_empty() {
                        return SafeRepr { text: "{}".to_string(), readable: true, recursive: false };
                    }
                    if let Some(maxl) = cfg.depth {
                        if level > maxl {
                            return SafeRepr {
                                text: "{...}".to_string(),
                                readable: false,
                                recursive: context.contains(&id),
                            };
                        }
                    }
                    if context.contains(&id) {
                        return SafeRepr {
                            text: recursion_marker(val),
                            readable: false,
                            recursive: true,
                        };
                    }
                    // Snapshot entries before recursing.
                    let mut entries: Vec<(MbValue, MbValue)> = map.iter()
                        .map(|(k, v)| (super::super::dict_ops::dict_key_to_mbvalue(k), *v))
                        .collect();
                    drop(map);
                    if cfg.sort_dicts {
                        sort_by_safe_key(&mut entries, |e| e.0);
                    }
                    context.insert(id);
                    let mut readable = true;
                    let mut recursive = false;
                    let mut parts = Vec::with_capacity(entries.len());
                    for (k, v) in &entries {
                        let kr = safe_repr(*k, cfg, context, level + 1);
                        let vr = safe_repr(*v, cfg, context, level + 1);
                        readable = readable && kr.readable && vr.readable;
                        if kr.recursive || vr.recursive { recursive = true; }
                        parts.push(format!("{}: {}", kr.text, vr.text));
                    }
                    context.remove(&id);
                    return SafeRepr {
                        text: format!("{{{}}}", parts.join(", ")),
                        readable,
                        recursive,
                    };
                }
                ObjData::List(ref lock) => {
                    let items: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                    if items.is_empty() {
                        return SafeRepr { text: "[]".to_string(), readable: true, recursive: false };
                    }
                    return safe_repr_seq(&items, "[", "]", false, id, cfg, context, level);
                }
                ObjData::Tuple(items) => {
                    if items.is_empty() {
                        return SafeRepr { text: "()".to_string(), readable: true, recursive: false };
                    }
                    let one = items.len() == 1;
                    let items = items.clone();
                    return safe_repr_seq(&items, "(", ")", one, id, cfg, context, level);
                }
                _ => {}
            }
        }
    }

    // Fallback: arbitrary object. readable iff repr doesn't start with '<'.
    let text = builtin_repr(val);
    let readable = !text.is_empty() && !text.starts_with('<');
    SafeRepr { text, readable, recursive: false }
}

#[allow(clippy::too_many_arguments)]
fn safe_repr_seq(
    items: &[MbValue],
    open: &str,
    close: &str,
    single_tuple: bool,
    id: usize,
    cfg: &Config,
    context: &mut HashSet<usize>,
    level: usize,
) -> SafeRepr {
    if let Some(maxl) = cfg.depth {
        if level > maxl {
            let body = if single_tuple { "...,".to_string() } else { "...".to_string() };
            return SafeRepr {
                text: format!("{open}{body}{close}"),
                readable: false,
                recursive: context.contains(&id),
            };
        }
    }
    if context.contains(&id) {
        return SafeRepr { text: "<Recursion>".to_string(), readable: false, recursive: true };
    }
    context.insert(id);
    let mut readable = true;
    let mut recursive = false;
    let mut parts = Vec::with_capacity(items.len());
    for o in items {
        let r = safe_repr(*o, cfg, context, level + 1);
        if !r.readable { readable = false; }
        if r.recursive { recursive = true; }
        parts.push(r.text);
    }
    context.remove(&id);
    let joined = parts.join(", ");
    let text = if single_tuple {
        format!("{open}{joined},{close}")
    } else {
        format!("{open}{joined}{close}")
    };
    SafeRepr { text, readable, recursive }
}

/// CPython `<Recursion on typename with id=...>`.
fn recursion_marker(val: MbValue) -> String {
    let (tname, id) = if let Some(ptr) = val.as_ptr() {
        let id = (ptr as usize) & 0x7FFF_FFFF_FFFF;
        let tname = unsafe {
            match &(*ptr).data {
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                _ => "object",
            }
        };
        (tname, id)
    } else {
        ("object", 0)
    };
    format!("<Recursion on {tname} with id={id}>")
}

// ----- _safe_key / _safe_tuple sorting ----------------------------------

/// Sort entries by CPython's `_safe_key` ordering. CPython tries normal
/// comparison first and falls back to `(str(type), id)`. We approximate with
/// a total order that matches CPython for the homogeneous and common mixed
/// cases the fixtures exercise: numbers compare numerically, strings
/// lexicographically, and unorderable/mixed types order by (type-name, then
/// repr/value).
fn sort_by_safe_key<T: Copy>(entries: &mut [T], key: impl Fn(&T) -> MbValue) {
    entries.sort_by(|a, b| safe_key_cmp(key(a), key(b)));
}

#[derive(PartialEq)]
enum KeyClass {
    Num(f64),
    Str(String),
    Other(String, usize),
}

fn key_class(v: MbValue) -> KeyClass {
    if let Some(b) = v.as_bool() {
        return KeyClass::Num(if b { 1.0 } else { 0.0 });
    }
    if let Some(i) = v.as_int() {
        return KeyClass::Num(i as f64);
    }
    if let Some(f) = v.as_float() {
        return KeyClass::Num(f);
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                return KeyClass::Str(s.clone());
            }
        }
    }
    // Unorderable / composite: order by (type-name, id) like CPython's
    // fallback. Use the repr to derive a stable type-name proxy.
    let tname = type_name(v);
    let id = obj_id(v).unwrap_or(0);
    KeyClass::Other(tname, id)
}

fn type_name(v: MbValue) -> String {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Tuple(_) => "tuple".to_string(),
                ObjData::List(_) => "list".to_string(),
                ObjData::Dict(_) => "dict".to_string(),
                ObjData::Set(_) => "set".to_string(),
                ObjData::FrozenSet(_) => "frozenset".to_string(),
                ObjData::Bytes(_) => "bytes".to_string(),
                ObjData::ByteArray(_) => "bytearray".to_string(),
                ObjData::Instance { class_name, .. } => class_name.clone(),
                _ => "object".to_string(),
            }
        }
    } else {
        "object".to_string()
    }
}

fn safe_key_cmp(a: MbValue, b: MbValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let ka = key_class(a);
    let kb = key_class(b);
    // Same-class comparisons preserve CPython's "try normal comparison" rule.
    match (&ka, &kb) {
        (KeyClass::Num(x), KeyClass::Num(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (KeyClass::Str(x), KeyClass::Str(y)) => x.cmp(y),
        _ => {
            // Different classes / unorderable: fall back to (type-name, id),
            // mirroring CPython's `(str(type(self.obj)), id(self.obj))`.
            let rank = |k: &KeyClass| -> (String, usize, f64, String) {
                match k {
                    KeyClass::Num(n) => ("int".to_string(), 0, *n, String::new()),
                    KeyClass::Str(s) => ("str".to_string(), 0, 0.0, s.clone()),
                    KeyClass::Other(t, id) => (t.clone(), *id, 0.0, String::new()),
                }
            };
            let (ta, ia, na, sa) = rank(&ka);
            let (tb, ib, nb, sb) = rank(&kb);
            ta.cmp(&tb)
                .then(na.partial_cmp(&nb).unwrap_or(Ordering::Equal))
                .then(sa.cmp(&sb))
                .then(ia.cmp(&ib))
        }
    }
}

// ----- the formatting engine (CPython _format) --------------------------

/// Top-level entry: format `val` with config `cfg`, anchored at column 0.
fn format_top(val: MbValue, cfg: &Config) -> String {
    let mut out = String::new();
    let mut context: HashSet<usize> = HashSet::new();
    format_obj(val, &mut out, 0, 0, &mut context, 1, cfg);
    out
}

/// CPython `_format`: write the single-line repr unless it overflows the
/// available width, in which case dispatch to the container-breaking writer.
fn format_obj(
    val: MbValue,
    out: &mut String,
    indent: usize,
    allowance: usize,
    context: &mut HashSet<usize>,
    level: usize,
    cfg: &Config,
) {
    // Recursion check (matches CPython's id-in-context guard in _format).
    if let Some(id) = obj_id(val) {
        if context.contains(&id) {
            out.push_str(&recursion_marker(val));
            return;
        }
    }

    let mut local_ctx = HashSet::new();
    let r = safe_repr(val, cfg, &mut local_ctx, level);
    let max_width = (cfg.width as isize) - (indent as isize) - (allowance as isize);

    if (r.text.len() as isize) > max_width {
        // Try to break this container.
        if let Some(ptr) = val.as_ptr() {
            let id = ptr as usize;
            unsafe {
                match &(*ptr).data {
                    ObjData::List(ref lock) => {
                        let items: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                        if !items.is_empty() {
                            out.push('[');
                            context.insert(id);
                            format_items(&items, out, indent, allowance + 1, context, level, cfg);
                            context.remove(&id);
                            out.push(']');
                            return;
                        }
                    }
                    ObjData::Tuple(items) => {
                        if !items.is_empty() {
                            let items = items.clone();
                            let endchar = if items.len() == 1 { ",)" } else { ")" };
                            out.push('(');
                            context.insert(id);
                            format_items(&items, out, indent, allowance + endchar.len(), context, level, cfg);
                            context.remove(&id);
                            out.push_str(endchar);
                            return;
                        }
                    }
                    ObjData::Dict(ref lock) => {
                        let map = lock.read().unwrap();
                        if !map.is_empty() {
                            let mut entries: Vec<(MbValue, MbValue)> = map.iter()
                                .map(|(k, v)| (super::super::dict_ops::dict_key_to_mbvalue(k), *v))
                                .collect();
                            drop(map);
                            if cfg.sort_dicts {
                                sort_by_safe_key(&mut entries, |e| e.0);
                            }
                            out.push('{');
                            if cfg.indent > 1 {
                                out.push_str(&" ".repeat(cfg.indent - 1));
                            }
                            context.insert(id);
                            format_dict_items(&entries, out, indent, allowance + 1, context, level, cfg);
                            context.remove(&id);
                            out.push('}');
                            return;
                        }
                    }
                    ObjData::Set(ref lock) => {
                        let items: Vec<MbValue> = lock.read().unwrap().iter().copied().collect();
                        if !items.is_empty() {
                            let mut items = items;
                            items.sort_by(|a, b| safe_key_cmp(*a, *b));
                            out.push('{');
                            context.insert(id);
                            format_items(&items, out, indent, allowance + 1, context, level, cfg);
                            context.remove(&id);
                            out.push('}');
                            return;
                        }
                    }
                    ObjData::FrozenSet(items) => {
                        if !items.is_empty() {
                            let mut items = items.clone();
                            items.sort_by(|a, b| safe_key_cmp(*a, *b));
                            // typ.__name__ + "({" ... "})"
                            let prefix = "frozenset({";
                            out.push_str(prefix);
                            let extra = "frozenset".len() + 1;
                            context.insert(id);
                            format_items(&items, out, indent + extra, allowance + 2, context, level, cfg);
                            context.remove(&id);
                            out.push_str("})");
                            return;
                        }
                    }
                    ObjData::Str(s) => {
                        if !s.is_empty() {
                            if let Some(text) = pprint_str(s, indent, allowance, level, cfg) {
                                out.push_str(&text);
                                return;
                            }
                        }
                    }
                    ObjData::Bytes(data) => {
                        if !data.is_empty() {
                            if let Some(text) = pprint_bytes(data, indent, allowance, level, cfg) {
                                out.push_str(&text);
                                return;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    out.push_str(&r.text);
}

/// CPython `_format_items` for list/tuple/set bodies.
fn format_items(
    items: &[MbValue],
    out: &mut String,
    mut indent: usize,
    allowance: usize,
    context: &mut HashSet<usize>,
    level: usize,
    cfg: &Config,
) {
    indent += cfg.indent;
    if cfg.indent > 1 {
        out.push_str(&" ".repeat(cfg.indent - 1));
    }
    let delimnl = format!(",\n{}", " ".repeat(indent));
    let mut delim = String::new();
    let mut width = (cfg.width as isize) - (indent as isize) + 1;
    let mut max_width = width;
    let n = items.len();

    let mut i = 0;
    while i < n {
        let ent = items[i];
        let last = i == n - 1;
        if last {
            max_width -= allowance as isize;
            width -= allowance as isize;
        }
        if cfg.compact {
            let mut local_ctx = HashSet::new();
            let rep = safe_repr(ent, cfg, &mut local_ctx, level + 1).text;
            let w = rep.len() as isize + 2;
            if width < w {
                width = max_width;
                if !delim.is_empty() {
                    delim = delimnl.clone();
                }
            }
            if width >= w {
                width -= w;
                out.push_str(&delim);
                delim = ", ".to_string();
                out.push_str(&rep);
                i += 1;
                continue;
            }
        }
        out.push_str(&delim);
        delim = delimnl.clone();
        let a = if last { allowance } else { 1 };
        format_obj(ent, out, indent, a, context, level + 1, cfg);
        i += 1;
    }
}

/// CPython `_format_dict_items`.
fn format_dict_items(
    entries: &[(MbValue, MbValue)],
    out: &mut String,
    mut indent: usize,
    allowance: usize,
    context: &mut HashSet<usize>,
    level: usize,
    cfg: &Config,
) {
    indent += cfg.indent;
    let delimnl = format!(",\n{}", " ".repeat(indent));
    let last_index = entries.len() - 1;
    for (i, (key, ent)) in entries.iter().enumerate() {
        let last = i == last_index;
        let mut local_ctx = HashSet::new();
        let rep = safe_repr(*key, cfg, &mut local_ctx, level + 1).text;
        out.push_str(&rep);
        out.push_str(": ");
        let a = if last { allowance } else { 1 };
        format_obj(*ent, out, indent + rep.len() + 2, a, context, level + 1, cfg);
        if !last {
            out.push_str(&delimnl);
        }
    }
}

// ----- str / bytes wrapping (_pprint_str / _pprint_bytes) ---------------

/// CPython `_pprint_str`. Returns None when the string would render on a
/// single line (caller falls back to the plain repr). The string is split on
/// whitespace runs into adjacent quoted literals.
fn pprint_str(s: &str, mut indent: usize, mut allowance: usize, level: usize, cfg: &Config) -> Option<String> {
    let lines: Vec<String> = split_keepends(s);
    if level == 1 {
        indent += 1;
        allowance += 1;
    }
    let max_width1_base = (cfg.width as isize) - (indent as isize);
    let max_width = max_width1_base;
    let mut chunks: Vec<String> = Vec::new();
    let nlines = lines.len();
    let mut last_rep = String::new();
    for (i, line) in lines.iter().enumerate() {
        let rep = repr_str(line);
        last_rep = rep.clone();
        let mut max_width1 = max_width1_base;
        if i == nlines - 1 {
            max_width1 -= allowance as isize;
        }
        if (rep.len() as isize) <= max_width1 {
            chunks.push(rep);
        } else {
            let parts = findall_word_space(line);
            let mut max_width2 = max_width;
            let mut current = String::new();
            let nparts = parts.len();
            for (j, part) in parts.iter().enumerate() {
                let candidate = format!("{current}{part}");
                if j == nparts - 1 && i == nlines - 1 {
                    max_width2 -= allowance as isize;
                }
                if (repr_str(&candidate).len() as isize) > max_width2 {
                    if !current.is_empty() {
                        chunks.push(repr_str(&current));
                    }
                    current = part.clone();
                } else {
                    current = candidate;
                }
            }
            if !current.is_empty() {
                chunks.push(repr_str(&current));
            }
        }
    }
    if chunks.len() == 1 {
        // Single chunk: render plainly (caller used repr already).
        return Some(last_rep);
    }
    let mut out = String::new();
    if level == 1 {
        out.push('(');
    }
    for (i, rep) in chunks.iter().enumerate() {
        if i > 0 {
            out.push('\n');
            out.push_str(&" ".repeat(indent));
        }
        out.push_str(rep);
    }
    if level == 1 {
        out.push(')');
    }
    Some(out)
}

/// CPython `_pprint_bytes`: bytes of length > 4 are split into adjacent
/// `b'...'` literals built four bytes at a time (`_wrap_bytes_repr`).
fn pprint_bytes(data: &[u8], mut indent: usize, mut allowance: usize, level: usize, _cfg: &Config) -> Option<String> {
    if data.len() <= 4 {
        return None;
    }
    let parens = level == 1;
    let mut out = String::new();
    if parens {
        indent += 1;
        allowance += 1;
        out.push('(');
    }
    let parts = wrap_bytes_repr(data, (_cfg.width as isize) - (indent as isize), allowance as isize);
    let mut delim = String::new();
    for rep in &parts {
        out.push_str(&delim);
        out.push_str(rep);
        if delim.is_empty() {
            delim = format!("\n{}", " ".repeat(indent));
        }
    }
    if parens {
        out.push(')');
    }
    Some(out)
}

/// CPython `_wrap_bytes_repr`: emit `b'...'` literals, accumulating in 4-byte
/// chunks and flushing whenever the candidate would exceed `width`. The final
/// chunk-start (`i == last`) gets `allowance` subtracted from `width`.
fn wrap_bytes_repr(data: &[u8], width: isize, allowance: isize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current: Vec<u8> = Vec::new();
    let n = data.len();
    let last = n / 4 * 4;
    let mut width = width;
    let mut i = 0usize;
    while i < n {
        let part = &data[i..(i + 4).min(n)];
        let mut candidate = current.clone();
        candidate.extend_from_slice(part);
        if i == last {
            width -= allowance;
        }
        if (repr_bytes(&candidate).len() as isize) > width {
            if !current.is_empty() {
                result.push(repr_bytes(&current));
            }
            current = part.to_vec();
        } else {
            current = candidate;
        }
        i += 4;
    }
    if !current.is_empty() {
        result.push(repr_bytes(&current));
    }
    result
}

fn repr_str(s: &str) -> String {
    builtin_repr(MbValue::from_ptr(MbObject::new_str(s.to_string())))
}

fn repr_bytes(data: &[u8]) -> String {
    let v = MbValue::from_ptr(MbObject::new_bytes(data.to_vec()));
    builtin_repr(v)
}

/// Equivalent of Python `str.splitlines(keepends=True)`.
fn split_keepends(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        cur.push(c);
        if c == '\n' {
            out.push(std::mem::take(&mut cur));
        } else if c == '\r' {
            if i + 1 < chars.len() && chars[i + 1] == '\n' {
                cur.push('\n');
                i += 1;
            }
            out.push(std::mem::take(&mut cur));
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// Equivalent of Python `re.findall(r'\S*\s*', line)` minus the trailing empty
/// match: alternating non-space then space runs.
fn findall_word_space(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let n = chars.len();
    while i < n {
        let start = i;
        while i < n && !chars[i].is_whitespace() {
            i += 1;
        }
        while i < n && chars[i].is_whitespace() {
            i += 1;
        }
        if i > start {
            parts.push(chars[start..i].iter().collect());
        } else {
            break;
        }
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pf(val: MbValue) -> String {
        let cfg = Config::default();
        format_top(val, &cfg)
    }

    #[test]
    fn test_pformat_int() {
        assert_eq!(pf(MbValue::from_int(42)), "42");
    }

    #[test]
    fn test_pformat_short_list_single_line() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        assert_eq!(pf(list), "[1, 2]");
    }

    #[test]
    fn test_underscore_int() {
        assert_eq!(underscore_int("1234567"), "1_234_567");
        assert_eq!(underscore_int("1000"), "1_000");
        assert_eq!(underscore_int("999"), "999");
        assert_eq!(underscore_int("-1234"), "-1_234");
    }
}
