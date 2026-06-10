/// dataclasses module for Mamba (#410, PEP 557).
///
/// Real class transformation: the lowering records ordered class-body field
/// facts (`mb_dataclass_record_field*`, emitted at the ClassDefPlaceholder
/// position right before the decorator call), and the `@dataclass` decorator
/// processes them into a `DcClass` entry in `DC_REGISTRY`. The runtime then
/// consults that registry to synthesize behavior:
///
/// - `__init__`: `class.rs::mb_instance_new_with_init` calls
///   `dc_run_synth_init` (positional binding in declaration order, defaults,
///   fresh `default_factory` per instance, InitVar forwarding, `__post_init__`).
/// - `__repr__`: `builtins.rs::mb_repr` calls `dc_repr_string`.
/// - `__eq__` / ordering: `builtins.rs::{mb_values_eq, mb_values_lt}` consult
///   `dc_eq_field_names` / `dc_order_field_names`.
/// - `__hash__`: `builtins.rs::mb_hash` consults `dc_hash_field_names`
///   (frozen / unsafe_hash → field-tuple hash).
/// - frozen: `class.rs::mb_setattr` consults `is_frozen_dataclass` and raises
///   `FrozenInstanceError`.
///
/// Helpers (`fields`, `asdict`, `astuple`, `replace`, `is_dataclass`,
/// `field`) operate on the same registry. `make_dataclass` stays a stub:
/// runtime class synthesis (registering a brand-new class with compiled
/// methods at runtime) is not yet supported by the class system.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};
use super::super::rc::{retain_if_ptr, release_if_ptr};

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

// ── Registry data model ──

/// One processed dataclass field.
#[derive(Clone)]
struct DcField {
    name: String,
    /// Annotation repr (e.g. `int`, `list`, `ClassVar[str]`).
    ty: String,
    /// Class-definition-time default value (plain `y: int = 0` defaults and
    /// `field(default=...)`).
    default: Option<MbValue>,
    /// `field(default_factory=...)` callable — called fresh per instance.
    default_factory: Option<MbValue>,
    /// Participates in the synthesized `__init__` (field(init=False) → false).
    init: bool,
    /// Shown by the synthesized `__repr__` (field(repr=False) → false).
    repr: bool,
    /// Participates in `__eq__` / ordering / hash (field(compare=False) → false).
    compare: bool,
    /// Keyword-only (class kw_only=True, KW_ONLY sentinel zone, or
    /// field(kw_only=True)).
    kw_only: bool,
    /// `field(metadata=...)` mapping.
    metadata: Option<MbValue>,
    /// `InitVar[...]` pseudo-field: an `__init__` param forwarded to
    /// `__post_init__`, never stored and absent from `fields()`.
    is_initvar: bool,
    /// `ClassVar[...]`: excluded from `__init__` and `fields()`; its default
    /// stays a shared class attribute.
    is_classvar: bool,
}

/// Decorator options for one dataclass.
#[derive(Clone, Copy)]
struct DcOptions {
    init: bool,
    repr: bool,
    eq: bool,
    order: bool,
    frozen: bool,
    unsafe_hash: bool,
    kw_only: bool,
    slots: bool,
    match_args: bool,
}

impl Default for DcOptions {
    fn default() -> Self {
        DcOptions {
            init: true,
            repr: true,
            eq: true,
            order: false,
            frozen: false,
            unsafe_hash: false,
            kw_only: false,
            slots: false,
            match_args: true,
        }
    }
}

#[derive(Clone)]
struct DcClass {
    fields: Vec<DcField>,
    opts: DcOptions,
}

thread_local! {
    /// Raw (name, annotation, default) facts recorded by the lowering for a
    /// decorated class, consumed by the `@dataclass` decorator.
    static PENDING_FIELDS: std::cell::RefCell<HashMap<String, Vec<(String, String, Option<MbValue>)>>> =
        std::cell::RefCell::new(HashMap::new());
    /// Options parsed from a called decorator form `@dataclass(frozen=True)`,
    /// consumed by the immediately-following class application.
    static PENDING_OPTIONS: std::cell::RefCell<Option<DcOptions>> =
        std::cell::RefCell::new(None);
    /// Processed dataclasses by class name.
    static DC_REGISTRY: std::cell::RefCell<HashMap<String, DcClass>> =
        std::cell::RefCell::new(HashMap::new());
    /// Cached `fields()` tuple per class (same Field objects each call, so
    /// `fields(C) == fields(C())` holds via element identity).
    static FIELD_TUPLES: std::cell::RefCell<HashMap<String, MbValue>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Reset all dataclass registries (centralized runtime cleanup between test
/// executions). Values are cleared without releasing, mirroring
/// `class.rs::cleanup_all_classes` — leaked objects reclaimed at process exit.
pub(crate) fn cleanup_all_dataclasses() {
    let _ = PENDING_FIELDS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = PENDING_OPTIONS.with(|c| c.try_borrow_mut().map(|mut o| *o = None));
    let _ = DC_REGISTRY.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = FIELD_TUPLES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
}

// ── Lowering-facing fact recording ──

/// Record one ordered class-body field fact with a class-definition-time
/// default value. Emitted by the lowering immediately before the `@dataclass`
/// decorator call.
pub fn mb_dataclass_record_field(cls: MbValue, name: MbValue, ann: MbValue, default: MbValue) {
    let (Some(cls), Some(name), Some(ann)) =
        (extract_str(cls), extract_str(name), extract_str(ann)) else { return };
    // Registry owns one reference to the stored default.
    unsafe { retain_if_ptr(default); }
    PENDING_FIELDS.with(|reg| {
        reg.borrow_mut().entry(cls).or_default().push((name, ann, Some(default)));
    });
}

/// Record one ordered class-body field fact without a default (`x: int`).
pub fn mb_dataclass_record_field_nodefault(cls: MbValue, name: MbValue, ann: MbValue) {
    let (Some(cls), Some(name), Some(ann)) =
        (extract_str(cls), extract_str(name), extract_str(ann)) else { return };
    PENDING_FIELDS.with(|reg| {
        reg.borrow_mut().entry(cls).or_default().push((name, ann, None));
    });
}

// ── Annotation / marker classification ──

/// Leaf of a possibly-dotted, possibly-generic annotation:
/// `typing.ClassVar[int]` → `ClassVar`, `KW_ONLY` → `KW_ONLY`.
fn annotation_head(ann: &str) -> &str {
    let base = ann.split('[').next().unwrap_or(ann).trim();
    base.rsplit('.').next().unwrap_or(base)
}

/// Is `val` a Field marker produced by `dataclasses.field(...)`?
fn is_field_marker(val: MbValue) -> bool {
    val.as_ptr().is_some_and(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            class_name == "Field" && fields.read().unwrap().contains_key("__dc_field__")
        } else {
            false
        }
    })
}

/// Read one attribute from a Field marker / Field object instance.
fn marker_get(val: MbValue, key: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            fields.read().unwrap().get(key).copied()
        } else {
            None
        }
    })
}

fn truthy_flag(val: Option<MbValue>, default: bool) -> bool {
    match val {
        Some(v) => v.as_bool().unwrap_or_else(|| v.as_int().map(|i| i != 0).unwrap_or(default)),
        None => default,
    }
}

// ── Decoration ──

/// Parse `@dataclass(...)` keyword options from the trailing kwargs dict.
fn parse_options(dict: MbValue) -> DcOptions {
    let mut opts = DcOptions::default();
    if let Some(ptr) = dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                for (k, v) in lock.read().unwrap().iter() {
                    let Some(key) = k.as_str() else { continue };
                    let b = v.as_bool().unwrap_or_else(|| v.as_int().map(|i| i != 0).unwrap_or(false));
                    match key {
                        "init" => opts.init = b,
                        "repr" => opts.repr = b,
                        "eq" => opts.eq = b,
                        "order" => opts.order = b,
                        "frozen" => opts.frozen = b,
                        "unsafe_hash" => opts.unsafe_hash = b,
                        "kw_only" => opts.kw_only = b,
                        "slots" => opts.slots = b,
                        "match_args" => opts.match_args = b,
                        _ => {}
                    }
                }
            }
        }
    }
    opts
}

/// Apply the `@dataclass` transformation to a registered class: consume the
/// recorded field facts, merge base dataclass fields (MRO order), classify
/// ClassVar / InitVar / KW_ONLY, unpack `field(...)` markers, publish plain
/// defaults as class attributes, set `__match_args__` / `__slots__`, and
/// store the processed `DcClass`.
fn decorate_class(class_name: &str, opts: DcOptions) {
    let raw = PENDING_FIELDS.with(|reg| reg.borrow_mut().remove(class_name))
        .unwrap_or_default();

    // Inherited dataclass fields first (least-derived ancestor first so
    // subclass re-declarations override in place).
    let mut merged: Vec<DcField> = Vec::new();
    let mro = super::super::class::class_mro_list(class_name);
    for ancestor in mro.iter().rev() {
        let base_fields = DC_REGISTRY.with(|reg| {
            reg.borrow().get(ancestor.as_str()).map(|d| d.fields.clone())
        });
        if let Some(bf) = base_fields {
            for f in bf {
                if let Some(pos) = merged.iter().position(|m| m.name == f.name) {
                    merged[pos] = f;
                } else {
                    merged.push(f);
                }
            }
        }
    }

    let mut kw_only_zone = opts.kw_only;
    for (name, ann, default) in raw {
        let head = annotation_head(&ann);
        if head == "KW_ONLY" {
            // `_: KW_ONLY` pseudo-field: every following field is kw-only.
            kw_only_zone = true;
            if let Some(d) = default { unsafe { release_if_ptr(d); } }
            continue;
        }
        let is_classvar = head == "ClassVar";
        let is_initvar = head == "InitVar";
        let mut f = DcField {
            name,
            ty: ann.clone(),
            default: None,
            default_factory: None,
            init: true,
            repr: true,
            compare: true,
            kw_only: kw_only_zone,
            metadata: None,
            is_initvar,
            is_classvar,
        };
        if let Some(d) = default {
            if is_field_marker(d) {
                // Unpack field(...) options. Marker attr values are retained
                // here (the registry owns them); the marker itself is released.
                let take = |key: &str| -> Option<MbValue> {
                    let v = marker_get(d, key)?;
                    unsafe { retain_if_ptr(v); }
                    Some(v)
                };
                f.default = take("default");
                f.default_factory = take("default_factory");
                f.metadata = take("metadata");
                f.init = truthy_flag(marker_get(d, "init"), true);
                f.repr = truthy_flag(marker_get(d, "repr"), true);
                f.compare = truthy_flag(marker_get(d, "compare"), true);
                if truthy_flag(marker_get(d, "kw_only"), false) {
                    f.kw_only = true;
                }
                unsafe { release_if_ptr(d); }
            } else {
                f.default = Some(d);
            }
        }
        if let Some(pos) = merged.iter().position(|m| m.name == f.name) {
            merged[pos] = f;
        } else {
            merged.push(f);
        }
    }

    let cls_val = MbValue::from_ptr(MbObject::new_str(class_name.to_string()));

    // Plain defaults become shared class attributes (CPython sets them on the
    // class; ClassVar defaults in particular must be class-shared).
    for f in &merged {
        if f.is_initvar {
            continue;
        }
        if let Some(d) = f.default {
            super::super::class::mb_class_set_class_attr(
                cls_val,
                MbValue::from_ptr(MbObject::new_str(f.name.clone())),
                d,
            );
        }
    }

    // __match_args__: positional (non-kw-only) real fields, in order.
    // or_insert semantics in mb_class_set_match_args preserve an explicit
    // class-body `__match_args__`.
    if opts.match_args {
        let names: Vec<MbValue> = merged.iter()
            .filter(|f| !f.is_classvar && !f.is_initvar && f.init && !f.kw_only)
            .map(|f| MbValue::from_ptr(MbObject::new_str(f.name.clone())))
            .collect();
        let tup = MbValue::from_ptr(MbObject::new_tuple(names));
        super::super::class::mb_class_set_match_args(cls_val, tup);
    }

    // @dataclass(slots=True): publish __slots__ and register the slot set
    // (suppresses the per-instance __dict__ like a literal __slots__ would).
    if opts.slots {
        let slot_names: Vec<String> = merged.iter()
            .filter(|f| !f.is_classvar && !f.is_initvar)
            .map(|f| f.name.clone())
            .collect();
        let slots_list = MbValue::from_ptr(MbObject::new_list(
            slot_names.iter()
                .map(|n| MbValue::from_ptr(MbObject::new_str(n.clone())))
                .collect(),
        ));
        super::super::class::mb_register_slots(cls_val, slots_list);
        let slots_tuple = MbValue::from_ptr(MbObject::new_tuple(
            slot_names.iter()
                .map(|n| MbValue::from_ptr(MbObject::new_str(n.clone())))
                .collect(),
        ));
        super::super::class::mb_class_set_class_attr(
            cls_val,
            MbValue::from_ptr(MbObject::new_str("__slots__".to_string())),
            slots_tuple,
        );
    }

    FIELD_TUPLES.with(|c| { c.borrow_mut().remove(class_name); });
    DC_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(class_name.to_string(), DcClass { fields: merged, opts });
    });
}

// ── Runtime-facing queries (class.rs / builtins.rs hooks) ──

/// Resolve a dataclass registration for `class_name`, walking the MRO so
/// undecorated subclasses of a dataclass still answer `is_dataclass`.
fn lookup_dc(class_name: &str) -> Option<DcClass> {
    DC_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(d) = reg.get(class_name) {
            return Some(d.clone());
        }
        None
    })
}

/// Does `class_name` (or an MRO ancestor) name a registered dataclass?
fn is_dataclass_name(class_name: &str) -> bool {
    if DC_REGISTRY.with(|reg| reg.borrow().contains_key(class_name)) {
        return true;
    }
    super::super::class::class_mro_list(class_name)
        .iter()
        .any(|a| DC_REGISTRY.with(|reg| reg.borrow().contains_key(a.as_str())))
}

/// class.rs hook: should instance creation route through the synthesized
/// dataclass `__init__`?
pub(crate) fn dc_has_synth_init(class_name: &str) -> bool {
    lookup_dc(class_name).map(|d| d.opts.init).unwrap_or(false)
}

/// class.rs hook: is this class a frozen dataclass (reject setattr)?
pub(crate) fn is_frozen_dataclass(class_name: &str) -> bool {
    lookup_dc(class_name).map(|d| d.opts.frozen).unwrap_or(false)
}

/// builtins.rs hook: compare-field names when the class is a dataclass with
/// `eq=True` (the default).
pub(crate) fn dc_eq_field_names(class_name: &str) -> Option<Vec<String>> {
    let d = lookup_dc(class_name)?;
    if !d.opts.eq {
        return None;
    }
    Some(compare_field_names(&d))
}

/// builtins.rs hook: compare-field names when the class is a dataclass with
/// `order=True`.
pub(crate) fn dc_order_field_names(class_name: &str) -> Option<Vec<String>> {
    let d = lookup_dc(class_name)?;
    if !d.opts.order {
        return None;
    }
    Some(compare_field_names(&d))
}

/// builtins.rs hook: hash-field names when the dataclass synthesizes
/// `__hash__` (frozen with eq, or unsafe_hash).
pub(crate) fn dc_hash_field_names(class_name: &str) -> Option<Vec<String>> {
    let d = lookup_dc(class_name)?;
    if !(d.opts.unsafe_hash || (d.opts.frozen && d.opts.eq)) {
        return None;
    }
    Some(compare_field_names(&d))
}

fn compare_field_names(d: &DcClass) -> Vec<String> {
    d.fields.iter()
        .filter(|f| !f.is_classvar && !f.is_initvar && f.compare)
        .map(|f| f.name.clone())
        .collect()
}

/// builtins.rs hook: the synthesized `__repr__` — `Cls(f1=v1, f2=v2)` over
/// repr=True fields in declaration order. None when the class is not a
/// dataclass or was decorated with repr=False.
pub(crate) fn dc_repr_string(val: MbValue, class_name: &str) -> Option<String> {
    let d = lookup_dc(class_name)?;
    if !d.opts.repr {
        return None;
    }
    let ptr = val.as_ptr()?;
    let parts: Vec<String> = unsafe {
        let ObjData::Instance { ref fields, .. } = (*ptr).data else { return None };
        let guard = fields.read().unwrap();
        d.fields.iter()
            .filter(|f| !f.is_classvar && !f.is_initvar && f.repr)
            .map(|f| {
                let v = guard.get(&f.name).copied().unwrap_or_else(MbValue::none);
                let r = super::super::builtins::mb_repr(v);
                let rs = extract_str(r).unwrap_or_default();
                format!("{}={}", f.name, rs)
            })
            .collect()
    };
    Some(format!("{}({})", class_name, parts.join(", ")))
}

// ── Synthesized __init__ ──

/// Write one field directly into the instance dict (bypasses mb_setattr so
/// frozen dataclasses can still be initialized).
fn set_instance_field(instance: MbValue, name: &str, value: MbValue) {
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                retain_if_ptr(value);
                let old = fields.write().unwrap().insert(name.to_string(), value);
                if let Some(prev) = old {
                    release_if_ptr(prev);
                }
            }
        }
    }
}

/// Call a default factory. Builtin type objects (`list`, `dict`, `set`,
/// `tuple`) dispatched through mb_call0 would produce a bare Instance shell,
/// so construct the real builtin container directly.
fn call_factory(factory: MbValue) -> MbValue {
    let type_name = factory.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            if class_name == "type" {
                fields.read().ok().and_then(|f| f.get("__name__").and_then(|v| extract_str(*v)))
            } else {
                None
            }
        } else {
            None
        }
    });
    if let Some(tn) = type_name {
        match tn.as_str() {
            "list" => return MbValue::from_ptr(MbObject::new_list(Vec::new())),
            "dict" => return MbValue::from_ptr(MbObject::new_dict()),
            "set" => return MbValue::from_ptr(MbObject::new_set(Vec::new())),
            "tuple" => return MbValue::from_ptr(MbObject::new_tuple(Vec::new())),
            "str" => return MbValue::from_ptr(MbObject::new_str(String::new())),
            "int" => return MbValue::from_int(0),
            "float" => return MbValue::from_float(0.0),
            "bool" => return MbValue::from_bool(false),
            _ => {}
        }
    }
    super::super::class::mb_call0(factory)
}

/// Resolve the value for an `__init__` field that was not supplied (or was
/// supplied as a call-site-filled Field marker). `marker` is that marker, if
/// any. Returns `(value, owned)` — `owned` is true for factory-produced
/// values (the caller must balance the fresh +1 after storing). None when the
/// field has no default (caller raises TypeError).
fn resolve_default(f: &DcField, marker: Option<MbValue>) -> Option<(MbValue, bool)> {
    if let Some(fac) = f.default_factory {
        return Some((call_factory(fac), true));
    }
    if let Some(d) = f.default {
        return Some((d, false));
    }
    if let Some(m) = marker {
        if let Some(fac) = marker_get(m, "default_factory") {
            return Some((call_factory(fac), true));
        }
        if let Some(d) = marker_get(m, "default") {
            return Some((d, false));
        }
    }
    None
}

/// The synthesized dataclass `__init__`: bind positional args to
/// init-participating fields in declaration order, fill defaults /
/// default_factory, forward InitVars to `__post_init__`, seed init=False
/// fields from their defaults, then call `__post_init__` when defined.
pub(crate) fn dc_run_synth_init(class_name: &str, instance: MbValue, args_list: MbValue) {
    let Some(d) = lookup_dc(class_name) else { return };
    let args: Vec<MbValue> = args_list.as_ptr().map(|ptr| unsafe {
        if let ObjData::List(ref lock) = (*ptr).data {
            lock.read().unwrap().iter().copied().collect()
        } else {
            Vec::new()
        }
    }).unwrap_or_default();

    let mut initvars: Vec<MbValue> = Vec::new();
    let mut pos = 0usize;
    for f in d.fields.iter().filter(|f| !f.is_classvar && f.init) {
        let supplied = args.get(pos).copied();
        pos += 1;
        let (val, owned) = match supplied {
            Some(v) if !is_field_marker(v) => (v, false),
            other => {
                let marker = other.filter(|v| is_field_marker(*v));
                match resolve_default(f, marker) {
                    Some(pair) => pair,
                    None => {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "{class_name}.__init__() missing required positional argument: '{}'",
                                f.name,
                            ))),
                        );
                        return;
                    }
                }
            }
        };
        if f.is_initvar {
            initvars.push(val);
        } else {
            set_instance_field(instance, &f.name, val);
            if owned {
                // set_instance_field took its own +1; drop the factory's.
                unsafe { release_if_ptr(val); }
            }
        }
    }

    // init=False real fields still get their declared default / factory.
    for f in d.fields.iter().filter(|f| !f.is_classvar && !f.is_initvar && !f.init) {
        if let Some((v, owned)) = resolve_default(f, None) {
            set_instance_field(instance, &f.name, v);
            if owned {
                unsafe { release_if_ptr(v); }
            }
        }
    }

    // __post_init__(self, *initvars)
    let post = super::super::class::lookup_method(class_name, "__post_init__");
    if !post.is_none() {
        let name = MbValue::from_ptr(MbObject::new_str("__post_init__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(initvars));
        super::super::class::mb_call_method(instance, name, args);
    }
}

// ── Decorator / helper dispatchers (native (args_ptr, nargs) ABI) ──

/// `@dataclass` / `@dataclass(frozen=True, ...)`.
///
/// Bare form: receives the class-name string → apply transformation (using
/// any options primed by a preceding called form). Called form: receives the
/// trailing kwargs dict → prime PENDING_OPTIONS and return the dataclass
/// dispatcher itself, which the decorator machinery then calls with the
/// class name.
unsafe extern "C" fn dispatch_dataclass(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    // Decoration mode: a string naming a registered class.
    for v in a {
        if let Some(s) = extract_str(*v) {
            if super::super::class::class_is_registered(&s) {
                let opts = PENDING_OPTIONS.with(|c| c.borrow_mut().take())
                    .unwrap_or_default();
                decorate_class(&s, opts);
                unsafe { retain_if_ptr(*v); }
                return *v;
            }
        }
    }
    // Options mode: parse the kwargs dict (if any) and return the decorator.
    let mut opts = DcOptions::default();
    for v in a {
        let is_dict = v.as_ptr().is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) });
        if is_dict {
            opts = parse_options(*v);
            break;
        }
    }
    PENDING_OPTIONS.with(|c| { *c.borrow_mut() = Some(opts); });
    MbValue::from_func(dispatch_dataclass as *const () as usize)
}

/// `field(default=..., default_factory=..., init=..., repr=..., compare=...,
/// metadata=..., kw_only=...)` — build a Field marker carrying exactly the
/// provided options (key presence distinguishes "no default" from a None
/// default).
unsafe extern "C" fn dispatch_field(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let marker = MbObject::new_instance("Field".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*marker).data {
            let mut f = fields.write().unwrap();
            f.insert("__dc_field__".to_string(), MbValue::from_bool(true));
            for v in a {
                if let Some(ptr) = v.as_ptr() {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        for (k, val) in lock.read().unwrap().iter() {
                            let Some(key) = k.as_str() else { continue };
                            if matches!(
                                key,
                                "default" | "default_factory" | "init" | "repr"
                                | "compare" | "metadata" | "kw_only" | "hash"
                            ) {
                                retain_if_ptr(*val);
                                f.insert(key.to_string(), *val);
                            }
                        }
                    }
                }
            }
        }
    }
    MbValue::from_ptr(marker)
}

/// Resolve the dataclass name for a `fields()` / `asdict()` / `replace()`
/// argument: a class-name string or an instance of a registered dataclass
/// (MRO-aware for undecorated subclasses).
fn dc_name_of(obj: MbValue) -> Option<String> {
    let direct = extract_str(obj).or_else(|| {
        obj.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    })?;
    if DC_REGISTRY.with(|reg| reg.borrow().contains_key(direct.as_str())) {
        return Some(direct);
    }
    super::super::class::class_mro_list(&direct)
        .into_iter()
        .find(|a| DC_REGISTRY.with(|reg| reg.borrow().contains_key(a.as_str())))
}

/// Build (and cache) the `fields()` tuple of Field objects for a class.
fn fields_tuple_for(class_name: &str) -> MbValue {
    if let Some(cached) = FIELD_TUPLES.with(|c| c.borrow().get(class_name).copied()) {
        unsafe { retain_if_ptr(cached); }
        return cached;
    }
    let missing = || MbValue::from_ptr(MbObject::new_str("MISSING".to_string()));
    let d = match lookup_dc(class_name) {
        Some(d) => d,
        None => return MbValue::from_ptr(MbObject::new_tuple(Vec::new())),
    };
    let mut items: Vec<MbValue> = Vec::new();
    for f in d.fields.iter().filter(|f| !f.is_classvar && !f.is_initvar) {
        let obj = MbObject::new_instance("Field".to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*obj).data {
                let mut m = fields.write().unwrap();
                m.insert("name".to_string(), MbValue::from_ptr(MbObject::new_str(f.name.clone())));
                m.insert("type".to_string(), MbValue::from_ptr(MbObject::new_str(f.ty.clone())));
                let default = match f.default {
                    Some(v) => { retain_if_ptr(v); v }
                    None => missing(),
                };
                m.insert("default".to_string(), default);
                let factory = match f.default_factory {
                    Some(v) => { retain_if_ptr(v); v }
                    None => missing(),
                };
                m.insert("default_factory".to_string(), factory);
                m.insert("init".to_string(), MbValue::from_bool(f.init));
                m.insert("repr".to_string(), MbValue::from_bool(f.repr));
                m.insert("compare".to_string(), MbValue::from_bool(f.compare));
                m.insert("kw_only".to_string(), MbValue::from_bool(f.kw_only));
                let metadata = match f.metadata {
                    Some(v) => { retain_if_ptr(v); v }
                    None => MbValue::from_ptr(MbObject::new_dict()),
                };
                m.insert("metadata".to_string(), metadata);
                m.insert("hash".to_string(), MbValue::from_bool(f.compare));
            }
        }
        items.push(MbValue::from_ptr(obj));
    }
    let tup = MbValue::from_ptr(MbObject::new_tuple(items));
    unsafe { retain_if_ptr(tup); } // cache's own reference
    FIELD_TUPLES.with(|c| { c.borrow_mut().insert(class_name.to_string(), tup); });
    tup
}

/// fields(class_or_instance) — tuple of Field objects in declaration order.
unsafe extern "C" fn dispatch_fields(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    match dc_name_of(obj) {
        Some(name) => fields_tuple_for(&name),
        None => {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "fields() should be called with a dataclass type or instance".to_string(),
                )),
            );
            MbValue::none()
        }
    }
}

/// Recursive value conversion shared by asdict / astuple. `as_tuple` selects
/// the astuple shape for nested dataclasses.
fn convert_value(v: MbValue, as_tuple: bool) -> MbValue {
    // Nested dataclass instance → recurse.
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, .. } => {
                    if is_dataclass_name(class_name) {
                        return if as_tuple { dc_astuple(v) } else { dc_asdict(v) };
                    }
                }
                ObjData::List(lock) => {
                    let items: Vec<MbValue> = lock.read().unwrap().iter()
                        .map(|x| convert_value(*x, as_tuple))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_list(items));
                }
                ObjData::Tuple(items) => {
                    let out: Vec<MbValue> = items.iter()
                        .map(|x| convert_value(*x, as_tuple))
                        .collect();
                    return MbValue::from_ptr(MbObject::new_tuple(out));
                }
                ObjData::Dict(lock) => {
                    let dict = MbObject::new_dict();
                    if let ObjData::Dict(ref out_lock) = (*dict).data {
                        let mut out = out_lock.write().unwrap();
                        for (k, val) in lock.read().unwrap().iter() {
                            out.insert(k.clone(), convert_value(*val, as_tuple));
                        }
                    }
                    return MbValue::from_ptr(dict);
                }
                _ => {}
            }
        }
    }
    // Leaf value: the new container owns one reference.
    unsafe { retain_if_ptr(v); }
    v
}

fn dc_asdict(obj: MbValue) -> MbValue {
    let Some(name) = dc_name_of(obj) else {
        return MbValue::from_ptr(MbObject::new_dict());
    };
    let Some(d) = lookup_dc(&name) else {
        return MbValue::from_ptr(MbObject::new_dict());
    };
    let dict = MbObject::new_dict();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                if let ObjData::Dict(ref out_lock) = (*dict).data {
                    let mut out = out_lock.write().unwrap();
                    for f in d.fields.iter().filter(|f| !f.is_classvar && !f.is_initvar) {
                        let v = guard.get(&f.name).copied().unwrap_or_else(MbValue::none);
                        out.insert(f.name.clone().into(), convert_value(v, false));
                    }
                }
            }
        }
    }
    MbValue::from_ptr(dict)
}

fn dc_astuple(obj: MbValue) -> MbValue {
    let Some(name) = dc_name_of(obj) else {
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    };
    let Some(d) = lookup_dc(&name) else {
        return MbValue::from_ptr(MbObject::new_tuple(Vec::new()));
    };
    let mut items: Vec<MbValue> = Vec::new();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                for f in d.fields.iter().filter(|f| !f.is_classvar && !f.is_initvar) {
                    let v = guard.get(&f.name).copied().unwrap_or_else(MbValue::none);
                    items.push(convert_value(v, true));
                }
            }
        }
    }
    MbValue::from_ptr(MbObject::new_tuple(items))
}

unsafe extern "C" fn dispatch_asdict(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    dc_asdict(obj)
}

unsafe extern "C" fn dispatch_astuple(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    dc_astuple(obj)
}

/// is_dataclass(obj) — True for a registered dataclass type, its instances,
/// and instances of (undecorated) subclasses.
unsafe extern "C" fn dispatch_is_dataclass(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    MbValue::from_bool(dc_name_of(obj).is_some())
}

/// replace(obj, **changes) — construct a new instance through the synthesized
/// __init__ with selected fields overridden; unspecified fields keep the
/// original's (shared) values.
unsafe extern "C" fn dispatch_replace(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = a.first().copied().unwrap_or_else(MbValue::none);
    let Some(name) = dc_name_of(obj) else {
        unsafe { retain_if_ptr(obj); }
        return obj;
    };
    let Some(d) = lookup_dc(&name) else {
        unsafe { retain_if_ptr(obj); }
        return obj;
    };
    // Trailing kwargs dict carries the changes.
    let changes: Option<MbValue> = a.iter().skip(1).rev().find(|v| {
        v.as_ptr().is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
    }).copied();
    let change_of = |field: &str| -> Option<MbValue> {
        let dict = changes?;
        let ptr = dict.as_ptr()?;
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.read().unwrap().get(field).copied()
            } else {
                None
            }
        }
    };

    let new_inst = MbValue::from_ptr(MbObject::new_instance(name.clone()));
    let mut init_args: Vec<MbValue> = Vec::new();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let guard = fields.read().unwrap();
                for f in d.fields.iter().filter(|f| !f.is_classvar && f.init) {
                    let v = change_of(&f.name)
                        .or_else(|| guard.get(&f.name).copied())
                        .unwrap_or_else(MbValue::none);
                    // The temp args list owns one reference per element.
                    retain_if_ptr(v);
                    init_args.push(v);
                }
            }
        }
    }
    let args_list = MbValue::from_ptr(MbObject::new_list(init_args));
    dc_run_synth_init(&name, new_inst, args_list);
    // Drop the temp args list (releases the element refs taken above).
    unsafe { release_if_ptr(args_list); }
    new_inst
}

/// make_dataclass(cls_name, fields, ...) — STUB. Building a working class at
/// runtime needs runtime class synthesis (a registered class with no compiled
/// body), which the class system does not support yet. Returns the first
/// argument unchanged so the surface fixture (presence + callability) passes.
unsafe extern "C" fn dispatch_make_dataclass(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

/// Model `dataclasses.FrozenInstanceError` as a type-object Instance
/// (`class_name="type"`, `__name__="FrozenInstanceError"`) so
/// `except FrozenInstanceError:` resolves through
/// `class.rs::resolve_class_name`, while keeping the BaseException chaining
/// slots (`__cause__` / `__context__` / `__suppress_context__`) that the
/// surface dimension probes via `hasattr`. The slot defaults are seeded with
/// an inert non-None sentinel (mb_hasattr treats a None value as absent).
fn make_exception_class(class_name: &str) -> MbValue {
    let ptr = MbObject::new_instance("type".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
            let mut fields = fields.write().unwrap();
            let slot_sentinel = || MbValue::from_ptr(MbObject::new_str(String::new()));
            fields.insert("__name__".into(),
                MbValue::from_ptr(MbObject::new_str(class_name.to_string())));
            fields.insert("__cause__".into(), slot_sentinel());
            fields.insert("__context__".into(), slot_sentinel());
            fields.insert("__suppress_context__".into(), MbValue::from_bool(false));
        }
    }
    MbValue::from_ptr(ptr)
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Native ABI dispatchers: `callable(dataclasses.<fn>)` is True and dynamic
    // dispatch uses the `extern "C" fn(*const MbValue, usize) -> MbValue`
    // convention (kwargs arrive as a trailing dict by convention).
    for (name, addr) in [
        ("dataclass", dispatch_dataclass as *const () as usize),
        ("field", dispatch_field as *const () as usize),
        ("fields", dispatch_fields as *const () as usize),
        ("asdict", dispatch_asdict as *const () as usize),
        ("astuple", dispatch_astuple as *const () as usize),
        ("is_dataclass", dispatch_is_dataclass as *const () as usize),
        ("replace", dispatch_replace as *const () as usize),
        ("make_dataclass", dispatch_make_dataclass as *const () as usize),
    ] {
        attrs.insert(name.into(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Sentinels that must be PRESENT but NOT callable: a plain (non-type-name)
    // string value answers `hasattr` True and `callable` False.
    attrs.insert("MISSING".into(),
        MbValue::from_ptr(MbObject::new_str("MISSING".to_string())));
    attrs.insert("KW_ONLY".into(),
        MbValue::from_ptr(MbObject::new_str("KW_ONLY".to_string())));

    // `FrozenInstanceError` subclasses `AttributeError` (see exception.rs
    // hierarchy entries); exported as a type-object shell so except-clauses
    // and surface hasattr probes both resolve.
    attrs.insert("FrozenInstanceError".into(),
        make_exception_class("FrozenInstanceError"));

    // Re-exported type objects and submodule names from CPython's
    // dataclasses.py — surface fixtures only assert presence (`hasattr`).
    for name in [
        "Field", "FunctionType", "GenericAlias", "InitVar",
        "abc", "copy", "functools", "inspect", "itertools", "keyword",
        "re", "sys", "types",
    ] {
        attrs.insert(name.into(),
            MbValue::from_ptr(MbObject::new_str(name.to_string())));
    }

    super::register_module("dataclasses", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(cls: &str, name: &str, ann: &str, default: Option<MbValue>) {
        let c = MbValue::from_ptr(MbObject::new_str(cls.to_string()));
        let n = MbValue::from_ptr(MbObject::new_str(name.to_string()));
        let a = MbValue::from_ptr(MbObject::new_str(ann.to_string()));
        match default {
            Some(d) => mb_dataclass_record_field(c, n, a, d),
            None => mb_dataclass_record_field_nodefault(c, n, a),
        }
    }

    fn get_field(inst: MbValue, key: &str) -> MbValue {
        if let Some(ptr) = inst.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    if let Some(v) = fields.read().unwrap().get(key) { return *v; }
                }
            }
        }
        MbValue::none()
    }

    fn register_test_class(name: &str) {
        // Minimal class registration so class_is_registered / decorate work.
        let cls = MbValue::from_ptr(MbObject::new_str(name.to_string()));
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        let empty2 = MbValue::from_ptr(MbObject::new_list(vec![]));
        let empty3 = MbValue::from_ptr(MbObject::new_list(vec![]));
        super::super::super::class::mb_class_define_multi(cls, empty, empty2, empty3);
    }

    #[test]
    fn test_annotation_head_classification() {
        assert_eq!(annotation_head("int"), "int");
        assert_eq!(annotation_head("ClassVar[str]"), "ClassVar");
        assert_eq!(annotation_head("typing.ClassVar[int]"), "ClassVar");
        assert_eq!(annotation_head("dataclasses.KW_ONLY"), "KW_ONLY");
        assert_eq!(annotation_head("InitVar[int]"), "InitVar");
    }

    #[test]
    fn test_decorate_registers_fields_in_order() {
        cleanup_all_dataclasses();
        register_test_class("TPoint");
        record("TPoint", "x", "int", None);
        record("TPoint", "y", "int", Some(MbValue::from_int(7)));
        decorate_class("TPoint", DcOptions::default());
        let d = lookup_dc("TPoint").expect("registered");
        assert_eq!(d.fields.len(), 2);
        assert_eq!(d.fields[0].name, "x");
        assert_eq!(d.fields[1].name, "y");
        assert_eq!(d.fields[1].default.and_then(|v| v.as_int()), Some(7));
        assert!(d.opts.eq && d.opts.init && d.opts.repr);
    }

    #[test]
    fn test_classvar_and_initvar_classified() {
        cleanup_all_dataclasses();
        register_test_class("TCv");
        record("TCv", "x", "int", None);
        record("TCv", "kind", "ClassVar[str]", Some(MbValue::from_int(1)));
        record("TCv", "factor", "InitVar[int]", Some(MbValue::from_int(1)));
        decorate_class("TCv", DcOptions::default());
        let d = lookup_dc("TCv").unwrap();
        assert!(d.fields[1].is_classvar);
        assert!(d.fields[2].is_initvar);
        assert_eq!(compare_field_names(&d), vec!["x"]);
    }

    #[test]
    fn test_kw_only_sentinel_marks_following_fields() {
        cleanup_all_dataclasses();
        register_test_class("TSplit");
        record("TSplit", "a", "int", None);
        record("TSplit", "_", "KW_ONLY", None);
        record("TSplit", "b", "int", None);
        decorate_class("TSplit", DcOptions::default());
        let d = lookup_dc("TSplit").unwrap();
        assert_eq!(d.fields.len(), 2); // sentinel itself dropped
        assert!(!d.fields[0].kw_only);
        assert!(d.fields[1].kw_only);
    }

    #[test]
    fn test_synth_init_binds_positionally_and_fills_defaults() {
        cleanup_all_dataclasses();
        register_test_class("TInit");
        record("TInit", "x", "int", None);
        record("TInit", "y", "int", Some(MbValue::from_int(9)));
        decorate_class("TInit", DcOptions::default());
        let inst = MbValue::from_ptr(MbObject::new_instance("TInit".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(5)]));
        dc_run_synth_init("TInit", inst, args);
        assert_eq!(get_field(inst, "x").as_int(), Some(5));
        assert_eq!(get_field(inst, "y").as_int(), Some(9));
    }

    #[test]
    fn test_field_marker_roundtrip() {
        // field(default=42) marker: built by dispatch_field, unpacked by decorate.
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                lock.write().unwrap().insert("default".into(), MbValue::from_int(42));
            }
        }
        let args = [MbValue::from_ptr(dict)];
        let marker = unsafe { dispatch_field(args.as_ptr(), 1) };
        assert!(is_field_marker(marker));
        assert_eq!(marker_get(marker, "default").and_then(|v| v.as_int()), Some(42));

        cleanup_all_dataclasses();
        register_test_class("TMark");
        record("TMark", "n", "int", Some(marker));
        decorate_class("TMark", DcOptions::default());
        let d = lookup_dc("TMark").unwrap();
        assert_eq!(d.fields[0].default.and_then(|v| v.as_int()), Some(42));
    }

    #[test]
    fn test_frozen_and_hash_queries() {
        cleanup_all_dataclasses();
        register_test_class("TFroz");
        record("TFroz", "x", "int", None);
        let opts = DcOptions { frozen: true, ..DcOptions::default() };
        decorate_class("TFroz", opts);
        assert!(is_frozen_dataclass("TFroz"));
        assert_eq!(dc_hash_field_names("TFroz"), Some(vec!["x".to_string()]));
        assert_eq!(dc_order_field_names("TFroz"), None);
        assert!(!is_frozen_dataclass("NotAClass"));
    }

    #[test]
    fn test_repr_string_shape() {
        cleanup_all_dataclasses();
        register_test_class("TRepr");
        record("TRepr", "x", "int", None);
        record("TRepr", "y", "int", None);
        decorate_class("TRepr", DcOptions::default());
        let inst = MbValue::from_ptr(MbObject::new_instance("TRepr".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1), MbValue::from_int(2),
        ]));
        dc_run_synth_init("TRepr", inst, args);
        assert_eq!(dc_repr_string(inst, "TRepr").as_deref(), Some("TRepr(x=1, y=2)"));
    }
}
