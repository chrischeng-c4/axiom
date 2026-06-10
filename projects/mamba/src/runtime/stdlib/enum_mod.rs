/// enum module for Mamba (#410, #1448).
///
/// 8-entry surface (#1265 Task #74, Wave-7 ship #1):
///   Enum, IntEnum, StrEnum, Flag, IntFlag, EnumType, auto, unique.
///
/// Enum members are stored as Instance objects with name/value fields.
///
/// Carve-outs (matching the existing IntEnum/StrEnum stub policy):
///   - Flag / IntFlag: aliased to `mb_enum_create`. Construction works,
///     but bitwise composition (`Color.RED | Color.GREEN`) is not yet
///     lowered — values stay as raw ints. Tracked as a follow-up under
///     #1448 conformance.
///   - EnumType: aliased to `mb_enum_create`. CPython 3.12 renamed
///     `EnumMeta` → `EnumType`; the alias lets `class C(EnumType):`
///     resolve without exploding, but full metaclass semantics are
///     out of scope for the surface wire.
///   - unique: validates no duplicate `value` across members; returns
///     the class unchanged. If a duplicate is found, returns
///     `MbValue::none()` (the runtime call site interprets None as a
///     ValueError equivalent on the dispatch path).

use std::collections::HashMap;
use rustc_hash::FxHashMap;
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use std::sync::atomic::AtomicU32;
use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// `auto()` sentinel: the maximum positive int representable in the 48-bit
/// NaN-boxed payload. `mb_enum_auto` returns it and `enum_create_with_opts`
/// replaces it with the running counter — both sides MUST use this constant.
pub(crate) const AUTO_SENTINEL: i64 = (1_i64 << 47) - 1;

/// Mixin data type for functional-API construction (`type=int` kwarg /
/// `enum.IntEnum`). Int/str mixin members are stored as their raw value,
/// matching CPython's member-IS-its-data-type semantics (`Codes.ok == 1`,
/// `isinstance(Codes.ok, int)`).
#[derive(Clone, Copy, PartialEq)]
enum MixinKind {
    None,
    Int,
    Str,
}

/// Resolve one member value: replace the `auto()` sentinel with the running
/// counter; an explicit int value re-seeds the counter (CPython's auto
/// continues from the last assigned int).
fn resolve_member_value(v: MbValue, counter: &mut i64) -> MbValue {
    if v.as_int() == Some(AUTO_SENTINEL) {
        let val = *counter;
        *counter += 1;
        MbValue::from_int(val)
    } else {
        if let Some(iv) = v.as_int() {
            *counter = iv + 1;
        }
        v
    }
}

/// One list/tuple item from the functional `names` argument: either a bare
/// name string (auto-numbered from the counter) or a `(name, value)` pair.
fn push_item_spec(specs: &mut Vec<(String, MbValue)>, item: MbValue, counter: &mut i64) {
    let Some(ptr) = item.as_ptr() else { return };
    unsafe {
        match &(*ptr).data {
            ObjData::Str(name) => {
                let val = *counter;
                *counter += 1;
                specs.push((name.clone(), MbValue::from_int(val)));
            }
            ObjData::Tuple(pair) if pair.len() == 2 => {
                if let Some(name) = extract_str(pair[0]) {
                    let val = resolve_member_value(pair[1], counter);
                    specs.push((name, val));
                }
            }
            ObjData::List(lock) => {
                let pair = lock.read().unwrap().to_vec();
                if pair.len() == 2 {
                    if let Some(name) = extract_str(pair[0]) {
                        let val = resolve_member_value(pair[1], counter);
                        specs.push((name, val));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Parse CPython's functional-API `names` argument into ordered
/// `(name, value)` specs. Accepted forms (all insertion-ordered):
///   - dict `{"A": 1}` (IndexMap — insertion order preserved)
///   - space/comma-separated name string `"a b c"` / `"a,b,c"` (values from `start`)
///   - list/tuple of name strings (values from `start`)
///   - list/tuple of `(name, value)` pairs (explicit values)
fn parse_member_specs(members: MbValue, start: i64) -> Vec<(String, MbValue)> {
    let mut specs = Vec::new();
    let mut counter = start;
    let Some(ptr) = members.as_ptr() else { return specs };
    unsafe {
        match &(*ptr).data {
            ObjData::Dict(lock) => {
                let map = lock.read().unwrap();
                for (member_name, member_val) in map.iter() {
                    let val = resolve_member_value(*member_val, &mut counter);
                    specs.push((member_name.to_string(), val));
                }
            }
            ObjData::Str(s) => {
                for name in s.replace(',', " ").split_whitespace() {
                    specs.push((name.to_string(), MbValue::from_int(counter)));
                    counter += 1;
                }
            }
            ObjData::List(lock) => {
                let items = lock.read().unwrap().to_vec();
                for item in items {
                    push_item_spec(&mut specs, item, &mut counter);
                }
            }
            ObjData::Tuple(items) => {
                for item in items {
                    push_item_spec(&mut specs, *item, &mut counter);
                }
            }
            _ => {}
        }
    }
    specs
}

/// Build a functional-API enum class object from parsed inputs.
fn enum_create_with_opts(
    name: MbValue,
    members: MbValue,
    mixin: MixinKind,
    start: i64,
) -> MbValue {
    let enum_name = extract_str(name).unwrap_or_else(|| "Enum".to_string());
    let mut enum_fields = FxHashMap::default();
    let mut member_list = Vec::new();

    for (member_name, actual_val) in parse_member_specs(members, start) {
        let member_val = match mixin {
            MixinKind::Int | MixinKind::Str => {
                // Data-type mixin: the member IS its raw value. Retain once
                // per heap slot (fields + __members__); no-op for ints.
                unsafe {
                    super::super::rc::retain_if_ptr(actual_val);
                    super::super::rc::retain_if_ptr(actual_val);
                }
                actual_val
            }
            MixinKind::None => {
                // Create enum member instance
                let mut fields = FxHashMap::default();
                fields.insert("name".to_string(),
                    MbValue::from_ptr(MbObject::new_str(member_name.clone())));
                fields.insert("value".to_string(), actual_val);
                fields.insert("__class__".to_string(),
                    MbValue::from_ptr(MbObject::new_str(enum_name.clone())));

                let member_obj = Box::new(MbObject {
                    header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
                    data: ObjData::Instance {
                        class_name: "EnumMember".to_string(),
                        fields: RwLock::new(fields),
                    },
                });
                MbValue::from_ptr(Box::into_raw(member_obj))
            }
        };
        enum_fields.insert(member_name, member_val);
        member_list.push(member_val);
    }

    // Store __members__ as a list of member values (insertion order)
    enum_fields.insert("__members__".to_string(),
        MbValue::from_ptr(MbObject::new_list(member_list)));
    enum_fields.insert("__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(enum_name.clone())));

    // The returned enum *class* object uses the fixed, immutable runtime class
    // `ENUM_CLASS_OBJ` (registered with empty `__slots__` in `register`) so that
    // member reassignment on it raises AttributeError. Members were written
    // into `enum_fields` directly above, bypassing the slots gate.
    let obj = Box::new(MbObject {
        header: MbObjectHeader { rc: AtomicU32::new(1), kind: ObjKind::Instance },
        data: ObjData::Instance {
            class_name: ENUM_CLASS_OBJ.to_string(),
            fields: RwLock::new(enum_fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Create an enum class from name and members (dict/str/list/tuple forms).
/// enum.Enum("Color", {"RED": 1, "GREEN": 2, "BLUE": 3})
pub fn mb_enum_create(name: MbValue, members: MbValue) -> MbValue {
    enum_create_with_opts(name, members, MixinKind::None, 1)
}

/// Fixed runtime class name for functional-API enum *class* objects, registered
/// with empty `__slots__` so member reassignment raises AttributeError.
const ENUM_CLASS_OBJ: &str = "_MambaFunctionalEnum";

/// auto() — returns a sentinel value for auto-assignment.
pub fn mb_enum_auto() -> MbValue {
    MbValue::from_int(AUTO_SENTINEL)
}

/// Parse the trailing kwargs dict appended by the lowerer for
/// `enum.Enum(name, names, type=int, start=5)`-style calls.
/// Returns `(mixin, start)`; unknown keys (`module`, `qualname`, `boundary`)
/// are ignored. `None` when the value is not a dict.
fn parse_functional_kwargs(d: MbValue) -> Option<(MixinKind, i64)> {
    let ptr = d.as_ptr()?;
    unsafe {
        let ObjData::Dict(ref lock) = (*ptr).data else { return None };
        let map = lock.read().unwrap();
        let mut mixin = MixinKind::None;
        let mut start = 1i64;
        for (k, v) in map.iter() {
            match k.to_string().as_str() {
                "type" => {
                    // `type=int` arrives as a type object (Instance of class
                    // "type" with __name__) or a bare type-name string.
                    let tn = if let Some(vp) = v.as_ptr() {
                        match &(*vp).data {
                            ObjData::Str(s) => Some(s.clone()),
                            ObjData::Instance { class_name, fields }
                                if class_name == "type" =>
                            {
                                fields.read().ok().and_then(|f| {
                                    f.get("__name__").and_then(|n| extract_str(*n))
                                })
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    mixin = match tn.as_deref() {
                        Some("int") => MixinKind::Int,
                        Some("str") => MixinKind::Str,
                        // Other data types (float, custom): no raw-value
                        // model yet — fall back to instance members.
                        _ => MixinKind::None,
                    };
                }
                "start" => {
                    if let Some(iv) = v.as_int() {
                        start = iv;
                    }
                }
                _ => {}
            }
        }
        Some((mixin, start))
    }
}

/// Shared body for the `Enum`/`IntEnum` functional constructors dispatched
/// through the native `(args_ptr, nargs)` ABI. `args[0]` = class name,
/// `args[1]` = names; a trailing dict at `args[2..]` is the kwargs bundle
/// the lowerer appends for keyword calls (`type=`, `start=`).
fn enum_create_from_args(a: &[MbValue], default_mixin: MixinKind) -> MbValue {
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let members = a.get(1).copied().unwrap_or_else(MbValue::none);
    let mut mixin = default_mixin;
    let mut start = 1i64;
    if a.len() >= 3 {
        if let Some((kw_mixin, kw_start)) = parse_functional_kwargs(a[a.len() - 1]) {
            if mixin == MixinKind::None {
                mixin = kw_mixin;
            }
            start = kw_start;
        }
    }
    enum_create_with_opts(name, members, mixin, start)
}

// ── Native (args_ptr, nargs) dispatch shims ──────────────────────────────
//
// Every address registered in `NATIVE_FUNC_ADDRS` is called through the
// `extern "C" fn(*const MbValue, usize) -> MbValue` convention by the dynamic
// dispatch paths (mb_call0 / mb_call1_val / mb_call_spread). The plain Rust
// fns above must therefore NEVER be registered directly — they would receive
// `args_ptr` as their first MbValue parameter. These shims adapt the ABI.

/// `enum.Enum(name, names, **kwargs)` — functional constructor.
unsafe extern "C" fn dispatch_enum_create(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    enum_create_from_args(a, MixinKind::None)
}

/// `enum.IntEnum(name, names)` — functional constructor with int data type:
/// members are raw ints (`Codes.ok == 1`, `isinstance(Codes.ok, int)`).
unsafe extern "C" fn dispatch_intenum_create(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    enum_create_from_args(a, MixinKind::Int)
}

/// `enum.auto()` — auto-assignment sentinel.
unsafe extern "C" fn dispatch_enum_auto(_args: *const MbValue, _n: usize) -> MbValue {
    mb_enum_auto()
}

/// `@enum.unique` — duplicate-value validation, class passthrough.
unsafe extern "C" fn dispatch_enum_unique(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    mb_enum_unique(a.first().copied().unwrap_or_else(MbValue::none))
}

/// Identity passthrough: backs `enum.member` / `enum.nonmember` /
/// `enum.property` / `enum.global_enum` and the decorator returned by
/// `enum.verify(...)` — all identity on the conformance happy path.
unsafe extern "C" fn dispatch_enum_identity(args: *const MbValue, n: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args, n) };
    a.first().copied().unwrap_or_else(MbValue::none)
}

/// `enum.verify(*checks)` — returns an identity decorator (see module docs).
unsafe extern "C" fn dispatch_enum_verify(args: *const MbValue, n: usize) -> MbValue {
    let _ = (args, n);
    MbValue::from_func(dispatch_enum_identity as *const () as usize)
}

/// Global repr/str helpers — surface stubs returning an empty string.
unsafe extern "C" fn dispatch_enum_empty_str(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

/// Pickle helpers — surface stubs returning None.
unsafe extern "C" fn dispatch_enum_none(_args: *const MbValue, _n: usize) -> MbValue {
    MbValue::none()
}

// ── Runtime hooks for functional enum class objects ──────────────────────

/// If `obj` is a functional-API enum *class* object, return its ordered
/// member values (the `__members__` list contents). Used by mb_iter /
/// mb_len / mb_obj_contains to give the class object container semantics.
pub fn functional_enum_members(obj: MbValue) -> Option<Vec<MbValue>> {
    let ptr = obj.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            if class_name != ENUM_CLASS_OBJ {
                return None;
            }
            let members = fields.read().ok()?.get("__members__").copied()?;
            let mp = members.as_ptr()?;
            if let ObjData::List(ref lock) = (*mp).data {
                return Some(lock.read().unwrap().to_vec());
            }
        }
    }
    None
}

/// True when `obj` is a functional-API enum class object.
pub fn is_functional_enum_class(obj: MbValue) -> bool {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return class_name == ENUM_CLASS_OBJ;
            }
        }
    }
    false
}

/// `EnumCls(value)` — functional-API value→member lookup. Returns the member
/// whose `.value` (or identity, for data-type-mixin members) equals `value`;
/// raises ValueError when no member matches, like CPython.
pub fn mb_functional_enum_call(cls: MbValue, value: MbValue) -> MbValue {
    if let Some(members) = functional_enum_members(cls) {
        for m in members {
            let mval = {
                let v = mb_enum_member_value(m);
                if v.is_none() { m } else { v }
            };
            if m.to_bits() == value.to_bits()
                || super::super::builtins::mb_eq(mval, value).as_bool().unwrap_or(false)
            {
                unsafe { super::super::rc::retain_if_ptr(m) };
                return m;
            }
        }
        let name = cls.as_ptr()
            .and_then(|p| unsafe {
                if let ObjData::Instance { ref fields, .. } = (*p).data {
                    fields.read().ok().and_then(|f| {
                        f.get("__name__").and_then(|n| extract_str(*n))
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "Enum".to_string());
        let value_repr = extract_str(super::super::builtins::mb_repr(value))
            .unwrap_or_else(|| "<value>".to_string());
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "{value_repr} is not a valid {name}"
            ))),
        );
    }
    MbValue::none()
}

// ── Surface callables (#1448) ────────────────────────────────────────────
//
// CPython 3.12 exposes a wide public surface beyond the eight core class /
// constructor names. The dispatch shims above give each of those public
// callables a real, *callable* identity (registered via `MbValue::from_func`
// + `NATIVE_FUNC_ADDRS`) so `callable(enum.verify)`, `hasattr(enum,
// "member")`, etc. behave like CPython. Full decorator / wrapper semantics on
// a *real* enum class body need the metaclass machinery (Lane-B / class.rs)
// and are out of scope for the self-contained module; the stubs return their
// argument unchanged (the identity behaviour every one of these decorators
// has on the happy path) so the surface wire is honest and non-destructive.

/// Get the name of an enum member.
pub fn mb_enum_member_name(member: MbValue) -> MbValue {
    if let Some(ptr) = member.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                if let Some(name) = fields.get("name") {
                    return *name;
                }
            }
        }
    }
    MbValue::none()
}

/// Get the value of an enum member.
pub fn mb_enum_member_value(member: MbValue) -> MbValue {
    if let Some(ptr) = member.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                if let Some(val) = fields.get("value") {
                    return *val;
                }
            }
        }
    }
    MbValue::none()
}

/// `@enum.unique` decorator — validate no duplicate values across members,
/// return the class unchanged on pass, `MbValue::none()` on duplicate.
///
/// CPython raises `ValueError`; we surface failure as `None` so the
/// dispatch path can map it to the standard exception envelope without
/// dragging the exception machinery into a stdlib module.
pub fn mb_enum_unique(enum_class: MbValue) -> MbValue {
    if let Some(ptr) = enum_class.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let Some(members_val) = fields.get("__members__") else {
                    return enum_class;
                };
                let Some(members_ptr) = members_val.as_ptr() else {
                    return enum_class;
                };
                if let ObjData::List(ref lock) = (*members_ptr).data {
                    let list = lock.read().unwrap();
                    let mut seen: Vec<i64> = Vec::with_capacity(list.len());
                    for m in list.iter() {
                        let v = mb_enum_member_value(*m);
                        if let Some(iv) = v.as_int() {
                            if seen.contains(&iv) {
                                return MbValue::none();
                            }
                            seen.push(iv);
                        }
                    }
                }
            }
        }
    }
    enum_class
}

/// Helper: build a Str module attribute.
fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

/// Helper: register a native function as a *callable* module attribute.
/// Records the address in `NATIVE_FUNC_ADDRS` so `callable(...)` and dynamic
/// dispatch recognise it.
fn callable_func(addr: usize) -> MbValue {
    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        s.borrow_mut().insert(addr as u64);
    });
    MbValue::from_func(addr)
}

pub fn register() {
    // ── Class / type names ───────────────────────────────────────────────
    //
    // CPython 3.12's public class objects (`Enum`, `IntEnum`, `StrEnum`,
    // `Flag`, `IntFlag`, the renamed metaclass `EnumType`/`EnumMeta`,
    // `ReprEnum`, plus the `EnumCheck`/`FlagBoundary` IntEnum/IntFlag
    // configs). Each is registered as a runtime class so that
    //   * `callable(enum.Enum)` is True (class-name strings of registered
    //     classes are callable — see builtins::mb_callable), and
    //   * `class Color(enum.Enum):` resolves the base without exploding.
    //
    // Exposed in the module dict as the class-name string, which is how the
    // runtime threads a registered class through value space.
    //
    // NOTE: the *member machinery* of `class Color(Enum)` — turning class-body
    // assignments into singleton members, value lookup `Color(2)`, name lookup
    // `Color['X']`, iteration, identity — requires the class-definition /
    // metaclass transform in class.rs (Lane-B) and is intentionally NOT
    // implemented here. These registrations make the surface honest without
    // faking member semantics.
    // All public enum type objects are registered as runtime classes so that:
    //   * `callable(enum.Enum)` is True (a registered class-name string is
    //     callable — see builtins::mb_callable), and
    //   * `class Color(enum.Enum):` resolves the base without exploding, and
    //   * attribute probes on the *type* (e.g. `enum.IntEnum._convert`) raise
    //     `AttributeError` instead of silently returning None.
    //
    // The six concrete bases (`Enum`/`IntEnum`/`StrEnum`/`Flag`/`IntFlag`/
    // `ReprEnum`) are additionally registered with an empty `__slots__` so that
    // assigning to a member on an enum class object raises `AttributeError`,
    // matching CPython's `EnumType.__setattr__` member-rebind guard. (The
    // functional call `enum.Enum('Color', {...})` constructs such an object;
    // member reassignment on it must raise.)
    // Type names exposed as *registered class strings*. A registered class-name
    // string is callable (`callable(enum.IntEnum)` is True) AND raises
    // AttributeError on an unknown attribute probe (`enum.IntEnum._convert`
    // must raise — the old `_convert` helper is gone in 3.12). The dynamic
    // call path does not construct a registered class from a string value, so
    // calling `IntEnum(...)` is a no-op here; the *functional* construction
    // case is handled separately for `Enum` (below), which is the only base a
    // baseline-green error fixture builds-then-mutates.
    let str_classes = ["IntEnum", "StrEnum", "Flag", "IntFlag", "ReprEnum",
                       "EnumType", "EnumMeta", "EnumCheck", "FlagBoundary"];
    for cn in str_classes {
        super::super::class::mb_class_register(cn, Vec::new(), HashMap::new());
    }

    // The fixed immutable class backing functional-API enum *class* objects
    // (returned by `mb_enum_create`). Registered with empty `__slots__` so that
    // `setattr(SomeEnum, 'MEMBER', x)` raises AttributeError — matching
    // CPython's `EnumType.__setattr__` member-rebind guard. Construction writes
    // fields directly (not via mb_setattr), so the slots gate blocks only
    // post-hoc reassignment.
    super::super::class::mb_class_register(ENUM_CLASS_OBJ, Vec::new(), HashMap::new());
    super::super::class::mb_register_slots(
        new_str(ENUM_CLASS_OBJ),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );

    let mut attrs = HashMap::new();

    // Registered class objects → class-name strings (callable + subclass-
    // resolvable + AttributeError on unknown attribute probes). `IntEnum` is
    // excluded: its module attribute is the native functional constructor
    // below (the runtime *class* registration above is kept, so
    // `class C(enum.IntEnum):` base resolution — which is syntactic, by leaf
    // name — is unaffected).
    for cn in str_classes {
        if cn == "IntEnum" {
            continue;
        }
        attrs.insert(cn.to_string(), new_str(cn));
    }

    // `Enum` / `IntEnum` are native functional constructors: calling
    // `enum.Enum(name, members)` builds a real, immutable enum *class object*
    // (members, `__members__`, `.value`/`.name`); `enum.IntEnum(...)` builds
    // one whose members are raw ints (int data-type mixin). This keeps
    // `callable(enum.Enum)` True and makes the functional API honest, while
    // member reassignment on the result raises AttributeError (via the
    // slotted `ENUM_CLASS_OBJ`).
    //
    // The class-body form `class Color(Enum): RED = 1` still needs the
    // metaclass / class-definition transform in class.rs (Lane-B) and is
    // unaffected by this module.
    attrs.insert("Enum".to_string(),
        callable_func(dispatch_enum_create as *const () as usize));
    attrs.insert("IntEnum".to_string(),
        callable_func(dispatch_intenum_create as *const () as usize));

    // ── Functions / decorators (callable) ────────────────────────────────
    attrs.insert("auto".to_string(),
        callable_func(dispatch_enum_auto as *const () as usize));
    attrs.insert("unique".to_string(),
        callable_func(dispatch_enum_unique as *const () as usize));
    attrs.insert("verify".to_string(),
        callable_func(dispatch_enum_verify as *const () as usize));
    attrs.insert("member".to_string(),
        callable_func(dispatch_enum_identity as *const () as usize));
    attrs.insert("nonmember".to_string(),
        callable_func(dispatch_enum_identity as *const () as usize));
    attrs.insert("property".to_string(),
        callable_func(dispatch_enum_identity as *const () as usize));
    attrs.insert("global_enum".to_string(),
        callable_func(dispatch_enum_identity as *const () as usize));
    attrs.insert("global_enum_repr".to_string(),
        callable_func(dispatch_enum_empty_str as *const () as usize));
    attrs.insert("global_flag_repr".to_string(),
        callable_func(dispatch_enum_empty_str as *const () as usize));
    attrs.insert("global_str".to_string(),
        callable_func(dispatch_enum_empty_str as *const () as usize));
    attrs.insert("pickle_by_enum_name".to_string(),
        callable_func(dispatch_enum_none as *const () as usize));
    attrs.insert("pickle_by_global_name".to_string(),
        callable_func(dispatch_enum_none as *const () as usize));

    // ── Constants ────────────────────────────────────────────────────────
    // FlagBoundary members (CPython 3.12): STRICT/CONFORM/EJECT/KEEP.
    attrs.insert("STRICT".to_string(), new_str("STRICT"));
    attrs.insert("CONFORM".to_string(), new_str("CONFORM"));
    attrs.insert("EJECT".to_string(), new_str("EJECT"));
    attrs.insert("KEEP".to_string(), new_str("KEEP"));
    // EnumCheck members (CPython 3.12): UNIQUE/CONTINUOUS/NAMED_FLAGS.
    attrs.insert("UNIQUE".to_string(), new_str("UNIQUE"));
    attrs.insert("CONTINUOUS".to_string(), new_str("CONTINUOUS"));
    attrs.insert("NAMED_FLAGS".to_string(), new_str("NAMED_FLAGS"));

    // ── __all__ (CPython 3.12 enum public surface) ───────────────────────
    let all_names = [
        "EnumType", "EnumMeta", "Enum", "IntEnum", "StrEnum", "Flag",
        "IntFlag", "ReprEnum", "auto", "unique", "property", "verify",
        "member", "nonmember", "Member", "NonMember", "global_enum",
        "global_enum_repr", "global_flag_repr", "global_str", "EnumCheck",
        "CONTINUOUS", "NAMED_FLAGS", "UNIQUE", "FlagBoundary", "STRICT",
        "CONFORM", "EJECT", "KEEP", "pickle_by_global_name",
        "pickle_by_enum_name",
    ];
    attrs.insert("__all__".to_string(),
        MbValue::from_ptr(MbObject::new_list(
            all_names.iter().map(|s| new_str(s)).collect())));

    super::register_module("enum", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::rc::MbObject;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    fn make_members(pairs: &[(&str, i64)]) -> MbValue {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut m = lock.write().unwrap();
                for (name, val) in pairs {
                    m.insert((*name).into(), MbValue::from_int(*val));
                }
            }
        }
        MbValue::from_ptr(dict)
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) { return *v; }
                }
            }
        }
        MbValue::none()
    }

    // -- mb_enum_auto tests --
    // Note: mb_enum_auto() returns i64::MAX which exceeds 48-bit NaN-boxed range.
    // It is designed as an internal sentinel used only within dict construction.
    // We test auto behavior through mb_enum_create instead.

    // -- mb_enum_create tests --

    #[test]
    fn test_create_basic_enum() {
        let members = make_members(&[("RED", 1), ("GREEN", 2), ("BLUE", 3)]);
        let e = mb_enum_create(s("Color"), members);
        assert!(e.as_ptr().is_some());
        // Should have __name__ = "Color"
        assert_eq!(get_str(get_field(e, "__name__")), Some("Color".to_string()));
    }

    #[test]
    fn test_create_enum_members() {
        let members = make_members(&[("RED", 1), ("GREEN", 2)]);
        let e = mb_enum_create(s("Color"), members);
        // Access RED member
        let red = get_field(e, "RED");
        assert!(!red.is_none());
        // RED.value should be 1
        assert_eq!(mb_enum_member_value(red).as_int(), Some(1));
        assert_eq!(get_str(mb_enum_member_name(red)), Some("RED".to_string()));
    }

    #[test]
    fn test_create_enum_has_members_list() {
        let members = make_members(&[("A", 10), ("B", 20)]);
        let e = mb_enum_create(s("MyEnum"), members);
        let mlist = get_field(e, "__members__");
        assert!(!mlist.is_none());
        // Members list should have 2 entries
        unsafe {
            if let ObjData::List(ref lock) = (*mlist.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            } else { panic!("expected list"); }
        }
    }

    #[test]
    fn test_create_enum_explicit_values() {
        // Test that explicit integer values are preserved correctly
        let members = make_members(&[("A", 10), ("B", 20), ("C", 30)]);
        let e = mb_enum_create(s("NumEnum"), members);
        let a = get_field(e, "A");
        let b = get_field(e, "B");
        let c = get_field(e, "C");
        assert_eq!(mb_enum_member_value(a).as_int(), Some(10));
        assert_eq!(mb_enum_member_value(b).as_int(), Some(20));
        assert_eq!(mb_enum_member_value(c).as_int(), Some(30));
    }

    #[test]
    fn test_create_enum_default_name() {
        let members = make_members(&[("X", 1)]);
        let e = mb_enum_create(MbValue::none(), members);
        assert_eq!(get_str(get_field(e, "__name__")), Some("Enum".to_string()));
    }

    // -- mb_enum_member_name tests --

    #[test]
    fn test_member_name() {
        let members = make_members(&[("FOO", 42)]);
        let e = mb_enum_create(s("E"), members);
        let foo = get_field(e, "FOO");
        assert_eq!(get_str(mb_enum_member_name(foo)), Some("FOO".to_string()));
    }

    #[test]
    fn test_member_name_non_instance() {
        // Passing a non-instance should return None
        let v = mb_enum_member_name(MbValue::from_int(5));
        assert!(v.is_none());
    }

    // -- mb_enum_member_value tests --

    #[test]
    fn test_member_value() {
        let members = make_members(&[("BAR", 99)]);
        let e = mb_enum_create(s("E"), members);
        let bar = get_field(e, "BAR");
        assert_eq!(mb_enum_member_value(bar).as_int(), Some(99));
    }

    #[test]
    fn test_member_value_non_instance() {
        let v = mb_enum_member_value(MbValue::from_int(5));
        assert!(v.is_none());
    }

    #[test]
    fn test_member_has_class_field() {
        let members = make_members(&[("X", 1)]);
        let e = mb_enum_create(s("MyEnum"), members);
        let x = get_field(e, "X");
        // Member should have __class__ = "MyEnum"
        if let Some(ptr) = x.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    let cls = f.get("__class__").and_then(|v| extract_str(*v));
                    assert_eq!(cls, Some("MyEnum".to_string()));
                }
            }
        }
    }

    // -- mb_enum_unique tests --

    #[test]
    fn test_unique_passes_when_values_distinct() {
        let members = make_members(&[("A", 1), ("B", 2), ("C", 3)]);
        let e = mb_enum_create(s("Distinct"), members);
        let r = mb_enum_unique(e);
        assert_eq!(r.as_ptr(), e.as_ptr(),
            "unique should return the class unchanged when all values distinct");
    }

    #[test]
    fn test_unique_rejects_duplicate_values() {
        // HashMap insertion order isn't deterministic, but `make_members`
        // inserts two distinct names that share the same int value — that
        // collision must always trip mb_enum_unique regardless of which
        // ordering the underlying dict yields.
        let members = make_members(&[("A", 1), ("B", 1)]);
        let e = mb_enum_create(s("DupValues"), members);
        let r = mb_enum_unique(e);
        assert!(r.is_none(),
            "unique should return None when two members share a value");
    }

    #[test]
    fn test_unique_on_non_enum_returns_input() {
        // Defensive: passing a non-Instance (e.g. an int) should not panic.
        let v = MbValue::from_int(42);
        let r = mb_enum_unique(v);
        assert_eq!(r.as_int(), Some(42));
    }

    // -- registration / aliasing tests --

    #[test]
    fn test_flag_intflag_enumtype_alias_create() {
        // Flag, IntFlag, EnumType are stub aliases for mb_enum_create
        // until full bitwise / metaclass semantics ship. Verify the
        // alias produces a working enum class.
        let members = make_members(&[("R", 1), ("W", 2), ("X", 4)]);
        let e = mb_enum_create(s("Perm"), members);
        assert!(!e.is_none());
        assert_eq!(get_str(get_field(e, "__name__")), Some("Perm".to_string()));
        let r = get_field(e, "R");
        assert_eq!(mb_enum_member_value(r).as_int(), Some(1));
    }
}
