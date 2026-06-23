//! Class-body enum member machinery (Lane-B of #1448).
//!
//! `class Color(enum.Enum): RED = 1` — the runtime side of CPython's
//! `EnumType.__new__` member transform. The lowering emits one
//! `mb_class_set_class_attr` call per class-body assignment (at the class's
//! textual position); `maybe_convert_class_attr` intercepts those calls for
//! classes whose MRO names an enum base and converts each eligible value into
//! a singleton member Instance (fields `name`/`_name_`/`value`/`_value_`,
//! `class_name` = the enum class) registered in a per-class member table.
//!
//! Hook surface (every entry point is gated on the `HAVE_ENUM_CLASSES` flag
//! plus a registry lookup so non-enum programs pay one `Cell` read at most):
//!   * iteration / len / contains on the class-name string,
//!   * `Color(value)` value→member lookup (+ `_missing_` hook),
//!   * `Color["NAME"]` name lookup,
//!   * `Color.__members__` (aliases included),
//!   * member equality (singleton identity; never equal to raw values),
//!   * member str/repr (`Color.RED` / `<Color.RED: 1>`),
//!   * Flag `|`/`&`/`^` composite members + composite containment,
//!   * `auto()` sentinel consumption (sequential for Enum, powers of two
//!     for Flag).
//!
//! Data-type mixins (`IntEnum`, `IntFlag`, `StrEnum`, `ReprEnum`, or an
//! explicit `int`/`str` base) are intentionally NOT converted: their members
//! stay raw values (member-IS-its-data-type), matching the pre-existing
//! behavior the IntEnum conformance fixtures rely on. The functional API
//! (`enum.Enum('Color', {...})` in enum_mod.rs) shares the same member field
//! shape (`name`/`value`) but keeps its own `_MambaFunctionalEnum` container
//! representation.

use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
use std::cell::{Cell, RefCell};

/// Enum flavor that determines auto() progression, iteration canonicality,
/// bitwise-composition support, and raw-value equality.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnumKind {
    Plain,
    Flag,
    /// `enum.IntFlag` or `class Bits(int, enum.Flag)`: Flag semantics plus
    /// the int data-type mixin — members DO equal their raw int values and
    /// are `isinstance(_, int)`.
    IntFlag,
    /// `enum.StrEnum`: members equal their raw str values, are
    /// `isinstance(_, str)`, and `str(member)` IS the value.
    StrEnum,
    /// `class Direction(str, enum.Enum)`: like StrEnum for equality and
    /// isinstance, but `str(member)` stays the qualified "Class.NAME"
    /// (CPython 3.12 distinction).
    StrMixin,
}

impl EnumKind {
    fn is_flag(self) -> bool {
        matches!(self, EnumKind::Flag | EnumKind::IntFlag)
    }
    fn is_str_mixin(self) -> bool {
        matches!(self, EnumKind::StrEnum | EnumKind::StrMixin)
    }
}

/// Bitwise ops bridged from mb_bitor / mb_bitand / mb_bitxor.
#[derive(Clone, Copy)]
pub enum FlagOp {
    Or,
    And,
    Xor,
}

struct EnumClassInfo {
    kind: EnumKind,
    /// Next auto() value (Enum: last int + 1; Flag: next power of two).
    next_auto: i64,
    /// Canonical members in definition order (aliases and, for Flag,
    /// multi-bit members excluded) — the iteration order surface.
    canonical: Vec<MbValue>,
    /// Every bound name (canonical + aliases) in definition order — the
    /// `__members__` mapping surface.
    by_name: Vec<(String, MbValue)>,
    /// Flag composite cache: composite int value → cached member Instance,
    /// so `Color.RED | Color.BLUE` returns the same singleton each time.
    composites: FxHashMap<i64, MbValue>,
}

thread_local! {
    /// class name → member machinery for converted class-body enums.
    static ENUM_CLASSES: RefCell<FxHashMap<String, EnumClassInfo>> =
        RefCell::new(FxHashMap::default());
    /// class name → detection result memo (None = not a convertible enum).
    static ENUM_KIND_MEMO: RefCell<FxHashMap<String, Option<EnumKind>>> =
        RefCell::new(FxHashMap::default());
    /// Cheap global gate: false until the first enum class registers a
    /// member. Keeps every hook on generic hot paths (string iteration,
    /// equality, instance creation) at a single Cell read for programs
    /// that never define a class-body enum.
    static HAVE_ENUM_CLASSES: Cell<bool> = const { Cell::new(false) };
}

#[inline]
fn have_enum_classes() -> bool {
    HAVE_ENUM_CLASSES.with(|c| c.get())
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

/// Detect (memoized) whether `class_name`'s registered MRO names a
/// convertible enum base. Data-type mixins keep raw-value members → `None`.
fn enum_kind_for(class_name: &str) -> Option<EnumKind> {
    let memo = ENUM_KIND_MEMO.with(|m| m.borrow().get(class_name).copied());
    if let Some(k) = memo {
        return k;
    }
    let mro = super::super::class::class_mro_list(class_name);
    if mro.is_empty() {
        // Class not registered (yet) — don't memoize a negative.
        return None;
    }
    let mut saw_flag = false;
    let mut saw_enum = false;
    let mut saw_int_flag = false;
    let mut saw_int = false;
    let mut saw_str_enum = false;
    let mut saw_str = false;
    let mut rejected = false;
    for ancestor in mro.iter().skip(1) {
        match ancestor.as_str() {
            // Data-type mixins / metaclass bases whose members stay raw
            // values (member-IS-its-data-type, e.g. IntEnum arithmetic).
            "IntEnum" | "ReprEnum" | "EnumType" | "EnumMeta" | "EnumCheck" | "FlagBoundary" => {
                rejected = true;
            }
            "IntFlag" => saw_int_flag = true,
            "int" => saw_int = true,
            "StrEnum" => saw_str_enum = true,
            "str" => saw_str = true,
            "Flag" => saw_flag = true,
            "Enum" => saw_enum = true,
            _ => {}
        }
    }
    let kind = if rejected {
        None
    } else if saw_int_flag || (saw_flag && saw_int) {
        Some(EnumKind::IntFlag)
    } else if saw_str_enum {
        Some(EnumKind::StrEnum)
    } else if saw_str && saw_enum && !saw_flag {
        Some(EnumKind::StrMixin)
    } else if saw_int || saw_str {
        // Other data-type mixins (IntEnum-like int+Enum, str+Flag): members
        // keep the pre-existing raw-value behavior.
        None
    } else if saw_flag {
        Some(EnumKind::Flag)
    } else if saw_enum {
        Some(EnumKind::Plain)
    } else {
        None
    };
    ENUM_KIND_MEMO.with(|m| {
        m.borrow_mut().insert(class_name.to_string(), kind);
    });
    kind
}

/// Reserved class-body names that never become members: dunders
/// (`__slots__`) and sunders (`_ignore_`, `_missing_`, `_order_`).
fn is_reserved_name(name: &str) -> bool {
    name.starts_with('_') && name.ends_with('_')
}

/// Values that are method-like / descriptors and therefore not members
/// (e.g. `__str__ = Loud.__str__`, `classmethod`/`staticmethod`/`property`
/// wrappers). NOTE: closure handles are int-tagged and indistinguishable
/// from plain int values, so lambdas assigned in an enum body DO convert —
/// a known, narrow divergence that keeps `RED = 1` safe.
fn is_method_like(v: MbValue) -> bool {
    if v.as_func().is_some() {
        return true;
    }
    if let Some(p) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                return matches!(
                    class_name.as_str(),
                    "__property__"
                        | "__classmethod__"
                        | "__staticmethod__"
                        | "__cached_property__"
                        | "__unbound_method__"
                );
            }
        }
    }
    false
}

/// Member's stored raw `value` field (borrowed copy; fields keep it alive).
fn member_value(m: MbValue) -> MbValue {
    if let Some(p) = m.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                if let Some(v) = fields.read().unwrap().get("value") {
                    return *v;
                }
            }
        }
    }
    MbValue::none()
}

/// `(class_name, member_name)` when `v` is a converted enum member.
fn member_class_and_name(v: MbValue) -> Option<(String, String)> {
    if !have_enum_classes() {
        return None;
    }
    let p = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*p).data
        {
            let registered = ENUM_CLASSES.with(|m| m.borrow().contains_key(class_name.as_str()));
            if registered {
                let name = fields
                    .read()
                    .unwrap()
                    .get("_name_")
                    .copied()
                    .and_then(extract_str)
                    .unwrap_or_default();
                return Some((class_name.clone(), name));
            }
        }
    }
    None
}

/// `(class_name, int value)` when `v` is a converted member with an int value.
fn member_class_and_int(v: MbValue) -> Option<(String, i64)> {
    let (cls, _) = member_class_and_name(v)?;
    let value = member_value(v).as_int()?;
    Some((cls, value))
}

fn class_kind(class_name: &str) -> Option<EnumKind> {
    ENUM_CLASSES.with(|m| m.borrow().get(class_name).map(|i| i.kind))
}

/// Build a fresh member Instance (rc=1, fields populated directly so the
/// `__slots__` gate never applies).
fn new_member(class_name: &str, member_name: &str, value: MbValue) -> MbValue {
    let inst_ptr = MbObject::new_instance(class_name.to_string());
    let inst = MbValue::from_ptr(inst_ptr);
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
            let mut f = fields.write().unwrap();
            f.insert(
                "name".to_string(),
                MbValue::from_ptr(MbObject::new_str(member_name.to_string())),
            );
            f.insert(
                "_name_".to_string(),
                MbValue::from_ptr(MbObject::new_str(member_name.to_string())),
            );
            // One retain per field slot — the caller keeps its own reference
            // to the original value.
            super::super::rc::retain_if_ptr(value);
            f.insert("value".to_string(), value);
            super::super::rc::retain_if_ptr(value);
            f.insert("_value_".to_string(), value);
        }
    }
    inst
}

/// mb_class_set_class_attr interception: when `class_name` is a convertible
/// class-body enum and `attr`/`value` are member-shaped, return the singleton
/// member Instance to store in `class_attrs` instead of the raw value.
/// Returns `None` to store the raw value unchanged (non-enum class, reserved
/// name, or method-like value).
pub fn maybe_convert_class_attr(class_name: &str, attr: &str, value: MbValue) -> Option<MbValue> {
    let kind = enum_kind_for(class_name)?;
    if is_reserved_name(attr) || is_method_like(value) {
        return None;
    }

    ENUM_CLASSES.with(|m| {
        let mut map = m.borrow_mut();
        let info = map.entry(class_name.to_string()).or_insert_with(|| {
            HAVE_ENUM_CLASSES.with(|c| c.set(true));
            EnumClassInfo {
                kind,
                next_auto: 1,
                canonical: Vec::new(),
                by_name: Vec::new(),
                composites: FxHashMap::default(),
            }
        });

        // auto() sentinel consumption / counter reseed on explicit ints.
        let resolved = if value.as_int() == Some(super::enum_mod::AUTO_SENTINEL) {
            let v = info.next_auto;
            info.next_auto = if kind.is_flag() {
                if v > 0 {
                    v << 1
                } else {
                    1
                }
            } else {
                v + 1
            };
            MbValue::from_int(v)
        } else {
            if let Some(iv) = value.as_int() {
                info.next_auto = if kind.is_flag() {
                    if iv > 0 {
                        1i64 << (64 - iv.leading_zeros() as i64)
                    } else {
                        1
                    }
                } else {
                    iv + 1
                };
            }
            value
        };

        // Alias: a value equal to an existing member's binds the new name to
        // the FIRST member with that value.
        let existing = info.by_name.iter().find_map(|(_, m)| {
            let mv = member_value(*m);
            if super::super::builtins::mb_eq(mv, resolved).as_bool() == Some(true) {
                Some(*m)
            } else {
                None
            }
        });
        if let Some(m) = existing {
            unsafe { super::super::rc::retain_if_ptr(m) }; // by_name slot
            info.by_name.push((attr.to_string(), m));
            unsafe { super::super::rc::retain_if_ptr(m) }; // returned reference
            return Some(m);
        }

        let member = new_member(class_name, attr, resolved);
        // Flag canonicality: only single-bit int values iterate; multi-bit
        // named members (B = 3) stay reachable by name/value but are
        // excluded from `list(FlagCls)` (CPython 3.12).
        let canonical = if kind.is_flag() {
            resolved
                .as_int()
                .map(|v| v.count_ones() == 1)
                .unwrap_or(true)
        } else {
            true
        };
        if canonical {
            unsafe { super::super::rc::retain_if_ptr(member) };
            info.canonical.push(member);
        }
        unsafe { super::super::rc::retain_if_ptr(member) };
        info.by_name.push((attr.to_string(), member));
        Some(member)
    })
}

/// True when `name` is a converted class-body enum class.
pub fn is_enum_class(name: &str) -> bool {
    have_enum_classes() && ENUM_CLASSES.with(|m| m.borrow().contains_key(name))
}

/// Kind of `v`'s enum class when `v` is a converted member.
fn member_kind(v: MbValue) -> Option<EnumKind> {
    if !have_enum_classes() {
        return None;
    }
    if let Some(p) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                return class_kind(class_name);
            }
        }
    }
    None
}

/// Equality override for converted enum members (`None` → caller falls
/// through to the generic rules). Members are singletons:
///   * member vs member — pointer identity (aliases/composites are cached),
///   * Plain/Flag member vs raw value — never equal (`Suit.CLUBS != 1`),
///   * IntFlag member vs raw number / StrEnum member vs raw str — data-type
///     mixin value equality (`Perm.READ == 1`, `Label.ONE == "1"`).
pub fn members_eq_override(a: MbValue, b: MbValue) -> Option<bool> {
    let ka = member_kind(a);
    let kb = member_kind(b);
    if ka.is_none() && kb.is_none() {
        return None;
    }
    if ka.is_some() && kb.is_some() {
        return Some(a.to_bits() == b.to_bits());
    }
    let (kind, member, raw) = if let Some(k) = ka {
        (k, a, b)
    } else {
        (kb.unwrap(), b, a)
    };
    match kind {
        EnumKind::Plain | EnumKind::Flag => Some(false),
        EnumKind::IntFlag => {
            let mv = member_value(member);
            Some(super::super::builtins::mb_eq(mv, raw).as_bool() == Some(true))
        }
        EnumKind::StrEnum | EnumKind::StrMixin => {
            let mv = member_value(member);
            match (extract_str(mv), extract_str(raw)) {
                (Some(ms), Some(rs)) => Some(ms == rs),
                _ => Some(false),
            }
        }
    }
}

/// Raw str value of a str-mixin member, for delegating str methods
/// (`Direction.EAST.upper()` dispatches against the value).
pub fn str_mixin_member_value(v: MbValue) -> Option<MbValue> {
    if member_kind(v).is_some_and(EnumKind::is_str_mixin) {
        let mv = member_value(v);
        if mv.as_ptr().is_some() {
            return Some(mv);
        }
    }
    None
}

/// `isinstance(member, int)` / `isinstance(member, str)` for data-type
/// mixin members (IntFlag → int, StrEnum/(str, Enum) → str).
pub fn member_isinstance_builtin(v: MbValue, target: &str) -> bool {
    match member_kind(v) {
        Some(EnumKind::IntFlag) => target == "int",
        Some(EnumKind::StrEnum) | Some(EnumKind::StrMixin) => target == "str",
        _ => false,
    }
}

/// True when `v` is a singleton member of a converted class-body enum.
/// Used by mb_values_eq: members never equal raw values or members of other
/// classes (identity was already handled by the bits fast path).
/// True when `v` is a member of a *plain* enum (`Enum` or non-int `Flag`),
/// which — unlike IntEnum/IntFlag/StrEnum — does NOT support ordering
/// comparisons; CPython raises `TypeError: '<' not supported between instances`
/// for them. The int/str data-mixin kinds compare via their raw value instead.
pub fn member_is_plain_unorderable(v: MbValue) -> bool {
    matches!(member_kind(v), Some(EnumKind::Plain) | Some(EnumKind::Flag))
}

pub fn is_enum_member(v: MbValue) -> bool {
    if !have_enum_classes() {
        return false;
    }
    if let Some(p) = v.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                return ENUM_CLASSES.with(|m| m.borrow().contains_key(class_name.as_str()));
            }
        }
    }
    false
}

/// Canonical members (definition order) for iteration: `list(Color)`.
pub fn class_canonical_members(class_name: &str) -> Option<Vec<MbValue>> {
    if !have_enum_classes() {
        return None;
    }
    ENUM_CLASSES.with(|m| m.borrow().get(class_name).map(|i| i.canonical.clone()))
}

/// `len(Color)` — canonical member count.
pub fn class_member_count(class_name: &str) -> Option<i64> {
    if !have_enum_classes() {
        return None;
    }
    ENUM_CLASSES.with(|m| m.borrow().get(class_name).map(|i| i.canonical.len() as i64))
}

/// `Color.__members__` — name→member mapping including aliases, in
/// definition order (mamba dicts are insertion-ordered).
pub fn members_map_dict(class_name: &str) -> Option<MbValue> {
    if !have_enum_classes() {
        return None;
    }
    let entries: Vec<(String, MbValue)> =
        ENUM_CLASSES.with(|m| m.borrow().get(class_name).map(|i| i.by_name.clone()))?;
    let dict = super::super::dict_ops::mb_dict_new();
    for (name, member) in entries {
        let key = MbValue::from_ptr(MbObject::new_str(name));
        // mb_dict_setitem takes its own retain on the member.
        super::super::dict_ops::mb_dict_setitem(dict, key, member);
    }
    Some(dict)
}

/// Value→member lookup over named members (canonical first by construction)
/// then Flag composites. Returns a borrowed reference (registry-owned).
fn lookup_by_value(class_name: &str, value: MbValue) -> Option<MbValue> {
    ENUM_CLASSES.with(|m| {
        let map = m.borrow();
        let info = map.get(class_name)?;
        for (_, member) in &info.by_name {
            if member.to_bits() == value.to_bits() {
                return Some(*member);
            }
            let mv = member_value(*member);
            if super::super::builtins::mb_eq(mv, value).as_bool() == Some(true) {
                return Some(*member);
            }
        }
        if let Some(iv) = value.as_int() {
            if let Some(c) = info.composites.get(&iv) {
                return Some(*c);
            }
        }
        None
    })
}

/// Name→member lookup including aliases (`Color["BLUE"]`).
fn lookup_by_name(class_name: &str, name: &str) -> Option<MbValue> {
    ENUM_CLASSES.with(|m| {
        let map = m.borrow();
        let info = map.get(class_name)?;
        info.by_name
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, member)| *member)
    })
}

/// `Color(value)` — value→member lookup with the `_missing_` classmethod
/// hook. Returns `None` when `class_name` is not a converted enum (caller
/// falls through to ordinary instance creation); otherwise returns the
/// member (retained) or raises ValueError/TypeError like CPython.
pub fn enum_class_call(class_name: &str, args_list: MbValue) -> Option<MbValue> {
    if !is_enum_class(class_name) {
        return None;
    }
    // Single positional lookup argument only; anything else falls through.
    let arg = {
        let p = args_list.as_ptr()?;
        unsafe {
            if let ObjData::List(ref lock) = (*p).data {
                let items = lock.read().unwrap();
                if items.len() != 1 {
                    return None;
                }
                items[0]
            } else {
                return None;
            }
        }
    };

    if let Some(member) = lookup_by_value(class_name, arg) {
        unsafe { super::super::rc::retain_if_ptr(member) };
        return Some(member);
    }

    // _missing_ hook: classmethod `_missing_(cls, value)` may map an
    // alternate key to a member; returning None keeps the ValueError.
    let missing = super::super::class::lookup_method(class_name, "_missing_");
    if !missing.is_none() {
        let addr = super::super::class::registered_callable_addr(missing);
        if addr != 0 {
            let cls_val = MbValue::from_ptr(MbObject::new_str(class_name.to_string()));
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            let result = func(cls_val, arg);
            if !result.is_none() {
                if is_enum_member(result) {
                    return Some(result);
                }
                super::super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "error in {class_name}._missing_: returned a non-member"
                    ))),
                );
                return Some(MbValue::none());
            }
        }
    }

    let value_repr = repr_string(arg);
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "{value_repr} is not a valid {class_name}"
        ))),
    );
    Some(MbValue::none())
}

/// `Color["BLUE"]` — name→member lookup. `None` when not an enum class;
/// raises KeyError on a missing name.
pub fn enum_class_getitem(class_name: &str, key: MbValue) -> Option<MbValue> {
    if !is_enum_class(class_name) {
        return None;
    }
    let name = extract_str(key)?;
    if let Some(member) = lookup_by_name(class_name, &name) {
        unsafe { super::super::rc::retain_if_ptr(member) };
        return Some(member);
    }
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!("'{name}'"))),
    );
    Some(MbValue::none())
}

/// `member in Color` / `value in Color` (CPython 3.12 value-contains).
pub fn class_contains(class_name: &str, item: MbValue) -> Option<bool> {
    if !is_enum_class(class_name) {
        return None;
    }
    if let Some(p) = item.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                class_name: ref icn,
                ..
            } = (*p).data
            {
                if icn == class_name {
                    return Some(true);
                }
            }
        }
    }
    Some(lookup_by_value(class_name, item).is_some())
}

/// Flag containment: `Color.RED in (Color.RED | Color.BLUE)` — bit test on
/// two members of the same Flag class.
pub fn flag_member_contains(container: MbValue, item: MbValue) -> Option<bool> {
    if !have_enum_classes() {
        return None;
    }
    let (cc, cv) = member_class_and_int(container)?;
    if !class_kind(&cc).is_some_and(EnumKind::is_flag) {
        return None;
    }
    let (ic, iv) = member_class_and_int(item)?;
    if cc != ic {
        return None;
    }
    Some(iv != 0 && (cv & iv) == iv)
}

/// Resolve a Flag int value to its singleton member: named member first,
/// then the composite cache, else build + cache a composite Instance whose
/// name is the bit-decomposition over canonical members ("RED|BLUE").
fn resolve_flag_value(class_name: &str, value: i64) -> MbValue {
    if let Some(m) = lookup_by_value(class_name, MbValue::from_int(value)) {
        unsafe { super::super::rc::retain_if_ptr(m) };
        return m;
    }
    let name = ENUM_CLASSES.with(|m| {
        let map = m.borrow();
        map.get(class_name)
            .map(|info| {
                let parts: Vec<String> = info
                    .canonical
                    .iter()
                    .filter_map(|c| {
                        let cv = member_value(*c).as_int()?;
                        if cv != 0 && (value & cv) == cv {
                            member_class_and_name(*c).map(|(_, n)| n)
                        } else {
                            None
                        }
                    })
                    .collect();
                if parts.is_empty() {
                    value.to_string()
                } else {
                    parts.join("|")
                }
            })
            .unwrap_or_else(|| value.to_string())
    });
    let composite = new_member(class_name, &name, MbValue::from_int(value));
    ENUM_CLASSES.with(|m| {
        if let Some(info) = m.borrow_mut().get_mut(class_name) {
            unsafe { super::super::rc::retain_if_ptr(composite) }; // cache slot
            info.composites.insert(value, composite);
        }
    });
    composite
}

/// Flag bitwise composition. Both operands members of the SAME Flag class,
/// or — for int-mixin flags (IntFlag) — one member plus a bare int
/// (`Bits.ONE | 2` returns a Bits member, CPython). `None` → caller falls
/// through to the ordinary int/set paths.
pub fn flag_binop(a: MbValue, b: MbValue, op: FlagOp) -> Option<MbValue> {
    if !have_enum_classes() {
        return None;
    }
    let (cls, va, vb) = match (member_class_and_int(a), member_class_and_int(b)) {
        (Some((ca, va)), Some((cb, vb))) => {
            if ca != cb {
                return None;
            }
            (ca, va, vb)
        }
        (Some((ca, va)), None) => {
            // int-mixin flags compose with bare ints; pure Flags do not.
            if class_kind(&ca) != Some(EnumKind::IntFlag) {
                return None;
            }
            (ca, va, b.as_int()?)
        }
        (None, Some((cb, vb))) => {
            if class_kind(&cb) != Some(EnumKind::IntFlag) {
                return None;
            }
            (cb, a.as_int()?, vb)
        }
        (None, None) => return None,
    };
    if !class_kind(&cls).is_some_and(EnumKind::is_flag) {
        return None;
    }
    let v = match op {
        FlagOp::Or => va | vb,
        FlagOp::And => va & vb,
        FlagOp::Xor => va ^ vb,
    };
    Some(resolve_flag_value(&cls, v))
}

/// Empty Flag members (value 0, e.g. `RED & BLUE`) are falsy; everything
/// else keeps the default-truthy instance behavior. Plain Enum members are
/// always truthy (CPython). Called AFTER user `__bool__` dispatch.
pub fn flag_member_is_empty(v: MbValue) -> bool {
    if !have_enum_classes() {
        return false;
    }
    match member_class_and_int(v) {
        Some((cls, value)) => class_kind(&cls).is_some_and(EnumKind::is_flag) && value == 0,
        None => false,
    }
}

fn repr_string(v: MbValue) -> String {
    let r = super::super::builtins::mb_repr(v);
    let s = extract_str(r).unwrap_or_else(|| "<value>".to_string());
    unsafe { super::super::rc::release_if_ptr(r) };
    s
}

/// `str(member)` → "Color.RED" (or the raw value for StrEnum) unless the
/// class defines its own __str__ (then `None` lets the normal dunder
/// dispatch run).
pub fn member_str(v: MbValue) -> Option<String> {
    let (cls, name) = member_class_and_name(v)?;
    if !super::super::class::lookup_method(&cls, "__str__").is_none() {
        return None;
    }
    // StrEnum: str(member) IS the member's value ("1", not "Label.ONE");
    // a plain (str, Enum) mixin keeps the qualified form (CPython 3.12).
    if class_kind(&cls) == Some(EnumKind::StrEnum) {
        if let Some(s) = extract_str(member_value(v)) {
            return Some(s);
        }
    }
    Some(format!("{cls}.{name}"))
}

/// `repr(member)` → "<Color.RED: 1>" unless the class defines its own
/// __repr__.
pub fn member_repr(v: MbValue) -> Option<String> {
    let (cls, name) = member_class_and_name(v)?;
    if !super::super::class::lookup_method(&cls, "__repr__").is_none() {
        return None;
    }
    let vr = repr_string(member_value(v));
    Some(format!("<{cls}.{name}: {vr}>"))
}

/// First alias found: (alias_name, canonical_name) — `@enum.unique` support.
/// An alias is a by_name entry whose member pointer already appeared under an
/// earlier name.
pub fn class_first_alias(class_name: &str) -> Option<(String, String)> {
    if !have_enum_classes() {
        return None;
    }
    ENUM_CLASSES.with(|m| {
        let map = m.borrow();
        let info = map.get(class_name)?;
        let mut seen: Vec<(u64, MbValue, &str)> = Vec::new();
        for (name, member) in &info.by_name {
            let bits = member.to_bits();
            let mv = member_value(*member);
            if let Some((_, _, first)) = seen.iter().find(|(b, sv, _)| {
                // Same member object, or distinct members carrying equal
                // values (the class-body translation can mint separate
                // member objects for an alias).
                *b == bits || super::super::builtins::mb_eq(*sv, mv).as_bool() == Some(true)
            }) {
                return Some((name.clone(), (*first).to_string()));
            }
            seen.push((bits, mv, name));
        }
        None
    })
}

/// Canonical member (name, int value) pairs — `@enum.verify` support.
pub fn class_member_int_values(class_name: &str) -> Option<Vec<(String, i64)>> {
    if !have_enum_classes() {
        return None;
    }
    ENUM_CLASSES.with(|m| {
        let map = m.borrow();
        let info = map.get(class_name)?;
        let mut out = Vec::new();
        let canonical_bits: Vec<u64> = info.canonical.iter().map(|v| v.to_bits()).collect();
        for (name, member) in &info.by_name {
            if !canonical_bits.contains(&member.to_bits()) {
                continue;
            }
            let v = member_value(*member);
            if let Some(i) = v.as_int() {
                out.push((name.clone(), i));
            }
        }
        Some(out)
    })
}

/// class_first_alias fallback for data-mixin enums (IntEnum et al.) that
/// keep raw values as class attrs instead of ENUM_CLASSES members.
pub fn attrs_first_alias(class_name: &str) -> Option<(String, String)> {
    let entries = super::super::class::class_attr_entries(class_name);
    let mut seen: Vec<(MbValue, String)> = Vec::new();
    for (name, value) in entries {
        if name.starts_with("__") {
            continue;
        }
        if value.as_int().is_none()
            && value
                .as_ptr()
                .map(|p| unsafe { !matches!((*p).data, super::super::rc::ObjData::Str(_)) })
                .unwrap_or(true)
        {
            continue; // only int/str member values participate
        }
        if let Some((_, first)) = seen
            .iter()
            .find(|(sv, _)| super::super::builtins::mb_eq(*sv, value).as_bool() == Some(true))
        {
            return Some((name, first.clone()));
        }
        seen.push((value, name));
    }
    None
}

/// class_member_int_values fallback over class attrs (data-mixin enums).
pub fn attrs_member_int_values(class_name: &str) -> Vec<(String, i64)> {
    super::super::class::class_attr_entries(class_name)
        .into_iter()
        .filter(|(n, _)| !n.starts_with("__"))
        .filter_map(|(n, v)| v.as_int().map(|i| (n, i)))
        .collect()
}
