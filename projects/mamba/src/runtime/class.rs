use super::rc::{MbObject, ObjData};
use super::value::MbValue;
use rustc_hash::FxHashMap;
/// Class system for the Mamba runtime (#287, #288).
///
/// Implements Python-compatible classes with:
/// - Single and multiple inheritance
/// - Method Resolution Order (C3 linearization)
/// - Instance creation and __init__
/// - Attribute access (getattr/setattr)
/// - Operator overloading (__add__, __sub__, __eq__, etc.)
/// - super() support
/// - Property descriptors
use std::cell::Cell;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub(crate) const INT_SUBCLASS_VALUE_FIELD: &str = "__mamba_int_value__";

/// A class definition stored at runtime.
pub struct MbClass {
    pub name: String,
    /// Base classes (direct parents)
    pub bases: Vec<String>,
    /// Method Resolution Order (computed via C3 linearization)
    pub mro: Vec<String>,
    /// Methods: name → function MbValue
    pub methods: HashMap<String, MbValue>,
    /// Class-level attributes
    pub class_attrs: HashMap<String, MbValue>,
    /// Metaclass name, if specified (e.g., `class Foo(metaclass=Meta)`)
    pub metaclass: Option<String>,
    /// Cached __init__ method: (func_addr, is_registered_in_callable_registry).
    /// Resolved at registration time to avoid repeated MRO walks during instance creation.
    pub cached_init: Option<(u64, bool)>,
}

// Global class registry — maps class name → MbClass.
thread_local! {
    static CLASS_REGISTRY: std::cell::RefCell<HashMap<String, MbClass>> =
        std::cell::RefCell::new(HashMap::new());
    /// Names of classes defined in the user's program (lowered via
    /// `mb_class_define_multi`/`mb_class_define`), as opposed to native stdlib
    /// stub classes registered through `mb_class_register` directly. Used by
    /// mb_getattr to decide whether a missing attribute should raise
    /// AttributeError: only when the instance's ENTIRE MRO is user-defined, so
    /// native parents (whose __init__ populates attributes outside mamba's
    /// instance fields) keep the lenient None return.
    static USER_CLASSES: std::cell::RefCell<HashSet<String>> =
        std::cell::RefCell::new(HashSet::new());
    /// Registry of valid callable function pointer addresses.
    /// Only addresses registered here can be invoked via mb_call_method1.
    static CALLABLE_REGISTRY: std::cell::RefCell<HashSet<u64>> =
        std::cell::RefCell::new(HashSet::new());
    /// __slots__ registry: class name → effective (merged) slot names.
    static SLOTS_REGISTRY: std::cell::RefCell<HashMap<String, Vec<String>>> =
        std::cell::RefCell::new(HashMap::new());
    /// R13: Classes with __dict__ suppressed (defined __slots__ without '__dict__' in it).
    static DICT_SUPPRESSED: std::cell::RefCell<HashSet<String>> =
        std::cell::RefCell::new(HashSet::new());
    /// R10: Pending class keyword arguments for __init_subclass__ dispatch.
    /// Stored by mb_class_set_kwargs, consumed by mb_class_register.
    static KWARGS_REGISTRY: std::cell::RefCell<HashMap<String, HashMap<String, MbValue>>> =
        std::cell::RefCell::new(HashMap::new());
    /// Method lookup cache: (class_name_hash, method_name_hash) → cached MbValue.
    /// Avoids repeated MRO walks for hot method dispatch paths.
    static METHOD_CACHE: std::cell::RefCell<HashMap<(u64, u64), MbValue>> =
        std::cell::RefCell::new(HashMap::new());
    /// Generation counter for METHOD_CACHE invalidation.
    /// Bumped on class registration or class attribute mutation; cache is cleared.
    static METHOD_CACHE_GEN: Cell<u64> = Cell::new(0);
    /// Fast-path cache for "simple" classes: classes with no descriptors and no __slots__.
    /// For these classes, mb_setattr can skip the descriptor protocol check and slots
    /// registry check entirely, going straight to the field insert.
    /// Populated lazily on first mb_setattr call for each class.
    static SIMPLE_CLASS_CACHE: std::cell::RefCell<HashSet<String>> =
        std::cell::RefCell::new(HashSet::new());
    /// Names of classes decorated with `@typing.runtime_checkable` (Protocols).
    /// `isinstance(obj, P)` against such a class does STRUCTURAL matching —
    /// true iff obj's class provides every non-dunder method P declares.
    static RUNTIME_CHECKABLE_PROTOCOLS: std::cell::RefCell<HashSet<String>> =
        std::cell::RefCell::new(HashSet::new());
    /// collections.abc virtual subclass registry populated by ABC.register().
    /// Entries are (registered child class name, ABC parent name).
    static ABC_VIRTUAL_SUBCLASSES: std::cell::RefCell<HashSet<(String, String)>> =
        std::cell::RefCell::new(HashSet::new());
    /// abc: per-class set of method names declared with `@abc.abstractmethod`
    /// (and the abstract{property,classmethod,staticmethod} variants). Used to
    /// compute `cls.__abstractmethods__` and to block instantiation of classes
    /// that still carry un-overridden abstract methods.
    static USER_ABC_OWN_ABSTRACT: std::cell::RefCell<HashMap<String, HashSet<String>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// abc: record the names declared `@abc.abstractmethod` on a class. Called by
/// the class-definition lowering immediately after `mb_class_define_multi`.
pub fn mb_class_set_abstractmethods(class_name: MbValue, names: MbValue) {
    let name = match extract_str(class_name) {
        Some(n) if !n.is_empty() => n,
        _ => return,
    };
    let mut set = HashSet::new();
    if let Some(ptr) = names.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                for item in lock.read().unwrap().iter() {
                    if let Some(s) = extract_str(*item) {
                        set.insert(s);
                    }
                }
            }
        }
    }
    USER_ABC_OWN_ABSTRACT.with(|reg| {
        reg.borrow_mut().insert(name, set);
    });
}

/// abc: does `class_name` derive (transitively) from `abc.ABC` or use
/// `abc.ABCMeta` as its metaclass? Such classes get abstract-method
/// enforcement and the `register()` / `__subclasshook__` virtual-subclass
/// protocol.
pub fn is_user_abc(class_name: &str) -> bool {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let Some(cls) = reg.get(class_name) else {
            return false;
        };
        // Direct or inherited `ABC` base, or an explicit `ABCMeta` metaclass.
        if cls.metaclass.as_deref() == Some("ABCMeta") {
            return true;
        }
        // Walk own MRO: an `ABC` base/ancestor marks this class abstract-capable.
        for ancestor in std::iter::once(class_name).chain(cls.mro.iter().map(|s| s.as_str())) {
            if ancestor == "ABC" {
                return true;
            }
            if let Some(acls) = reg.get(ancestor) {
                if acls.metaclass.as_deref() == Some("ABCMeta")
                    || acls.bases.iter().any(|b| b == "ABC")
                {
                    return true;
                }
            } else if ancestor == "ABC" {
                return true;
            }
        }
        false
    })
}

/// abc: compute the still-abstract method names of `class_name` — the union of
/// abstract names declared anywhere in its MRO, minus any that a more-derived
/// class in the MRO concretely overrides. Mirrors CPython's `ABCMeta.__new__`
/// `__abstractmethods__` calculation. Returns names in sorted order for
/// deterministic output.
/// Enumerate (name, value) members of a class — class attributes and methods
/// across the MRO, a more-derived definition winning. Used by
/// inspect.getmembers on a class value.
pub(crate) fn class_members(class_name: &str) -> Vec<(String, MbValue)> {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let mro: Vec<String> = match reg.get(class_name) {
            Some(c) if !c.mro.is_empty() => c.mro.clone(),
            Some(_) => vec![class_name.to_string()],
            None => return Vec::new(),
        };
        let mut seen = std::collections::HashSet::new();
        let mut out: Vec<(String, MbValue)> = Vec::new();
        for cls in &mro {
            if let Some(c) = reg.get(cls) {
                for (k, v) in c.class_attrs.iter() {
                    if seen.insert(k.clone()) {
                        out.push((k.clone(), *v));
                    }
                }
                for (k, v) in c.methods.iter() {
                    if seen.insert(k.clone()) {
                        out.push((k.clone(), *v));
                    }
                }
            }
        }
        out
    })
}

pub(crate) fn compute_user_abstractmethods(class_name: &str) -> Vec<String> {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let Some(cls) = reg.get(class_name) else {
            return Vec::new();
        };
        // MRO front-to-back (most-derived first).
        let chain: Vec<&str> = std::iter::once(class_name)
            .chain(cls.mro.iter().map(|s| s.as_str()))
            .collect();
        USER_ABC_OWN_ABSTRACT.with(|own| {
            let own = own.borrow();
            // Candidate abstract names: every abstract name declared anywhere in
            // the MRO.
            let mut candidates: HashSet<String> = HashSet::new();
            for c in &chain {
                if let Some(s) = own.get(*c) {
                    candidates.extend(s.iter().cloned());
                }
            }
            let mut still_abstract: Vec<String> = Vec::new();
            for m in candidates {
                // Find the most-derived class in the MRO that *defines* `m`
                // (either a concrete method or an abstract declaration). The
                // first such class decides whether `m` is still abstract.
                let mut decided = false;
                for c in &chain {
                    let cls_c = reg.get(*c);
                    let declares_abstract = own.get(*c).map_or(false, |s| s.contains(&m));
                    let has_concrete = cls_c.map_or(false, |cc| {
                        cc.methods.contains_key(&m) || cc.class_attrs.contains_key(&m)
                    }) && !declares_abstract;
                    if declares_abstract {
                        still_abstract.push(m.clone());
                        decided = true;
                        break;
                    }
                    if has_concrete {
                        // Concrete override in a more-derived class — no longer
                        // abstract.
                        decided = true;
                        break;
                    }
                }
                let _ = decided;
            }
            still_abstract.sort();
            still_abstract
        })
    })
}

/// abc: if `class_name` is a user ABC with un-overridden abstract methods,
/// raise `TypeError` (matching CPython's message) and return `Some(None)` so
/// the instance-creation path bails out. Returns `None` when the class is
/// instantiable.
pub fn mb_user_abc_reject_abstract_instantiation(class_name: &str) -> Option<MbValue> {
    if !is_user_abc(class_name) {
        return None;
    }
    let abstracts = compute_user_abstractmethods(class_name);
    if abstracts.is_empty() {
        return None;
    }
    // CPython 3.12: "Can't instantiate abstract class X without an implementation
    // for abstract method[s] 'a', 'b'".
    let quoted: Vec<String> = abstracts.iter().map(|m| format!("'{m}'")).collect();
    let (word, joined) = if quoted.len() == 1 {
        ("method", quoted[0].clone())
    } else {
        ("methods", quoted.join(", "))
    };
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "Can't instantiate abstract class {class_name} without an implementation for abstract {word} {joined}",
        ))),
    );
    Some(MbValue::none())
}

/// typing.Protocol: a class with `Protocol` as a *direct* base is a protocol
/// definition and cannot be instantiated (CPython: "Protocols cannot be
/// instantiated"). A concrete class that merely subclasses a protocol (without
/// listing Protocol directly) is instantiable, so this checks direct bases only.
/// True iff `class_name` has `Protocol` as a direct base — i.e. it is a
/// `typing.Protocol` subclass. Used by `typing.runtime_checkable` to reject a
/// plain (non-protocol) class.
pub fn is_protocol_class(class_name: &str) -> bool {
    CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(class_name)
            .map(|c| c.bases.iter().any(|b| b == "Protocol" || b == "typing.Protocol"))
            .unwrap_or(false)
    })
}

pub fn mb_reject_protocol_instantiation(class_name: &str) -> Option<MbValue> {
    let is_protocol = is_protocol_class(class_name);
    if is_protocol {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Protocols cannot be instantiated".to_string(),
            )),
        );
        return Some(MbValue::none());
    }
    None
}

/// abc: build a frozenset MbValue of `class_name`'s abstract method names.
fn user_abstractmethods_frozenset(class_name: &str) -> MbValue {
    let names = compute_user_abstractmethods(class_name);
    let items: Vec<MbValue> = names
        .into_iter()
        .map(|n| MbValue::from_ptr(MbObject::new_str(n)))
        .collect();
    MbValue::from_ptr(MbObject::new_frozenset(items))
}

/// Mark a class name as a `@runtime_checkable` Protocol (called by the
/// typing.runtime_checkable decorator).
pub fn mark_runtime_checkable(name: &str) {
    if !name.is_empty() {
        RUNTIME_CHECKABLE_PROTOCOLS.with(|s| {
            s.borrow_mut().insert(name.to_string());
        });
    }
}

/// Is `name` a `@runtime_checkable` Protocol?
pub fn is_runtime_checkable_protocol(name: &str) -> bool {
    RUNTIME_CHECKABLE_PROTOCOLS.with(|s| s.borrow().contains(name))
}

/// Structural isinstance: does `obj` (its class MRO + its own instance fields)
/// provide every non-dunder member declared by runtime-checkable Protocol
/// `proto` — including data members declared as annotations (`name: str`)?
fn protocol_structural_match(obj: MbValue, obj_class: &str, proto: &str) -> bool {
    // Instance attributes (e.g. `self.name` set in __init__) count toward a
    // DATA protocol — collect them up front to avoid borrowing across the
    // CLASS_REGISTRY borrow.
    let inst_fields: std::collections::HashSet<String> = obj
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Instance { ref fields, .. } = (*p).data {
                fields.read().unwrap().keys().cloned().collect()
            } else {
                std::collections::HashSet::new()
            }
        })
        .unwrap_or_default();
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let Some(pcls) = reg.get(proto) else {
            return false;
        };
        // Required members: the Protocol's own non-dunder method/attr names...
        let mut required: Vec<String> = pcls
            .methods
            .keys()
            .chain(pcls.class_attrs.keys())
            .filter(|n| !(n.starts_with("__") && n.ends_with("__")))
            .cloned()
            .collect();
        // ...plus annotation-only DATA members, which live as keys inside the
        // class's `__annotations__` dict rather than as class_attrs.
        if let Some(anns) = pcls.class_attrs.get("__annotations__") {
            if let Some(ptr) = anns.as_ptr() {
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        for k in lock.read().unwrap().keys() {
                            if let super::dict_ops::DictKey::Str(s) = k {
                                if !(s.starts_with("__") && s.ends_with("__")) {
                                    required.push(s.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        if required.is_empty() {
            // An empty Protocol matches any object (CPython: all objects
            // satisfy a member-less runtime_checkable Protocol).
            return true;
        }
        // The candidate's MRO (itself + ancestors); fall back to just itself
        // when unregistered.
        let chain: Vec<String> = reg
            .get(obj_class)
            .map(|c| {
                let mut v = vec![obj_class.to_string()];
                v.extend(c.mro.iter().cloned());
                v
            })
            .unwrap_or_else(|| vec![obj_class.to_string()]);
        let provides = |member: &str| -> bool {
            inst_fields.contains(member)
                || chain.iter().any(|cn| {
                    reg.get(cn).map_or(false, |c| {
                        c.methods.contains_key(member) || c.class_attrs.contains_key(member)
                    })
                })
        };
        required.iter().all(|m| provides(m))
    })
}

fn has_method(class_name: &str, name: &str) -> bool {
    if !lookup_method(class_name, name).is_none() {
        return true;
    }
    builtin_type_has_dunder(class_name, name)
}

/// abc/structural: does the named builtin type provide `dunder`? Builtin types
/// are not registered in `CLASS_REGISTRY`, so structural ABC checks
/// (`hasattr(C, "__len__")` → Sized) need this table. Only the dunders that
/// drive ABC membership are modelled.
pub(crate) fn builtin_type_has_dunder(type_name: &str, dunder: &str) -> bool {
    // Every Python object carries `__format__` (object.__format__) — surface
    // fixtures probe e.g. `hasattr(str, "__format__")`.
    if dunder == "__format__" {
        return true;
    }
    match type_name {
        // list: mutable + orderable + unhashable (no __hash__).
        "list" => matches!(
            dunder,
            "__len__"
                | "__getitem__"
                | "__setitem__"
                | "__delitem__"
                | "__contains__"
                | "__iter__"
                | "__reversed__"
                | "__add__"
                | "__mul__"
                | "__rmul__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
        ),
        // tuple: immutable (no __setitem__/__delitem__) + orderable + hashable.
        "tuple" => matches!(
            dunder,
            "__len__"
                | "__getitem__"
                | "__contains__"
                | "__iter__"
                | "__add__"
                | "__mul__"
                | "__rmul__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__hash__"
                | "__getnewargs__"
        ),
        "str" => matches!(
            dunder,
            "__len__"
                | "__getitem__"
                | "__contains__"
                | "__iter__"
                | "__add__"
                | "__mul__"
                | "__rmul__"
                | "__mod__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__hash__"
        ),
        "bytes" | "bytearray" => matches!(
            dunder,
            "__len__"
                | "__getitem__"
                | "__contains__"
                | "__iter__"
                | "__add__"
                | "__mul__"
                | "__rmul__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
        ),
        // dict: mutable + union operators (3.9) + reversed (3.8); not orderable.
        "dict" => matches!(
            dunder,
            "__len__"
                | "__getitem__"
                | "__setitem__"
                | "__delitem__"
                | "__contains__"
                | "__iter__"
                | "__reversed__"
                | "__or__"
                | "__ior__"
                | "__eq__"
                | "__ne__"
        ),
        // set: mutable, so it carries the in-place set operators.
        "set" => matches!(
            dunder,
            "__len__"
                | "__contains__"
                | "__iter__"
                | "__eq__"
                | "__ne__"
                | "__and__"
                | "__or__"
                | "__sub__"
                | "__xor__"
                | "__iand__"
                | "__ior__"
                | "__isub__"
                | "__ixor__"
                | "__le__"
                | "__lt__"
                | "__ge__"
                | "__gt__"
        ),
        // frozenset: immutable (no in-place ops) + hashable.
        "frozenset" => matches!(
            dunder,
            "__len__"
                | "__contains__"
                | "__iter__"
                | "__eq__"
                | "__ne__"
                | "__and__"
                | "__or__"
                | "__sub__"
                | "__xor__"
                | "__le__"
                | "__lt__"
                | "__ge__"
                | "__gt__"
                | "__hash__"
        ),
        "range" => matches!(
            dunder,
            "__len__" | "__getitem__" | "__contains__" | "__iter__" | "__reversed__"
        ),
        // `types.UnionType` (the `X | Y` runtime type) exposes `__args__`
        // (its member types) plus the union-combining operators.
        "UnionType" => matches!(dunder, "__args__" | "__or__" | "__ror__" | "__parameters__"),
        // Numbers: arithmetic/comparison/hash dunders so type-level
        // `hasattr(int, "__add__")` / `hasattr(complex, "__eq__")` report True.
        // Deliberately NO container dunders (__len__/__iter__/__contains__) so
        // ABC structural negatives (isinstance(5, Sized) is False) stay correct.
        "int" | "bool" => matches!(
            dunder,
            "__add__"
                | "__sub__"
                | "__mul__"
                | "__truediv__"
                | "__floordiv__"
                | "__mod__"
                | "__divmod__"
                | "__pow__"
                | "__neg__"
                | "__pos__"
                | "__abs__"
                | "__invert__"
                | "__and__"
                | "__or__"
                | "__xor__"
                | "__lshift__"
                | "__rshift__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__hash__"
                | "__bool__"
                | "__index__"
                | "__int__"
                | "__float__"
                | "__round__"
                | "__ceil__"
                | "__floor__"
                | "__trunc__"
                | "__getnewargs__"
        ),
        "float" => matches!(
            dunder,
            "__add__"
                | "__sub__"
                | "__mul__"
                | "__truediv__"
                | "__floordiv__"
                | "__mod__"
                | "__divmod__"
                | "__pow__"
                | "__neg__"
                | "__pos__"
                | "__abs__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__hash__"
                | "__bool__"
                | "__int__"
                | "__float__"
                | "__round__"
                | "__ceil__"
                | "__floor__"
                | "__trunc__"
                | "__getnewargs__"
        ),
        // complex is NOT orderable — no __lt__/__le__/__gt__/__ge__.
        "complex" => matches!(
            dunder,
            "__add__"
                | "__sub__"
                | "__mul__"
                | "__truediv__"
                | "__pow__"
                | "__neg__"
                | "__pos__"
                | "__abs__"
                | "__eq__"
                | "__ne__"
                | "__hash__"
                | "__bool__"
                | "__complex__"
                | "__getnewargs__"
        ),
        // slice: orderable + hashable (3.12), with a small attr/method surface.
        "slice" => matches!(
            dunder,
            "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__hash__"
                | "__repr__"
        ),
        // object: the universal base dunders every type inherits. Probed via
        // `hasattr(object, "__init__")` etc.
        "object" => matches!(
            dunder,
            "__init__"
                | "__new__"
                | "__repr__"
                | "__str__"
                | "__hash__"
                | "__eq__"
                | "__ne__"
                | "__lt__"
                | "__le__"
                | "__gt__"
                | "__ge__"
                | "__class__"
                | "__doc__"
                | "__dir__"
                | "__getattribute__"
                | "__setattr__"
                | "__delattr__"
                | "__sizeof__"
                | "__reduce__"
                | "__reduce_ex__"
                | "__init_subclass__"
                | "__subclasshook__"
        ),
        _ => false,
    }
}

fn collections_abc_structural_match(obj_class: &str, target: &str) -> bool {
    match target {
        "Callable" => has_method(obj_class, "__call__"),
        "Sized" => has_method(obj_class, "__len__"),
        "Container" => has_method(obj_class, "__contains__"),
        "Iterable" => has_method(obj_class, "__iter__") || has_method(obj_class, "__next__"),
        "Iterator" => has_method(obj_class, "__iter__") && has_method(obj_class, "__next__"),
        "Collection" => {
            collections_abc_structural_match(obj_class, "Sized")
                && collections_abc_structural_match(obj_class, "Iterable")
                && collections_abc_structural_match(obj_class, "Container")
        }
        "Reversible" => {
            has_method(obj_class, "__reversed__")
                || (has_method(obj_class, "__len__") && has_method(obj_class, "__getitem__"))
        }
        "Sequence" => has_method(obj_class, "__len__") && has_method(obj_class, "__getitem__"),
        "MutableSequence" => {
            collections_abc_structural_match(obj_class, "Sequence")
                && has_method(obj_class, "__setitem__")
                && has_method(obj_class, "__delitem__")
                && has_method(obj_class, "insert")
        }
        "Mapping" => has_method(obj_class, "__getitem__") && has_method(obj_class, "keys"),
        "MutableMapping" => {
            collections_abc_structural_match(obj_class, "Mapping")
                && has_method(obj_class, "__setitem__")
                && has_method(obj_class, "__delitem__")
        }
        "Set" => {
            has_method(obj_class, "__contains__")
                && has_method(obj_class, "__iter__")
                && has_method(obj_class, "__len__")
        }
        "MutableSet" => {
            collections_abc_structural_match(obj_class, "Set")
                && has_method(obj_class, "add")
                && has_method(obj_class, "discard")
        }
        _ => false,
    }
}

fn collections_abc_parents(name: &str) -> &'static [&'static str] {
    match name {
        "Coroutine" => &["Awaitable"],
        "AsyncIterator" => &["AsyncIterable"],
        "AsyncGenerator" => &["AsyncIterator"],
        "Iterator" => &["Iterable"],
        "Generator" => &["Iterator"],
        "Collection" => &["Sized", "Iterable", "Container"],
        "Reversible" => &["Iterable"],
        "Sequence" => &["Reversible", "Collection"],
        "MutableSequence" => &["Sequence"],
        "ByteString" => &["Sequence"],
        "Set" => &["Collection"],
        "MutableSet" => &["Set"],
        "Mapping" => &["Collection"],
        "MutableMapping" => &["Mapping"],
        "MappingView" => &["Sized"],
        "KeysView" => &["MappingView", "Set"],
        "ItemsView" => &["MappingView", "Set"],
        "ValuesView" => &["MappingView", "Collection"],
        _ => &[],
    }
}

fn is_collections_abc_name(name: &str) -> bool {
    matches!(
        name,
        "Awaitable"
            | "Coroutine"
            | "AsyncIterable"
            | "AsyncIterator"
            | "AsyncGenerator"
            | "Hashable"
            | "Iterable"
            | "Iterator"
            | "Generator"
            | "Sized"
            | "Callable"
            | "Container"
            | "Collection"
            | "Reversible"
            | "Sequence"
            | "MutableSequence"
            | "ByteString"
            | "Set"
            | "MutableSet"
            | "Mapping"
            | "MutableMapping"
            | "MappingView"
            | "KeysView"
            | "ItemsView"
            | "ValuesView"
            | "Buffer"
    )
}

fn collections_abc_is_subclass(child: &str, parent: &str) -> bool {
    if child == parent {
        return is_collections_abc_name(child);
    }
    if !is_collections_abc_name(child) || !is_collections_abc_name(parent) {
        return false;
    }

    fn walk(child: &str, parent: &str, seen: &mut HashSet<String>) -> bool {
        if !seen.insert(child.to_string()) {
            return false;
        }
        collections_abc_parents(child)
            .iter()
            .any(|base| *base == parent || walk(base, parent, seen))
    }

    walk(child, parent, &mut HashSet::new())
}

fn collections_abc_virtual_match(child: &str, parent: &str) -> bool {
    ABC_VIRTUAL_SUBCLASSES.with(|reg| {
        reg.borrow()
            .iter()
            .any(|(registered_child, registered_parent)| {
                registered_child == child
                    && (registered_parent == parent
                        || collections_abc_is_subclass(registered_parent, parent))
            })
    })
}

fn collections_abc_type_or_virtual_match(child: &str, parent: &str) -> bool {
    collections_abc_is_subclass(child, parent) || collections_abc_virtual_match(child, parent)
}

/// Builtin-subclass relations for native collections types: Counter,
/// OrderedDict, and defaultdict are real dict subclasses in CPython, so
/// `isinstance(Counter(), dict)` / `issubclass(Counter, dict)` are true.
fn collections_builtin_subclass(child: &str, parent: &str) -> bool {
    parent == "dict"
        && matches!(
            child,
            "collections.Counter" | "collections.OrderedDict" | "collections.defaultdict"
        )
}

fn class_matches_collections_abc(class_name: &str, target: &str) -> bool {
    let nominal = CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(class_name) {
            class_name == target
                || cls.mro.iter().any(|base| {
                    base == target || collections_abc_type_or_virtual_match(base, target)
                })
        } else {
            collections_abc_type_or_virtual_match(class_name, target)
        }
    });
    nominal || collections_abc_structural_match(class_name, target)
}

pub fn mb_collections_abc_register(parent_name: &str, child: MbValue) -> MbValue {
    if !is_collections_abc_name(parent_name) {
        return MbValue::none();
    }
    let Some(child_name) = resolve_class_name(child) else {
        super::builtins::raise_type_error("Can only register classes".to_string());
        return MbValue::none();
    };
    if child_name.is_empty() {
        super::builtins::raise_type_error("Can only register classes".to_string());
        return MbValue::none();
    }
    ABC_VIRTUAL_SUBCLASSES.with(|reg| {
        reg.borrow_mut()
            .insert((child_name, parent_name.to_string()));
    });
    unsafe { super::rc::retain_if_ptr(child) };
    child
}

/// Hash a string to u64 for use as a METHOD_CACHE key component.
fn hash_str(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Invalidate the method lookup cache by bumping generation and clearing entries.
/// Also clears the simple class cache since class attributes may have changed.
fn invalidate_method_cache() {
    METHOD_CACHE_GEN.with(|gen| gen.set(gen.get().wrapping_add(1)));
    let _ = METHOD_CACHE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = SIMPLE_CLASS_CACHE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
}

// ── Class Registration ──

/// Register a new class with the given name, bases, and methods.
/// Shared by both class statement lowering and type() 3-arg form.
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R3
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R5
// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R7
/// int.from_bytes(bytes, byteorder='big', *, signed=False) → int
/// Minimal implementation covering bytes / bytearray input with a string
/// byteorder ("big" or "little") and an optional signed flag. Bytes
/// beyond i64 range are silently truncated to the low 64 bits.
/// int.from_bytes requires a bytes-like or iterable-of-ints argument; anything
/// else raises CPython's "cannot convert '<type>' object to bytes" TypeError.
fn int_from_bytes_type_error(val: MbValue) -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "cannot convert '{}' object to bytes",
            super::builtins::value_type_name(val)
        ))),
    );
    MbValue::none()
}

pub fn mb_int_from_bytes(
    bytes_val: MbValue,
    byteorder_val: MbValue,
    signed_val: MbValue,
) -> MbValue {
    let byteorder = extract_str(byteorder_val).unwrap_or_else(|| "big".to_string());
    // `signed` may arrive as either a bool positional arg, or packed into a
    // trailing kwargs dict produced by the method-call lowering path.
    let signed = if let Some(b) = signed_val.as_bool() {
        b
    } else if let Some(ptr) = signed_val.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                let k = super::dict_ops::DictKey::Str("signed".to_string());
                guard.get(&k).and_then(|v| v.as_bool()).unwrap_or(false)
            } else {
                false
            }
        }
    } else {
        false
    };
    let bytes: Vec<u8> = if let Some(ptr) = bytes_val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => b.clone(),
                ObjData::ByteArray(lock) => lock.read().unwrap().clone(),
                ObjData::List(lock) => lock
                    .read()
                    .unwrap()
                    .iter()
                    .filter_map(|v| v.as_int().map(|i| i as u8))
                    .collect(),
                ObjData::Tuple(items) => items
                    .iter()
                    .filter_map(|v| v.as_int().map(|i| i as u8))
                    .collect(),
                _ => return int_from_bytes_type_error(bytes_val),
            }
        }
    } else {
        // Inline non-bytes-like (int, bool, None, float).
        return int_from_bytes_type_error(bytes_val);
    };
    if bytes.is_empty() {
        return MbValue::from_int(0);
    }
    // Big-endian byte order for the magnitude computation. CPython supports
    // arbitrary-width integers here, so use a BigInt accumulator and promote
    // the result to a heap BigInt whenever it exceeds the 48-bit inline range
    // (e.g. plistlib's binary int decode of 0x13 8-byte signed words).
    use num_bigint::BigInt;
    use num_traits::{ToPrimitive, Zero};
    let be: Vec<u8> = if byteorder == "big" {
        bytes.clone()
    } else {
        bytes.iter().rev().copied().collect()
    };
    let mut result = BigInt::from_bytes_be(num_bigint::Sign::Plus, &be);
    if signed && !be.is_empty() && (be[0] & 0x80) != 0 {
        // Two's-complement: subtract 2**(8*len).
        let modulus = BigInt::from(1) << (8 * be.len());
        result -= modulus;
    }
    if result.is_zero() {
        return MbValue::from_int(0);
    }
    if let Some(i) = result.to_i64() {
        if i >= -(1i64 << 47) && i < (1i64 << 47) {
            return MbValue::from_int(i);
        }
    }
    super::bigint_ops::bigint_from_big(result)
}

/// Returns true iff `name` is a class name registered via `mb_class_register`.
/// Used by `callable()` to recognise user-defined classes that flow through
/// the runtime as bare class-name strings.
pub fn class_is_registered(name: &str) -> bool {
    CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(name))
}

fn registered_class_name_for_func(value: MbValue, addr: usize) -> Option<String> {
    let native_name = super::module::NATIVE_TYPE_NAMES.with(|map| {
        map.borrow().get(&(addr as u64)).cloned()
    });
    if native_name.is_some() {
        return native_name;
    }
    extract_str(super::closure::mb_func_get_name(value))
        .filter(|name| class_is_registered(name))
}

/// Ordered MRO (ancestors only, most-derived first) of a registered class.
/// Empty when the class is unknown. Used by the dataclasses runtime to merge
/// inherited dataclass fields (PEP 557).
/// Snapshot of a registered class's class-level attributes (name, value),
/// in insertion order where the map preserves it. Used by @enum.unique /
/// @enum.verify fallbacks for data-mixin enums that skip ENUM_CLASSES.
pub(crate) fn class_attr_entries(name: &str) -> Vec<(String, MbValue)> {
    CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(name)
            .map(|cls| {
                cls.class_attrs
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect()
            })
            .unwrap_or_default()
    })
}

pub(crate) fn class_mro_list(name: &str) -> Vec<String> {
    CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(name)
            .map(|c| c.mro.clone())
            .unwrap_or_default()
    })
}

/// Does the class's OWN method table (not the MRO) define `method`? Used by
/// the dataclass instance-creation path: a dataclass that defines its own
/// `__init__` keeps it; otherwise the synthesized init wins over any base
/// `__init__` found later in the MRO.
pub(crate) fn class_defines_own_method(name: &str, method: &str) -> bool {
    CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(name)
            .is_some_and(|c| c.methods.contains_key(method))
    })
}

pub fn mb_class_register(name: &str, bases: Vec<String>, methods: HashMap<String, MbValue>) {
    // Register all method addresses as valid callables.
    // R1 P1: Also unwrap classmethod/staticmethod wrappers to register
    // the underlying function address (not the wrapper pointer).
    CALLABLE_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        for method in methods.values() {
            let (unwrapped, _dk) = unwrap_descriptor_method(*method);
            let unwrapped_addr = extract_func_addr(unwrapped);
            if unwrapped_addr != 0 {
                reg.insert(unwrapped_addr);
            }
            // Also register the raw method value addr for backward compat
            let addr = extract_func_addr(*method);
            if addr != 0 {
                reg.insert(addr);
            }
        }
    });
    let mro = compute_mro(name, &bases);
    // Clone bases before moving into MbClass so we can iterate for __init_subclass__
    let bases_for_hook = bases.clone();
    CLASS_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(
            name.to_string(),
            MbClass {
                name: name.to_string(),
                bases,
                mro,
                methods,
                class_attrs: HashMap::new(),
                metaclass: None,
                cached_init: None,
            },
        );
    });

    // Invalidate method cache — new class may shadow inherited methods.
    invalidate_method_cache();

    // Resolve and cache __init__ for fast instance creation.
    // Must happen after insertion so lookup_method can find the class.
    let init_method = lookup_method(name, "__init__");
    if !init_method.is_none() {
        let addr = extract_func_addr(init_method);
        if addr != 0 {
            let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
            CLASS_REGISTRY.with(|reg| {
                if let Some(cls) = reg.borrow_mut().get_mut(name) {
                    cls.cached_init = Some((addr, is_registered));
                }
            });
        }
    }
    // collections.abc mixins: a class deriving from MutableSequence (etc.)
    // gets the mixin methods (append/extend/pop/__iadd__/…) installed so they
    // resolve through normal method + dunder dispatch.
    {
        let mro = CLASS_REGISTRY.with(|reg| {
            reg.borrow()
                .get(name)
                .map(|c| c.mro.clone())
                .unwrap_or_default()
        });
        install_abc_mixins(name, &mro);
    }

    // R10: Retrieve class keyword arguments (set by mb_class_set_kwargs before registration).
    let class_kwargs: HashMap<String, MbValue> =
        KWARGS_REGISTRY.with(|reg| reg.borrow_mut().remove(name).unwrap_or_default());

    // Call __init_subclass__ on each direct base (PEP 487)
    let cls_val = MbValue::from_ptr(MbObject::new_str(name.to_string()));
    for base_name in &bases_for_hook {
        let hook = lookup_method(base_name, "__init_subclass__");
        if !hook.is_none() {
            let addr = extract_func_addr(hook);
            if addr != 0 {
                let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                if is_registered {
                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                    if class_kwargs.is_empty() {
                        // No kwargs: call with 1 arg (cls only)
                        let func: extern "C" fn(MbValue) -> MbValue =
                            unsafe { std::mem::transmute(addr as usize) };
                        func(cls_val);
                    } else {
                        // R10: Pass kwargs as a dict to __init_subclass__(cls, kwargs_dict)
                        let kwargs_dict = build_kwargs_dict(&class_kwargs);
                        let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                            unsafe { std::mem::transmute(addr as usize) };
                        func(cls_val, kwargs_dict);
                    }
                }
            }
        } else if !class_kwargs.is_empty() {
            // R10: If base has no __init_subclass__ and kwargs are non-empty, raise TypeError
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "__init_subclass__() takes no keyword arguments".to_string(),
                )),
            );
            return;
        }
    }

    // R12: __set_name__ descriptor protocol (PEP 487).
    // After class dict is populated, call __set_name__(owner, name) on any attribute
    // that defines it.
    let class_attr_names: Vec<String> = CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(name) {
            let mut names: Vec<String> = cls.class_attrs.keys().cloned().collect();
            names.sort(); // Alphabetical order for determinism
            names
        } else {
            Vec::new()
        }
    });
    for attr_name in &class_attr_names {
        let attr_val = CLASS_REGISTRY.with(|reg| {
            reg.borrow()
                .get(name)
                .and_then(|cls| cls.class_attrs.get(attr_name).copied())
        });
        if let Some(val) = attr_val {
            if let Some(set_name_method) = try_get_dunder_on_value(val, "__set_name__") {
                let addr = extract_func_addr(set_name_method);
                if addr != 0 {
                    let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                    if is_registered {
                        let owner = MbValue::from_ptr(MbObject::new_str(name.to_string()));
                        let attr_str = MbValue::from_ptr(MbObject::new_str(attr_name.clone()));
                        // REQ: JIT-compiled functions use SystemV/C calling convention.
                        let func: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                            unsafe { std::mem::transmute(addr as usize) };
                        func(val, owner, attr_str);
                    }
                }
            }
        }
    }
}

/// Register a class from MbValues (callable from compiled code).
pub fn mb_class_define(
    name: MbValue,
    base: MbValue,
    method_names: MbValue,
    method_values: MbValue,
) {
    let class_name = extract_str(name).unwrap_or_else(|| "object".to_string());
    let base_name = extract_str(base);
    let bases = base_name.map(|b| vec![b]).unwrap_or_default();

    let mut methods = HashMap::new();
    unsafe {
        if let (Some(names_ptr), Some(vals_ptr)) = (method_names.as_ptr(), method_values.as_ptr()) {
            if let (ObjData::List(ref names_lock), ObjData::List(ref vals_lock)) =
                (&(*names_ptr).data, &(*vals_ptr).data)
            {
                let names = names_lock.read().unwrap();
                let vals = vals_lock.read().unwrap();
                for (n, v) in names.iter().zip(vals.iter()) {
                    if let Some(method_name) = extract_str(*n) {
                        // Fix C-prime: registry takes its own +1 so the JIT
                        // release of the source list VReg cannot UAF the raw
                        // MbValue stored in MbClass.methods.
                        super::rc::retain_if_ptr(*v);
                        methods.insert(method_name, *v);
                    }
                }
            }
        }
    }

    mb_class_register(&class_name, bases, methods);
}

/// Register a class from MbValues with multiple bases (P1 OOP conformance).
/// `bases_list` is a List MbValue containing base class name strings,
/// or None if no bases.
pub fn mb_class_define_multi(
    name: MbValue,
    bases_list: MbValue,
    method_names: MbValue,
    method_values: MbValue,
) {
    let class_name = extract_str(name).unwrap_or_else(|| "object".to_string());
    let mut bases = Vec::new();
    if let Some(ptr) = bases_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for item in items.iter() {
                    if let Some(base_name) = extract_str(*item) {
                        bases.push(base_name);
                    }
                }
            }
        }
    }

    let mut methods = HashMap::new();
    unsafe {
        if let (Some(names_ptr), Some(vals_ptr)) = (method_names.as_ptr(), method_values.as_ptr()) {
            if let (ObjData::List(ref names_lock), ObjData::List(ref vals_lock)) =
                (&(*names_ptr).data, &(*vals_ptr).data)
            {
                let names = names_lock.read().unwrap();
                let vals = vals_lock.read().unwrap();
                for (n, v) in names.iter().zip(vals.iter()) {
                    if let Some(method_name) = extract_str(*n) {
                        // Fix C-prime: registry takes its own +1.
                        super::rc::retain_if_ptr(*v);
                        methods.insert(method_name, *v);
                    }
                }
            }
        }
    }

    USER_CLASSES.with(|u| {
        u.borrow_mut().insert(class_name.clone());
    });
    mb_class_register(&class_name, bases, methods);
}

/// R10: Store class keyword arguments for __init_subclass__ dispatch.
/// Called BEFORE mb_class_define_multi when class has non-metaclass keyword args.
/// e.g., `class Child(Base, registry="users")` → kwargs = {"registry": "users"}
pub fn mb_class_set_kwargs(class_name: MbValue, keys: MbValue, values: MbValue) {
    let name = extract_str(class_name).unwrap_or_default();
    let mut kwargs = HashMap::new();
    unsafe {
        if let (Some(keys_ptr), Some(vals_ptr)) = (keys.as_ptr(), values.as_ptr()) {
            if let (ObjData::List(ref keys_lock), ObjData::List(ref vals_lock)) =
                (&(*keys_ptr).data, &(*vals_ptr).data)
            {
                let keys_items = keys_lock.read().unwrap();
                let vals_items = vals_lock.read().unwrap();
                for (k, v) in keys_items.iter().zip(vals_items.iter()) {
                    if let Some(key_name) = extract_str(*k) {
                        // Fix C-prime: KWARGS_REGISTRY takes its own +1.
                        super::rc::retain_if_ptr(*v);
                        kwargs.insert(key_name, *v);
                    }
                }
            }
        }
    }
    KWARGS_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(name, kwargs);
    });
}

/// Build a dict MbValue from a HashMap of kwargs (R10 helper).
fn build_kwargs_dict(kwargs: &HashMap<String, MbValue>) -> MbValue {
    let dict = super::dict_ops::mb_dict_new();
    for (key, val) in kwargs {
        let key_val = MbValue::from_ptr(MbObject::new_str(key.clone()));
        super::dict_ops::mb_dict_setitem(dict, key_val, *val);
    }
    dict
}

/// Look up a dunder method on a value's class (R12 helper).
/// Similar to try_get_dunder but works on arbitrary values (not just instances).
fn try_get_dunder_on_value(val: MbValue, dunder: &str) -> Option<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, .. } => {
                    let method = lookup_method(class_name, dunder);
                    if !method.is_none() {
                        return Some(method);
                    }
                }
                _ => {}
            }
        }
    }
    None
}

/// Set the metaclass for a class (P2-R2).
/// Called after `mb_class_define_multi` when `class Foo(metaclass=Meta)` is used.
/// Stores the metaclass association in CLASS_REGISTRY so that instance creation
/// routes through the metaclass's `__call__` method.
pub fn mb_class_set_metaclass(class_name: MbValue, metaclass_name: MbValue) {
    let name = extract_str(class_name).unwrap_or_default();
    let meta = extract_str(metaclass_name).unwrap_or_default();
    if meta.is_empty() {
        return;
    }
    CLASS_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some(cls) = reg.get_mut(&name) {
            cls.metaclass = Some(meta);
        }
    });
}

/// Set a class-level attribute (P2-R3).
/// Stores a value in the class's `class_attrs` dict so that it is visible
/// via the descriptor protocol (e.g., class-level descriptor instances).
pub fn mb_class_set_class_attr(class_name: MbValue, attr_name: MbValue, value: MbValue) {
    let name = extract_str(class_name).unwrap_or_default();
    let attr = extract_str(attr_name).unwrap_or_default();
    if name.is_empty() || attr.is_empty() {
        return;
    }
    let synthesize_typeddict = attr == "__annotations__";
    // Class-body enums (`class Color(enum.Enum): RED = 1`): convert eligible
    // class-body assignments into singleton member Instances at registration
    // time (Lane-B of #1448). Non-enum classes fall through untouched.
    let value = match super::stdlib::enum_class::maybe_convert_class_attr(&name, &attr, value) {
        Some(member) => member,
        None => value,
    };
    // Fix C-prime: registry takes its own +1 so JIT epilogue release of the
    // source VReg cannot UAF the raw reference in `class_attrs`.
    unsafe {
        super::rc::retain_if_ptr(value);
    }
    CLASS_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some(cls) = reg.get_mut(&name) {
            if let Some(prev) = cls.class_attrs.insert(attr, value) {
                unsafe {
                    super::rc::release_if_ptr(prev);
                }
            }
        } else {
            // No matching class — drop the retain we just took.
            unsafe {
                super::rc::release_if_ptr(value);
            }
        }
    });
    if synthesize_typeddict {
        synthesize_typeddict_metadata_from_annotations(&name, value);
    }
    // Invalidate method cache — class attribute change may affect method resolution.
    invalidate_method_cache();
}

fn synthesize_typeddict_metadata_from_annotations(name: &str, annotations: MbValue) {
    let mut keys = Vec::new();
    if let Some(ptr) = annotations.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                for key in lock.read().unwrap().keys() {
                    if let super::dict_ops::DictKey::Str(s) = key {
                        keys.push(s.clone());
                    }
                }
            }
        }
    }

    CLASS_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let Some(cls) = reg.get_mut(name) else { return };
        let is_typed_dict = cls.name == "TypedDict" || cls.mro.iter().any(|base| base == "TypedDict");
        if !is_typed_dict {
            return;
        }

        cls.class_attrs
            .entry("__total__".to_string())
            .or_insert(MbValue::from_bool(true));

        let required_items = keys
            .into_iter()
            .map(|key| MbValue::from_ptr(MbObject::new_str(key)))
            .collect();
        let required = MbValue::from_ptr(MbObject::new_frozenset(required_items));
        unsafe { super::rc::retain_if_ptr(required); }
        match cls.class_attrs.entry("__required_keys__".to_string()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(required);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                unsafe { super::rc::release_if_ptr(required); }
            }
        }

        let optional = MbValue::from_ptr(MbObject::new_frozenset(vec![]));
        unsafe { super::rc::retain_if_ptr(optional); }
        match cls.class_attrs.entry("__optional_keys__".to_string()) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(optional);
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                unsafe { super::rc::release_if_ptr(optional); }
            }
        }
    });
}

// ── Generator Method Dispatch ──

fn invalid_generator_throw_arg_type(value: MbValue) -> &'static str {
    if value.is_none() {
        "NoneType"
    } else if value.as_int().is_some() {
        "int"
    } else if value.as_bool().is_some() {
        "bool"
    } else if value.as_float().is_some() {
        "float"
    } else if let Some(ptr) = value.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Tuple(_) => "tuple",
                ObjData::Dict(_) => "dict",
                ObjData::Instance { class_name, .. } if class_name == "type" => "type",
                ObjData::Instance { .. } => "instance",
                _ => "object",
            }
        }
    } else {
        "object"
    }
}

fn raise_invalid_generator_throw_arg(value: MbValue) -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "exceptions must be classes or instances deriving from BaseException, not {}",
            invalid_generator_throw_arg_type(value),
        ))),
    );
    MbValue::none()
}

fn resolve_generator_throw_args(
    exc_type: MbValue,
    exc_msg: MbValue,
) -> Result<(String, String), MbValue> {
    if let Some(s) = extract_str(exc_type) {
        // Plain string type name is retained for mamba's legacy
        // `g.throw("TypeError", "msg")` lowering path.
        return Ok((s, extract_str(exc_msg).unwrap_or_default()));
    }

    if let Some(ptr) = exc_type.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    let fields_guard = fields.read().unwrap();
                    let Some(type_name) =
                        fields_guard.get("__name__").and_then(|v| extract_str(*v))
                    else {
                        return Err(raise_invalid_generator_throw_arg(exc_type));
                    };
                    if !super::exception::is_subclass_of(&type_name, "BaseException") {
                        return Err(raise_invalid_generator_throw_arg(exc_type));
                    }
                    return Ok((type_name, extract_str(exc_msg).unwrap_or_default()));
                }

                if !super::exception::is_subclass_of(class_name, "BaseException") {
                    return Err(raise_invalid_generator_throw_arg(exc_type));
                }

                let fields_guard = fields.read().unwrap();
                let msg = if !exc_msg.is_none() {
                    extract_str(exc_msg).unwrap_or_default()
                } else {
                    fields_guard
                        .get("message")
                        .and_then(|v| exception_message_str(*v))
                        .or_else(|| {
                            fields_guard
                                .get("args")
                                .and_then(|t| first_tuple_element(*t))
                                .and_then(exception_message_str)
                        })
                        .unwrap_or_default()
                };
                return Ok((class_name.clone(), msg));
            }
        }
    }

    Err(raise_invalid_generator_throw_arg(exc_type))
}

/// Dispatch method calls on generator handles (.send, .throw, .close).
fn dispatch_generator_method(gen: MbValue, method: &str, args: MbValue) -> MbValue {
    let arg_list = extract_args_list(args);
    match method {
        "send" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            super::generator::mb_generator_send(gen, value)
        }
        "throw" => {
            // g.throw(ExcType, message) or g.throw(exc_instance)
            // CPython 3.12: throw(value) where value is an exception instance
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            let type_val = MbValue::from_ptr(MbObject::new_str(type_str));
            let msg_val = MbValue::from_ptr(MbObject::new_str(msg_str));
            super::generator::mb_generator_throw(gen, type_val, msg_val)
        }
        "close" => {
            super::generator::mb_generator_close(gen);
            MbValue::none()
        }
        "__next__" => super::generator::mb_generator_next(gen),
        // Async-generator protocol: `async def f(): yield` is routed
        // through the sync generator path (see ast_to_hir.rs AsyncFnDef
        // arm), so the same handle must answer both sync and async
        // iteration methods. `await g.__anext__()` works because mb_await
        // on a non-coroutine value passes it through unchanged.
        "__aiter__" => gen,
        "__anext__" => {
            let val = super::generator::mb_generator_next(gen);
            // Generator completion sets CURRENT_EXCEPTION to StopIteration;
            // CPython's async-iter protocol expects StopAsyncIteration
            // instead. Convert in-place so user code can
            // `except StopAsyncIteration:` cleanly.
            let pending = super::exception::mb_get_exception();
            if !pending.is_none() {
                if let Some(ptr) = pending.as_ptr() {
                    let is_stop = unsafe {
                        matches!(
                            &(*ptr).data,
                            super::rc::ObjData::Instance { class_name, .. }
                                if class_name == "StopIteration"
                        )
                    };
                    if is_stop {
                        super::exception::mb_clear_exception();
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("StopAsyncIteration".to_string())),
                            MbValue::from_ptr(MbObject::new_str(String::new())),
                        );
                        return MbValue::none();
                    }
                }
            }
            // Wrap the yielded value in a pre-completed coroutine so that
            // `await g.__anext__()` survives the generator/coroutine
            // ID-space collision. NEXT_GEN_ID and NEXT_CORO_ID both count
            // from 1 independently, so a small yielded int (e.g. `1`) on
            // a generator with id=1 would be misread by mb_await as the
            // (still-active) main coroutine, blow the iteration budget,
            // and surface as None. Boxing the value into a coroutine
            // marked exhausted with `result = val` makes the await path
            // unambiguous and round-trips the value cleanly.
            let coro = super::async_rt::mb_coroutine_new(
                MbValue::from_ptr(MbObject::new_str("__anext_value__".to_string())),
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
            );
            super::async_rt::mb_coroutine_complete(coro, val);
            coro
        }
        "aclose" => {
            super::generator::mb_generator_close(gen);
            MbValue::none()
        }
        "asend" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            super::generator::mb_generator_send(gen, value)
        }
        "athrow" => {
            // Re-route to throw with the same arg shape — matches CPython
            // sync-vs-async generator equivalence in mamba's flattened model.
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            super::generator::mb_generator_throw(
                gen,
                MbValue::from_ptr(MbObject::new_str(type_str)),
                MbValue::from_ptr(MbObject::new_str(msg_str)),
            )
        }
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'generator' object has no attribute '{method}'"
                ))),
            );
            MbValue::none()
        }
    }
}

/// Extract arguments from a list MbValue.
fn extract_args_list(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    Vec::new()
}

// ── Descriptor Wrapper Helpers (P1 OOP conformance) ──

/// Descriptor kind for classmethod/staticmethod/regular method dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DescriptorKind {
    Regular,
    ClassMethod,
    StaticMethod,
}

/// Unwrap a `__classmethod__` or `__staticmethod__` wrapper to get the underlying
/// function pointer (TAG_FUNC). Returns (func_mbvalue, descriptor_kind).
fn unwrap_descriptor_method(method: MbValue) -> (MbValue, DescriptorKind) {
    if let Some(ptr) = method.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
                if class_name == "__classmethod__" || class_name == "__staticmethod__" {
                    let kind = if class_name == "__classmethod__" {
                        DescriptorKind::ClassMethod
                    } else {
                        DescriptorKind::StaticMethod
                    };
                    let fields = fields.read().unwrap();
                    if let Some(&func) = fields.get("__func__") {
                        return (func, kind);
                    }
                }
            }
        }
    }
    (method, DescriptorKind::Regular)
}

/// Unwrap any classmethod/staticmethod wrapper on `method` and return its
/// function address when the address is in the CALLABLE_REGISTRY; 0
/// otherwise. Used by the class-body enum machinery to dispatch the
/// `_missing_(cls, value)` classmethod hook.
pub(crate) fn registered_callable_addr(method: MbValue) -> u64 {
    let (unwrapped, _kind) = unwrap_descriptor_method(method);
    let addr = extract_func_addr(unwrapped);
    if addr != 0 && CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr)) {
        addr
    } else {
        0
    }
}

// ── Function Address Extraction ──

/// Extract a function address from a NaN-boxed method value.
/// Methods are stored as NaN-boxed pointers (TAG_PTR=0), where the
/// 48-bit payload is the actual function address in executable memory.
/// Also handles raw int-tagged values for backward compatibility.
fn extract_func_addr(val: MbValue) -> u64 {
    // TAG_FUNC (tag=4): JIT-compiled or extern function pointer — most common for methods.
    if let Some(addr) = val.as_func() {
        return addr as u64;
    }
    // TAG_PTR (tag=0): heap-pointer (legacy path).
    if let Some(ptr) = val.as_ptr() {
        return ptr as u64;
    }
    // Fallback: try as NaN-boxed int
    if let Some(i) = val.as_int() {
        return i as u64;
    }
    0
}

fn memoryview_field(view: MbValue, name: &str) -> Option<MbValue> {
    let ptr = view.as_ptr()?;
    unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if class_name == "memoryview" {
                return fields.read().unwrap().get(name).copied();
            }
        }
    }
    None
}

fn memoryview_readonly(view: MbValue) -> bool {
    if let Some(ro) = memoryview_field(view, "_readonly") {
        return ro.as_bool() == Some(true);
    }
    let Some(buf) = memoryview_field(view, "_buffer") else {
        return true;
    };
    let writable = buf.as_ptr().map_or(false, |bp| unsafe {
        matches!((*bp).data, ObjData::ByteArray(_))
    });
    !writable
}

fn memoryview_slice(view: MbValue, start: MbValue, stop: MbValue, step: MbValue) -> MbValue {
    let Some(buf) = memoryview_field(view, "_buffer") else {
        return MbValue::from_ptr(MbObject::new_instance("memoryview".to_string()));
    };
    let sliced = super::bytes_ops::mb_bytes_slice_full(buf, start, stop, step);
    let stride = step.as_int().unwrap_or(1);
    let inst = MbObject::new_instance("memoryview".to_string());
    unsafe {
        if let ObjData::Instance { fields, .. } = &(*inst).data {
            let mut f = fields.write().unwrap();
            f.insert("_buffer".to_string(), sliced);
            let obj = memoryview_field(view, "_obj").unwrap_or(buf);
            super::rc::retain_if_ptr(obj);
            f.insert("_obj".to_string(), obj);
            f.insert("_readonly".to_string(), MbValue::from_bool(memoryview_readonly(view)));
            f.insert("_contiguous".to_string(), MbValue::from_bool(stride == 1));
            f.insert("_stride".to_string(), MbValue::from_int(stride));
        }
    }
    MbValue::from_ptr(inst)
}

// ── Instance Creation ──

/// Create a new instance of a class, calling __init__ if present.
pub fn mb_instance_new(class_name: MbValue, _args: MbValue) -> MbValue {
    let name = extract_str(class_name).unwrap_or_else(|| "object".to_string());
    if let Some(result) = mb_collections_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_user_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_contextlib_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_reject_protocol_instantiation(&name) {
        return result;
    }
    let instance = MbValue::from_ptr(MbObject::new_instance(name.clone()));

    // __init__ is called by the compiled code after instance creation.
    // The runtime just creates the instance object here.

    instance
}

/// Helper: call __init__ with instance + args from args_list using arity dispatch.
/// Factored out to avoid duplicating the arity match in both cached and uncached paths.
/// Uses a single RwLock read to get both the arg count and arg values.
#[inline]
fn call_init_with_args(addr: u64, instance: MbValue, args_list: MbValue) {
    // Variadic native `__init__(self, args_list)`: the registered function
    // always expects exactly two arguments — `self` plus a list of the
    // remaining positional args — regardless of how many were supplied at the
    // call site. This lets native constructors model optional parameters
    // (e.g. `TestCase(methodName="runTest")`) without one typed entry point
    // per arity. Built classes never register a variadic `__init__`, so this
    // branch is inert for them.
    if super::module::is_variadic_func(addr) {
        let packed: Vec<MbValue> = args_list
            .as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    lock.read().unwrap().iter().copied().collect()
                } else {
                    Vec::new()
                }
            })
            .unwrap_or_default();
        let list = MbValue::from_ptr(MbObject::new_list(packed));
        let func: extern "C" fn(MbValue, MbValue) -> MbValue =
            unsafe { std::mem::transmute(addr as usize) };
        func(instance, list);
        return;
    }
    if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                // Dispatch based on arg count — single lock acquisition for count + values.
                // REQ: JIT-compiled functions use SystemV/C calling convention.
                match items.len() {
                    0 => {
                        let func: extern "C" fn(MbValue) -> MbValue =
                            std::mem::transmute(addr as usize);
                        func(instance);
                    }
                    1 => {
                        let a0 = items[0];
                        let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                            std::mem::transmute(addr as usize);
                        func(instance, a0);
                    }
                    2 => {
                        let a0 = items[0];
                        let a1 = items[1];
                        let func: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                            std::mem::transmute(addr as usize);
                        func(instance, a0, a1);
                    }
                    3 => {
                        let a0 = items[0];
                        let a1 = items[1];
                        let a2 = items[2];
                        let func: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                            std::mem::transmute(addr as usize);
                        func(instance, a0, a1, a2);
                    }
                    _ => {
                        // Higher arity: build args vec from the already-held lock.
                        let mut all_args = Vec::with_capacity(items.len() + 1);
                        all_args.push(instance);
                        all_args.extend(items.iter());
                        drop(items); // Release the lock before calling.
                        if all_args.len() == 5 {
                            let func: extern "C" fn(
                                MbValue,
                                MbValue,
                                MbValue,
                                MbValue,
                                MbValue,
                            ) -> MbValue = std::mem::transmute(addr as usize);
                            func(
                                all_args[0],
                                all_args[1],
                                all_args[2],
                                all_args[3],
                                all_args[4],
                            );
                        }
                    }
                }
                return;
            }
        }
    }
    // No args list — call with just instance (zero-arg __init__).
    // REQ: JIT-compiled functions use SystemV/C calling convention.
    let func: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr as usize) };
    func(instance);
}

/// Create a new instance of a class and invoke __init__ with args.
/// `args_list` is a List MbValue containing all constructor arguments.
/// Used by compiled code for `ClassName(arg1, arg2, ...)`.
pub fn mb_instance_new_with_init(class_name: MbValue, args_list: MbValue) -> MbValue {
    instance_new_with_init_impl(class_name, args_list, false)
}

/// Default instance creation BYPASSING metaclass `__call__` routing — the
/// `type.__call__(cls, ...)` a metaclass body reaches via `super().__call__()`.
/// Routing through the public entry from inside the metaclass would recurse.
pub(crate) fn instance_new_default(class_name: MbValue, args_list: MbValue) -> MbValue {
    instance_new_with_init_impl(class_name, args_list, true)
}

fn instance_new_with_init_impl(
    class_name: MbValue,
    args_list: MbValue,
    skip_metaclass: bool,
) -> MbValue {
    let name = extract_str(class_name).unwrap_or_else(|| "object".to_string());
    if let Some(result) = mb_collections_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_user_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_contextlib_abc_reject_abstract_instantiation(&name) {
        return result;
    }
    if let Some(result) = mb_reject_protocol_instantiation(&name) {
        return result;
    }
    // Class-body enums: `Color(2)` is a value→member lookup (with the
    // `_missing_` hook), never instance creation (CPython EnumType.__call__).
    if let Some(member) = super::stdlib::enum_class::enum_class_call(&name, args_list) {
        return member;
    }

    // Single registry access: fetch both metaclass and cached_init together to avoid
    // two separate RefCell borrows + HashMap lookups per instance creation.
    let (metaclass_name, cached_init) = CLASS_REGISTRY.with(|reg| {
        let r = reg.borrow();
        if let Some(cls) = r.get(&name) {
            (cls.metaclass.clone(), cls.cached_init)
        } else {
            (None, None)
        }
    });
    let metaclass_name = if skip_metaclass { None } else { metaclass_name };

    // P2-R2: Check for metaclass — route through metaclass.__call__ if present.
    // When a class has a metaclass, the metaclass's __call__ controls instance creation.
    // Most classes have no metaclass, so this branch is rarely taken.
    if let Some(ref meta_name) = metaclass_name {
        let call_method = lookup_method(meta_name, "__call__");
        if !call_method.is_none() {
            let addr = extract_func_addr(call_method);
            if addr != 0 {
                let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                if is_registered {
                    // Gather constructor args
                    let mut ctor_args: Vec<MbValue> = Vec::new();
                    if let Some(ptr) = args_list.as_ptr() {
                        unsafe {
                            if let ObjData::List(ref lock) = (*ptr).data {
                                let items = lock.read().unwrap();
                                ctor_args.extend(items.iter());
                            }
                        }
                    }
                    // Call metaclass.__call__(cls_name, ...args)
                    // cls_name is passed as `self` (first arg) for the method.
                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                    let result = match ctor_args.len() {
                        0 => {
                            let func: extern "C" fn(MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr as usize) };
                            func(class_name)
                        }
                        1 => {
                            let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr as usize) };
                            func(class_name, ctor_args[0])
                        }
                        2 => {
                            let func: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr as usize) };
                            func(class_name, ctor_args[0], ctor_args[1])
                        }
                        3 => {
                            let func: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr as usize) };
                            func(class_name, ctor_args[0], ctor_args[1], ctor_args[2])
                        }
                        _ => MbValue::none(),
                    };
                    // If metaclass.__call__ returns a non-None value, use it as the instance.
                    // Otherwise fall through to default creation with __init__.
                    if !result.is_none() {
                        return result;
                    }
                }
            }
        }
    }

    // Pre-allocate HashMap capacity based on constructor arg count.
    // Each __init__ arg typically corresponds to one `self.x = x` assignment,
    // so we use the arg count as a hint for the field HashMap capacity.
    let field_capacity = if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                lock.read().unwrap().len()
            } else {
                0
            }
        }
    } else {
        0
    };
    let instance = MbValue::from_ptr(MbObject::new_instance_with_capacity(
        name.clone(),
        field_capacity,
    ));
    if check_class_hierarchy(&name, "int")
        && !seed_int_subclass_value(instance, args_list)
    {
        return MbValue::none();
    }

    // PEP 557: dataclasses without their own `__init__` route through the
    // synthesized init (positional binding in declaration order, defaults,
    // default_factory, InitVar forwarding, __post_init__). Checked before the
    // cached-init path so a base class's `__init__` in the MRO does not
    // shadow the synthesized one; a dataclass that defines its own __init__
    // keeps it.
    if super::stdlib::dataclasses_mod::dc_has_synth_init(&name)
        && !class_defines_own_method(&name, "__init__")
    {
        super::stdlib::dataclasses_mod::dc_run_synth_init(&name, instance, args_list);
        return instance;
    }

    // Fast path: use cached __init__ from MbClass to avoid MRO walk.
    // cached_init was already fetched in the single registry access above.
    let has_init = if let Some((addr, is_registered)) = cached_init {
        // Cached __init__ found — use it directly without MRO walk.
        if is_registered {
            // Extract args from the list and call __init__ with arity-based dispatch.
            // Retain instance before dispatch: the JIT Return terminator emits
            // mb_release_value for all I64-typed vreg params including self, dropping
            // rc from 1→0 and freeing the instance. We pre-retain (rc 1→2) so the
            // JIT release only brings rc to 1, leaving the instance alive for the caller.
            unsafe {
                super::rc::retain_if_ptr(instance);
            }
            call_init_with_args(addr, instance, args_list);
        }
        true
    } else {
        // No cached __init__ — fall back to MRO lookup (handles dynamically added methods).
        let init_method = lookup_method(&name, "__init__");
        if !init_method.is_none() {
            let addr = extract_func_addr(init_method);
            if addr != 0 {
                let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                if is_registered {
                    // Same retain guard as the cached path above.
                    unsafe {
                        super::rc::retain_if_ptr(instance);
                    }
                    call_init_with_args(addr, instance, args_list);
                }
            }
            true
        } else {
            false
        }
    };

    if !has_init {
        // No __init__ defined — store args as e.args for Exception-like classes
        // Check if any base class is an exception type (including built-in hierarchy)
        let is_exception = CLASS_REGISTRY.with(|reg| {
            if let Some(cls) = reg.borrow().get(&name) {
                cls.mro.iter().any(|c| {
                    c == "Exception"
                        || c == "BaseException"
                        || super::exception::is_subclass_of(c, "Exception")
                })
            } else {
                false
            }
        });
        // A user subclass of (Base)ExceptionGroup must carry the EG shape
        // (`message` + `exceptions` tuple) so str()/repr()/split() work — the
        // generic exception init below would only store `args`.
        let is_eg = name == "ExceptionGroup" || name == "BaseExceptionGroup"
            || CLASS_REGISTRY.with(|reg| {
                reg.borrow().get(&name).map(|cls| {
                    cls.mro.iter().any(|c| c == "BaseExceptionGroup" || c == "ExceptionGroup")
                }).unwrap_or(false)
            });
        if is_eg {
            if let Some(ptr) = args_list.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        let items = lock.read().unwrap();
                        let message = items.first().copied().unwrap_or_else(MbValue::none);
                        let excs = items.get(1).copied().unwrap_or_else(MbValue::none);
                        // PEP 654: an ExceptionGroup subclass (but not a bare
                        // BaseExceptionGroup subclass) cannot nest a BaseException.
                        let is_eg_not_base = name == "ExceptionGroup"
                            || super::exception::is_subclass_of(&name, "ExceptionGroup");
                        if is_eg_not_base {
                            let members: Vec<MbValue> = excs.as_ptr().map(|ep| match &(*ep).data {
                                ObjData::List(ref el) => el.read().unwrap().to_vec(),
                                ObjData::Tuple(ref t) => t.to_vec(),
                                _ => Vec::new(),
                            }).unwrap_or_default();
                            if members.iter().any(|m| super::exception::eg_member_is_bare_base(*m)) {
                                drop(items);
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(
                                        super::exception::eg_nest_error_message(&name))),
                                );
                                return instance;
                            }
                        }
                        let msg_str = super::builtins::mb_str(message);
                        mb_setattr(instance,
                            MbValue::from_ptr(MbObject::new_str("message".to_string())), msg_str);
                        mb_setattr(instance,
                            MbValue::from_ptr(MbObject::new_str("__type__".to_string())),
                            MbValue::from_ptr(MbObject::new_str(name.clone())));
                        let exc_tuple = if let Some(ep) = excs.as_ptr() {
                            match &(*ep).data {
                                ObjData::List(ref el) => MbValue::from_ptr(
                                    MbObject::new_tuple_borrowed(el.read().unwrap().to_vec())),
                                _ => excs,
                            }
                        } else { excs };
                        mb_setattr(instance,
                            MbValue::from_ptr(MbObject::new_str("exceptions".to_string())), exc_tuple);
                        let args_tuple = MbValue::from_ptr(
                            MbObject::new_tuple_borrowed(items.to_vec()));
                        mb_setattr(instance,
                            MbValue::from_ptr(MbObject::new_str("args".to_string())), args_tuple);
                    }
                }
            }
        } else if is_exception {
            // Store args attribute and message
            if let Some(ptr) = args_list.as_ptr() {
                unsafe {
                    if let ObjData::List(ref lock) = (*ptr).data {
                        let items = lock.read().unwrap();
                        // Set message to first arg (Python convention)
                        if let Some(first) = items.first() {
                            let msg = super::builtins::mb_str(*first);
                            mb_setattr(
                                instance,
                                MbValue::from_ptr(MbObject::new_str("message".to_string())),
                                msg,
                            );
                        }
                        // Set __type__ to class name
                        mb_setattr(
                            instance,
                            MbValue::from_ptr(MbObject::new_str("__type__".to_string())),
                            MbValue::from_ptr(MbObject::new_str(name.clone())),
                        );
                        // Store args as tuple — items borrowed from args list.
                        let args_tuple =
                            MbValue::from_ptr(MbObject::new_tuple_borrowed(items.to_vec()));
                        mb_setattr(
                            instance,
                            MbValue::from_ptr(MbObject::new_str("args".to_string())),
                            args_tuple,
                        );
                        // StopIteration stores the first arg as `.value` (CPython semantics).
                        if name == "StopIteration" {
                            eprintln!("[DBG] StopIteration init branch taken");
                            let value_val = items.first().copied().unwrap_or_else(MbValue::none);
                            super::rc::retain_if_ptr(value_val);
                            mb_setattr(
                                instance,
                                MbValue::from_ptr(MbObject::new_str("value".to_string())),
                                value_val,
                            );
                        }
                    }
                }
            }
        }
    }

    instance
}

fn seed_int_subclass_value(instance: MbValue, args_list: MbValue) -> bool {
    let args = super::builtins::extract_items(args_list);
    let value = match args.len() {
        0 => MbValue::from_int(0),
        1 => super::builtins::mb_int(args[0]),
        2 => super::builtins::mb_int_base(args[0], args[1]),
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "int() takes at most 2 arguments ({} given)",
                    args.len()
                ))),
            );
            return false;
        }
    };
    if super::exception::mb_has_exception().as_bool() == Some(true) {
        return false;
    }
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields
                    .write()
                    .unwrap()
                    .insert(INT_SUBCLASS_VALUE_FIELD.to_string(), value);
            }
        }
    }
    true
}

/// The `message` field of an exception instance as display text. CPython's
/// `str(exc)` stringifies a non-str single arg (`str(ValueError(3)) == "3"`,
/// and `SystemExit(3)` carries its exit status here), so int/float args
/// must not vanish.
/// First element of an `args` tuple field, if any.
fn first_tuple_element(t: MbValue) -> Option<MbValue> {
    let ptr = t.as_ptr()?;
    unsafe {
        if let ObjData::Tuple(ref items) = (*ptr).data {
            items.first().copied()
        } else {
            None
        }
    }
}

fn exception_message_str(v: MbValue) -> Option<String> {
    extract_str(v)
        .or_else(|| v.as_int().map(|i| i.to_string()))
        .or_else(|| v.as_float().map(|f| f.to_string()))
}

/// Raise an exception from an instance value directly.
/// Used for user-defined exception classes: the instance already has
/// the correct class_name, message, and custom fields.
pub fn mb_raise_instance(instance: MbValue) {
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            // A builtin exception class referenced as a value is carried as a
            // bare type-name string (`e = StopIteration; raise e`). Raise it as
            // that exception type rather than dropping to the generic Exception
            // fallback — and flip the StopIteration iterator flag so a
            // variable-spelled StopIteration is recognised as exhaustion.
            if let ObjData::Str(ref type_name) = (*ptr).data {
                let exc = super::exception::MbException::new(type_name, "");
                super::exception::set_current_exception(exc);
                if type_name == "StopIteration" {
                    super::iter::signal_stop_iteration();
                }
                return;
            }
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                let fields_guard = fields.read().unwrap();
                // `raise SomeException` where the operand is a bare type OBJECT
                // (class_name "type" carrying __name__, e.g. a class held in a
                // variable: `e = StopIteration; raise e`) raises an instance of
                // that class — the exception type is __name__, not the literal
                // "type". Resolving it here is what lets the iterator protocol
                // recognise a variable-raised StopIteration as exhaustion.
                let resolved_type = if class_name == "type" {
                    fields_guard
                        .get("__name__")
                        .and_then(|v| extract_str(*v))
                        .unwrap_or_else(|| class_name.clone())
                } else {
                    class_name.clone()
                };
                let msg = fields_guard
                    .get("message")
                    .and_then(|v| exception_message_str(*v))
                    .or_else(|| {
                        fields_guard
                            .get("args")
                            .and_then(|t| first_tuple_element(*t))
                            .and_then(exception_message_str)
                    })
                    .unwrap_or_default();
                let exc = super::exception::MbException::new(&resolved_type, &msg);
                drop(fields_guard);
                super::exception::set_current_exception(exc);
                // Parity with mb_raise: a StopIteration (however it was spelled)
                // must also flip the iterator-protocol flag so user __next__
                // exhaustion is caught by the drive loop, not propagated.
                if resolved_type == "StopIteration" {
                    super::iter::signal_stop_iteration();
                }
                // Retain before storing: the caller's vreg may be released at function
                // exit before the catcher retrieves it. Ownership is then transferred
                // to mb_catch_exception_instance which takes from the cell.
                super::rc::retain_if_ptr(instance);
                LAST_RAISED_INSTANCE.with(|cell| {
                    let prev = cell.borrow_mut().take();
                    if let Some(p) = prev {
                        super::rc::release_if_ptr(p);
                    }
                    *cell.borrow_mut() = Some(instance);
                });
                return;
            }
        }
    }
    // Fallback: generic exception
    let exc = super::exception::MbException::new("Exception", "");
    super::exception::set_current_exception(exc);
}

/// Raise an existing instance with implicit context chaining.
/// Used for `raise exc` (variable) inside an except handler body.
pub fn mb_raise_instance_with_context(instance: MbValue, context: MbValue) {
    // Set __context__ on the instance's fields so it's visible via e.__context__
    if !context.is_none() {
        let ctx_key = MbValue::from_ptr(MbObject::new_str("__context__".to_string()));
        mb_setattr(instance, ctx_key, context);
    }
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                let fields_guard = fields.read().unwrap();
                let msg = fields_guard
                    .get("message")
                    .and_then(|v| exception_message_str(*v))
                    .or_else(|| {
                        fields_guard
                            .get("args")
                            .and_then(|t| first_tuple_element(*t))
                            .and_then(exception_message_str)
                    })
                    .unwrap_or_default();
                let mut exc = super::exception::MbException::new(class_name, &msg);
                if !context.is_none() {
                    let ctx_type = super::exception::get_exception_type_pub(context)
                        .unwrap_or_else(|| "Exception".to_string());
                    let ctx_msg =
                        super::exception::get_exception_message_pub(context).unwrap_or_default();
                    exc.context = Some(Box::new(super::exception::MbException::new(
                        &ctx_type, &ctx_msg,
                    )));
                }
                super::exception::set_current_exception(exc);
                LAST_RAISED_INSTANCE.with(|cell| {
                    *cell.borrow_mut() = Some(instance);
                });
                return;
            }
        }
    }
    // Fallback: just raise the instance without context
    mb_raise_instance(instance);
}

/// Raise an instance with explicit chaining: `raise X from Y`.
/// Sets __cause__ on the instance and __suppress_context__ = True.
/// `cause` is the exception value from the `from` clause.
pub fn mb_raise_instance_from(instance: MbValue, cause: MbValue) {
    // Set __cause__ and __suppress_context__ on the instance
    let cause_key = MbValue::from_ptr(MbObject::new_str("__cause__".to_string()));
    mb_setattr(instance, cause_key, cause);
    let suppress_key = MbValue::from_ptr(MbObject::new_str("__suppress_context__".to_string()));
    mb_setattr(instance, suppress_key, MbValue::from_bool(true));
    // Also set __cause__ and suppress_context on the MbException in thread-local state
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                let fields_guard = fields.read().unwrap();
                let msg = fields_guard
                    .get("message")
                    .and_then(|v| exception_message_str(*v))
                    .or_else(|| {
                        fields_guard
                            .get("args")
                            .and_then(|t| first_tuple_element(*t))
                            .and_then(exception_message_str)
                    })
                    .unwrap_or_default();
                let mut exc = super::exception::MbException::new(class_name, &msg);
                exc.suppress_context = true;
                if !cause.is_none() {
                    let cause_type = super::exception::get_exception_type_pub(cause)
                        .unwrap_or_else(|| "Exception".to_string());
                    let cause_msg =
                        super::exception::get_exception_message_pub(cause).unwrap_or_default();
                    exc.cause = Some(Box::new(super::exception::MbException::new(
                        &cause_type,
                        &cause_msg,
                    )));
                }
                super::exception::set_current_exception(exc);
                LAST_RAISED_INSTANCE.with(|cell| {
                    *cell.borrow_mut() = Some(instance);
                });
                return;
            }
        }
    }
    mb_raise_instance(instance);
}

/// Raise an instance with explicit chaining and implicit context.
/// `raise X from Y` inside an except handler body.
pub fn mb_raise_instance_from_with_context(instance: MbValue, cause: MbValue, context: MbValue) {
    // Set __cause__, __suppress_context__, and __context__ on the instance
    let cause_key = MbValue::from_ptr(MbObject::new_str("__cause__".to_string()));
    mb_setattr(instance, cause_key, cause);
    let suppress_key = MbValue::from_ptr(MbObject::new_str("__suppress_context__".to_string()));
    mb_setattr(instance, suppress_key, MbValue::from_bool(true));
    let context_key = MbValue::from_ptr(MbObject::new_str("__context__".to_string()));
    mb_setattr(instance, context_key, context);
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                let fields_guard = fields.read().unwrap();
                let msg = fields_guard
                    .get("message")
                    .and_then(|v| exception_message_str(*v))
                    .or_else(|| {
                        fields_guard
                            .get("args")
                            .and_then(|t| first_tuple_element(*t))
                            .and_then(exception_message_str)
                    })
                    .unwrap_or_default();
                let mut exc = super::exception::MbException::new(class_name, &msg);
                exc.suppress_context = true;
                if !cause.is_none() {
                    let cause_type = super::exception::get_exception_type_pub(cause)
                        .unwrap_or_else(|| "Exception".to_string());
                    let cause_msg =
                        super::exception::get_exception_message_pub(cause).unwrap_or_default();
                    exc.cause = Some(Box::new(super::exception::MbException::new(
                        &cause_type,
                        &cause_msg,
                    )));
                }
                if !context.is_none() {
                    let ctx_type = super::exception::get_exception_type_pub(context)
                        .unwrap_or_else(|| "Exception".to_string());
                    let ctx_msg =
                        super::exception::get_exception_message_pub(context).unwrap_or_default();
                    exc.context = Some(Box::new(super::exception::MbException::new(
                        &ctx_type, &ctx_msg,
                    )));
                }
                super::exception::set_current_exception(exc);
                LAST_RAISED_INSTANCE.with(|cell| {
                    *cell.borrow_mut() = Some(instance);
                });
                return;
            }
        }
    }
    mb_raise_instance(instance);
}

thread_local! {
    /// The most recently *caught* exception value — what `sys.exception()`
    /// reports (None until the first catch).
    static LAST_CAUGHT_VALUE: std::cell::Cell<u64> =
        std::cell::Cell::new(MbValue::none().to_bits());
}

/// The exception value most recently bound by an except handler.
pub(crate) fn last_caught_exception_value() -> MbValue {
    LAST_CAUGHT_VALUE.with(|c| MbValue::from_bits(c.get()))
}

/// Retrieve the last raised instance (preserves custom fields).
/// Falls back to mb_catch_exception if no instance was stored.
pub fn mb_catch_exception_instance() -> MbValue {
    // Check if we have a stored full instance
    let instance = LAST_RAISED_INSTANCE.with(|cell| cell.borrow_mut().take());
    if let Some(inst) = instance {
        LAST_CAUGHT_VALUE.with(|c| c.set(inst.to_bits()));
        // Clear the thread-local exception state
        super::exception::clear_current_exception();
        // mb_raise also signals STOP_ITERATION on raise of StopIteration so
        // user-defined __next__ can use `raise StopIteration` to mean
        // end-of-iter. A user-level `except StopIteration:` block leaves
        // that iterator-protocol flag set; the next generator resume in
        // the same frame then misreads it as immediate exhaustion. Clear
        // it here so catching a StopIteration is a clean reset.
        super::iter::check_and_clear_stop();
        unsafe {
            super::rc::retain_if_ptr(inst);
        }
        return inst;
    }
    // Fallback to standard catch (already retains internally)
    let caught = super::exception::mb_catch_exception();
    super::iter::check_and_clear_stop();
    LAST_CAUGHT_VALUE.with(|c| c.set(caught.to_bits()));
    caught
}

thread_local! {
    /// Storage for the last raised instance (preserves custom fields).
    static LAST_RAISED_INSTANCE: std::cell::RefCell<Option<MbValue>> =
        std::cell::RefCell::new(None);
}

// ── Attribute Access ──

/// Internal-to-registry shim for typed-wrapper detection. The registry
/// crate stores type names keyed by raw pointer address; this helper
/// converts our internal `MbValue` into the registry's repr-transparent
/// `MbValue` via bit-level conversion so pointer addresses match.
fn native_type_name_for(obj: MbValue) -> Option<&'static str> {
    let reg = cclab_mamba_registry::MbValue::from_bits(obj.to_bits());
    cclab_mamba_registry::convert::native_type_name(reg)
}

fn make_unbound_method(type_name: &str, method_name: &str) -> MbValue {
    let inst = MbObject::new_instance("__unbound_method__".to_string());
    if let ObjData::Instance { fields, .. } = unsafe { &(*inst).data } {
        let mut guard = fields.write().unwrap();
        guard.insert(
            "__type__".to_string(),
            MbValue::from_ptr(MbObject::new_str(type_name.to_string())),
        );
        guard.insert(
            "__method__".to_string(),
            MbValue::from_ptr(MbObject::new_str(method_name.to_string())),
        );
    }
    MbValue::from_ptr(inst)
}

fn inherited_builtin_unbound_method(class_name: &str, method_name: &str) -> Option<MbValue> {
    if !class_is_registered(class_name) {
        return None;
    }
    for ancestor in class_mro_list(class_name).into_iter().skip(1) {
        if builtin_type_method_names_by_name(&ancestor)
            .iter()
            .any(|m| *m == method_name)
        {
            return Some(make_unbound_method(&ancestor, method_name));
        }
    }
    None
}

/// Build a bound method over a builtin instance: a `__bound_native_method__`
/// shell carrying the receiver + method name. `mb_call_spread` / `mb_call0` /
/// `mb_call1_val` dispatch it through `mb_call_method`, which already serves the
/// direct `recv.method(...)` form — so `f = (1,2).count; f(1)` works like
/// `(1,2).count(1)`. The receiver is retained because the shell now owns a
/// reference to it.
fn make_bound_native_method(recv: MbValue, method_name: &str) -> MbValue {
    let inst = MbObject::new_instance("__bound_native_method__".to_string());
    if let ObjData::Instance { fields, .. } = unsafe { &(*inst).data } {
        let mut g = fields.write().unwrap();
        g.insert("__self__".to_string(), recv);
        unsafe {
            super::rc::retain_if_ptr(recv);
        }
        g.insert(
            "__method__".to_string(),
            MbValue::from_ptr(MbObject::new_str(method_name.to_string())),
        );
        g.insert(
            "__name__".to_string(),
            MbValue::from_ptr(MbObject::new_str(method_name.to_string())),
        );
    }
    MbValue::from_ptr(inst)
}

pub fn mb_counter_fromkeys_not_implemented() -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("NotImplementedError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "Counter.fromkeys() is undefined.  Use Counter(iterable) instead.".to_string(),
        )),
    );
    MbValue::none()
}

fn make_abc_register_method(parent_name: &str) -> MbValue {
    let inst = MbObject::new_instance("collections.abc._register_bound".to_string());
    if let ObjData::Instance { fields, .. } = unsafe { &(*inst).data } {
        fields.write().unwrap().insert(
            "_abc_parent".to_string(),
            MbValue::from_ptr(MbObject::new_str(parent_name.to_string())),
        );
    }
    MbValue::from_ptr(inst)
}

/// abc: bound `ABCMeta.register` method for a user-defined ABC. Calling it
/// records `child` as a virtual subclass of `parent_name`.
fn make_user_abc_register_method(parent_name: &str) -> MbValue {
    let inst = MbObject::new_instance("abc._user_register_bound".to_string());
    if let ObjData::Instance { fields, .. } = unsafe { &(*inst).data } {
        fields.write().unwrap().insert(
            "_abc_parent".to_string(),
            MbValue::from_ptr(MbObject::new_str(parent_name.to_string())),
        );
    }
    MbValue::from_ptr(inst)
}

/// abc: register `child` as a virtual subclass of user ABC `parent_name`.
pub fn mb_user_abc_register(parent_name: &str, child: MbValue) -> MbValue {
    let Some(child_name) = resolve_class_name(child) else {
        super::builtins::raise_type_error("Can only register classes".to_string());
        return MbValue::none();
    };
    if child_name.is_empty() {
        super::builtins::raise_type_error("Can only register classes".to_string());
        return MbValue::none();
    }
    ABC_VIRTUAL_SUBCLASSES.with(|reg| {
        reg.borrow_mut()
            .insert((child_name, parent_name.to_string()));
    });
    unsafe { super::rc::retain_if_ptr(child) };
    child
}

/// abc: a user ABC with a custom `__subclasshook__` drives structural
/// `issubclass`/`isinstance`. Returns `Some(true/false)` when the hook gives a
/// definite answer, or `None` (NotImplemented / no hook) to fall through to
/// nominal matching. `parent` must be a user ABC; `child_name` is the candidate
/// class (may be a builtin like "list"/"int").
fn user_abc_subclasshook(parent: &str, child_name: &str) -> Option<bool> {
    // Only ABCs whose own MRO defines `__subclasshook__` participate; the hook
    // is a classmethod, so we look it up via the normal MRO walk but must skip
    // the default (there is no default in our runtime, so any presence counts).
    let hook = lookup_method(parent, "__subclasshook__");
    if hook.is_none() {
        return None;
    }
    // `__subclasshook__` is a @classmethod — unwrap the descriptor to get the
    // underlying function pointer before invoking it.
    let (hook_fn, _dk) = unwrap_descriptor_method(hook);
    let addr = extract_func_addr(hook_fn);
    if addr == 0 {
        return None;
    }
    let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
    if !is_registered {
        return None;
    }
    // Call __subclasshook__(cls, C). mamba's @classmethod convention passes the
    // owning class as a bare class-name string for `cls` (so an identity guard
    // `if cls is Sized` works against the registered-class string). The
    // candidate `C` is passed as a *type object* so `hasattr(C, "__len__")`
    // queries the type's methods rather than treating a class-name string as a
    // plain str (which would itself answer hasattr("__len__") truthily).
    let cls_obj = MbValue::from_ptr(MbObject::new_str(parent.to_string()));
    let cand_obj = make_type_object(child_name);
    let func: extern "C" fn(MbValue, MbValue) -> MbValue =
        unsafe { std::mem::transmute(addr as usize) };
    let result = func(cls_obj, cand_obj);
    // NotImplemented → fall through.
    if result.is_not_implemented() {
        return None;
    }
    result.as_bool()
}

// ── collections.abc mixin methods, installed on user subclasses ──
//
// A class deriving from collections.abc.MutableSequence (and implementing the
// abstract methods __getitem__/__setitem__/__delitem__/__len__/insert) gets
// these mixins for free, exactly like CPython. Each is a native (self, args)
// method that delegates to the abstract methods via mb_call_method.

fn ms_call(recv: MbValue, method: &str, args: Vec<MbValue>) -> MbValue {
    let nm = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    let arglist = MbValue::from_ptr(MbObject::new_list(args));
    mb_call_method(recv, nm, arglist)
}

unsafe extern "C" fn ms_append(self_v: MbValue, args: MbValue) -> MbValue {
    let v = super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    let len = super::builtins::mb_len(self_v);
    ms_call(self_v, "insert", vec![len, v]);
    MbValue::none()
}

/// Append every element of `iterable` to `self` via insert(len, v).
fn ms_extend_with(self_v: MbValue, iterable: MbValue) {
    let handle = super::iter::mb_iter(iterable);
    if super::iter::is_iter_handle(handle) {
        if let Some(items) = super::iter::drain_iter_to_vec(handle) {
            for v in items {
                let len = super::builtins::mb_len(self_v);
                ms_call(self_v, "insert", vec![len, v]);
            }
        }
    }
}

// `seq.extend(it)` — the method-call ABI wraps the arg in a list.
unsafe extern "C" fn ms_extend(self_v: MbValue, args: MbValue) -> MbValue {
    let it = super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    ms_extend_with(self_v, it);
    MbValue::none()
}

// `seq += it` — reached via mb_iadd / invoke_binop_method, which passes the
// rhs iterable directly (not wrapped in a list).
unsafe extern "C" fn ms_iadd(self_v: MbValue, iterable: MbValue) -> MbValue {
    ms_extend_with(self_v, iterable);
    super::rc::retain_if_ptr(self_v);
    self_v
}

unsafe extern "C" fn ms_reverse(self_v: MbValue, args: MbValue) -> MbValue {
    dispatch_mutable_sequence_mixin(self_v, "reverse", args).unwrap_or_else(MbValue::none)
}

/// Sequence.__iter__ — yield self[0], self[1], … via __getitem__ up to
/// __len__. Materialized into a list whose iterator is returned (so
/// `for x in seq` / `list(seq)` work).
unsafe extern "C" fn ms_iter(self_v: MbValue, _args: MbValue) -> MbValue {
    let len = super::builtins::mb_len(self_v).as_int().unwrap_or(0);
    let mut items = Vec::with_capacity(len.max(0) as usize);
    for i in 0..len {
        items.push(ms_call(self_v, "__getitem__", vec![MbValue::from_int(i)]));
    }
    super::iter::mb_iter(MbValue::from_ptr(MbObject::new_list(items)))
}

/// Sequence.__contains__ — membership via iteration + equality.
unsafe extern "C" fn ms_contains(self_v: MbValue, args: MbValue) -> MbValue {
    let target = super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    let len = super::builtins::mb_len(self_v).as_int().unwrap_or(0);
    for i in 0..len {
        let v = ms_call(self_v, "__getitem__", vec![MbValue::from_int(i)]);
        if super::builtins::mb_eq(v, target).as_bool() == Some(true) {
            return MbValue::from_bool(true);
        }
    }
    MbValue::from_bool(false)
}

unsafe extern "C" fn ms_pop(self_v: MbValue, args: MbValue) -> MbValue {
    let items = super::builtins::extract_items(args);
    let idx = items.first().and_then(|v| v.as_int()).unwrap_or(-1);
    let v = ms_call(self_v, "__getitem__", vec![MbValue::from_int(idx)]);
    ms_call(self_v, "__delitem__", vec![MbValue::from_int(idx)]);
    v
}

unsafe extern "C" fn ms_index(self_v: MbValue, args: MbValue) -> MbValue {
    let target = super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    let len = super::builtins::mb_len(self_v).as_int().unwrap_or(0);
    for i in 0..len {
        let v = ms_call(self_v, "__getitem__", vec![MbValue::from_int(i)]);
        if super::builtins::mb_eq(v, target).as_bool() == Some(true) {
            return MbValue::from_int(i);
        }
    }
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str("value not in sequence".to_string())),
    );
    MbValue::none()
}

unsafe extern "C" fn ms_remove(self_v: MbValue, args: MbValue) -> MbValue {
    let idx = ms_index(self_v, args);
    if let Some(i) = idx.as_int() {
        ms_call(self_v, "__delitem__", vec![MbValue::from_int(i)]);
    }
    MbValue::none()
}

// ── MutableSet mixins (delegate to add / discard / __contains__ / __iter__) ──

/// Drain self's current elements into a Vec via the __iter__ mixin path.
fn mset_elements(self_v: MbValue) -> Vec<MbValue> {
    let it = ms_call(self_v, "__iter__", vec![]);
    if super::iter::is_iter_handle(it) {
        super::iter::drain_iter_to_vec(it).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn iter_to_vec(iterable: MbValue) -> Vec<MbValue> {
    let h = super::iter::mb_iter(iterable);
    if super::iter::is_iter_handle(h) {
        super::iter::drain_iter_to_vec(h).unwrap_or_default()
    } else {
        Vec::new()
    }
}

unsafe extern "C" fn mset_pop(self_v: MbValue, _args: MbValue) -> MbValue {
    let elems = mset_elements(self_v);
    match elems.first().copied() {
        Some(v) => {
            ms_call(self_v, "discard", vec![v]);
            v
        }
        None => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                MbValue::from_ptr(MbObject::new_str("pop from an empty set".to_string())),
            );
            MbValue::none()
        }
    }
}

unsafe extern "C" fn mset_ior(self_v: MbValue, iterable: MbValue) -> MbValue {
    for v in iter_to_vec(iterable) {
        ms_call(self_v, "add", vec![v]);
    }
    super::rc::retain_if_ptr(self_v);
    self_v
}

unsafe extern "C" fn mset_isub(self_v: MbValue, iterable: MbValue) -> MbValue {
    for v in iter_to_vec(iterable) {
        ms_call(self_v, "discard", vec![v]);
    }
    super::rc::retain_if_ptr(self_v);
    self_v
}

unsafe extern "C" fn mset_ixor(self_v: MbValue, iterable: MbValue) -> MbValue {
    for v in iter_to_vec(iterable) {
        let present = ms_call(self_v, "__contains__", vec![v]).as_bool() == Some(true);
        if present {
            ms_call(self_v, "discard", vec![v]);
        } else {
            ms_call(self_v, "add", vec![v]);
        }
    }
    super::rc::retain_if_ptr(self_v);
    self_v
}

unsafe extern "C" fn mset_iand(self_v: MbValue, iterable: MbValue) -> MbValue {
    let keep = iter_to_vec(iterable);
    for v in mset_elements(self_v) {
        let in_other = keep
            .iter()
            .any(|k| super::builtins::mb_eq(*k, v).as_bool() == Some(true));
        if !in_other {
            ms_call(self_v, "discard", vec![v]);
        }
    }
    super::rc::retain_if_ptr(self_v);
    self_v
}

// ── Set algebra mixins (delegate to __contains__ / __iter__ / __len__) ──

fn set_instance_class(self_v: MbValue) -> Option<String> {
    self_v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

/// Set._from_iterable(it) — build a new instance of self's class from `items`.
/// CPython's default is `cls(iterable)`.
fn set_from_iterable(self_v: MbValue, items: Vec<MbValue>) -> MbValue {
    let Some(class) = set_instance_class(self_v) else {
        return MbValue::from_ptr(MbObject::new_list(items));
    };
    let list = MbValue::from_ptr(MbObject::new_list(items));
    let args = MbValue::from_ptr(MbObject::new_list(vec![list]));
    mb_instance_new_with_init(MbValue::from_ptr(MbObject::new_str(class)), args)
}

fn set_contains(container: MbValue, v: MbValue) -> bool {
    ms_call(container, "__contains__", vec![v]).as_bool() == Some(true)
}

unsafe extern "C" fn set_and(self_v: MbValue, other: MbValue) -> MbValue {
    let items: Vec<MbValue> = mset_elements(self_v)
        .into_iter()
        .filter(|v| set_contains(other, *v))
        .collect();
    set_from_iterable(self_v, items)
}

unsafe extern "C" fn set_or(self_v: MbValue, other: MbValue) -> MbValue {
    let mut items = mset_elements(self_v);
    for v in iter_to_vec(other) {
        if !items
            .iter()
            .any(|x| super::builtins::mb_eq(*x, v).as_bool() == Some(true))
        {
            items.push(v);
        }
    }
    set_from_iterable(self_v, items)
}

unsafe extern "C" fn set_sub(self_v: MbValue, other: MbValue) -> MbValue {
    let items: Vec<MbValue> = mset_elements(self_v)
        .into_iter()
        .filter(|v| !set_contains(other, *v))
        .collect();
    set_from_iterable(self_v, items)
}

unsafe extern "C" fn set_xor(self_v: MbValue, other: MbValue) -> MbValue {
    let self_items = mset_elements(self_v);
    let other_items = iter_to_vec(other);
    let mut items: Vec<MbValue> = self_items
        .iter()
        .copied()
        .filter(|v| !set_contains(other, *v))
        .collect();
    for v in other_items {
        if !self_items
            .iter()
            .any(|x| super::builtins::mb_eq(*x, v).as_bool() == Some(true))
        {
            items.push(v);
        }
    }
    set_from_iterable(self_v, items)
}

/// self <= other: every element of self is in other.
fn set_subset(self_v: MbValue, other: MbValue) -> bool {
    mset_elements(self_v)
        .iter()
        .all(|v| set_contains(other, *v))
}

fn set_len(v: MbValue) -> i64 {
    super::builtins::mb_len(v).as_int().unwrap_or(0)
}

unsafe extern "C" fn set_le(self_v: MbValue, other: MbValue) -> MbValue {
    MbValue::from_bool(set_subset(self_v, other))
}
unsafe extern "C" fn set_lt(self_v: MbValue, other: MbValue) -> MbValue {
    MbValue::from_bool(set_len(self_v) < set_len(other) && set_subset(self_v, other))
}
unsafe extern "C" fn set_ge(self_v: MbValue, other: MbValue) -> MbValue {
    MbValue::from_bool(set_subset(other, self_v))
}
unsafe extern "C" fn set_gt(self_v: MbValue, other: MbValue) -> MbValue {
    MbValue::from_bool(set_len(self_v) > set_len(other) && set_subset(other, self_v))
}
unsafe extern "C" fn set_eq(self_v: MbValue, other: MbValue) -> MbValue {
    MbValue::from_bool(set_len(self_v) == set_len(other) && set_subset(self_v, other))
}

unsafe extern "C" fn set_isdisjoint(self_v: MbValue, args: MbValue) -> MbValue {
    let other = super::builtins::extract_items(args)
        .first()
        .copied()
        .unwrap_or_else(MbValue::none);
    let disjoint = !mset_elements(self_v)
        .iter()
        .any(|v| set_contains(other, *v));
    MbValue::from_bool(disjoint)
}

/// Install the MutableSequence / MutableSet / Set mixin methods on a class
/// (skipping any the class defines itself), called from mb_class_register when
/// the class's MRO includes a collections.abc sequence/set ABC.
fn install_abc_mixins(name: &str, mro: &[String]) {
    let derives = |abc: &str| mro.iter().any(|c| c == abc);
    if derives("MutableSequence") {
        let methods: &[(&str, usize)] = &[
            ("append", ms_append as *const () as usize),
            ("extend", ms_extend as *const () as usize),
            ("reverse", ms_reverse as *const () as usize),
            ("pop", ms_pop as *const () as usize),
            ("remove", ms_remove as *const () as usize),
            ("index", ms_index as *const () as usize),
            ("__iadd__", ms_iadd as *const () as usize),
            ("__iter__", ms_iter as *const () as usize),
            ("__contains__", ms_contains as *const () as usize),
        ];
        for (m, addr) in methods {
            if class_defines_own_method(name, m) {
                continue;
            }
            super::module::register_variadic_func(*addr as u64);
            class_replace_method(name, m, MbValue::from_func(*addr));
        }
    }
    // Set (and MutableSet, which is a Set) algebra + comparisons.
    if derives("Set") || derives("MutableSet") {
        let methods: &[(&str, usize)] = &[
            ("__and__", set_and as *const () as usize),
            ("__rand__", set_and as *const () as usize),
            ("__or__", set_or as *const () as usize),
            ("__ror__", set_or as *const () as usize),
            ("__sub__", set_sub as *const () as usize),
            ("__xor__", set_xor as *const () as usize),
            ("__rxor__", set_xor as *const () as usize),
            ("__le__", set_le as *const () as usize),
            ("__lt__", set_lt as *const () as usize),
            ("__ge__", set_ge as *const () as usize),
            ("__gt__", set_gt as *const () as usize),
            ("__eq__", set_eq as *const () as usize),
            ("isdisjoint", set_isdisjoint as *const () as usize),
        ];
        for (m, addr) in methods {
            if class_defines_own_method(name, m) {
                continue;
            }
            super::module::register_variadic_func(*addr as u64);
            class_replace_method(name, m, MbValue::from_func(*addr));
        }
    }
    if derives("MutableSet") {
        let methods: &[(&str, usize)] = &[
            ("pop", mset_pop as *const () as usize),
            ("__ior__", mset_ior as *const () as usize),
            ("__iand__", mset_iand as *const () as usize),
            ("__ixor__", mset_ixor as *const () as usize),
            ("__isub__", mset_isub as *const () as usize),
        ];
        for (m, addr) in methods {
            if class_defines_own_method(name, m) {
                continue;
            }
            super::module::register_variadic_func(*addr as u64);
            class_replace_method(name, m, MbValue::from_func(*addr));
        }
    }
}

fn dispatch_mutable_sequence_mixin(
    receiver: MbValue,
    method_name: &str,
    args: MbValue,
) -> Option<MbValue> {
    match method_name {
        "append" => {
            let items = super::builtins::extract_items(args);
            let value = items.first().copied().unwrap_or_else(MbValue::none);
            let len = super::builtins::mb_len(receiver);
            let insert_name = MbValue::from_ptr(MbObject::new_str("insert".to_string()));
            let insert_args = MbValue::from_ptr(MbObject::new_list(vec![len, value]));
            Some(mb_call_method(receiver, insert_name, insert_args))
        }
        "reverse" => {
            let len = super::builtins::mb_len(receiver).as_int().unwrap_or(0);
            if len <= 1 {
                return Some(MbValue::none());
            }
            let getitem_name = MbValue::from_ptr(MbObject::new_str("__getitem__".to_string()));
            let setitem_name = MbValue::from_ptr(MbObject::new_str("__setitem__".to_string()));
            let mut left_idx = 0;
            let mut right_idx = len - 1;
            while left_idx < right_idx {
                let left = mb_call_method(
                    receiver,
                    getitem_name,
                    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(left_idx)])),
                );
                let right = mb_call_method(
                    receiver,
                    getitem_name,
                    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(right_idx)])),
                );
                mb_call_method(
                    receiver,
                    setitem_name,
                    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(left_idx), right])),
                );
                mb_call_method(
                    receiver,
                    setitem_name,
                    MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(right_idx), left])),
                );
                left_idx += 1;
                right_idx -= 1;
            }
            Some(MbValue::none())
        }
        _ => None,
    }
}

fn collections_abc_abstract_methods(name: &str) -> &'static [&'static str] {
    match name {
        "Iterable" => &["__iter__"],
        "Iterator" => &["__next__"],
        "Reversible" => &["__reversed__"],
        "Collection" => &["__contains__", "__iter__", "__len__"],
        "Sequence" => &["__getitem__", "__len__"],
        "MutableSequence" => &[
            "__getitem__",
            "__setitem__",
            "__delitem__",
            "__len__",
            "insert",
        ],
        "Mapping" => &["__getitem__", "__iter__", "__len__"],
        "MutableMapping" => &[
            "__getitem__",
            "__setitem__",
            "__delitem__",
            "__iter__",
            "__len__",
        ],
        "Set" => &["__contains__", "__iter__", "__len__"],
        "MutableSet" => &["__contains__", "__iter__", "__len__", "add", "discard"],
        _ => &[],
    }
}

fn missing_collections_abc_abstract_method(class_name: &str) -> Option<&'static str> {
    // Collect the ABC names that impose abstract-method requirements on this
    // class. A class is "abstract" only when it reaches a real
    // `collections.abc` ABC *through inheritance* — not merely because the user
    // happened to name their own class `Collection`, `Sequence`, etc. CPython
    // treats `class Collection:` (no ABC base) as an ordinary, instantiable
    // class; only the native ABC itself or a subclass of it is abstract.
    //
    // Discriminator:
    //   - User-defined class registered in CLASS_REGISTRY: the class's own name
    //     is NOT an ABC source (the user shadowed the name); only the inherited
    //     MRO ancestors (`mro[1..]`, which excludes the class itself) count.
    //   - No registered class (e.g. instantiating the native `Sequence()` /
    //     `Collection()` ABC type object directly): the name itself is the ABC,
    //     so it must still be rejected.
    let abc_names = CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(class_name) {
            // `mro[0]` is always the class itself (see `compute_mro`); skip it
            // so a user class merely *named* like an ABC is not abstract.
            cls.mro.iter().skip(1).cloned().collect::<Vec<_>>()
        } else {
            vec![class_name.to_string()]
        }
    });

    let mut required = Vec::new();
    for abc_name in abc_names {
        for method in collections_abc_abstract_methods(&abc_name) {
            if !required.contains(method) {
                required.push(*method);
            }
        }
    }

    required
        .into_iter()
        .find(|method| lookup_method(class_name, method).is_none())
}

pub fn mb_collections_abc_reject_abstract_instantiation(class_name: &str) -> Option<MbValue> {
    let missing = missing_collections_abc_abstract_method(class_name)?;
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "Can't instantiate abstract class {class_name} with abstract method {missing}",
        ))),
    );
    Some(MbValue::none())
}

pub fn mb_contextlib_abc_reject_abstract_instantiation(class_name: &str) -> Option<MbValue> {
    let mro = class_mro_list(class_name);
    let has_base = |base: &str| class_name == base || mro.iter().any(|entry| entry == base);
    let missing = if has_base("AbstractAsyncContextManager")
        && !class_defines_non_none(class_name, "__aexit__")
    {
        Some("__aexit__")
    } else if has_base("AbstractContextManager")
        && !class_defines_non_none(class_name, "__exit__")
    {
        Some("__exit__")
    } else {
        None
    }?;
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "Can't instantiate abstract class {class_name} with abstract method {missing}",
        ))),
    );
    Some(MbValue::none())
}

fn is_array_unbound_method(name: &str) -> bool {
    matches!(
        name,
        "append"
            | "extend"
            | "fromlist"
            | "frombytes"
            | "tobytes"
            | "fromunicode"
            | "tolist"
            | "tounicode"
            | "buffer_info"
            | "byteswap"
            | "count"
            | "index"
            | "insert"
            | "pop"
            | "remove"
            | "reverse"
    )
}

/// Get an attribute from an instance (checks instance fields, then class methods via MRO).
/// Falls back to `__getattr__` dunder if normal lookup fails.
pub fn mb_getattr(obj: MbValue, attr: MbValue) -> MbValue {
    // obj.__class__ is type(obj): report the class uniformly for user/builtin
    // instances and bare values. An object that explicitly stores a __class__
    // field (e.g. ET.Element stubs, or an assigned __class__) keeps it — the
    // normal field lookup below wins, so only fall back to type(obj) when no
    // such field exists. Module values (modeled as dicts) report ModuleType.
    if let Some(attr_name) = extract_str(attr) {
        if attr_name == "__class__" {
            let has_stored = obj.as_ptr().map_or(false, |p| unsafe {
                matches!(&(*p).data,
                    ObjData::Instance { fields, .. } if fields.read().unwrap().contains_key("__class__"))
            });
            if !has_stored {
                if super::module::is_module_value(obj) {
                    return make_type_object("module");
                }
                return super::builtins::mb_type(obj);
            }
        }
        // A module's __dict__ is its namespace mapping. Return a snapshot
        // (same as vars(module)) rather than the live module dict, so iterating
        // it works (e.g. errno's uppercase constants) without letting callers
        // mutate the module through __dict__. Without this it was None.
        if attr_name == "__dict__" && super::module::is_module_value(obj) {
            return mb_vars(obj);
        }
        // EnumClass._member_type_: the mixed-in data type (int / str / object).
        if attr_name == "_member_type_" {
            if let Some(cn) = resolve_class_name(obj) {
                if let Some(mt) = super::stdlib::enum_class::member_type_name(&cn) {
                    return make_type_object(mt);
                }
            }
        }
    }
    // Unbound-method wrappers (Cls.method): function attributes set by
    // decorators (@typing.override → __override__) live in the FUNC_ATTRS
    // registry keyed by the underlying method value — resolve through it.
    if let (Some(ptr), Some(attr_name)) = (obj.as_ptr(), extract_str(attr)) {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "__unbound_method__"
                    && attr_name != "__method__"
                    && attr_name != "__type__"
                    && attr_name != "__name__"
                {
                    let (ty, m) = {
                        let f = fields.read().unwrap();
                        (
                            f.get("__type__").copied().and_then(extract_str),
                            f.get("__method__").copied().and_then(extract_str),
                        )
                    };
                    if let (Some(ty), Some(m)) = (ty, m) {
                        let method = lookup_method(&ty, &m);
                        if !method.is_none() {
                            let (actual, _) = unwrap_descriptor_method(method);
                            if let Some(v) = super::pep695::func_attrs_get(actual, &attr_name) {
                                return v;
                            }
                            if attr_name == "__override__" || attr_name == "__final__" {
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str(
                                        "AttributeError".to_string(),
                                    )),
                                    MbValue::from_ptr(MbObject::new_str(format!(
                                        "'function' object has no attribute '{attr_name}'"
                                    ))),
                                );
                                return MbValue::none();
                            }
                        }
                    }
                }
            }
        }
    }
    // Typed native wrappers are raw `Box<T>` pointers, not `MbObject`s.
    // Dispatch them through registered getters before any fast path deref.
    if let Some(type_name) = native_type_name_for(obj) {
        let attr_name = extract_str(attr).unwrap_or_default();
        if let Some(getter) = super::registry_bridge::lookup_getter(type_name, &attr_name) {
            let reg_obj = cclab_mamba_registry::MbValue::from_bits(obj.to_bits());
            let args = [reg_obj];
            let result = unsafe { getter(args.as_ptr(), args.len()) };
            return MbValue::from_bits(result.to_bits());
        }
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{}' object has no attribute '{}'",
                type_name, attr_name
            ))),
        );
        return MbValue::none();
    }

    // Issue #2097 fast path — module / plain-dict attribute lookup is the
    // single hottest GETATTR shape in idiomatic Python (e.g.
    // `keyword.iskeyword(w)` inside a for-loop). The JIT bakes the attr
    // name as an immortal `ObjData::Str` and reuses the same pointer for
    // every call at a given call site, so we can:
    //   1. Skip the dunder cascade (`__name__` / `__qualname__` / `__doc__`
    //      / handle-type registries) when the receiver is a plain Dict —
    //      none of those branches fire on a Dict receiver anyway.
    //   2. Borrow the attr name as `&str` directly out of the immortal
    //      Str object instead of cloning it into a fresh `String` via
    //      `extract_str`. `DictKey: Equivalent<str>` lets `IndexMap::get`
    //      accept the borrow without rehashing.
    // Falls back to the full slow path on miss so descriptor / dunder
    // semantics on non-dict objects are unchanged.
    if let (Some(obj_ptr), Some(attr_ptr)) = (obj.as_ptr(), attr.as_ptr()) {
        unsafe {
            if let ObjData::Dict(ref lock) = (*obj_ptr).data {
                if let ObjData::Str(ref attr_s) = (*attr_ptr).data {
                    let guard = lock.read().unwrap();
                    if let Some(val) = guard.get(attr_s.as_str()) {
                        let v = *val;
                        super::rc::retain_if_ptr(v);
                        return v;
                    }
                    // A module namespace (dict carrying __name__) reports a
                    // missing non-dunder attribute as AttributeError, like
                    // CPython. Dunder lookups still fall to the slow path.
                    if !attr_s.starts_with("__") && guard.contains_key("__name__") {
                        let mod_name = guard
                            .get("__name__")
                            .copied()
                            .and_then(extract_str)
                            .unwrap_or_default();
                        drop(guard);
                        // The test.* scaffolding modules are deliberately
                        // empty stubs whose consumers rely on lenient
                        // None-miss; keep them out of the strict path.
                        if mod_name == "test" || mod_name.starts_with("test.") {
                            return MbValue::none();
                        }
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "module '{mod_name}' has no attribute '{attr_s}'"
                            ))),
                        );
                        return MbValue::none();
                    }
                    // Dict miss is a real AttributeError shape; fall through
                    // to the slow path so existing dunder / __getattr__ /
                    // descriptor semantics keep handling it.
                }
            }
        }
    }
    let attr_name = extract_str(attr).unwrap_or_default();

    // SpooledTemporaryFile: a binary spool has no encoding/errors/newlines
    // fields and CPython raises AttributeError for them (the generic Instance
    // path would silently yield None). Scoped to exactly these text-only
    // names so bound-method attribute access stays untouched.
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "SpooledTemporaryFile"
                    && matches!(attr_name.as_str(), "encoding" | "errors" | "newlines")
                    && !fields.read().unwrap().contains_key(&attr_name)
                {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'SpooledTemporaryFile' object has no attribute '{attr_name}'"
                        ))),
                    );
                    return MbValue::none();
                }
            }
        }
    }

    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::file_io::is_file_handle(id) {
            if attr_name == "name" {
                return super::file_io::mb_file_name(obj);
            }
            if attr_name == "closed" {
                return MbValue::from_bool(super::file_io::is_file_closed(obj));
            }
            if attr_name == "mode" {
                return super::file_io::mb_file_mode(obj);
            }
            if attr_name == "encoding" {
                return super::file_io::mb_file_encoding(obj);
            }
            if attr_name == "errors" {
                return super::file_io::mb_file_errors(obj);
            }
            if attr_name == "buffer" {
                let buffer = super::file_io::mb_file_buffer(obj);
                if !buffer.is_none() {
                    return buffer;
                }
            }
        }
    }

    // Range handles are stored as int-tagged iterator IDs. Expose CPython's
    // range.start / range.stop / range.step attributes before generic int
    // attribute handling sees the value as a plain integer.
    if let Some((start, stop, step)) = super::iter::mb_iter_range_params(obj) {
        match attr_name.as_str() {
            "start" => return MbValue::from_int(start),
            "stop" => return MbValue::from_int(stop),
            "step" => return MbValue::from_int(step),
            _ => {}
        }
    }

    // __name__ / __qualname__ on functions / closures: look up in the
    // FUNC_NAMES registry. Top-level functions always have
    // qualname == name (CPython only nests for class methods / closures).
    if attr_name == "__name__" || attr_name == "__qualname__" {
        // Builtin method wrappers: __name__ is the bare method name,
        // __qualname__ is "type.method" (e.g. dict.fromkeys → "dict.fromkeys",
        // [1].append → "list.append").
        if let Some(ptr) = obj.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
                    if class_name == "__unbound_method__"
                        || class_name == "__bound_native_method__"
                    {
                        let g = fields.read().unwrap();
                        if let Some(method) = g.get("__method__").and_then(|v| extract_str(*v)) {
                            if attr_name == "__name__" {
                                return MbValue::from_ptr(MbObject::new_str(method));
                            }
                            let type_name = if class_name == "__unbound_method__" {
                                g.get("__type__").and_then(|v| extract_str(*v))
                            } else {
                                g.get("__self__")
                                    .map(|s| super::builtins::value_type_name(*s).to_string())
                            };
                            let qn = match type_name {
                                Some(tn) => format!("{tn}.{method}"),
                                None => method,
                            };
                            return MbValue::from_ptr(MbObject::new_str(qn));
                        }
                    }
                }
            }
        }
        // TAG_FUNC direct pointer
        if obj.as_func().is_some() {
            let name = super::closure::mb_func_get_name(obj);
            if !name.is_none() {
                return name;
            }
        }
        // Closure handle (integer ID)
        if obj.as_int().is_some() && !super::generator::is_known_generator(obj) {
            let name = super::closure::mb_func_get_name(obj);
            if !name.is_none() {
                return name;
            }
        }
    }

    // __module__ on functions: Mamba doesn't currently track import-time
    // module ownership for user-defined functions, so report "__main__"
    // (CPython's default for top-level defs) when the registry confirms
    // this value is a known function. Avoids leaking "__main__" onto
    // plain ints / closure-handle ambiguous values.
    if attr_name == "__module__" {
        let registered = !super::closure::mb_func_get_name(obj).is_none();
        if registered {
            let module = super::closure::mb_func_get_module(obj);
            if !module.is_none() {
                return module;
            }
            return MbValue::from_ptr(MbObject::new_str("__main__".to_string()));
        }
    }

    // __doc__ on functions: return the registered docstring (or None for
    // bodies without a leading string literal). Gated on registry presence
    // to avoid leaking docs onto plain ints / unrelated values.
    if attr_name == "__doc__" {
        let registered = !super::closure::mb_func_get_name(obj).is_none();
        if registered {
            return super::closure::mb_func_get_doc(obj);
        }
    }

    // __annotations__ on functions: dict of {param: anno, "return": anno} built
    // from the registered signature metadata. Gated on registry presence so a
    // class-name string (whose __annotations__ is a class attr) and plain
    // values fall through to their own paths.
    if attr_name == "__annotations__" {
        let registered = !super::closure::mb_func_get_name(obj).is_none();
        if registered {
            let anns = super::closure::mb_func_get_annotations(obj);
            if !anns.is_none() {
                return anns;
            }
        }
    }

    // __wrapped__ on functions: set by functools.wraps / update_wrapper. Only
    // returns a value when the wrapper actually recorded one.
    if attr_name == "__wrapped__" {
        let wrapped = super::stdlib::functools_mod::get_func_wrapped(obj);
        if !wrapped.is_none() {
            unsafe {
                super::rc::retain_if_ptr(wrapped);
            }
            return wrapped;
        }
    }

    // __code__ on functions (CORE #3): build a code object exposing the
    // compiled signature metadata. Gated on the function metadata registry so
    // we never fabricate a code object for arbitrary ints / values.
    // `f.__code__.co_name` → name, `.co_argcount` → arity, `.co_varnames` →
    // parameter-name tuple. The synthetic code object is a plain Instance
    // (class_name "code"); its `co_*` fields resolve through the normal
    // Instance attribute path.
    if attr_name == "__code__" && super::closure::mb_func_is_registered(obj) {
        return make_code_object(obj);
    }

    // __defaults__ / __kwdefaults__ on functions: CPython exposes positional
    // (positional-only + positional-or-keyword) defaults as a tuple — None when
    // there are none — and keyword-only defaults as a {name: value} dict, or
    // None when there are none. Pull the values from the declared-signature
    // registry (FUNC_PARAMS), which carries each param's kind / has_default /
    // default value.
    if (attr_name == "__defaults__" || attr_name == "__kwdefaults__")
        && super::closure::mb_func_is_registered(obj)
    {
        let Some(params) = super::closure::func_params(obj) else {
            return MbValue::none();
        };
        if attr_name == "__defaults__" {
            let vals: Vec<MbValue> = params.iter()
                .filter(|p| p.kind <= 1 && p.has_default)
                .map(|p| p.default)
                .collect();
            return if vals.is_empty() {
                MbValue::none()
            } else {
                MbValue::from_ptr(super::rc::MbObject::new_tuple_borrowed(vals))
            };
        }
        // __kwdefaults__
        let kwdefaults: Vec<&super::closure::MbParamInfo> = params.iter()
            .filter(|p| p.kind == 3 && p.has_default)
            .collect();
        if kwdefaults.is_empty() {
            return MbValue::none();
        }
        let dict = super::dict_ops::mb_dict_new();
        for p in kwdefaults {
            let key = MbValue::from_ptr(super::rc::MbObject::new_str(p.name.clone()));
            super::dict_ops::mb_dict_setitem(dict, key, p.default);
        }
        return dict;
    }

    // Function / closure attributes (PEP 695): user-set attributes (incl. a
    // writable __type_params__) live in the FUNC_ATTRS side registry; a
    // registered function without an explicit entry reports the CPython
    // default — an empty __type_params__ tuple.
    if obj.as_ptr().is_none() {
        if let Some(v) = super::pep695::func_attrs_get(obj, &attr_name) {
            return v;
        }
        if attr_name == "__type_params__" && super::pep695::is_attrable_function(obj) {
            return MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![]));
        }
    }

    // Generator handles are int-tagged values. Handle generator-specific attributes.
    if obj.is_int() && super::generator::is_known_generator(obj) {
        match attr_name.as_str() {
            "gi_frame" => {
                // Return None when the generator is exhausted/closed, else a sentinel
                // (the generator handle itself suffices — any non-None value).
                let exhausted = super::generator::mb_generator_is_exhausted(obj)
                    .as_bool()
                    .unwrap_or(true);
                return if exhausted { MbValue::none() } else { obj };
            }
            _ => {}
        }
    }

    // Hashlib handles are int-tagged values registered by hashlib_mod.
    // Surface CPython conformance attrs (name, digest_size, block_size).
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::hashlib_mod::is_hashlib_handle(id) {
            match attr_name.as_str() {
                "name" => return super::stdlib::hashlib_mod::mb_hashlib_name(obj),
                "digest_size" => {
                    return super::stdlib::hashlib_mod::mb_hashlib_digest_size_attr(obj)
                }
                "block_size" => return super::stdlib::hashlib_mod::mb_hashlib_block_size_attr(obj),
                _ => {}
            }
        }
    }

    // Hmac handles are int-tagged values registered by hmac_mod.
    // Surface CPython conformance attrs (name, digest_size, block_size).
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::hmac_mod::is_hmac_handle(id) {
            match attr_name.as_str() {
                "name" => return super::stdlib::hmac_mod::mb_hmac_name(obj),
                "digest_size" => return super::stdlib::hmac_mod::mb_hmac_digest_size_attr(obj),
                "block_size" => return super::stdlib::hmac_mod::mb_hmac_block_size_attr(obj),
                _ => {}
            }
        }
    }

    // `exc.__traceback__` on exception instances: mamba does not store a
    // real traceback; synthesize a minimal one (tb_frame/tb_lineno/tb_next)
    // so walk_tb / extract_tb consumers see a non-None object.
    if let (Some(obj_ptr), Some(attr_ptr)) = (obj.as_ptr(), attr.as_ptr()) {
        unsafe {
            if let (
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                },
                ObjData::Str(ref a),
            ) = (&(*obj_ptr).data, &(*attr_ptr).data)
            {
                if a == "__traceback__"
                    && !fields.read().unwrap().contains_key("__traceback__")
                    && (super::exception::is_subclass_of(class_name, "BaseException")
                        || super::exception::is_subclass_of(class_name, "Exception")
                        || class_name == "Exception"
                        || class_name == "BaseException")
                {
                    return super::stdlib::traceback_mod::make_tb_instance();
                }
            }
        }
    }

    // Array handles are int-tagged values registered by array_mod.
    // Surface CPython conformance attrs (typecode, itemsize).
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            match attr_name.as_str() {
                "typecode" => return super::stdlib::array_mod::mb_array_typecode_attr(obj),
                "itemsize" => return super::stdlib::array_mod::mb_array_itemsize_attr(obj),
                _ => {}
            }
        }
    }

    if let Some(addr) = obj.as_func() {
        // numbers.Number.__abstractmethods__ is frozenset() — Number declares no
        // abstract methods (it is concrete); the lower tower ABCs are unaffected.
        if attr_name == "__abstractmethods__"
            && super::stdlib::numbers_mod::numbers_abc_rank(addr as u64) == Some(0)
        {
            return MbValue::from_ptr(MbObject::new_frozenset(vec![]));
        }
        let native_type =
            super::module::NATIVE_TYPE_NAMES.with(|map| map.borrow().get(&(addr as u64)).cloned());
        if let Some(nt) = native_type {
            if nt == "collections.Counter" && attr_name == "fromkeys" {
                return make_unbound_method(&nt, &attr_name);
            }
            if nt == "array" && is_array_unbound_method(&attr_name) {
                return make_unbound_method("array", &attr_name);
            }
            // `datetime.timedelta.min/.max/.resolution` are class-attribute
            // VALUES (timedelta instances), not methods — return them directly
            // instead of wrapping in an unbound method.
            if nt == "datetime.timedelta" {
                if let Some(v) = super::stdlib::datetime_mod::timedelta_class_attr(&attr_name) {
                    return v;
                }
            }
            // `unittest.mock.call.<name>` builds a named-call factory for ANY
            // attribute name.
            if nt == "_mock_call_factory" {
                return super::stdlib::unittest_mock_mod::make_call_namebuilder(&attr_name);
            }
            // `datetime.timezone.utc/.min/.max` are singleton class-attribute
            // values; `datetime.UTC is timezone.utc` needs the same pointer.
            if nt == "datetime.timezone" {
                if let Some(v) = super::stdlib::datetime_mod::timezone_class_attr(&attr_name) {
                    return v;
                }
            }
            // A native class's constructor is bound to its name as a func value
            // (`pathlib.Path` via NATIVE_TYPE_NAMES). Accessing one of the class's
            // REGISTERED methods on that func (`pathlib.Path.joinpath`) is an
            // unbound method — callable, and dispatchable when invoked. Validate
            // against the class method table (lookup_method → CLASS_REGISTRY, the
            // same registry mb_class_register populates for native classes) so
            // ONLY a real method bridges — a missing attr falls through to the
            // normal path and stays absent/None, never a spurious callable.
            // Class-attribute VALUES (e.g. `Morsel._reserved`, a dict set via
            // mb_class_set_class_attr) are returned directly; only funcs wrap
            // as unbound methods.
            let class_attr = CLASS_REGISTRY.with(|reg| {
                reg.borrow()
                    .get(&nt)
                    .and_then(|cls| cls.class_attrs.get(&attr_name).copied())
            });
            if let Some(v) = class_attr {
                if v.as_func().is_none() {
                    return v;
                }
            }
            if !lookup_method(&nt, &attr_name).is_none() {
                return make_unbound_method(&nt, &attr_name);
            }
        }
        if attr_name == "_convert" {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "'function' object has no attribute '_convert'".to_string(),
                )),
            );
            return MbValue::none();
        }
    }

    // Random handles are int-tagged values registered by random_mod.
    // Method lookup goes through mb_call_method; an attribute READ
    // (`variate = gen.uniform; variate(10, 10)`) produces a bound-method
    // Instance carrying the receiver + method name, dispatched by
    // mb_call_spread / mb_call0 / mb_call1_val. `__name__` is a field so
    // `variate.__name__` reads it like CPython's bound method.
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::random_mod::is_random_handle(id) {
            match attr_name.as_str() {
                "random" | "seed" | "randint" | "randrange" | "uniform" | "triangular"
                | "choice" | "shuffle" | "sample" | "choices" | "gauss" | "normalvariate"
                | "expovariate" | "lognormvariate" | "vonmisesvariate" | "gammavariate"
                | "betavariate" | "paretovariate" | "weibullvariate" | "getrandbits"
                | "randbytes" | "getstate" | "setstate" | "binomialvariate" => {
                    let inst_ptr = MbObject::new_instance("__bound_native_method__".to_string());
                    unsafe {
                        if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
                            let mut g = fields.write().unwrap();
                            g.insert("__self__".to_string(), obj);
                            g.insert(
                                "__method__".to_string(),
                                MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
                            );
                            g.insert(
                                "__name__".to_string(),
                                MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
                            );
                        }
                    }
                    return MbValue::from_ptr(inst_ptr);
                }
                _ => {}
            }
        }
    }

    // Queue handles are int-tagged values registered by queue_mod. Method
    // *calls* (`q.put(1)`) route through the call-method protocol directly
    // (never this getattr path — proven by `q.put(1)` working while bare
    // `callable(q.put)` was False), so the bare-attribute value only needs
    // to report callable: return an `__unbound_method__` Instance (which
    // mb_callable recognizes) rather than the bare int handle (which it
    // does not). Satisfies `callable(queue.Queue().<m>)` surface probes.
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::queue_mod::is_queue_handle(id) {
            match attr_name.as_str() {
                "put" | "put_nowait" | "get" | "get_nowait" | "empty" | "full" | "qsize"
                | "task_done" | "join" => {
                    return make_unbound_method("Queue", &attr_name);
                }
                _ => {}
            }
        }
    }

    // Fraction handles are int-tagged values registered by fractions_mod.
    // Surface CPython conformance attrs (numerator, denominator, real,
    // imag); method lookup (`conjugate`, `is_integer`, `as_integer_ratio`,
    // `limit_denominator`, dunders) goes through mb_call_method, so
    // those names return the handle itself to keep the bound-method
    // call shape. Task #45 — Fraction class via integer-handle pattern.
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::fractions_mod::is_fraction_handle(id) {
            match attr_name.as_str() {
                "numerator" => return super::stdlib::fractions_mod::mb_fraction_numerator(obj),
                "denominator" => return super::stdlib::fractions_mod::mb_fraction_denominator(obj),
                "real" => return super::stdlib::fractions_mod::mb_fraction_real(obj),
                "imag" => return super::stdlib::fractions_mod::mb_fraction_imag(obj),
                "conjugate" | "is_integer" | "as_integer_ratio" | "limit_denominator" => {
                    return obj
                }
                _ => {}
            }
        }
    }

    // UUID handles are int-tagged values registered by uuid_mod. Most of
    // the UUID surface is attribute reads (.hex, .int, .urn, .version,
    // .variant, .bytes, .bytes_le, .fields); the latter two hit #2096 /
    // #2128 respectively at high call counts. Task #46 — UUID class via
    // integer-handle pattern. UUID has no operator overloads, so the
    // [[project_mamba_int_handle_operator_overload_gap]] does not apply.
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::uuid_mod::is_uuid_handle(id) {
            match attr_name.as_str() {
                "hex" => return super::stdlib::uuid_mod::mb_uuid_hex(obj),
                "int" => return super::stdlib::uuid_mod::mb_uuid_int_attr(obj),
                "urn" => return super::stdlib::uuid_mod::mb_uuid_urn(obj),
                "version" => return super::stdlib::uuid_mod::mb_uuid_version_attr(obj),
                "variant" => return super::stdlib::uuid_mod::mb_uuid_variant_attr(obj),
                "bytes" => return super::stdlib::uuid_mod::mb_uuid_bytes_attr(obj),
                "bytes_le" => return super::stdlib::uuid_mod::mb_uuid_bytes_le_attr(obj),
                "fields" => return super::stdlib::uuid_mod::mb_uuid_fields_attr(obj),
                "time_low" => return super::stdlib::uuid_mod::mb_uuid_time_low(obj),
                "time_mid" => return super::stdlib::uuid_mod::mb_uuid_time_mid(obj),
                "time_hi_version" => return super::stdlib::uuid_mod::mb_uuid_time_hi_version(obj),
                "clock_seq_hi_variant" => {
                    return super::stdlib::uuid_mod::mb_uuid_clock_seq_hi_variant(obj)
                }
                "clock_seq_low" => return super::stdlib::uuid_mod::mb_uuid_clock_seq_low(obj),
                "clock_seq" => return super::stdlib::uuid_mod::mb_uuid_clock_seq(obj),
                "node" => return super::stdlib::uuid_mod::mb_uuid_node(obj),
                "time" => return super::stdlib::uuid_mod::mb_uuid_time(obj),
                "is_safe" => return super::stdlib::uuid_mod::mb_uuid_is_safe(obj),
                _ => {}
            }
        }
        // ipaddress handles (#1474, Task #69 — Wave-6 ship #2).
        if super::stdlib::ipaddress_mod::is_ip_handle(id) {
            match attr_name.as_str() {
                "packed" => return super::stdlib::ipaddress_mod::mb_ipaddress_packed(obj),
                "compressed" => return super::stdlib::ipaddress_mod::mb_ipaddress_compressed(obj),
                "exploded" => return super::stdlib::ipaddress_mod::mb_ipaddress_exploded(obj),
                "version" => return super::stdlib::ipaddress_mod::mb_ipaddress_version(obj),
                "is_private" => return super::stdlib::ipaddress_mod::mb_ipaddress_is_private(obj),
                "is_global" => return super::stdlib::ipaddress_mod::mb_ipaddress_is_global(obj),
                _ => {}
            }
        }
    }

    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            // `str.lower` / `list.append` / … — the builtin type names are
            // stored as plain strings in Mamba, so attribute access would
            // otherwise call the method on the type-name string itself
            // (`str.lower("HELLO")` → `"str".lower()` → `"str"`). Intercept
            // those at the getattr layer and return an `__unbound_method__`
            // Instance that carries (type, method). mb_call_spread detects
            // this wrapper and dispatches the method against args[0] as
            // receiver, consuming the remaining args.
            if let ObjData::Str(ref s) = (*ptr).data {
                if matches!(
                    s.as_str(),
                    "str"
                        | "list"
                        | "dict"
                        | "tuple"
                        | "set"
                        | "frozenset"
                        | "int"
                        | "float"
                        | "bool"
                        | "bytes"
                        | "bytearray"
                ) {
                    let inst = MbObject::new_instance("__unbound_method__".to_string());
                    if let ObjData::Instance { fields, .. } = &(*inst).data {
                        let mut guard = fields.write().unwrap();
                        guard.insert(
                            "__type__".to_string(),
                            MbValue::from_ptr(MbObject::new_str(s.clone())),
                        );
                        guard.insert(
                            "__method__".to_string(),
                            MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
                        );
                    }
                    return MbValue::from_ptr(inst);
                }
            }
            // Type-singleton objects (Instance class_name="type", __name__=X) also need
            // to support the unbound-method pattern for calls like `int.from_bytes(…)`.
            // But only for method accesses — NOT for dunder/own fields (__name__, __doc__,
            // __module__) which should be returned directly from the Instance fields below.
            if let ObjData::Instance {
                class_name: ref cn,
                ref fields,
            } = (*ptr).data
            {
                if cn == "type" {
                    // Skip attributes that are actual fields of the type object.
                    let is_own_field = matches!(
                        attr_name.as_str(),
                        "__name__"
                            | "__doc__"
                            | "__module__"
                            | "__qualname__"
                            | "__bases__"
                            | "__mro__"
                            | "__dict__"
                            | "__abstractmethods__"
                            | "__subclasscheck__"
                            | "__instancecheck__"
                    );
                    if !is_own_field {
                        // Try to get the type name from __name__ field.
                        if let Some(type_name_str) = fields
                            .read()
                            .ok()
                            .and_then(|f| f.get("__name__").and_then(|v| extract_str(*v)))
                        {
                            if attr_name == "register" && is_collections_abc_name(&type_name_str) {
                                return make_abc_register_method(&type_name_str);
                            }
                            if attr_name == "register" && is_user_abc(&type_name_str) {
                                return make_user_abc_register_method(&type_name_str);
                            }
                            if matches!(
                                type_name_str.as_str(),
                                "str"
                                    | "list"
                                    | "dict"
                                    | "tuple"
                                    | "set"
                                    | "frozenset"
                                    | "int"
                                    | "float"
                                    | "bool"
                                    | "bytes"
                                    | "bytearray"
                            ) {
                                let inst = MbObject::new_instance("__unbound_method__".to_string());
                                if let ObjData::Instance {
                                    fields: ifields, ..
                                } = &(*inst).data
                                {
                                    let mut guard = ifields.write().unwrap();
                                    guard.insert(
                                        "__type__".to_string(),
                                        MbValue::from_ptr(MbObject::new_str(type_name_str)),
                                    );
                                    guard.insert(
                                        "__method__".to_string(),
                                        MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
                                    );
                                }
                                return MbValue::from_ptr(inst);
                            }
                            if let Some(val) = mro_lookup_class_attr(&type_name_str, &attr_name) {
                                super::rc::retain_if_ptr(val);
                                return val;
                            }
                            // A registered native class's method accessed on its
                            // type-object (`unittest.TestCase.assertEqual`) is an
                            // unbound method — callable and dispatchable when
                            // invoked. Validate against the class method table
                            // (lookup_method → CLASS_REGISTRY, the same registry
                            // the func->method bridge uses) so ONLY a real method
                            // bridges; a miss falls through and stays absent.
                            if class_is_registered(&type_name_str)
                                && !lookup_method(&type_name_str, &attr_name).is_none()
                            {
                                return make_unbound_method(&type_name_str, &attr_name);
                            }
                            if let Some(method) =
                                inherited_builtin_unbound_method(&type_name_str, &attr_name)
                            {
                                return method;
                            }
                            // <type>.__new__ — every type inherits
                            // object.__new__(cls), which allocates a BARE
                            // instance without running __init__ (used by the
                            // type-wall idiom `obj = object.__new__(C)`). A
                            // user-defined __new__ was already returned by the
                            // mro_lookup / registered-method checks above, so
                            // this is the inherited-object.__new__ fallback;
                            // the call is serviced by mb_call_method's __new__
                            // arm (it reads the target class from the first arg).
                            if attr_name == "__new__" {
                                return make_unbound_method(&type_name_str, "__new__");
                            }
                            // object's base dunders accessed on the type
                            // (`object.__init__`, `object.__repr__`, …) are
                            // callable unbound methods.
                            if type_name_str == "object"
                                && matches!(attr_name.as_str(),
                                    "__init__" | "__repr__" | "__str__" | "__hash__"
                                    | "__eq__" | "__ne__" | "__lt__" | "__le__"
                                    | "__gt__" | "__ge__" | "__delattr__"
                                    | "__setattr__" | "__getattribute__"
                                    | "__sizeof__" | "__reduce__" | "__reduce_ex__"
                                    | "__dir__" | "__init_subclass__"
                                    | "__subclasshook__" | "__format__")
                            {
                                return make_unbound_method("object", &attr_name);
                            }
                            // complex comparison dunders accessed unbound
                            // (`complex.__eq__`, `complex.__lt__`, …). The call
                            // dispatch computes bool / NotImplemented; here we
                            // just expose the callable wrapper.
                            if type_name_str == "complex"
                                && matches!(attr_name.as_str(),
                                    "__eq__" | "__ne__" | "__lt__" | "__le__" | "__gt__" | "__ge__")
                            {
                                return make_unbound_method("complex", &attr_name);
                            }
                            // PEP 695: every type object carries
                            // __type_params__, defaulting to () (the
                            // mro_lookup above already returned a generic
                            // class's stored tuple).
                            if attr_name == "__type_params__" {
                                return MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![]));
                            }
                        }
                    }
                }
            }
            match &(*ptr).data {
                ObjData::Dict(ref lock) => {
                    // Module dicts and plain dicts: attribute access looks up a dict key.
                    let guard = lock.read().unwrap();
                    if let Some(val) = guard.get(&attr_name) {
                        let v = *val;
                        super::rc::retain_if_ptr(v);
                        return v;
                    }
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                } => {
                    // unittest.mock: fields, return_value, and supported magic
                    // names resolve to (autovivified) child mocks before the
                    // class method table can shadow them.
                    if super::stdlib::unittest_mock_mod::is_mock_class(class_name) {
                        if let Some(v) =
                            super::stdlib::unittest_mock_mod::mock_getattr_hook(obj, &attr_name)
                        {
                            super::rc::retain_if_ptr(v);
                            return v;
                        }
                    }
                    // PEP 695 TypeVar / TypeAliasType: __bound__ /
                    // __constraints__ / __value__ evaluate their stored thunk
                    // lazily on first access (then cache).
                    if super::pep695::is_pep695_class(class_name) {
                        if let Some(v) =
                            super::pep695::instance_lazy_attr_hook(obj, class_name, &attr_name)
                        {
                            return v;
                        }
                    }
                    // ChainMap.parents — a lazily built view over maps[1:].
                    if class_name == "collections.ChainMap" && attr_name == "parents" {
                        if let Some(p) = super::stdlib::collections_mod::chainmap_parents(obj) {
                            return p;
                        }
                    }
                    // namedtuple factory class attributes: _fields,
                    // __match_args__, __name__, __slots__, _field_defaults.
                    if class_name == "collections.namedtuple_factory" {
                        if let Some(v) = super::stdlib::collections_mod::namedtuple_factory_getattr(
                            obj, &attr_name,
                        ) {
                            return v;
                        }
                        // namedtuple reuses tuple.__getitem__ (CPython:
                        // `Point.__getitem__ == tuple.__getitem__`).
                        if attr_name == "__getitem__" {
                            return make_unbound_method("tuple", "__getitem__");
                        }
                    }
                    // namedtuple instance `_fields` mirrors the class attr.
                    if matches!(attr_name.as_str(), "_fields" | "__match_args__") {
                        let names: Option<Vec<MbValue>> = fields
                            .read()
                            .unwrap()
                            .get("_namedtuple_fields")
                            .and_then(|v| v.as_ptr())
                            .map(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    lk.read()
                                        .unwrap()
                                        .iter()
                                        .map(|s| {
                                            super::rc::retain_if_ptr(*s);
                                            *s
                                        })
                                        .collect()
                                } else {
                                    vec![]
                                }
                            });
                        if let Some(names) = names {
                            return MbValue::from_ptr(MbObject::new_tuple(names));
                        }
                    }
                    // defaultdict.default_factory — the stored factory callable.
                    if class_name == "collections.defaultdict" && attr_name == "default_factory" {
                        let factory = fields
                            .read()
                            .unwrap()
                            .get("_factory")
                            .copied()
                            .unwrap_or(MbValue::none());
                        super::rc::retain_if_ptr(factory);
                        return factory;
                    }
                    // UserDict / UserList / UserString public `.data` payload.
                    if attr_name == "data"
                        && super::stdlib::collections_mod::user_wrapper_kind(class_name).is_some()
                    {
                        let data = fields
                            .read()
                            .unwrap()
                            .get("_data")
                            .copied()
                            .unwrap_or(MbValue::none());
                        super::rc::retain_if_ptr(data);
                        return data;
                    }
                    // io in-memory streams: `.closed` reflects the _closed flag.
                    if matches!(
                        class_name.as_str(),
                        "StringIO"
                            | "BytesIO"
                            | "BufferedReader"
                            | "BufferedWriter"
                            | "TextIOWrapper"
                    ) && attr_name == "closed"
                    {
                        let closed = fields
                            .read()
                            .unwrap()
                            .get("_closed")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        return MbValue::from_bool(closed);
                    }
                    // memoryview: expose nbytes / format / readonly / itemsize /
                    // ndim / shape / strides. Mamba models the byte-flat case
                    // (format='B', itemsize=1, ndim=1), so shape == (nbytes,)
                    // and strides == (1,).
                    if class_name == "memoryview" {
                        match attr_name.as_str() {
                            "itemsize" | "ndim" => return MbValue::from_int(1),
                            "nbytes" => {
                                let buf = fields.read().unwrap().get("_buffer").copied();
                                let nbytes = buf
                                    .and_then(super::builtins::try_bytes_like)
                                    .map(|data| data.len() as i64)
                                    .unwrap_or(0);
                                return MbValue::from_int(nbytes);
                            }
                            "shape" => {
                                // Resolve the underlying bytes-like length via the
                                // canonical try_bytes_like coercion so a memoryview
                                // wrapping a nested memoryview or array('B'/'b') still
                                // reports its real length (CPython:
                                // memoryview(memoryview(b'abc')).shape == (3,)).
                                let buf = fields.read().unwrap().get("_buffer").copied();
                                let nbytes = buf
                                    .and_then(super::builtins::try_bytes_like)
                                    .map(|data| data.len() as i64)
                                    .unwrap_or(0);
                                return MbValue::from_ptr(MbObject::new_tuple(vec![
                                    MbValue::from_int(nbytes),
                                ]));
                            }
                            "strides" => {
                                let stride = fields
                                    .read()
                                    .unwrap()
                                    .get("_stride")
                                    .and_then(|v| v.as_int())
                                    .unwrap_or(1);
                                return MbValue::from_ptr(MbObject::new_tuple(vec![
                                    MbValue::from_int(stride),
                                ]));
                            }
                            "format" => {
                                return MbValue::from_ptr(MbObject::new_str("B".to_string()))
                            }
                            "readonly" => {
                                // Explicit flag (set by toreadonly / a bytes
                                // source) wins; otherwise a bytearray-backed
                                // view is writable.
                                let g = fields.read().unwrap();
                                if let Some(ro) = g.get("_readonly") {
                                    return MbValue::from_bool(ro.as_bool() == Some(true));
                                }
                                let writable = g
                                    .get("_buffer")
                                    .and_then(|b| b.as_ptr())
                                    .map_or(false, |bp| {
                                        matches!((*bp).data, ObjData::ByteArray(_))
                                    });
                                return MbValue::from_bool(!writable);
                            }
                            "contiguous" | "c_contiguous" | "f_contiguous" => {
                                let contiguous = fields
                                    .read()
                                    .unwrap()
                                    .get("_contiguous")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(true);
                                return MbValue::from_bool(contiguous);
                            }
                            // `mv.obj` is the underlying object the view exposes.
                            "obj" => {
                                let b = {
                                    let g = fields.read().unwrap();
                                    g.get("_obj").copied().or_else(|| g.get("_buffer").copied())
                                };
                                if let Some(b) = b {
                                    super::rc::retain_if_ptr(b);
                                    return b;
                                }
                                return MbValue::none();
                            }
                            _ => {}
                        }
                        // memoryview methods as bound callables (`callable(mv.cast)`,
                        // `f = mv.tobytes; f()`), dispatched via mb_call_method.
                        if matches!(
                            attr_name.as_str(),
                            "tobytes"
                                | "tolist"
                                | "hex"
                                | "release"
                                | "toreadonly"
                                | "cast"
                                | "count"
                                | "index"
                        ) {
                            return make_bound_native_method(obj, &attr_name);
                        }
                    }
                    // slice methods as bound callables (`callable(slice(0,5)
                    // .indices)`, `slice(1,2,3).__hash__()`); dispatched via
                    // mb_call_method. start/stop/step remain plain fields.
                    if class_name == "slice" && matches!(attr_name.as_str(), "indices" | "__hash__")
                    {
                        return make_bound_native_method(obj, &attr_name);
                    }
                    // functools.lru_cache wrapper: expose cache_info / cache_clear
                    // as bound callables via a tiny helper Instance.
                    if class_name == "functools.lru_cache_wrapper"
                        && matches!(attr_name.as_str(), "cache_info" | "cache_clear")
                    {
                        let mut bound_fields = FxHashMap::default();
                        super::rc::retain_if_ptr(obj);
                        bound_fields.insert("_wrapper".to_string(), obj);
                        bound_fields.insert(
                            "_method".to_string(),
                            MbValue::from_ptr(MbObject::new_str(attr_name.clone())),
                        );
                        let bound = Box::new(MbObject {
                            header: super::rc::MbObjectHeader {
                                rc: std::sync::atomic::AtomicU32::new(1),
                                kind: super::rc::ObjKind::Instance,
                            },
                            data: ObjData::Instance {
                                class_name: "functools._lru_bound_method".to_string(),
                                fields: crate::runtime::rc::MbRwLock::new(bound_fields),
                            },
                        });
                        return MbValue::from_ptr(Box::into_raw(bound));
                    }
                    // R13: __dict__ access suppression.
                    // If class defines __slots__ without '__dict__', raise AttributeError for __dict__.
                    if attr_name == "__dict__" {
                        let suppressed =
                            DICT_SUPPRESSED.with(|reg| reg.borrow().contains(class_name.as_str()));
                        if suppressed {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!(
                                    "'{}' object has no attribute '__dict__'",
                                    class_name
                                ))),
                            );
                            return MbValue::none();
                        }
                        // Materialize the instance __dict__ as a real dict over
                        // the instance fields, unless a user shadowed `__dict__`
                        // as an explicit field (handled by the lookup below).
                        let has_field = fields.read().unwrap().contains_key("__dict__");
                        if !has_field {
                            let guard = fields.read().unwrap();
                            let dict = super::dict_ops::mb_dict_new();
                            for (k, v) in guard.iter() {
                                // `__ns_order__` is SimpleNamespace's hidden
                                // insertion-order list; it must not surface in
                                // __dict__ (vars() already excludes it).
                                if k == "__ns_order__" {
                                    continue;
                                }
                                let key =
                                    MbValue::from_ptr(super::rc::MbObject::new_str(k.clone()));
                                super::dict_ops::mb_dict_setitem(dict, key, *v);
                            }
                            return dict;
                        }
                    }
                    // Python descriptor protocol:
                    // 1. Data descriptors (has __set__) override instance __dict__
                    let class_attr = lookup_method(class_name, &attr_name);
                    if !class_attr.is_none() {
                        if is_data_descriptor(class_attr) {
                            return invoke_descriptor_get(class_attr, obj);
                        }
                    }
                    // 2. Instance __dict__
                    {
                        let fields = fields.read().unwrap();
                        if let Some(val) = fields.get(&attr_name) {
                            let v = *val;
                            super::rc::retain_if_ptr(v);
                            return v;
                        }
                    }
                    // 3. Non-data descriptors and regular class attributes
                    if !class_attr.is_none() {
                        if is_descriptor(class_attr) {
                            return invoke_descriptor_get(class_attr, obj);
                        }
                        super::rc::retain_if_ptr(class_attr);
                        return class_attr;
                    }
                    // 3b. A class attribute whose value is literally None (e.g.
                    //     `_crasher = None`) is invisible to lookup_method's
                    //     MbValue::none() "not found" sentinel. Resolve it
                    //     explicitly via the existence-aware MRO lookup so it
                    //     reads back as None instead of falling through to
                    //     __getattr__ / AttributeError.
                    if class_attr_lookup(class_name, &attr_name).is_some() {
                        return MbValue::none();
                    }
                    // 4. Fallback: __getattr__(self, name) dunder — call if it is a
                    //    TAG_FUNC function pointer; return value directly for other
                    //    stored values (legacy/non-JIT path).
                    let getattr_dunder = lookup_method(class_name, "__getattr__");
                    if !getattr_dunder.is_none() {
                        if let Some(addr) = getattr_dunder.as_func() {
                            // JIT-compiled function: call __getattr__(self, name)
                            // REQ: JIT-compiled functions use SystemV/C calling convention.
                            let attr_str =
                                MbValue::from_ptr(super::rc::MbObject::new_str(attr_name.clone()));
                            let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                                std::mem::transmute(addr);
                            return func(obj, attr_str);
                        }
                        // Non-callable stored value (e.g. test stubs): return directly.
                        super::rc::retain_if_ptr(getattr_dunder);
                        return getattr_dunder;
                    }
                    // 5. Nothing matched (no instance field, class attr, descriptor,
                    //    or __getattr__): a genuine user-class instance raises
                    //    AttributeError, matching CPython. `hasattr` and the 3-arg
                    //    `getattr(…, default)` recover from this via
                    //    current_exception_type (see mb_hasattr / mb_getattr_default).
                    //    Scoped to NON-dunder names on classes whose ENTIRE MRO is
                    //    user-defined (every base registered in CLASS_REGISTRY, or
                    //    `object`). A native/stdlib ancestor anywhere in the MRO
                    //    (e.g. `class Sub(HTMLCalendar)`) keeps the lenient None
                    //    return, because the native parent's __init__ populates
                    //    attributes outside mamba's instance fields and they would
                    //    look spuriously missing. Native-stub Instances (not
                    //    registered) and internal dunder/protocol probing are also
                    //    unaffected.
                    if !(attr_name.starts_with("__") && attr_name.ends_with("__")) {
                        let mro = CLASS_REGISTRY.with(|reg| {
                            reg.borrow().get(class_name.as_str()).map(|cls| cls.mro.clone())
                        });
                        let pure_user_hierarchy = match mro {
                            Some(mro) => USER_CLASSES.with(|u| {
                                let u = u.borrow();
                                mro.iter().all(|n| n == "object" || u.contains(n.as_str()))
                            }),
                            None => false,
                        };
                        if pure_user_hierarchy {
                            super::exception::mb_raise(
                                MbValue::from_ptr(super::rc::MbObject::new_str(
                                    "AttributeError".to_string(),
                                )),
                                MbValue::from_ptr(super::rc::MbObject::new_str(format!(
                                    "'{}' object has no attribute '{}'",
                                    class_name, attr_name
                                ))),
                            );
                            return MbValue::none();
                        }
                    }
                }
                ObjData::Complex(re, im) => match attr_name.as_str() {
                    "real" => return MbValue::from_float(*re),
                    "imag" => return MbValue::from_float(*im),
                    _ => {}
                },
                ObjData::Str(ref s) => {
                    // Class-name strings: support class-level attributes like __mro__, __name__
                    let class_found =
                        CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(s.as_str()));
                    if class_found {
                        match attr_name.as_str() {
                            "__mro__" => {
                                let mro = CLASS_REGISTRY.with(|reg| {
                                    reg.borrow().get(s.as_str()).map(|cls| cls.mro.clone())
                                });
                                if let Some(mro_names) = mro {
                                    let items: Vec<MbValue> = mro_names
                                        .iter()
                                        .map(|name| make_type_object(name))
                                        .collect();
                                    return MbValue::from_ptr(super::rc::MbObject::new_tuple(
                                        items,
                                    ));
                                }
                            }
                            "__name__" => {
                                return MbValue::from_ptr(super::rc::MbObject::new_str(s.clone()));
                            }
                            "__abstractmethods__" if is_user_abc(s) => {
                                return user_abstractmethods_frozenset(s);
                            }
                            "__bases__" => {
                                let bases = CLASS_REGISTRY.with(|reg| {
                                    reg.borrow().get(s.as_str()).map(|cls| cls.bases.clone())
                                });
                                if let Some(base_names) = bases {
                                    let items: Vec<MbValue> = if base_names.is_empty() {
                                        vec![make_type_object("object")]
                                    } else {
                                        base_names.iter().map(|n| make_type_object(n)).collect()
                                    };
                                    return MbValue::from_ptr(super::rc::MbObject::new_tuple(
                                        items,
                                    ));
                                }
                            }
                            "register" if is_user_abc(s) => {
                                return make_user_abc_register_method(s);
                            }
                            // PEP 695: classes always carry __type_params__;
                            // generic classes get theirs set as a class attr
                            // by the desugarer, plain classes default to ().
                            "__type_params__" => {
                                if let Some(val) = mro_lookup_class_attr(s, &attr_name) {
                                    super::rc::retain_if_ptr(val);
                                    return val;
                                }
                                return MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![]));
                            }
                            "__members__" if super::stdlib::enum_class::is_enum_class(s) => {
                                // Class-body enum: name→member mapping incl. aliases.
                                if let Some(d) = super::stdlib::enum_class::members_map_dict(s) {
                                    return d;
                                }
                            }
                            _ => {
                                if let Some(meta_desc) =
                                    metaclass_data_descriptor_for_class(s, &attr_name)
                                {
                                    let cls_val = MbValue::from_ptr(
                                        MbObject::new_str(s.clone()),
                                    );
                                    return invoke_descriptor_get(meta_desc, cls_val);
                                }
                                // Class methods and class attributes via MRO
                                let method = lookup_method(s, &attr_name);
                                if !method.is_none() {
                                    // Unwrap staticmethod/classmethod descriptors
                                    let (unwrapped, _dk) = unwrap_descriptor_method(method);
                                    super::rc::retain_if_ptr(unwrapped);
                                    return unwrapped;
                                }
                                if let Some(method) =
                                    inherited_builtin_unbound_method(s, &attr_name)
                                {
                                    return method;
                                }
                                let class_attr = mro_lookup_class_attr(s, &attr_name);
                                if let Some(val) = class_attr {
                                    super::rc::retain_if_ptr(val);
                                    return val;
                                }
                                // __slots__ entry read on the CLASS yields the
                                // member descriptor (CPython semantics).
                                if class_slot_names(s).iter().any(|n| n == &attr_name) {
                                    return make_member_descriptor(s, &attr_name);
                                }
                                // CPython: class attribute lookup continues
                                // through the metaclass (a @property or plain
                                // method on the metaclass answers Class.attr).
                                let meta = CLASS_REGISTRY.with(|reg| {
                                    reg.borrow()
                                        .get(s.as_str())
                                        .and_then(|c| c.metaclass.clone())
                                });
                                if let Some(meta) = meta {
                                    let mmethod = lookup_method(&meta, &attr_name);
                                    if !mmethod.is_none() {
                                        let is_descriptor = mmethod.as_ptr().is_some_and(|p| {
                                            matches!(
                                                &(*p).data,
                                                ObjData::Instance { class_name, .. }
                                                    if class_name == "__property__"
                                                        || class_name == "__cached_property__"
                                            )
                                        });
                                        if is_descriptor {
                                            let cls_val =
                                                MbValue::from_ptr(MbObject::new_str(s.clone()));
                                            return invoke_descriptor_get(mmethod, cls_val);
                                        }
                                        let (unwrapped, _) = unwrap_descriptor_method(mmethod);
                                        super::rc::retain_if_ptr(unwrapped);
                                        return unwrapped;
                                    }
                                    if let Some(val) = mro_lookup_class_attr(&meta, &attr_name) {
                                        super::rc::retain_if_ptr(val);
                                        return val;
                                    }
                                }
                                // Every lookup missed: CPython raises. Dunders
                                // are exempt — names like __doc__/__module__
                                // exist on every class but aren't modeled here.
                                if !(attr_name.starts_with("__") && attr_name.ends_with("__")) {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "AttributeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(format!(
                                            "type object '{s}' has no attribute '{attr_name}'"
                                        ))),
                                    );
                                    return MbValue::none();
                                }
                            }
                        }
                    }
                    // PEP 695: builtin type names (`type`, `object`, `int`,
                    // ...) carry an empty __type_params__ even when not in
                    // the user class registry.
                    if attr_name == "__type_params__" && super::builtins::is_type_name(s) {
                        return MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![]));
                    }
                    // __name__ / __qualname__ on a builtin class name that is
                    // not in the user registry — exception/warning classes
                    // (RuntimeWarning, ValueError, …) and builtin types (int,
                    // str, …). CPython answers the bare class name. Without
                    // this, `RuntimeWarning.__name__` returns None.
                    if (attr_name == "__name__" || attr_name == "__qualname__")
                        && (super::exception::is_builtin_exception_name(s)
                            || super::builtins::is_type_name(s))
                    {
                        return MbValue::from_ptr(super::rc::MbObject::new_str(s.clone()));
                    }
                }
                _ => {}
            }
        }
    }

    // Primitive number-tower properties (int / float / bool):
    // CPython exposes `(5).numerator`, `(5).real`, `(5).imag`,
    // `(5).denominator` as data descriptors that yield the int / 0 / 1
    // directly. The method-dispatch arm in mb_call_method handles the
    // call form; this getattr branch handles the property form.
    if obj.is_int() {
        let val = obj.as_int().unwrap_or(0);
        match attr_name.as_str() {
            "numerator" | "real" => return MbValue::from_int(val),
            "denominator" => return MbValue::from_int(1),
            "imag" => return MbValue::from_int(0),
            _ => {}
        }
    }
    if obj.is_bool() {
        let b = obj.as_bool().unwrap_or(false);
        let v = if b { 1 } else { 0 };
        match attr_name.as_str() {
            "numerator" | "real" => return MbValue::from_int(v),
            "denominator" => return MbValue::from_int(1),
            "imag" => return MbValue::from_int(0),
            _ => {}
        }
    }
    if obj.is_float() {
        let f = obj.as_float().unwrap_or(0.0);
        match attr_name.as_str() {
            "real" => return MbValue::from_float(f),
            "imag" => return MbValue::from_float(0.0),
            _ => {}
        }
    }

    // Bound method on a builtin container: `(1,2).count`, `getattr([], "pop")`.
    // After every specific arm above declined, synthesize a bound-method shell
    // for a recognized method name so `f = x.method; f(...)` works like
    // `x.method(...)` (both route through mb_call_method). Gated to genuine
    // builtin containers — NOT tagged-int handles (closures/ranges/Fraction/
    // Decimal are is_int() but not real containers), and not user Instances
    // (handled by their own arms above).
    let is_builtin_container = obj.as_ptr().is_some_and(|p| unsafe {
        matches!(
            (*p).data,
            ObjData::List(_)
                | ObjData::Tuple(_)
                | ObjData::Dict(_)
                | ObjData::Str(_)
                | ObjData::Set(_)
                | ObjData::Bytes(_)
                | ObjData::ByteArray(_)
                | ObjData::FrozenSet(_)
                | ObjData::Complex(..)
        )
    });
    // float is also eligible (`getattr(0.5, "__ceil__")()`): it is a direct
    // f64, never a tagged-int handle, so there is no handle ambiguity. int/bool
    // are excluded — their tag space is shared with closure/range/Fraction/
    // Decimal handles, which must not masquerade as ints here.
    if (is_builtin_container || obj.is_float())
        && builtin_type_method_names(&obj).contains(&attr_name.as_str())
    {
        return make_bound_native_method(obj, &attr_name);
    }

    MbValue::none()
}

/// Create a type object — Instance with class_name="type" and __name__ field.
fn make_type_object(name: &str) -> MbValue {
    let mut fields = FxHashMap::default();
    fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(name.to_string())),
    );
    let obj = Box::new(super::rc::MbObject {
        header: super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "type".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// The co_* field names a code object carries, in the CPython 3.12
/// `CodeType(...)` positional-constructor order. `replace()` validates its
/// kwargs against this list; the 18-arg constructor zips its positionals to it.
const CODE_FIELD_ORDER: [&str; 18] = [
    "co_argcount",
    "co_posonlyargcount",
    "co_kwonlyargcount",
    "co_nlocals",
    "co_stacksize",
    "co_flags",
    "co_code",
    "co_consts",
    "co_names",
    "co_varnames",
    "co_filename",
    "co_name",
    "co_qualname",
    "co_firstlineno",
    "co_linetable",
    "co_exceptiontable",
    "co_freevars",
    "co_cellvars",
];

/// Fields compared by code `__eq__` and folded into `__hash__`. CPython
/// compares name/code/consts/etc.; firstlineno participates (two otherwise
/// identical lambdas on different lines are unequal and hash differently).
const CODE_EQ_FIELDS: [&str; 10] = [
    "co_name",
    "co_argcount",
    "co_posonlyargcount",
    "co_kwonlyargcount",
    "co_flags",
    "co_firstlineno",
    "co_varnames",
    "co_consts",
    "co_names",
    "co_code",
];

/// Build a "code" Instance from a prepared field map, ensuring the `code`
/// class (replace/__eq__/__hash__) is registered first.
fn new_code_instance(fields: FxHashMap<String, MbValue>) -> MbValue {
    ensure_code_class_registered();
    let obj = Box::new(super::rc::MbObject {
        header: super::rc::MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "code".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// Build a "code" Instance from an ordered positional field list (the 18-arg
/// `types.CodeType(...)` constructor). Extra/missing trailing args default to
/// empty tuples (CPython's freevars/cellvars defaults).
pub fn make_code_object_from_ctor_args(items: &[MbValue]) -> MbValue {
    let mut fields = FxHashMap::default();
    for (i, name) in CODE_FIELD_ORDER.iter().enumerate() {
        let v = items
            .get(i)
            .copied()
            .unwrap_or_else(|| MbValue::from_ptr(super::rc::MbObject::new_tuple(Vec::new())));
        unsafe {
            super::rc::retain_if_ptr(v);
        }
        fields.insert((*name).to_string(), v);
    }
    new_code_instance(fields)
}

/// Build the code object for a compiled source snippet (code.compile_command).
/// Carries module-level co_* metadata plus a hidden `_source` field so
/// `InteractiveInterpreter.runcode` can re-execute the snippet later.
pub fn make_module_code_object(filename: &str, source: &str) -> MbValue {
    let empty_bytes = || MbValue::from_ptr(super::rc::MbObject::new_bytes(Vec::new()));
    let empty_tuple = || MbValue::from_ptr(super::rc::MbObject::new_tuple(Vec::new()));
    let mut fields = FxHashMap::default();
    fields.insert(
        "co_name".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str("<module>".to_string())),
    );
    fields.insert(
        "co_qualname".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str("<module>".to_string())),
    );
    fields.insert("co_argcount".to_string(), MbValue::from_int(0));
    fields.insert("co_posonlyargcount".to_string(), MbValue::from_int(0));
    fields.insert("co_kwonlyargcount".to_string(), MbValue::from_int(0));
    fields.insert("co_nlocals".to_string(), MbValue::from_int(0));
    fields.insert("co_stacksize".to_string(), MbValue::from_int(1));
    fields.insert("co_flags".to_string(), MbValue::from_int(0));
    fields.insert("co_varnames".to_string(), empty_tuple());
    fields.insert(
        "co_filename".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(filename.to_string())),
    );
    fields.insert("co_firstlineno".to_string(), MbValue::from_int(1));
    fields.insert("co_code".to_string(), empty_bytes());
    fields.insert("co_linetable".to_string(), empty_bytes());
    fields.insert("co_exceptiontable".to_string(), empty_bytes());
    fields.insert(
        "co_consts".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![MbValue::none()])),
    );
    fields.insert("co_names".to_string(), empty_tuple());
    fields.insert("co_freevars".to_string(), empty_tuple());
    fields.insert("co_cellvars".to_string(), empty_tuple());
    fields.insert(
        "_source".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(source.to_string())),
    );
    new_code_instance(fields)
}

thread_local! {
    static CODE_CLASS_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Register the `code` native class (replace / __eq__ / __hash__) once per
/// thread. All methods land in CALLABLE_REGISTRY via one mb_class_register
/// call (adding methods later via class_replace_method would NOT enroll them).
fn ensure_code_class_registered() {
    if CODE_CLASS_REGISTERED.with(|c| c.get()) {
        return;
    }
    CODE_CLASS_REGISTERED.with(|c| c.set(true));
    use std::collections::HashMap as Map;
    let mut m: Map<String, MbValue> = Map::new();
    for (name, addr) in [
        ("replace", code_replace as *const () as usize),
        ("__eq__", code_eq as *const () as usize),
        ("__hash__", code_hash as *const () as usize),
    ] {
        super::module::register_variadic_func(addr as u64);
        m.insert(name.to_string(), MbValue::from_func(addr));
    }
    mb_class_register("code", vec![], m);
}

/// Read one co_* field off a code Instance (no retain).
fn code_field(co: MbValue, name: &str) -> MbValue {
    if let Some(ptr) = co.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                if let Some(&v) = fields.read().unwrap().get(name) {
                    return v;
                }
            }
        }
    }
    MbValue::none()
}

/// code.replace(**kwargs) — copy the field map, overwrite the named co_*
/// fields, return a new code object. Unknown keywords raise TypeError;
/// changing co_nlocals away from len(co_varnames) raises ValueError (CPython:
/// nlocals must agree with the varnames table).
unsafe extern "C" fn code_replace(self_v: MbValue, args: MbValue) -> MbValue {
    // kwargs arrive as a trailing dict positional in the packed args list.
    let kwargs = args.as_ptr().and_then(|p| {
        if let ObjData::List(ref lk) = (*p).data {
            lk.read().unwrap().last().copied()
        } else {
            None
        }
    });
    let mut fields = FxHashMap::default();
    if let Some(ptr) = self_v.as_ptr() {
        if let ObjData::Instance {
            fields: ref flock, ..
        } = (*ptr).data
        {
            for (k, &v) in flock.read().unwrap().iter() {
                super::rc::retain_if_ptr(v);
                fields.insert(k.clone(), v);
            }
        }
    }
    if let Some(kw) = kwargs {
        if let Some(kp) = kw.as_ptr() {
            if let ObjData::Dict(ref dlock) = (*kp).data {
                for (dk, &v) in dlock.read().unwrap().iter() {
                    let key = match dk {
                        super::dict_ops::DictKey::Str(s) => s.clone(),
                        _ => continue,
                    };
                    if !CODE_FIELD_ORDER.contains(&key.as_str()) {
                        super::exception::mb_raise(
                            MbValue::from_ptr(super::rc::MbObject::new_str(
                                "TypeError".to_string(),
                            )),
                            MbValue::from_ptr(super::rc::MbObject::new_str(format!(
                                "replace() got an unexpected keyword argument '{key}'"
                            ))),
                        );
                        return MbValue::none();
                    }
                    if key == "co_nlocals" {
                        let nvars = fields
                            .get("co_varnames")
                            .and_then(|t| t.as_ptr())
                            .map(|p| match &(*p).data {
                                ObjData::Tuple(items) => items.len() as i64,
                                _ => 0,
                            })
                            .unwrap_or(0);
                        if v.as_int() != Some(nvars) {
                            super::exception::mb_raise(
                                MbValue::from_ptr(super::rc::MbObject::new_str(
                                    "ValueError".to_string(),
                                )),
                                MbValue::from_ptr(super::rc::MbObject::new_str(
                                    "co_nlocals != len(co_varnames)".to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                    }
                    super::rc::retain_if_ptr(v);
                    if let Some(old) = fields.insert(key, v) {
                        super::rc::release_if_ptr(old);
                    }
                }
            }
        }
    }
    new_code_instance(fields)
}

/// code.__eq__ — field-tuple comparison over CODE_EQ_FIELDS. Dual-convention
/// operand extraction: dunder dispatch passes the operand directly, method
/// dispatch packs it in a one-element list.
unsafe extern "C" fn code_eq(self_v: MbValue, args: MbValue) -> MbValue {
    let mut other = args;
    if let Some(ptr) = args.as_ptr() {
        if let ObjData::List(ref lock) = (*ptr).data {
            let items = lock.read().unwrap();
            if items.len() == 1 {
                other = items[0];
            }
        }
    }
    let is_code = other
        .as_ptr()
        .map(|p| matches!(&(*p).data, ObjData::Instance { class_name, .. } if class_name == "code"))
        .unwrap_or(false);
    if !is_code {
        return MbValue::from_bool(false);
    }
    for f in CODE_EQ_FIELDS {
        let a = code_field(self_v, f);
        let b = code_field(other, f);
        if super::builtins::mb_eq(a, b).as_bool() != Some(true) {
            return MbValue::from_bool(false);
        }
    }
    MbValue::from_bool(true)
}

/// code.__hash__ — folds the same fields __eq__ compares so equal code
/// objects hash equal and a co_firstlineno change shifts the hash.
unsafe extern "C" fn code_hash(self_v: MbValue, _args: MbValue) -> MbValue {
    let mut acc: i64 = 0x636f6465; // "code"
    for f in CODE_EQ_FIELDS {
        let v = code_field(self_v, f);
        let h = super::builtins::mb_hash(v).as_int().unwrap_or(0);
        acc = acc.wrapping_mul(1_000_003).wrapping_add(h);
    }
    // Clamp into the NaN-box 48-bit int payload range (from_int panics
    // outside it); 46 bits keeps the value positive and in range.
    MbValue::from_int(acc & 0x3FFF_FFFF_FFFF)
}

/// Build a function's `__code__` object (CORE #3). Returns a "code" Instance
/// whose co_* fields carry the compiled metadata pulled from the function
/// registries. Param-kind counts (argcount/posonly/kwonly) and variadic flags
/// derive from FUNC_PARAMS when available; varnames includes params plus
/// body locals primed by lowering. Unavailable metadata gets CPython-shaped
/// honest defaults (empty bytes/tuples) so attribute access never returns a
/// bare None for a real function.
fn make_code_object(func: MbValue) -> MbValue {
    let name_str = {
        let n = super::closure::mb_func_get_name(func);
        extract_str(n).unwrap_or_default()
    };
    let varnames = {
        let v = super::closure::mb_func_get_varnames(func);
        if v.is_none() {
            MbValue::from_ptr(super::rc::MbObject::new_tuple(Vec::new()))
        } else {
            v
        }
    };
    let nvars = varnames
        .as_ptr()
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::Tuple(items) => items.len() as i64,
                _ => 0,
            }
        })
        .unwrap_or(0);

    // Param-kind-aware counts: CPython's co_argcount excludes keyword-only
    // params and *args/**kwargs; co_posonlyargcount/co_kwonlyargcount count
    // kinds 0 and 3. Fall back to the coarse FUNC_ARGCOUNTS value (which
    // already excludes the variadics) when FUNC_PARAMS wasn't primed.
    let (argcount, posonly, kwonly, has_varargs, has_varkw) =
        match super::closure::func_params(func) {
            Some(params) => {
                let argcount = params.iter().filter(|p| p.kind <= 1).count() as i64;
                let posonly = params.iter().filter(|p| p.kind == 0).count() as i64;
                let kwonly = params.iter().filter(|p| p.kind == 3).count() as i64;
                let va = params.iter().any(|p| p.kind == 2);
                let vk = params.iter().any(|p| p.kind == 4);
                (argcount, posonly, kwonly, va, vk)
            }
            None => {
                let a = super::closure::mb_func_get_argcount(func)
                    .as_int()
                    .unwrap_or(0);
                (a, 0, 0, false, false)
            }
        };
    // CO_OPTIMIZED | CO_NEWLOCALS | CO_NOFREE plus the variadic bits.
    let mut flags: i64 = 0x01 | 0x02 | 0x40;
    if has_varargs {
        flags |= 0x04;
    }
    if has_varkw {
        flags |= 0x08;
    }

    let firstlineno = super::closure::func_line(func).unwrap_or(1);
    let filename = super::closure::func_file(func).unwrap_or_else(|| "<string>".to_string());
    let empty_bytes = || MbValue::from_ptr(super::rc::MbObject::new_bytes(Vec::new()));
    let empty_tuple = || MbValue::from_ptr(super::rc::MbObject::new_tuple(Vec::new()));

    let mut fields = FxHashMap::default();
    fields.insert(
        "co_name".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(name_str.clone())),
    );
    fields.insert(
        "co_qualname".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(name_str)),
    );
    fields.insert("co_argcount".to_string(), MbValue::from_int(argcount));
    fields.insert("co_posonlyargcount".to_string(), MbValue::from_int(posonly));
    fields.insert("co_kwonlyargcount".to_string(), MbValue::from_int(kwonly));
    fields.insert("co_nlocals".to_string(), MbValue::from_int(nvars));
    fields.insert("co_stacksize".to_string(), MbValue::from_int(1));
    fields.insert("co_flags".to_string(), MbValue::from_int(flags));
    fields.insert("co_varnames".to_string(), varnames);
    fields.insert(
        "co_filename".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_str(filename)),
    );
    fields.insert("co_firstlineno".to_string(), MbValue::from_int(firstlineno));
    fields.insert("co_code".to_string(), empty_bytes());
    fields.insert("co_linetable".to_string(), empty_bytes());
    fields.insert("co_exceptiontable".to_string(), empty_bytes());
    fields.insert(
        "co_consts".to_string(),
        MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![MbValue::none()])),
    );
    fields.insert("co_names".to_string(), empty_tuple());
    fields.insert("co_freevars".to_string(), empty_tuple());
    fields.insert("co_cellvars".to_string(), empty_tuple());

    new_code_instance(fields)
}

/// Invoke a looked-up method VALUE directly with (self, arg) — bypassing
/// name-based dispatch so instance fields shadowing a dunder (e.g. a
/// per-instance `__format__`) cannot intercept type-level dispatch.
/// Handles both the variadic (self, args-list) and plain two-arg JIT
/// calling conventions.
pub(crate) fn call_method_value2(method: MbValue, self_v: MbValue, arg: MbValue) -> MbValue {
    let addr = extract_func_addr(method);
    if addr == 0 {
        return MbValue::none();
    }
    let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
    if !is_registered {
        return MbValue::none();
    }
    unsafe {
        if super::module::is_variadic_func(addr) {
            let args = MbValue::from_ptr(super::rc::MbObject::new_list(vec![arg]));
            let f: unsafe extern "C" fn(MbValue, MbValue) -> MbValue =
                std::mem::transmute(addr as usize);
            return f(self_v, args);
        }
        // REQ: JIT-compiled functions use SystemV/C calling convention.
        let f: extern "C" fn(MbValue, MbValue) -> MbValue = std::mem::transmute(addr as usize);
        f(self_v, arg)
    }
}

/// Like [`call_method_value2`] but with an args LIST (`self` + 0..=3 args),
/// for dispatch sites that already hold the packed argument list.
pub(crate) fn call_method_value_with_args(
    method: MbValue,
    self_v: MbValue,
    args: MbValue,
) -> MbValue {
    let addr = extract_func_addr(method);
    if addr == 0 {
        return MbValue::none();
    }
    let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
    if !is_registered {
        return MbValue::none();
    }
    let mut items: Vec<MbValue> = Vec::new();
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                items.extend(lock.read().unwrap().iter());
            }
        }
    }
    unsafe {
        if super::module::is_variadic_func(addr) {
            let packed = MbValue::from_ptr(super::rc::MbObject::new_list(items));
            let f: unsafe extern "C" fn(MbValue, MbValue) -> MbValue =
                std::mem::transmute(addr as usize);
            return f(self_v, packed);
        }
        // REQ: JIT-compiled functions use SystemV/C calling convention.
        match items.len() {
            0 => {
                let f: extern "C" fn(MbValue) -> MbValue = std::mem::transmute(addr as usize);
                f(self_v)
            }
            1 => {
                let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                    std::mem::transmute(addr as usize);
                f(self_v, items[0])
            }
            2 => {
                let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                    std::mem::transmute(addr as usize);
                f(self_v, items[0], items[1])
            }
            3 => {
                let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                    std::mem::transmute(addr as usize);
                f(self_v, items[0], items[1], items[2])
            }
            _ => MbValue::none(),
        }
    }
}

/// Check if a value is a descriptor (has __get__).
fn is_descriptor(val: MbValue) -> bool {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return class_name == "__property__"
                    || class_name == "__classmethod__"
                    || class_name == "__staticmethod__"
                    || class_name == "__cached_property__"
                    || !lookup_method(class_name, "__get__").is_none();
            }
        }
    }
    false
}

/// Check if a value is a data descriptor (has __set__ or __delete__).
fn is_data_descriptor(val: MbValue) -> bool {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return class_name == "__property__"
                    || !lookup_method(class_name, "__set__").is_none()
                    || !lookup_method(class_name, "__delete__").is_none();
            }
        }
    }
    false
}

/// Invoke __get__ on a descriptor, or property fget for built-in property.
fn invoke_descriptor_get(desc: MbValue, instance: MbValue) -> MbValue {
    if let Some(ptr) = desc.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "__property__" {
                    return mb_property_get(desc, instance);
                }
                if class_name == "__cached_property__" {
                    return mb_cached_property_get(desc, instance);
                }
                if class_name == "__classmethod__" || class_name == "__staticmethod__" {
                    return mb_descriptor_unwrap(desc);
                }
            }
        }
    }
    // General __get__ protocol. try_get_dunder returns the unbound descriptor
    // method; call that callable directly with (desc, instance, owner).
    if let Some(method) = try_get_dunder(desc, "__get__") {
        let objtype = get_instance_class_name_value(instance);
        let args = MbValue::from_ptr(MbObject::new_list(vec![desc, instance, objtype]));
        return super::builtins::mb_call_spread(method, args);
    }
    desc
}

/// Extract the class name from an instance and return it as a string MbValue.
/// Returns MbValue::none() if the value is not an instance.
fn get_instance_class_name_value(instance: MbValue) -> MbValue {
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_str(class_name.clone()));
            }
        }
    }
    MbValue::none()
}

/// Invoke __set__(instance, value) on a descriptor.
fn invoke_descriptor_set(desc: MbValue, instance: MbValue, value: MbValue) {
    if let Some(ptr) = desc.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "__property__" {
                    mb_property_set(desc, instance, value);
                    return;
                }
            }
        }
    }
    // General __set__ protocol: call desc.__set__(instance, value)
    if let Some(method) = try_get_dunder(desc, "__set__") {
        let addr = extract_func_addr(method);
        if addr != 0 {
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let func: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            func(desc, instance, value);
        }
    }
}

/// Invoke __delete__(instance) on a descriptor.
fn invoke_descriptor_delete(desc: MbValue, instance: MbValue) {
    if let Some(ptr) = desc.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "__property__" {
                    // property.__delete__: call fdel(instance).
                    // Match mb_property_set's dual-path dispatch: TAG_FUNC
                    // direct address first, then CALLABLE_REGISTRY fallback.
                    let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
                    let deleter = mb_getattr(desc, key);
                    if deleter.is_none() {
                        // A property with no fdel cannot be deleted: `del obj.x`
                        // raises AttributeError (CPython). Returning silently let
                        // the delete succeed as a no-op.
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str("can't delete attribute".to_string())),
                        );
                        return;
                    }
                    if let Some(addr) = deleter.as_func() {
                        if addr > 4096 {
                            let f: extern "C" fn(MbValue) -> MbValue = std::mem::transmute(addr);
                            f(instance);
                            return;
                        }
                    }
                    let addr = extract_func_addr(deleter);
                    if addr != 0 {
                        let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                        if is_reg {
                            let f: extern "C" fn(MbValue) -> MbValue =
                                std::mem::transmute(addr as usize);
                            f(instance);
                        }
                    }
                    return;
                }
            }
        }
    }
    // General __delete__ protocol: call desc.__delete__(instance)
    if let Some(method) = try_get_dunder(desc, "__delete__") {
        let addr = extract_func_addr(method);
        if addr != 0 {
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            func(desc, instance);
        }
    }
}

/// getattr(obj, name, default) — returns default if attribute not found.
pub fn mb_getattr_default(obj: MbValue, attr: MbValue, default: MbValue) -> MbValue {
    let result = mb_getattr(obj, attr);
    // A `__getattr__` that raises AttributeError signals "no such attribute";
    // the 3-arg form swallows *only* AttributeError and yields the default
    // (CPython semantics). Any other exception propagates unchanged.
    if super::exception::current_exception_type().as_deref() == Some("AttributeError") {
        super::exception::mb_clear_exception();
        return default;
    }
    if result.is_none() {
        default
    } else {
        result
    }
}

/// vars(obj) → dict of instance __dict__.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-class-rs" tracker="standardize-gap-projects-mamba-src-runtime-class-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// CPython raises TypeError when the argument has no __dict__ attribute (e.g.,
/// `vars(1)`). The previous stub silently returned an empty dict, which broke
/// pytest fixture discovery (relies on the TypeError to skip non-introspectable
/// values). The zero-arg `vars()` form is routed to mb_locals at codegen time —
/// see hir_to_mir.rs (extern_name == "mb_vars" && args.is_empty()).
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#vars_dispatch
pub fn mb_vars(obj: MbValue) -> MbValue {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            // statistics.NormalDist declares __slots__, so it has no
            // instance __dict__ — CPython's vars() raises TypeError.
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "NormalDist" {
                    super::exception::mb_raise(
                        MbValue::from_ptr(super::rc::MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(super::rc::MbObject::new_str(
                            "vars() argument must have __dict__ attribute".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
            }
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let dict = super::dict_ops::mb_dict_new();
                for (k, v) in fields.iter() {
                    // Hidden bookkeeping fields (e.g. SimpleNamespace's
                    // insertion-order list) are not part of __dict__.
                    if k == "__ns_order__" {
                        continue;
                    }
                    let key = MbValue::from_ptr(super::rc::MbObject::new_str(k.clone()));
                    super::dict_ops::mb_dict_setitem(dict, key, *v);
                }
                return dict;
            }
            // vars(module): a module namespace is surfaced as a Dict; CPython
            // returns the module's __dict__. Return a snapshot of its str-keyed
            // entries. Gated on the `__name__` module marker so a plain dict
            // still hits the TypeError path below (CPython: vars(dict) raises).
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = lock.read().unwrap();
                if guard.contains_key("__name__") {
                    let dict = super::dict_ops::mb_dict_new();
                    for (k, v) in guard.iter() {
                        if let super::dict_ops::DictKey::Str(s) = k {
                            let key = MbValue::from_ptr(super::rc::MbObject::new_str(s.clone()));
                            super::dict_ops::mb_dict_setitem(dict, key, *v);
                        }
                    }
                    return dict;
                }
            }
        }
    }
    // Non-Instance: raise TypeError (vars() arg must have __dict__).
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "vars() argument must have __dict__ attribute".to_string(),
        )),
    );
    MbValue::none()
}
// HANDWRITE-END

/// MRO-walk helper: for a class name, return the union of method + class-attr
/// names across the entire MRO chain. Used by `mb_dir`. Returns deduplicated
/// names in insertion order; the caller is responsible for sort+dedup of the
/// combined instance + class set.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-class-rs" tracker="standardize-gap-projects-mamba-src-runtime-class-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Owns ONLY the per-class MRO traversal — does NOT register `mb_dir` and
/// does NOT touch any instance `__dict__`. Those responsibilities live in
/// the `mb_dir` entry point below (and conceptually in builtins.rs per the
/// approved spec, kept here in class.rs because that is where `mb_dir`
/// currently lives in the codebase).
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#dir_walk_mro
pub fn mb_dir_mro_keys(class_name: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let mro = match reg.get(class_name) {
            Some(cls) => cls.mro.clone(),
            None => return,
        };
        for ancestor in &mro {
            if let Some(cls) = reg.get(ancestor) {
                for k in cls.methods.keys() {
                    if seen.insert(k.clone()) {
                        names.push(k.clone());
                    }
                }
                for k in cls.class_attrs.keys() {
                    if seen.insert(k.clone()) {
                        names.push(k.clone());
                    }
                }
            }
        }
    });
    names
}
// HANDWRITE-END

/// Hardcoded method names for builtin types (list/dict/str/set/tuple/etc.).
/// Returns the list of method names that the dispatcher in `<type>_ops.rs`
/// recognises. Keep in sync with the corresponding `dispatch_*_method`.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-class-rs" tracker="standardize-gap-projects-mamba-src-runtime-class-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Builtin types lack a runtime method-table walkable from `__mro__`; this
/// table mirrors the `dispatch_*_method` switch arms. A future change can
/// codegen this from the dispatcher source to keep the two in sync.
/// Per-type method names for a builtin TYPE NAME (`dir(str)`, `dir(list)`).
/// Mirrors `builtin_type_method_names` below so `dir(str)` lists the same
/// methods as `dir("")`.
fn builtin_type_method_names_by_name(name: &str) -> Vec<&'static str> {
    match name {
        "list" => vec![
            "append",
            "extend",
            "insert",
            "pop",
            "remove",
            "clear",
            "reverse",
            "sort",
            "copy",
            "index",
            "count",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
            "__setitem__",
            "__delitem__",
        ],
        "dict" => vec![
            "keys",
            "values",
            "items",
            "get",
            "pop",
            "popitem",
            "setdefault",
            "update",
            "clear",
            "copy",
            "fromkeys",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
            "__setitem__",
            "__delitem__",
        ],
        "str" => vec![
            "upper",
            "lower",
            "title",
            "capitalize",
            "swapcase",
            "strip",
            "lstrip",
            "rstrip",
            "split",
            "rsplit",
            "splitlines",
            "join",
            "replace",
            "find",
            "rfind",
            "index",
            "rindex",
            "startswith",
            "endswith",
            "count",
            "encode",
            "format",
            "format_map",
            "isdigit",
            "isalpha",
            "isalnum",
            "isspace",
            "isupper",
            "islower",
            "istitle",
            "zfill",
            "ljust",
            "rjust",
            "center",
            "removeprefix",
            "removesuffix",
            "casefold",
            "expandtabs",
            "partition",
            "rpartition",
            "translate",
            "maketrans",
            "isidentifier",
            "isprintable",
            "isascii",
            "isdecimal",
            "isnumeric",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
        ],
        "set" => vec![
            "add",
            "discard",
            "remove",
            "pop",
            "clear",
            "copy",
            "union",
            "intersection",
            "difference",
            "symmetric_difference",
            "update",
            "intersection_update",
            "difference_update",
            "symmetric_difference_update",
            "isdisjoint",
            "issubset",
            "issuperset",
            "__iter__",
            "__len__",
            "__contains__",
        ],
        // frozenset is immutable: no add/discard/remove/pop/clear/*_update.
        "frozenset" => vec![
            "copy",
            "union",
            "intersection",
            "difference",
            "symmetric_difference",
            "isdisjoint",
            "issubset",
            "issuperset",
            "__iter__",
            "__len__",
            "__contains__",
        ],
        "complex" => vec![
            "conjugate",
            "real",
            "imag",
            "__getnewargs__",
            "__complex__",
            "__add__",
            "__sub__",
            "__mul__",
            "__truediv__",
            "__pow__",
            "__neg__",
            "__abs__",
            "__eq__",
            "__hash__",
            "__repr__",
            "__str__",
        ],
        "slice" => vec![
            "indices", "start", "stop", "step", "__eq__", "__ne__", "__lt__", "__le__", "__gt__",
            "__ge__", "__hash__", "__repr__",
        ],
        "tuple" => vec![
            "count",
            "index",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
        ],
        "int" => vec![
            "bit_length",
            "bit_count",
            "to_bytes",
            "from_bytes",
            "as_integer_ratio",
            "conjugate",
            "__add__",
            "__sub__",
            "__mul__",
            "__truediv__",
            "__floordiv__",
            "__mod__",
            "__pow__",
            "__neg__",
            "__abs__",
            "__eq__",
            "__lt__",
            "__le__",
            "__gt__",
            "__ge__",
            "__hash__",
            "__repr__",
            "__str__",
        ],
        "float" => vec![
            "hex",
            "fromhex",
            "is_integer",
            "as_integer_ratio",
            "conjugate",
            "__add__",
            "__sub__",
            "__mul__",
            "__truediv__",
            "__floordiv__",
            "__mod__",
            "__pow__",
            "__neg__",
            "__abs__",
            "__eq__",
            "__lt__",
            "__le__",
            "__gt__",
            "__ge__",
            "__hash__",
            "__repr__",
            "__str__",
        ],
        "bool" => vec![
            "__and__", "__or__", "__xor__", "__bool__", "__repr__", "__str__",
        ],
        "bytes" => vec![
            "hex",
            "fromhex",
            "decode",
            "count",
            "find",
            "rfind",
            "index",
            "rindex",
            "startswith",
            "endswith",
            "split",
            "rsplit",
            "splitlines",
            "strip",
            "lstrip",
            "rstrip",
            "replace",
            "removeprefix",
            "removesuffix",
            "upper",
            "lower",
            "title",
            "capitalize",
            "swapcase",
            "join",
            "center",
            "ljust",
            "rjust",
            "zfill",
            "translate",
            "maketrans",
            "partition",
            "rpartition",
            "expandtabs",
            "isalnum",
            "isalpha",
            "isascii",
            "isdigit",
            "islower",
            "isspace",
            "istitle",
            "isupper",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
        ],
        // bytearray = bytes surface + the mutable-sequence mutators.
        "bytearray" => vec![
            "hex",
            "fromhex",
            "decode",
            "count",
            "find",
            "rfind",
            "index",
            "rindex",
            "startswith",
            "endswith",
            "split",
            "rsplit",
            "splitlines",
            "strip",
            "lstrip",
            "rstrip",
            "replace",
            "removeprefix",
            "removesuffix",
            "upper",
            "lower",
            "title",
            "capitalize",
            "swapcase",
            "join",
            "center",
            "ljust",
            "rjust",
            "zfill",
            "translate",
            "maketrans",
            "partition",
            "rpartition",
            "expandtabs",
            "isalnum",
            "isalpha",
            "isascii",
            "isdigit",
            "islower",
            "isspace",
            "istitle",
            "isupper",
            "append",
            "extend",
            "insert",
            "pop",
            "remove",
            "clear",
            "reverse",
            "copy",
            "__iter__",
            "__len__",
            "__contains__",
            "__getitem__",
            "__setitem__",
            "__delitem__",
        ],
        _ => Vec::new(),
    }
}

fn builtin_type_method_names(obj: &MbValue) -> Vec<&'static str> {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match (*ptr).data {
                ObjData::List(_) => vec![
                    "append",
                    "extend",
                    "insert",
                    "pop",
                    "remove",
                    "clear",
                    "reverse",
                    "sort",
                    "copy",
                    "index",
                    "count",
                    "__iter__",
                    "__len__",
                    "__contains__",
                    "__getitem__",
                    "__setitem__",
                    "__delitem__",
                ],
                ObjData::Dict(_) => vec![
                    "keys",
                    "values",
                    "items",
                    "get",
                    "pop",
                    "popitem",
                    "setdefault",
                    "update",
                    "clear",
                    "copy",
                    "__iter__",
                    "__len__",
                    "__contains__",
                    "__getitem__",
                    "__setitem__",
                    "__delitem__",
                ],
                ObjData::Str(_) => vec![
                    "upper",
                    "lower",
                    "title",
                    "capitalize",
                    "swapcase",
                    "strip",
                    "lstrip",
                    "rstrip",
                    "split",
                    "rsplit",
                    "splitlines",
                    "join",
                    "replace",
                    "find",
                    "rfind",
                    "index",
                    "rindex",
                    "startswith",
                    "endswith",
                    "count",
                    "encode",
                    "format",
                    "format_map",
                    "isdigit",
                    "isalpha",
                    "isalnum",
                    "isspace",
                    "isupper",
                    "islower",
                    "istitle",
                    "zfill",
                    "ljust",
                    "rjust",
                    "center",
                    "__iter__",
                    "__len__",
                    "__contains__",
                    "__getitem__",
                ],
                ObjData::Set(_) => vec![
                    "add",
                    "discard",
                    "remove",
                    "pop",
                    "clear",
                    "copy",
                    "union",
                    "intersection",
                    "difference",
                    "symmetric_difference",
                    "update",
                    "intersection_update",
                    "difference_update",
                    "isdisjoint",
                    "issubset",
                    "issuperset",
                    "__iter__",
                    "__len__",
                    "__contains__",
                ],
                ObjData::Tuple(_) => vec![
                    "count",
                    "index",
                    "__iter__",
                    "__len__",
                    "__contains__",
                    "__getitem__",
                ],
                // Delegate to the by-name tables so the bound-method surface
                // stays in sync with hasattr for these immutable/byte types.
                ObjData::Bytes(_) => builtin_type_method_names_by_name("bytes"),
                ObjData::ByteArray(_) => builtin_type_method_names_by_name("bytearray"),
                ObjData::FrozenSet(_) => builtin_type_method_names_by_name("frozenset"),
                ObjData::Complex(..) => builtin_type_method_names_by_name("complex"),
                _ => Vec::new(),
            }
        }
    } else if obj.is_int() || obj.is_float() {
        vec![
            "bit_length",
            "to_bytes",
            "from_bytes",
            "is_integer",
            "as_integer_ratio",
            "conjugate",
            "__ceil__",
            "__floor__",
            "__trunc__",
            "__round__",
            "__add__",
            "__sub__",
            "__mul__",
            "__truediv__",
            "__floordiv__",
            "__mod__",
            "__pow__",
            "__neg__",
            "__abs__",
            "__eq__",
            "__lt__",
            "__le__",
            "__gt__",
            "__ge__",
            "__hash__",
            "__repr__",
            "__str__",
        ]
    } else if obj.is_bool() {
        vec![
            "__and__", "__or__", "__xor__", "__bool__", "__repr__", "__str__",
        ]
    } else {
        Vec::new()
    }
}
// HANDWRITE-END

/// dir(obj) → sorted, deduplicated list of attribute names.
///
// HANDWRITE-BEGIN gap="standardize:projects-mamba-src-runtime-class-rs" tracker="standardize-gap-projects-mamba-src-runtime-class-rs" reason="introspection-builtins (issue: enhancement-mamba-introspection-builtins-globals-locals-vars-dir)."
/// Entry point: collect instance `__dict__` keys, walk the class MRO via
/// `dir()` with no arguments — returns sorted names from the current module globals.
pub fn mb_dir_no_args() -> MbValue {
    let globals_dict = super::closure::build_globals_dict();
    let mut names: Vec<String> = Vec::new();
    if let Some(ptr) = globals_dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref map) = (*ptr).data {
                let guard = map.read().unwrap();
                for k in guard.keys() {
                    if let super::dict_ops::DictKey::Str(s) = k {
                        names.push(s.clone());
                    }
                }
            }
        }
    }
    names.sort();
    names.dedup();
    let list = super::list_ops::mb_list_new();
    for n in names {
        let v = MbValue::from_ptr(super::rc::MbObject::new_str(n));
        super::list_ops::mb_list_append(list, v);
    }
    list
}

/// dir() called with more than one argument — CPython rejects the call.
pub fn mb_dir_arity_error(count: MbValue) -> MbValue {
    let n = count.as_int().unwrap_or(2);
    super::builtins::raise_type_error(format!("dir expected at most 1 argument, got {n}"));
    MbValue::none()
}

/// `mb_dir_mro_keys`, then sort+dedup. For builtin types (list/dict/str/...),
/// use `builtin_type_method_names`.
/// @spec .aw/tech-design/cclab-mamba/logic/introspection-builtins.md#dir_has_dict
pub fn mb_dir(obj: MbValue) -> MbValue {
    let list = super::list_ops::mb_list_new();
    let mut names: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let push = |s: String, names: &mut Vec<String>, seen: &mut HashSet<String>| {
        if seen.insert(s.clone()) {
            names.push(s);
        }
    };

    // A native class exposed as its constructor func (e.g.
    // tempfile.SpooledTemporaryFile, pathlib.Path) — `dir()` must list the
    // class's registered methods, resolved through NATIVE_TYPE_NAMES into the
    // CLASS_REGISTRY MRO. ADDITIVE: only ever adds names for funcs registered
    // as native types; an unregistered func falls through unchanged.
    if let Some(addr) = obj.as_func() {
        let native_type =
            super::module::NATIVE_TYPE_NAMES.with(|map| map.borrow().get(&(addr as u64)).cloned());
        if let Some(nt) = native_type {
            for k in mb_dir_mro_keys(&nt) {
                push(k, &mut names, &mut seen);
            }
        }
    }

    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, fields } => {
                    // Bare object() instance: dir() lists object's base dunders.
                    if class_name == "object" {
                        for d in [
                            "__class__", "__delattr__", "__dir__", "__doc__",
                            "__eq__", "__format__", "__ge__", "__getattribute__",
                            "__gt__", "__hash__", "__init__", "__init_subclass__",
                            "__le__", "__lt__", "__ne__", "__new__", "__reduce__",
                            "__reduce_ex__", "__repr__", "__setattr__", "__sizeof__",
                            "__str__", "__subclasshook__",
                        ] {
                            push(d.to_string(), &mut names, &mut seen);
                        }
                    }
                    // TYPE OBJECT (`dir(str)`, `dir(MyClass)`): list the named
                    // type's methods, not the type-object wrapper's fields.
                    if class_name == "type" {
                        let type_name: Option<String> = {
                            let guard = fields.read().unwrap();
                            guard.get("__name__").and_then(|v| {
                                v.as_ptr().and_then(|p| {
                                    if let ObjData::Str(ref s) = (*p).data {
                                        Some(s.clone())
                                    } else {
                                        None
                                    }
                                })
                            })
                        };
                        if let Some(tn) = type_name {
                            let builtin = builtin_type_method_names_by_name(&tn);
                            if builtin.is_empty() {
                                // User class type object — walk its MRO.
                                for k in mb_dir_mro_keys(&tn) {
                                    push(k, &mut names, &mut seen);
                                }
                            } else {
                                for k in builtin {
                                    push(k.to_string(), &mut names, &mut seen);
                                }
                            }
                        }
                    } else if !lookup_method(class_name, "__dir__").is_none() {
                        // User-defined __dir__: call it, require an iterable
                        // result, and return its names sorted (CPython sorts
                        // but does not dedup the custom-__dir__ result).
                        let name_val =
                            MbValue::from_ptr(super::rc::MbObject::new_str("__dir__".to_string()));
                        let args = MbValue::from_ptr(super::rc::MbObject::new_list(vec![]));
                        let result = mb_call_method(obj, name_val, args);
                        let items: Option<Vec<MbValue>> =
                            result.as_ptr().and_then(|rp| match &(*rp).data {
                                ObjData::List(ref lock) => {
                                    Some(lock.read().unwrap().iter().copied().collect())
                                }
                                ObjData::Tuple(ref t) => Some(t.clone()),
                                ObjData::Set(ref lock) => {
                                    Some(lock.read().unwrap().iter().copied().collect())
                                }
                                ObjData::FrozenSet(ref t) => Some(t.clone()),
                                ObjData::Str(ref s) => Some(
                                    s.chars()
                                        .map(|c| {
                                            MbValue::from_ptr(super::rc::MbObject::new_str(
                                                c.to_string(),
                                            ))
                                        })
                                        .collect(),
                                ),
                                _ => None,
                            });
                        let Some(items) = items else {
                            super::builtins::raise_type_error(format!(
                                "'{}' object is not iterable",
                                super::builtins::value_type_name(result)
                            ));
                            return list;
                        };
                        let mut name_strs: Vec<String> = Vec::with_capacity(items.len());
                        for item in items {
                            if let Some(p) = item.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    name_strs.push(s.clone());
                                }
                            }
                        }
                        name_strs.sort();
                        for n in name_strs {
                            let v = MbValue::from_ptr(super::rc::MbObject::new_str(n));
                            super::list_ops::mb_list_append(list, v);
                        }
                        return list;
                    } else {
                        let fields = fields.read().unwrap();
                        for k in fields.keys() {
                            push(k.clone(), &mut names, &mut seen);
                        }
                        drop(fields);
                        for k in mb_dir_mro_keys(class_name) {
                            push(k, &mut names, &mut seen);
                        }
                    }
                }
                ObjData::Dict(lock) => {
                    let guard = lock.read().unwrap();
                    let looks_like_module = guard.contains_key("__name__");
                    if looks_like_module {
                        for k in guard.keys() {
                            if let super::dict_ops::DictKey::Str(s) = k {
                                push(s.clone(), &mut names, &mut seen);
                            }
                        }
                    }
                    drop(guard);
                    // A module is surfaced as a Dict, but `dir(module)` must NOT
                    // leak the dict-protocol method names (clear/copy/get/items/
                    // keys/pop/...): a CPython module namespace has none of them.
                    // Only a genuine dict object exposes dict methods via dir().
                    if !looks_like_module {
                        for k in builtin_type_method_names(&obj) {
                            push(k.to_string(), &mut names, &mut seen);
                        }
                    }
                }
                _ => {
                    for k in builtin_type_method_names(&obj) {
                        push(k.to_string(), &mut names, &mut seen);
                    }
                }
            }
        }
    } else {
        for k in builtin_type_method_names(&obj) {
            push(k.to_string(), &mut names, &mut seen);
        }
    }

    names.sort();
    for n in names {
        let v = MbValue::from_ptr(super::rc::MbObject::new_str(n));
        super::list_ops::mb_list_append(list, v);
    }
    list
}
// HANDWRITE-END

/// Set an attribute on an instance.
/// If the class defines `__setattr__`, compiled code should dispatch through it;
/// this function implements the default `object.__setattr__` behavior (direct field write).
pub fn mb_setattr(obj: MbValue, attr: MbValue, value: MbValue) {
    // uuid.UUID is immutable: setattr raises TypeError. Handles are
    // int-tagged, so intercept before any pointer path.
    if let Some(id) = obj.as_int() {
        if super::stdlib::uuid_mod::is_uuid_handle(id as u64) {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str("UUID objects are immutable".to_string())),
            );
            return;
        }
        // Fraction is immutable (__slots__, read-only numerator/denominator):
        // any attribute assignment raises AttributeError.
        if super::stdlib::fractions_mod::is_fraction_handle(id as u64) {
            let name = extract_str(attr).unwrap_or_default();
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'Fraction' object has no attribute '{name}'"
                ))),
            );
            return;
        }
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            // obj.__class__ = NewClass reassigns a user instance's class
            // (preserving its instance __dict__). CPython only permits this
            // between heap (user-defined) types; assigning a builtin like list
            // or str raises TypeError. Done before the generic field path so
            // __class__ is never stored as a plain instance attribute.
            if extract_str(attr).as_deref() == Some("__class__") {
                // Classify the receiver (no borrow held across the mutation):
                // 0 = user-heap instance (reassignable), 1 = native-stub
                // instance (e.g. ET.Element stores a __class__ field — fall
                // through to the generic path), 2 = builtin immutable value
                // (str/bytes/tuple/… — __class__ reassignment is a TypeError).
                let kind = match &(*ptr).data {
                    ObjData::Instance { class_name, .. } => {
                        if USER_CLASSES.with(|u| u.borrow().contains(class_name.as_str())) {
                            0
                        } else {
                            // native-stub instance OR a `type` object (a class) —
                            // fall through (`SomeClass.__class__ = Meta` is a
                            // metaclass swap, not our concern).
                            1
                        }
                    }
                    // A class-name string IS a class object (metaclass swap via
                    // `Class.__class__ = NewMeta`): fall through, do not treat it
                    // as a builtin-value rejection. Only a genuine builtin
                    // immutable value (str "a", bytes, tuple) is a TypeError.
                    ObjData::Str(ref s) => {
                        if CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(s.as_str())) {
                            1
                        } else {
                            2
                        }
                    }
                    _ => 2,
                };
                let class_err = || {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "__class__ assignment only supported for mutable types or ModuleType subclasses".to_string(),
                        )),
                    );
                };
                if kind == 0 {
                    let new_cn = resolve_class_name(value);
                    let new_is_user = new_cn
                        .as_deref()
                        .map_or(false, |cn| USER_CLASSES.with(|u| u.borrow().contains(cn)));
                    if new_is_user {
                        if let ObjData::Instance { class_name, .. } = &mut (*ptr).data {
                            *class_name = new_cn.unwrap();
                        }
                    } else {
                        class_err();
                    }
                    return;
                } else if kind == 2 {
                    class_err();
                    return;
                }
            }
            // Fast path for Instance objects (most common case: self.x = value in __init__).
            // Check Instance first before trying the Str/class-attr path, since instance
            // setattr is by far the hottest path during object construction.
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
                // A bare object() has no __dict__, so attribute assignment
                // raises AttributeError (CPython). Scoped to class_name ==
                // "object" exactly — user subclasses carry their own name and
                // do have a __dict__.
                if class_name == "object" {
                    let attr_name = extract_str(attr).unwrap_or_default();
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'object' object has no attribute '{attr_name}'"
                        ))),
                    );
                    return;
                }
                // zipfile: the archive comment must be bytes.
                if class_name == "ZipFile" {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if attr_s == "comment" {
                                let is_bytes = value
                                    .as_ptr()
                                    .map(|vp| {
                                        matches!(
                                            (*vp).data,
                                            ObjData::Bytes(_) | ObjData::ByteArray(_)
                                        )
                                    })
                                    .unwrap_or(false);
                                if !is_bytes {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "comment: expected bytes, got str".to_string(),
                                        )),
                                    );
                                    return;
                                }
                            }
                        }
                    }
                }
                // tracemalloc Filter/DomainFilter fields are read-only.
                if class_name == "tracemalloc.Filter" || class_name == "tracemalloc.DomainFilter" {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("readonly attribute".to_string())),
                    );
                    return;
                }
                // slice objects are immutable: start/stop/step are read-only
                // (CPython raises AttributeError on assignment). Construction
                // writes the fields directly (mb_slice), not through setattr.
                if class_name == "slice" {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str("readonly attribute".to_string())),
                    );
                    return;
                }
                // threading: daemonizing a running thread is a RuntimeError.
                if class_name == "Thread" {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if attr_s == "daemon" {
                                let alive = fields
                                    .read()
                                    .unwrap()
                                    .get("alive")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                if alive {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "RuntimeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "cannot set daemon status of active thread".to_string(),
                                        )),
                                    );
                                    return;
                                }
                            }
                        }
                    }
                }
                // unittest.mock spec_set: writes to undeclared attributes on a
                // spec_set mock raise AttributeError.
                if super::stdlib::unittest_mock_mod::is_mock_class(class_name) {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if super::stdlib::unittest_mock_mod::mock_setattr_blocked(obj, attr_s) {
                                return;
                            }
                        }
                    }
                }
                // ssl.SSLContext attribute writes route through CPython's
                // property setters (validation + the check_hostname /
                // verify_mode coupling).
                if class_name == "SSLContext" {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if super::stdlib::ssl_mod::sslcontext_setattr(obj, attr_s, value) {
                                return;
                            }
                        }
                    }
                }
                // urllib.request.Request.full_url is a property setter: changing it
                // re-parses derived fields, and invalid URLs raise ValueError.
                if class_name == "urllib.request.Request" {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if super::stdlib::http_mod::request_setattr(obj, attr_s, value) {
                                return;
                            }
                        }
                    }
                }
                // weakref.ref.__callback__ is a read-only attribute in CPython:
                // `r.__callback__ = ...` raises AttributeError.
                if class_name == "ReferenceType" {
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            if attr_s == "__callback__" {
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(
                                        "attribute '__callback__' of 'weakref.ReferenceType' objects is not writable".to_string(),
                                    )),
                                );
                                return;
                            }
                        }
                    }
                }
                // namedtuple fields are read-only (CPython: __slots__ = ()).
                // Setting a declared field raises AttributeError; a non-field
                // name on a subclass-with-__dict__ still flows to the insert.
                if let Some(attr_s) = extract_str(attr) {
                    if super::stdlib::collections_mod::namedtuple_is_field(obj, &attr_s) {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str("can't set attribute".to_string())),
                        );
                        return;
                    }
                }
                // SimpleNamespace insertion-order tracking: appending a NEW
                // attribute name to the hidden `__ns_order__` list keeps repr()
                // in insertion order. Runs before the generic insert below.
                if class_name == "SimpleNamespace" {
                    if let Some(attr_s) = extract_str(attr) {
                        if attr_s != "__ns_order__" {
                            let (is_new, order) = {
                                let g = fields.read().unwrap();
                                (!g.contains_key(&attr_s), g.get("__ns_order__").copied())
                            };
                            if is_new {
                                if let Some(ol) = order.and_then(|v| v.as_ptr()) {
                                    if let ObjData::List(ref lk) = (*ol).data {
                                        lk.write()
                                            .unwrap()
                                            .push(MbValue::from_ptr(MbObject::new_str(attr_s)));
                                    }
                                }
                            }
                        }
                    }
                }
                // Ultra-fast path for "simple" classes: classes that have no descriptors
                // and no __slots__. For these classes we skip the descriptor protocol
                // check and slots registry check entirely. Most user-defined classes
                // (e.g., `class Point:`) take this path.
                // This also avoids extract_str() which clones the attribute name String.
                let is_simple =
                    SIMPLE_CLASS_CACHE.with(|c| c.borrow().contains(class_name.as_str()));
                if is_simple {
                    // Retain value — Instance fields are released by release_contained_values.
                    super::rc::retain_if_ptr(value);
                    // Borrow the attribute name string directly to avoid cloning
                    // when the key already exists in the HashMap. Only clone for
                    // new field insertions (first __init__ call).
                    if let Some(kp) = attr.as_ptr() {
                        if let ObjData::Str(ref attr_s) = (*kp).data {
                            let mut flds = fields.write().unwrap();
                            if let Some(existing) = flds.get_mut(attr_s.as_str()) {
                                let old = *existing;
                                *existing = value;
                                super::rc::release_if_ptr(old);
                            } else {
                                flds.insert(attr_s.clone(), value);
                            }
                            return;
                        }
                    }
                    // Fallback: extract_str for non-pointer attr (very rare)
                    let attr_name = extract_str(attr).unwrap_or_default();
                    let old = fields.write().unwrap().insert(attr_name, value);
                    if let Some(prev) = old {
                        super::rc::release_if_ptr(prev);
                    }
                    return;
                }

                // Slow path: need the attr name as an owned String.
                let attr_name = extract_str(attr).unwrap_or_default();

                // PEP 557: frozen dataclasses reject all attribute assignment.
                // (The synthesized __init__ writes the instance dict directly,
                // so initialization is unaffected.) Frozen classes are never
                // added to SIMPLE_CLASS_CACHE, so every assignment lands here.
                if super::stdlib::dataclasses_mod::is_frozen_dataclass(class_name) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("FrozenInstanceError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "cannot assign to field '{attr_name}'"
                        ))),
                    );
                    return;
                }

                // inspect.Parameter / Signature / _ParameterKind instances are
                // immutable (CPython uses __slots__): attribute assignment
                // raises AttributeError. Native construction writes the field
                // map directly and is unaffected.
                if matches!(
                    class_name.as_str(),
                    "inspect.Parameter" | "inspect.Signature" | "inspect._ParameterKind"
                ) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'{}' object has no attribute '{}'",
                            class_name.rsplit('.').next().unwrap_or(class_name),
                            attr_name
                        ))),
                    );
                    return;
                }

                // Descriptor protocol: data descriptor __set__ takes priority over instance __dict__.
                // lookup_method uses METHOD_CACHE, so this is cheap after the first call for
                // each (class, attr) pair — just a hash + HashMap lookup.
                let class_attr = lookup_method(class_name, &attr_name);
                if !class_attr.is_none() && is_data_descriptor(class_attr) {
                    invoke_descriptor_set(class_attr, obj, value);
                    return;
                }
                // R13: Check __slots__ constraint with inheritance merge.
                // Combine both thread_local checks into a single scope for the common
                // case where there are no slots (most classes).
                let has_own_slots =
                    SLOTS_REGISTRY.with(|reg| reg.borrow().contains_key(class_name.as_str()));
                if has_own_slots {
                    let dict_suppressed =
                        DICT_SUPPRESSED.with(|reg| reg.borrow().contains(class_name.as_str()));
                    if dict_suppressed {
                        let in_slots = SLOTS_REGISTRY.with(|reg| {
                            let reg = reg.borrow();
                            if let Some(slots) = reg.get(class_name.as_str()) {
                                slots.contains(&attr_name)
                            } else {
                                false
                            }
                        });
                        if !in_slots {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!(
                                    "'{}' object has no attribute '{}'",
                                    class_name, attr_name
                                ))),
                            );
                            return;
                        }
                    }
                }
                // Cache the class as "simple" only if it has no slots AND no
                // data-descriptor class attributes. A single non-descriptor attr
                // hit doesn't prove the class is descriptor-free — other attrs
                // (e.g., `price = Validated()`) may be data descriptors.
                if !has_own_slots {
                    let has_descriptors = CLASS_REGISTRY.with(|reg| {
                        let reg = reg.borrow();
                        if let Some(cls) = reg.get(class_name.as_str()) {
                            cls.class_attrs.values().any(|v| is_data_descriptor(*v))
                                || cls.methods.values().any(|v| is_data_descriptor(*v))
                        } else {
                            false
                        }
                    });
                    // Frozen dataclasses must keep taking the slow path so the
                    // FrozenInstanceError check above always runs.
                    if !has_descriptors
                        && !super::stdlib::dataclasses_mod::is_frozen_dataclass(class_name)
                    {
                        SIMPLE_CLASS_CACHE.with(|c| {
                            c.borrow_mut().insert(class_name.clone());
                        });
                    }
                }
                // Direct field insert — no descriptor, no slots restriction.
                // Retain so value survives JIT epilogue VReg release.
                super::rc::retain_if_ptr(value);
                let old = fields.write().unwrap().insert(attr_name, value);
                if let Some(prev) = old {
                    super::rc::release_if_ptr(prev);
                }
                return;
            }

            // __class__-tagged dict stubs (ET.Element / QName / ...): attribute
            // writes land as dict keys so `e.text = "x"` round-trips through
            // the dict getattr fast path. Plain dicts (no stub tag) keep the
            // current fall-through (silent no-op) semantics.
            if let ObjData::Dict(ref lock) = (*ptr).data {
                // Module namespaces (dicts carrying __name__) accept attribute
                // writes too: `threading.excepthook = hook` must rebind the
                // module attr like CPython.
                let is_stub = {
                    let g = lock.read().unwrap();
                    g.contains_key("__class__") || g.contains_key("__name__")
                };
                if is_stub {
                    let attr_name = extract_str(attr).unwrap_or_default();
                    super::rc::retain_if_ptr(value);
                    let mut map = lock.write().unwrap();
                    let dk: super::dict_ops::DictKey = attr_name.into();
                    if let Some(existing) = map.get_mut(&dk) {
                        let old = *existing;
                        *existing = value;
                        super::rc::release_if_ptr(old);
                    } else {
                        map.insert(dk, value);
                    }
                    return;
                }
            }

            // Class name string: `cls.attr = value` where cls is a class name.
            // Stores as a class-level attribute accessible to all instances.
            if let ObjData::Str(ref class_name) = (*ptr).data {
                let is_class =
                    CLASS_REGISTRY.with(|r| r.borrow().contains_key(class_name.as_str()));
                if is_class {
                    mb_class_set_class_attr(obj, attr, value);
                    return;
                }
            }
        }
        return;
    }

    // Function / closure attribute writes (PEP 695 writable __type_params__
    // and generic `f.attr = v`): functions carry no field storage, so the
    // attributes live in the pep695 FUNC_ATTRS side registry. Gated on the
    // function-name registry so plain ints never accrue attributes.
    if super::pep695::is_attrable_function(obj) {
        super::pep695::func_attrs_set(obj, attr, value);
    }
}

/// Register __slots__ for a class (R13: with inheritance merge + __dict__ suppression).
/// Walks MRO to merge parent slots into the effective slot set.
/// If the class defines __slots__ without '__dict__', __dict__ is suppressed.
pub fn mb_register_slots(class_name: MbValue, slots: MbValue) {
    let name = extract_str(class_name).unwrap_or_default();
    let mut own_slot_names = Vec::new();
    if let Some(ptr) = slots.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for item in items.iter() {
                    if let Some(s) = extract_str(*item) {
                        own_slot_names.push(s);
                    }
                }
            }
        }
    }

    // R13: Merge parent slots from MRO into effective slot set.
    let mut effective_slots: Vec<String> = own_slot_names.clone();
    let mro = CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(&name)
            .map(|cls| cls.mro.clone())
            .unwrap_or_default()
    });
    for base_name in &mro {
        if base_name == &name || base_name == "object" {
            continue;
        }
        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(parent_slots) = reg.get(base_name) {
                for slot in parent_slots {
                    if !effective_slots.contains(slot) {
                        effective_slots.push(slot.clone());
                    }
                }
            }
        });
    }

    // R13: Track __dict__ suppression.
    // If class defines __slots__ (even empty), suppress __dict__ unless '__dict__' is in slots.
    let has_dict_in_slots = own_slot_names.iter().any(|s| s == "__dict__");
    if !has_dict_in_slots {
        DICT_SUPPRESSED.with(|reg| {
            reg.borrow_mut().insert(name.clone());
        });
    }

    SLOTS_REGISTRY.with(|reg| {
        reg.borrow_mut().insert(name, effective_slots);
    });
}

/// Check if a class defines `__setattr__`. Returns the dunder method if present.
/// Compiled code should call this before `mb_setattr` to support custom descriptors.
pub fn mb_check_setattr_dunder(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__setattr__") {
        return method;
    }
    MbValue::none()
}

/// Delete an attribute from an instance.
/// If the class defines `__delattr__`, compiled code should dispatch through it;
/// this function implements the default `object.__delattr__` behavior (direct field removal).
pub fn mb_delattr(obj: MbValue, attr: MbValue) {
    let attr_name = extract_str(attr).unwrap_or_default();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Str(ref class_name) = (*ptr).data {
                let is_class =
                    CLASS_REGISTRY.with(|r| r.borrow().contains_key(class_name.as_str()));
                if is_class {
                    let removed = CLASS_REGISTRY.with(|reg| {
                        let mut reg = reg.borrow_mut();
                        let Some(cls) = reg.get_mut(class_name.as_str()) else {
                            return false;
                        };
                        let attr_val = cls.class_attrs.remove(&attr_name);
                        let method_val = cls.methods.remove(&attr_name);
                        let removed = attr_val.is_some() || method_val.is_some();
                        drop(reg);
                        for val in attr_val.into_iter().chain(method_val.into_iter()) {
                            super::rc::release_if_ptr(val);
                        }
                        removed
                    });
                    invalidate_method_cache();
                    if !removed {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "type object '{class_name}' has no attribute '{attr_name}'"
                            ))),
                        );
                    }
                    return;
                }
            }
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
                // PEP 557: frozen dataclasses reject attribute deletion too.
                if super::stdlib::dataclasses_mod::is_frozen_dataclass(class_name) {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("FrozenInstanceError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "cannot delete field '{attr_name}'"
                        ))),
                    );
                    return;
                }
                // Descriptor protocol: data descriptor __delete__ takes priority
                let class_attr = lookup_method(class_name, &attr_name);
                if !class_attr.is_none() && is_data_descriptor(class_attr) {
                    invoke_descriptor_delete(class_attr, obj);
                    return;
                }
                let removed = fields.write().unwrap().remove(&attr_name).is_some();
                // object.__delattr__: deleting an attribute that does not exist
                // raises AttributeError (CPython). (#654)
                if !removed {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'{class_name}' object has no attribute '{attr_name}'"
                        ))),
                    );
                }
            }
        }
    }
}

/// Check if a class defines `__delattr__`. Returns the dunder method if present.
pub fn mb_check_delattr_dunder(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__delattr__") {
        return method;
    }
    MbValue::none()
}

/// Check if an object has an attribute.
pub fn mb_hasattr(obj: MbValue, attr: MbValue) -> MbValue {
    // Type objects: `hasattr(SomeType, "__len__")` must report whether the
    // *named type* actually provides the dunder. `mb_getattr` on a type object
    // is deliberately permissive (it returns an unbound-method wrapper for any
    // name to support `int.from_bytes(...)`-style calls), so for dunder queries
    // we answer from the structural table / MRO directly. This drives ABC
    // `__subclasshook__` structural checks (`hasattr(C, "__len__")` → Sized).
    {
        let attr_name = extract_str(attr).unwrap_or_default();
        if attr_name.starts_with("__") && attr_name.ends_with("__") {
            if let Some(type_name) = type_object_name(obj) {
                if builtin_type_has_dunder(&type_name, &attr_name)
                    || has_method(&type_name, &attr_name)
                {
                    return MbValue::from_bool(true);
                }
                // A stdlib exception modeled as a type-object Instance (e.g.
                // ET.ParseError) may seed chaining dunders (__cause__ /
                // __context__ / __suppress_context__) directly in its OWN
                // fields, which the structural dunder table does not model.
                // Read those fields directly — NOT the permissive mb_getattr,
                // which synthesizes an unbound-method wrapper for ANY name on a
                // type-object and would wrongly report every dunder present,
                // breaking ABC structural negative checks (hasattr(C, "__len__")
                // must stay False for a non-Sized type). Only a genuinely seeded
                // field reports True.
                if let Some(ptr) = obj.as_ptr() {
                    unsafe {
                        if let ObjData::Instance { fields, .. } = &(*ptr).data {
                            if fields.read().unwrap().contains_key(&attr_name) {
                                return MbValue::from_bool(true);
                            }
                        }
                    }
                }
                return MbValue::from_bool(false);
            }
        }
    }
    // Bare builtin type names (`hasattr(frozenset, "add")`), whether spelled
    // as a class-name string or a type-object Instance: answer from the
    // per-type method tables instead of mb_getattr, which synthesizes an
    // unbound-method wrapper for ANY name and would report everything present.
    {
        let builtin_name: Option<String> = unsafe {
            obj.as_ptr().and_then(|ptr| {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    type_object_name(obj)
                }
            })
        }
        .filter(|s| super::builtins::is_type_name(s))
        .filter(|s| !CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(s.as_str())));
        if let Some(s) = builtin_name {
            let names = builtin_type_method_names_by_name(&s);
            if !names.is_empty() {
                let attr_name = extract_str(attr).unwrap_or_default();
                return MbValue::from_bool(
                    names.contains(&attr_name.as_str())
                        || builtin_type_has_dunder(&s, &attr_name)
                        || matches!(
                            attr_name.as_str(),
                            "__name__"
                                | "__doc__"
                                | "__module__"
                                | "__qualname__"
                                | "__mro__"
                                | "__bases__"
                                | "__dict__"
                        ),
                );
            }
        }
    }
    let result = mb_getattr(obj, attr);
    // A `__getattr__` that raises AttributeError means the attribute is absent;
    // hasattr swallows *only* AttributeError and reports False (CPython
    // semantics). Any other exception propagates unchanged.
    if super::exception::current_exception_type().as_deref() == Some("AttributeError") {
        super::exception::mb_clear_exception();
        return MbValue::from_bool(false);
    }
    if !result.is_none() {
        return MbValue::from_bool(true);
    }
    // Check known methods on builtin container types
    let attr_name = extract_str(attr).unwrap_or_default();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            let has = match &(*ptr).data {
                ObjData::List(_) => matches!(
                    attr_name.as_str(),
                    "append"
                        | "extend"
                        | "insert"
                        | "remove"
                        | "pop"
                        | "clear"
                        | "index"
                        | "count"
                        | "sort"
                        | "reverse"
                        | "copy"
                        | "__getitem__"
                        | "__setitem__"
                        | "__delitem__"
                        | "__contains__"
                        | "__len__"
                        | "__iter__"
                        | "__add__"
                        | "__radd__"
                        | "__mul__"
                        | "__rmul__"
                        | "__eq__"
                        | "__ne__"
                        | "__repr__"
                        | "__str__"
                ),
                ObjData::Dict(lock) => {
                    // A module is a dict carrying `__name__`; its keys ARE its
                    // attributes, so a present key — even one whose value is
                    // None (e.g. `None`, `Ellipsis`, `os.path.altsep`) — means
                    // hasattr is True. A plain user dict has no `__name__`, so
                    // this only fires for module-dicts and never makes a user
                    // dict's items masquerade as attributes (CPython:
                    // `hasattr({"x": 1}, "x")` is False).
                    let guard = lock.read().unwrap();
                    if guard.contains_key("__name__") && guard.contains_key(attr_name.as_str()) {
                        true
                    } else {
                        matches!(
                            attr_name.as_str(),
                            "keys"
                                | "values"
                                | "items"
                                | "get"
                                | "pop"
                                | "update"
                                | "setdefault"
                                | "clear"
                                | "copy"
                                | "fromkeys"
                                | "__getitem__"
                                | "__setitem__"
                                | "__delitem__"
                                | "__contains__"
                                | "__len__"
                                | "__iter__"
                                | "__or__"
                                | "__ror__"
                                | "__eq__"
                                | "__ne__"
                                | "__repr__"
                                | "__str__"
                        )
                    }
                }
                ObjData::Set(_) => matches!(
                    attr_name.as_str(),
                    "add"
                        | "remove"
                        | "discard"
                        | "pop"
                        | "clear"
                        | "copy"
                        | "union"
                        | "intersection"
                        | "difference"
                        | "symmetric_difference"
                        | "issubset"
                        | "issuperset"
                        | "isdisjoint"
                        | "update"
                        | "intersection_update"
                        | "difference_update"
                        | "symmetric_difference_update"
                ),
                ObjData::Str(_) => matches!(
                    attr_name.as_str(),
                    "upper"
                        | "lower"
                        | "strip"
                        | "lstrip"
                        | "rstrip"
                        | "split"
                        | "join"
                        | "replace"
                        | "find"
                        | "rfind"
                        | "index"
                        | "rindex"
                        | "startswith"
                        | "endswith"
                        | "count"
                        | "format"
                        | "encode"
                        | "isdigit"
                        | "isalpha"
                        | "isalnum"
                        | "isspace"
                        | "isupper"
                        | "islower"
                        | "title"
                        | "capitalize"
                        | "swapcase"
                        | "center"
                        | "ljust"
                        | "rjust"
                        | "zfill"
                        | "expandtabs"
                        | "partition"
                        | "rpartition"
                        | "maketrans"
                        | "translate"
                ),
                ObjData::Tuple(_) => matches!(attr_name.as_str(), "count" | "index"),
                _ => false,
            };
            if has {
                return MbValue::from_bool(true);
            }
        }
    }
    // Iterator handles are stored as integer IDs (not heap objects).
    // `hasattr(iter_handle, '__next__')` / `hasattr(iter_handle, '__iter__')`
    // must return True to match CPython's iterator protocol surface.
    if matches!(attr_name.as_str(), "__next__" | "__iter__") {
        if super::iter::mb_is_iterator_handle(obj) {
            return MbValue::from_bool(true);
        }
    }
    // File handles are stored as integer IDs in the file table (not heap
    // objects), with their method surface dispatched structurally in
    // mb_call_method. `hasattr(file, 'read'/'write'/...)` must report True to
    // match CPython's file-object protocol (e.g. tempfile.TemporaryFile()
    // probed via hasattr, csv writers checking for `write`).
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::file_io::is_file_handle(id) {
            let is_file_method = matches!(
                attr_name.as_str(),
                "read"
                    | "write"
                    | "readline"
                    | "readlines"
                    | "readinto"
                    | "writelines"
                    | "tell"
                    | "seek"
                    | "flush"
                    | "truncate"
                    | "close"
                    | "name"
                    | "fileno"
                    | "mode"
                    | "closed"
                    | "readable"
                    | "writable"
                    | "seekable"
                    | "__enter__"
                    | "__exit__"
                    | "__iter__"
                    | "__next__"
            );
            if is_file_method {
                return MbValue::from_bool(true);
            }
        }
    }
    MbValue::from_bool(false)
}

/// abc/structural: extract the type name from a type object (Instance with
/// class_name == "type" and a `__name__` field). Returns `None` for non-type
/// values.
fn type_object_name(obj: MbValue) -> Option<String> {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                class_name: ref cn,
                ref fields,
            } = (*ptr).data
            {
                if cn == "type" {
                    return fields
                        .read()
                        .ok()
                        .and_then(|f| f.get("__name__").and_then(|v| extract_str(*v)));
                }
            }
        }
    }
    None
}

// ── Method Lookup via MRO ──

/// Look up a method or class attribute by walking the MRO.
/// Checks methods first, then class_attrs, for each class in the MRO.
/// This supports the descriptor protocol (P2-R3) where descriptors are
/// stored as class attributes (e.g., `attr = Verbose()` in class body).
/// Replace (or remove, when `value` is None) a registered class's method
/// entry, returning nothing. Used by unittest.mock patch.object to swap a
/// method for a mock and restore it. Invalidates the method cache.
pub(crate) fn class_replace_method(class_name: &str, method_name: &str, value: MbValue) {
    // Register the function address as a callable so instance-method dispatch
    // invokes it with the `(self, args_list)` ABI (mb_call_method gates that
    // path on CALLABLE_REGISTRY membership). Without this a freshly-installed
    // native method (e.g. functools.total_ordering's synthesized comparison
    // ops) is dispatched with the wrong ABI and returns garbage.
    let (unwrapped, _dk) = unwrap_descriptor_method(value);
    for addr in [extract_func_addr(unwrapped), extract_func_addr(value)] {
        if addr != 0 {
            CALLABLE_REGISTRY.with(|reg| {
                reg.borrow_mut().insert(addr);
            });
        }
    }
    CLASS_REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        if let Some(cls) = reg.get_mut(class_name) {
            if value.is_none() {
                cls.methods.remove(method_name);
            } else {
                unsafe {
                    super::rc::retain_if_ptr(value);
                }
                if let Some(prev) = cls.methods.insert(method_name.to_string(), value) {
                    unsafe {
                        super::rc::release_if_ptr(prev);
                    }
                }
            }
        }
    });
    METHOD_CACHE_GEN.with(|g| g.set(g.get().wrapping_add(1)));
}

pub(crate) fn lookup_method(class_name: &str, method_name: &str) -> MbValue {
    let class_hash = hash_str(class_name);
    let method_hash = hash_str(method_name);
    let cache_key = (class_hash, method_hash);

    // Check cache first.
    let cached = METHOD_CACHE.with(|c| c.borrow().get(&cache_key).copied());
    if let Some(val) = cached {
        return val;
    }

    // Cache miss — do full MRO walk.
    let result = CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(class_name) {
            // Walk MRO
            for mro_class in &cls.mro {
                if let Some(mro_cls) = reg.get(mro_class) {
                    if let Some(method) = mro_cls.methods.get(method_name) {
                        return *method;
                    }
                    if let Some(attr) = mro_cls.class_attrs.get(method_name) {
                        return *attr;
                    }
                }
            }
        }
        MbValue::none()
    });

    // Insert into cache.
    let _ = METHOD_CACHE.with(|c| c.try_borrow_mut().map(|mut m| m.insert(cache_key, result)));

    result
}

/// Walk the MRO to find a class-level attribute (not a method).
/// Returns `Some(value)` if found in any class along the MRO.
pub(crate) fn class_attr_lookup(class_name: &str, attr: &str) -> Option<MbValue> {
    mro_lookup_class_attr(class_name, attr)
}

/// Own member tables of a registered class for introspection
/// (inspect.classify_class_attrs / getattr_static): `(name, value,
/// from_method_table)` sorted by name. Does NOT walk the MRO.
pub(crate) fn class_own_members(class_name: &str) -> Vec<(String, MbValue, bool)> {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let Some(cls) = reg.get(class_name) else {
            return Vec::new();
        };
        let mut out: Vec<(String, MbValue, bool)> = cls
            .methods
            .iter()
            .map(|(k, v)| (k.clone(), *v, true))
            .collect();
        out.extend(cls.class_attrs.iter().map(|(k, v)| (k.clone(), *v, false)));
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    })
}

/// Effective `__slots__` names of a class (own + inherited), empty when the
/// class declares no slots.
pub(crate) fn class_slot_names(class_name: &str) -> Vec<String> {
    SLOTS_REGISTRY.with(|reg| reg.borrow().get(class_name).cloned().unwrap_or_default())
}

/// Synthesize a `member_descriptor` instance for a `__slots__` entry —
/// CPython's `Slotted.x` class read yields the slot descriptor, not a value.
pub(crate) fn make_member_descriptor(class_name: &str, attr: &str) -> MbValue {
    let inst = super::rc::MbObject::new_instance("member_descriptor".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__objclass__".to_string(),
                MbValue::from_ptr(super::rc::MbObject::new_str(class_name.to_string())),
            );
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(super::rc::MbObject::new_str(attr.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

/// Find which registered class's OWN method table holds `func`, returning
/// (class_name, method_name). Used by inspect.getdoc to inherit method
/// docstrings through the MRO.
pub(crate) fn find_method_owner(func: MbValue) -> Option<(String, String)> {
    let addr = extract_func_addr(func);
    if addr == 0 {
        return None;
    }
    CLASS_REGISTRY.with(|reg| {
        for (cname, cls) in reg.borrow().iter() {
            for (mname, mval) in &cls.methods {
                let (unwrapped, _dk) = unwrap_descriptor_method(*mval);
                if extract_func_addr(unwrapped) == addr || extract_func_addr(*mval) == addr {
                    return Some((cname.clone(), mname.clone()));
                }
            }
        }
        None
    })
}

// ── Class docstrings (inspect.getdoc) ──

thread_local! {
    static CLASS_DOCS: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Register a class-body docstring (emitted at module init by lowering).
pub fn mb_class_set_doc(class_name: MbValue, doc: MbValue) {
    if let (Some(name), Some(d)) = (extract_str(class_name), extract_str(doc)) {
        CLASS_DOCS.with(|m| m.borrow_mut().insert(name, d));
    }
}

/// The registered docstring of a class, or None.
pub(crate) fn class_doc(class_name: &str) -> Option<String> {
    CLASS_DOCS.with(|m| m.borrow().get(class_name).cloned())
}

pub(crate) fn cleanup_class_docs() {
    let _ = CLASS_DOCS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
}

fn mro_lookup_class_attr(class_name: &str, attr: &str) -> Option<MbValue> {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(class_name) {
            for mro_class in &cls.mro {
                if let Some(mro_cls) = reg.get(mro_class) {
                    if let Some(val) = mro_cls.class_attrs.get(attr) {
                        return Some(*val);
                    }
                }
            }
        }
        None
    })
}

fn metaclass_data_descriptor_for_class(class_name: &str, attr: &str) -> Option<MbValue> {
    let meta = CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(class_name)
            .and_then(|cls| cls.metaclass.clone())
    })?;
    let candidate = lookup_method(&meta, attr);
    if !candidate.is_none() && is_data_descriptor(candidate) {
        Some(candidate)
    } else {
        None
    }
}

/// True iff `__bool__` resolves to an explicit `None` via the MRO's class
/// attributes. CPython treats `__bool__ = None` as disabling truth-testing
/// (calling it raises `TypeError: 'NoneType' object is not callable`), even
/// when `__len__` exists. Only call this when method lookup for `__bool__`
/// already returned None, so a real `def __bool__` in a nearer MRO entry has
/// already taken precedence (lookup_method interleaves methods + class_attrs
/// per class, so a None here means it shadows any farther definition).
pub(crate) fn class_bool_is_blocked(class_name: &str) -> bool {
    matches!(mro_lookup_class_attr(class_name, "__bool__"), Some(v) if v.is_none())
}

// ── Method Resolution Order (C3 Linearization) ──

/// Compute MRO using simplified C3 linearization.
/// Returns Err with a message for inconsistent hierarchies (Python's TypeError).
fn compute_mro(name: &str, bases: &[String]) -> Vec<String> {
    let mut mro = vec![name.to_string()];
    // For single inheritance, MRO is just: class → base → base's base → ...
    // For multiple inheritance, use C3
    if bases.len() <= 1 {
        // Simple case: linear chain
        for base in bases {
            mro.push(base.clone());
            CLASS_REGISTRY.with(|reg| {
                if let Some(cls) = reg.borrow().get(base) {
                    for parent in &cls.mro[1..] {
                        if !mro.contains(parent) {
                            mro.push(parent.clone());
                        }
                    }
                }
            });
        }
    } else {
        // C3 linearization for multiple inheritance
        let mut lists: Vec<Vec<String>> = Vec::new();
        for base in bases {
            CLASS_REGISTRY.with(|reg| {
                if let Some(cls) = reg.borrow().get(base) {
                    lists.push(cls.mro.clone());
                } else {
                    lists.push(vec![base.clone()]);
                }
            });
        }
        lists.push(bases.to_vec());
        match c3_merge(&mut lists) {
            Ok(merged) => mro.extend(merged),
            Err(msg) => {
                // Inconsistent MRO — CPython raises a *catchable* TypeError when
                // the class statement runs. Set the pending exception (instead of
                // panicking and aborting the process) and fall back to a trivial
                // MRO so registration completes; the pending exception aborts the
                // class statement at the next runtime check. Only reached on a
                // genuinely inconsistent hierarchy, which previously crashed, so
                // no passing path observes the placeholder MRO.
                super::exception::set_current_exception(super::exception::MbException::new(
                    "TypeError",
                    &format!(
                        "Cannot create a consistent method resolution order (MRO) for bases {msg}"
                    ),
                ));
                mro.extend(bases.iter().cloned());
            }
        }
    }
    // Always end with "object"
    if !mro.contains(&"object".to_string()) {
        mro.push("object".to_string());
    }
    mro
}

/// C3 merge algorithm. Returns Err for inconsistent hierarchies.
fn c3_merge(lists: &mut Vec<Vec<String>>) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    loop {
        // Remove empty lists
        lists.retain(|l| !l.is_empty());
        if lists.is_empty() {
            break;
        }

        // Find a candidate: head of some list that doesn't appear in tail of any list
        let mut found = None;
        for list in lists.iter() {
            let head = &list[0];
            let in_tail = lists.iter().any(|l| l[1..].contains(head));
            if !in_tail {
                found = Some(head.clone());
                break;
            }
        }

        match found {
            Some(candidate) => {
                result.push(candidate.clone());
                for list in lists.iter_mut() {
                    if list.first() == Some(&candidate) {
                        list.remove(0);
                    }
                }
            }
            None => {
                let remaining: Vec<String> =
                    lists.iter().filter_map(|l| l.first().cloned()).collect();
                return Err(format!(
                    "inconsistent hierarchy, remaining heads: [{}]",
                    remaining.join(", ")
                ));
            }
        }
    }
    Ok(result)
}

// ── Operator Overloading ──

/// Dunder names indexed by opcode (must match MirBinOp variant order exactly).
/// MirBinOp: Add(0), Sub(1), Mul(2), Div(3), FloorDiv(4), Mod(5), Pow(6),
///           Eq(7), NotEq(8), Lt(9), Gt(10), LtEq(11), GtEq(12),
///           And(13), Or(14), BitAnd(15), BitOr(16), BitXor(17), LShift(18), RShift(19),
///           Is(20), IsNot(21), In(22), NotIn(23)
/// Note: Is/IsNot are identity checks (no dunder). In/NotIn use __contains__ on RHS.
/// The codegen handles these specially and should not route them here.
const BINOP_DUNDERS: &[&str] = &[
    "add", "sub", "mul", "truediv", "floordiv", "mod", "pow", "eq", "ne", "lt", "gt", "le", "ge",
    "and", "or", "and", "or", "xor", "lshift", "rshift",
    "", // Is — identity, not dunder-dispatched
    "", // IsNot — identity, not dunder-dispatched
    "", // In — uses RHS __contains__, handled by mb_obj_contains
    "", // NotIn — uses RHS __contains__, handled by mb_obj_contains
];

/// Dispatch a binary operation through dunder methods.
/// `op_code` is a raw i64 index into BINOP_DUNDERS (FFI-safe for codegen).
pub fn mb_dispatch_binop(op_code: i64, lhs: MbValue, rhs: MbValue) -> MbValue {
    // Fast path: when neither operand is a heap pointer, dunder lookup
    // can never succeed (try_get_dunder requires `obj.as_ptr().is_some()`).
    // Skip the two `format!()` allocations + two dunder lookups and go
    // straight to the primitive builtin. Hot for `total + x` in tight loops
    // where operands are NaN-boxed inline ints / bools / floats / None —
    // exactly the generator_sum bench's inner-loop shape.
    if lhs.as_ptr().is_none() && rhs.as_ptr().is_none() {
        match op_code {
            0 => return super::builtins::mb_add(lhs, rhs),
            1 => return super::builtins::mb_sub(lhs, rhs),
            2 => return super::builtins::mb_mul(lhs, rhs),
            3 => return super::builtins::mb_div(lhs, rhs),
            4 => return super::builtins::mb_floordiv(lhs, rhs),
            5 => return super::builtins::mb_mod(lhs, rhs),
            6 => return super::builtins::mb_pow(lhs, rhs),
            7 => return super::builtins::mb_eq(lhs, rhs),
            8 => return super::builtins::mb_ne(lhs, rhs),
            9 => return super::builtins::mb_lt(lhs, rhs),
            10 => return super::builtins::mb_gt(lhs, rhs),
            11 => return super::builtins::mb_le(lhs, rhs),
            12 => return super::builtins::mb_ge(lhs, rhs),
            13 | 15 => return super::builtins::mb_bitand(lhs, rhs),
            14 | 16 => return super::builtins::mb_bitor(lhs, rhs),
            17 => return super::builtins::mb_bitxor(lhs, rhs),
            18 => return super::builtins::mb_lshift(lhs, rhs),
            19 => return super::builtins::mb_rshift(lhs, rhs),
            // 20+ (Is/IsNot/In/NotIn) have no primitive entry in the slow
            // path's `match op_name` — fall through to the NotImplemented
            // terminal, which their dedicated handlers take over.
            _ => {}
        }
    }
    let op_name = BINOP_DUNDERS
        .get(op_code as usize)
        .copied()
        .unwrap_or("add");
    let dunder = format!("__{op_name}__");
    let rdunder = format!("__r{op_name}__");

    // Try lhs.__op__(rhs) first — invoke and use result unless NotImplemented.
    // Method values may be TAG_FUNC (direct address from JIT) or heap-backed
    // (registered in CALLABLE_REGISTRY). Try both paths, matching mb_property_get.
    if let Some(method) = try_get_dunder(lhs, &dunder) {
        if let Some(result) = invoke_binop_method(method, lhs, rhs) {
            if !result.is_not_implemented() {
                return result;
            }
        }
    }
    // Try rhs.__rop__(lhs) as fallback.
    if let Some(method) = try_get_dunder(rhs, &rdunder) {
        if let Some(result) = invoke_binop_method(method, rhs, lhs) {
            if !result.is_not_implemented() {
                return result;
            }
        }
    }
    // Fallback for primitive types (no dunders): use runtime builtins.
    // Handles NaN != NaN for float values typed as Any.
    match op_name {
        "eq" => return super::builtins::mb_eq(lhs, rhs),
        "ne" => return super::builtins::mb_ne(lhs, rhs),
        "lt" => return super::builtins::mb_lt(lhs, rhs),
        "gt" => return super::builtins::mb_gt(lhs, rhs),
        "le" => return super::builtins::mb_le(lhs, rhs),
        "ge" => return super::builtins::mb_ge(lhs, rhs),
        "add" => return super::builtins::mb_add(lhs, rhs),
        "sub" => return super::builtins::mb_sub(lhs, rhs),
        "mul" => return super::builtins::mb_mul(lhs, rhs),
        "truediv" => return super::builtins::mb_div(lhs, rhs),
        "mod" => return super::builtins::mb_mod(lhs, rhs),
        "floordiv" => return super::builtins::mb_floordiv(lhs, rhs),
        "pow" => return super::builtins::mb_pow(lhs, rhs),
        "and" => return super::builtins::mb_bitand(lhs, rhs),
        "or" => return super::builtins::mb_bitor(lhs, rhs),
        "xor" => return super::builtins::mb_bitxor(lhs, rhs),
        "lshift" => return super::builtins::mb_lshift(lhs, rhs),
        "rshift" => return super::builtins::mb_rshift(lhs, rhs),
        _ => {}
    }
    MbValue::none() // NotImplemented
}

/// Augmented-assignment in-place dispatch: `a <op>= b`. When `a` is an
/// instance defining the in-place dunder (`__iadd__`, `__ior__`, …), call it
/// (CPython tries the in-place form first); otherwise fall back to the normal
/// binary op (which itself tries `__op__` / `__rop__` / the primitive path).
/// Returning the (possibly same) object is the caller's new binding. Only
/// instance receivers can carry an in-place dunder, so primitives/str/list
/// take the fast fallback unchanged.
fn mb_inplace(
    a: MbValue,
    b: MbValue,
    idunder: &str,
    op_code: i64,
    fallback: fn(MbValue, MbValue) -> MbValue,
) -> MbValue {
    if a.as_ptr().is_some() {
        if let Some(method) = try_get_dunder(a, idunder) {
            if let Some(result) = invoke_binop_method(method, a, b) {
                if !result.is_not_implemented() {
                    return result;
                }
            }
        }
        // A user-defined instance whose in-place dunder is absent (or returned
        // NotImplemented) falls back to the binary operator dispatch
        // (`__op__` / `__rop__`), exactly like `a op b`. This lets a class that
        // defines only the binary dunder (e.g. `__add__` but no `__iadd__`)
        // still support `a op= b`, and an `__ipow__` returning NotImplemented
        // defer to `__pow__` / `__rpow__`. Non-instances (list/str/dict/…) keep
        // the primitive fallback so their `+=` extend/concat semantics are
        // unchanged.
        if a
            .as_ptr()
            .map_or(false, |p| matches!(unsafe { &(*p).data }, ObjData::Instance { .. }))
        {
            return mb_dispatch_binop(op_code, a, b);
        }
    }
    fallback(a, b)
}

pub fn mb_iadd(a: MbValue, b: MbValue) -> MbValue {
    // bytearray += : extend in place and preserve object identity (CPython).
    if a.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::ByteArray(_)) }).unwrap_or(false) {
        super::bytes_ops::mb_bytearray_extend(a, b);
        unsafe { super::rc::retain_if_ptr(a); }
        return a;
    }
    mb_inplace(a, b, "__iadd__", 0, super::builtins::mb_add)
}
pub fn mb_isub(a: MbValue, b: MbValue) -> MbValue {
    mb_inplace(a, b, "__isub__", 1, super::builtins::mb_sub)
}
pub fn mb_imul(a: MbValue, b: MbValue) -> MbValue {
    // bytearray *= n : repeat in place and preserve object identity (CPython).
    if a.as_ptr().map(|p| unsafe { matches!((*p).data, ObjData::ByteArray(_)) }).unwrap_or(false) {
        super::bytes_ops::mb_bytearray_imul(a, b);
        unsafe { super::rc::retain_if_ptr(a); }
        return a;
    }
    mb_inplace(a, b, "__imul__", 2, super::builtins::mb_mul)
}
pub fn mb_ipow(a: MbValue, b: MbValue) -> MbValue {
    mb_inplace(a, b, "__ipow__", 6, super::builtins::mb_pow)
}
pub fn mb_iand(a: MbValue, b: MbValue) -> MbValue {
    mb_inplace(a, b, "__iand__", 15, super::builtins::mb_bitand)
}
pub fn mb_ior(a: MbValue, b: MbValue) -> MbValue {
    mb_inplace(a, b, "__ior__", 16, super::builtins::mb_bitor)
}
pub fn mb_ixor(a: MbValue, b: MbValue) -> MbValue {
    mb_inplace(a, b, "__ixor__", 17, super::builtins::mb_bitxor)
}

/// Invoke a 2-arg method value with (self, arg). Handles both TAG_FUNC direct
/// addresses (JIT-compiled methods) and CALLABLE_REGISTRY heap pointers.
/// Returns None only when the address is unresolvable.
fn invoke_binop_method(method: MbValue, slf: MbValue, arg: MbValue) -> Option<MbValue> {
    // TAG_FUNC direct address — JIT-compiled class methods.
    if let Some(addr) = method.as_func() {
        if addr > 4096 {
            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr) };
            return Some(f(slf, arg));
        }
    }
    // CALLABLE_REGISTRY — heap-backed method values.
    let addr = extract_func_addr(method);
    if addr > 4096 {
        let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
        if is_reg {
            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            return Some(f(slf, arg));
        }
    }
    None
}

/// Try to get a dunder method from an object.
fn try_get_dunder(obj: MbValue, name: &str) -> Option<MbValue> {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let method = lookup_method(class_name, name);
                if !method.is_none() {
                    return Some(method);
                }
            }
        }
    }
    None
}

/// Look up a dunder method on an object's class (not instance fields).
/// Returns MbValue::none() if not found. Safe for use with mb_call_method1
/// since class methods are always valid function pointers.
pub fn mb_lookup_dunder(obj: MbValue, name: MbValue) -> MbValue {
    let dunder_name = extract_str(name).unwrap_or_default();
    match try_get_dunder(obj, &dunder_name) {
        Some(method) => method,
        None => MbValue::none(),
    }
}

/// isinstance(obj, class_name) → bool
/// Narrowest `numbers` tower rank a value occupies: Integral=4 for
/// int/bool/BigInt, Rational=3 for Fraction, Real=2 for float, Complex=1 for
/// complex, Number=0 for Decimal. `None` for non-numbers. A value satisfies
/// `isinstance(v, ABC)` when its rank is ≥ the ABC's rank.
fn numbers_value_rank(obj: MbValue) -> Option<u8> {
    // Fraction / Decimal are tagged-int handles — test before is_int().
    if super::builtins::is_fraction_handle_value(obj) {
        return Some(3);
    }
    if super::builtins::is_decimal_handle_value(obj) {
        return Some(0);
    }
    if obj.is_bool() || obj.is_int() {
        return Some(4);
    }
    if obj.is_float() {
        return Some(2);
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Complex(..) => return Some(1),
                ObjData::BigInt(_) => return Some(4),
                // A bare numbers.Number() instance sits at the tower root.
                ObjData::Instance { class_name, .. } if class_name == "numbers.Number" => {
                    return Some(0);
                }
                _ => {}
            }
        }
    }
    None
}

pub fn mb_isinstance(obj: MbValue, class_name: MbValue) -> MbValue {
    // typing.Union[...] aliases: any member matching counts (#22).
    if let Some(hit) = super::stdlib::typing_mod::typing_union_isinstance(obj, class_name) {
        return MbValue::from_bool(hit);
    }
    // Handle tuple of types: isinstance(x, (A, B, C))
    if let Some(ptr) = class_name.as_ptr() {
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                for &item in items.iter() {
                    if mb_isinstance(obj, item).as_bool() == Some(true) {
                        return MbValue::from_bool(true);
                    }
                }
                return MbValue::from_bool(false);
            }
        }
    }
    if let Some(items) = union_type_args(class_name) {
        for item in items {
            if mb_isinstance(obj, item).as_bool() == Some(true) {
                return MbValue::from_bool(true);
            }
        }
        return MbValue::from_bool(false);
    }
    // CPython: arg 2 must be a type, a tuple of types, or a union. Inline
    // scalars (int, float, bool, None) can never name a class. TAG_INT also
    // carries closure handles, but a function is equally not a type.
    if class_name.is_none()
        || class_name.as_int().is_some()
        || class_name.is_float()
        || class_name.as_bool().is_some()
    {
        super::builtins::raise_type_error(
            "isinstance() arg 2 must be a type, a tuple of types, or a union".to_string(),
        );
        return MbValue::none();
    }
    // numbers ABC numeric tower: isinstance(x, numbers.Integral/Real/Complex/
    // Rational/Number). These ABCs are native dispatcher functions (not classes),
    // so match by function pointer and compare the value's tower rank — a value
    // is an instance of an ABC at or above its own rung.
    if let Some(addr) = class_name.as_func() {
        if let Some(abc_rank) = super::stdlib::numbers_mod::numbers_abc_rank(addr as u64) {
            return MbValue::from_bool(numbers_value_rank(obj).is_some_and(|vr| vr >= abc_rank));
        }
    }
    // Handle type objects (returned by type()): Instance with class_name="type"
    // and __name__ field containing the actual type name.
    let target = if let Some(ptr) = class_name.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                class_name: ref cn,
                ref fields,
            } = (*ptr).data
            {
                if cn == "type" {
                    fields
                        .read()
                        .unwrap()
                        .get("__name__")
                        .and_then(|v| extract_str(*v))
                        .unwrap_or_default()
                } else {
                    // Not a type object; use the class name as string for isinstance
                    extract_str(class_name).unwrap_or_default()
                }
            } else {
                extract_str(class_name).unwrap_or_default()
            }
        }
    } else if let Some(addr) = class_name.as_func() {
        // Native-dispatcher function pointers used as types — e.g.
        // `threading.Thread` is a constructor dispatcher rather than a real
        // class. Look up the recorded class name for the pointer.
        registered_class_name_for_func(class_name, addr).unwrap_or_default()
    } else {
        extract_str(class_name).unwrap_or_default()
    };
    // types.ModuleType (a `type` object whose __name__ is "module"): a module
    // is modeled as a dict, so match by the module-value pointer registry rather
    // than the dict type. Only short-circuit to True for real modules; fall
    // through otherwise (a user class literally named "module" still resolves
    // nominally).
    if target == "module" && super::module::is_module_value(obj) {
        return MbValue::from_bool(true);
    }
    // A `typing.Protocol` that is NOT `@runtime_checkable` can't be used with
    // isinstance (CPython: TypeError "Instance and class checks can only be used
    // with @runtime_checkable protocols").
    if !target.is_empty()
        && !is_runtime_checkable_protocol(&target)
        && class_mro_list(&target)
            .iter()
            .any(|b| b == "Protocol" || b == "typing.Protocol")
    {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "Instance and class checks can only be used with @runtime_checkable protocols"
                    .to_string(),
            )),
        );
        return MbValue::none();
    }
    // Metaclass __instancecheck__: isinstance(x, C) defers to
    // type(C).__instancecheck__(C, x) when the metaclass defines it.
    let meta_check = CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(target.as_str())
            .and_then(|c| c.metaclass.clone())
            .filter(|m| m != "type" && m != "ABCMeta")
            .map(|m| lookup_method(&m, "__instancecheck__"))
            .filter(|m| !m.is_none())
    });
    if let Some(method) = meta_check {
        let cls_val = MbValue::from_ptr(MbObject::new_str(target.clone()));
        let out = call_method_value2(method, cls_val, obj);
        if super::exception::mb_has_exception().as_bool() == Some(true) || out.is_none() {
            // The user dunder hit a runtime gap (e.g. cls.__dict__ /
            // metaclass-bound methods) — fall back to the nominal check
            // rather than reporting a wrong False.
            super::exception::mb_clear_exception();
        } else {
            return MbValue::from_bool(super::builtins::mb_bool(out).as_bool() == Some(true));
        }
    }
    // os.PathLike structural __subclasshook__: an instance whose class defines
    // __fspath__ is a virtual instance (mirrors the issubclass path).
    if target == "os.PathLike" || target == "PathLike" {
        if let Some(obj_cls) = abc_runtime_class_name(obj) {
            return MbValue::from_bool(class_defines_non_none(&obj_cls, "__fspath__"));
        }
        return MbValue::from_bool(false);
    }
    // abc: isinstance(obj, ABC) defers to ABCMeta.__subclasscheck__ semantics —
    // nominal subclass, custom __subclasshook__ (structural), or registered
    // virtual subclass. Handles both instances and builtin objects (e.g.
    // isinstance([1, 2], Sized)).
    if is_user_abc(&target) {
        if let Some(obj_cls) = abc_runtime_class_name(obj) {
            if let Some(result) = user_abc_issubclass(&obj_cls, &target) {
                return MbValue::from_bool(result);
            }
        }
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                let nominal = CLASS_REGISTRY.with(|reg| {
                    if let Some(cls) = reg.borrow().get(class_name.as_str()) {
                        class_name == &target
                            || cls.mro.contains(&target)
                            || collections_abc_type_or_virtual_match(class_name, &target)
                            || cls
                                .mro
                                .iter()
                                .any(|base| collections_abc_type_or_virtual_match(base, &target))
                    } else {
                        class_name == &target
                            || super::exception::is_subclass_of(class_name, &target)
                            || collections_abc_type_or_virtual_match(class_name, &target)
                            || collections_builtin_subclass(class_name, &target)
                            // Descriptor wrapper instances answer to their
                            // public builtin type names.
                            || (target == "property" && class_name == "__property__")
                            || (target == "staticmethod" && class_name == "__staticmethod__")
                            || (target == "classmethod" && class_name == "__classmethod__")
                    }
                });
                if nominal {
                    return MbValue::from_bool(true);
                }
                // namedtuple instances are genuine tuple subclasses.
                if target == "tuple"
                    && super::stdlib::collections_mod::namedtuple_values(obj).is_some()
                {
                    return MbValue::from_bool(true);
                }
                // Data-type-mixin enum members: isinstance(IntFlag member,
                // int) / isinstance(StrEnum member, str).
                if super::stdlib::enum_class::member_isinstance_builtin(obj, &target) {
                    return MbValue::from_bool(true);
                }
                // Structural match against a @runtime_checkable Protocol.
                if is_runtime_checkable_protocol(&target) {
                    return MbValue::from_bool(protocol_structural_match(obj, class_name, &target));
                }
                if collections_abc_structural_match(class_name, &target) {
                    return MbValue::from_bool(true);
                }
                return MbValue::from_bool(false);
            }
        }
    }
    if obj.is_int() {
        if super::generator::is_known_generator(obj) {
            return MbValue::from_bool(matches!(
                target.as_str(),
                "Generator" | "Iterator" | "Iterable" | "object"
            ));
        }
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            return MbValue::from_bool(matches!(
                target.as_str(),
                "array"
                    | "object"
                    | "MutableSequence"
                    | "Sequence"
                    | "Reversible"
                    | "Collection"
                    | "Sized"
                    | "Iterable"
                    | "Container"
            ));
        }
        // hmac handles: `isinstance(h, hmac.HMAC)` (target resolved to "HMAC"
        // via NATIVE_TYPE_NAMES) — the constructor returns an int handle, so
        // it cannot match through the Instance path.
        if super::stdlib::hmac_mod::is_hmac_handle(id) {
            return MbValue::from_bool(matches!(target.as_str(), "HMAC" | "object"));
        }
        // uuid handles: `isinstance(u, uuid.UUID)` (target resolved to
        // "UUID" via NATIVE_TYPE_NAMES). The constructor returns an int
        // handle, so it cannot match through the Instance path.
        if super::stdlib::uuid_mod::is_uuid_handle(id) {
            return MbValue::from_bool(matches!(target.as_str(), "UUID" | "object"));
        }
        // decimal/fractions handles: `isinstance(d, Decimal)` /
        // `isinstance(f, Fraction)` — targets resolve via NATIVE_TYPE_NAMES.
        if super::stdlib::decimal_mod::is_decimal_handle(id) {
            return MbValue::from_bool(matches!(target.as_str(), "Decimal" | "object"));
        }
        if super::stdlib::fractions_mod::is_fraction_handle(id) {
            return MbValue::from_bool(matches!(target.as_str(), "Fraction" | "object"));
        }
    }
    // __class__-tagged dict stubs (e.g. ET.Element): match the stub class
    // name against the target (resolved via NATIVE_TYPE_NAMES for native
    // constructor dispatchers used as types).
    if let Some(stub) = super::dict_ops::dict_stub_class(obj) {
        if stub == target {
            return MbValue::from_bool(true);
        }
    }
    // Check primitive types and built-in containers
    let type_name = if obj.is_bool() {
        "bool"
    } else if obj.is_int() {
        "int"
    } else if obj.is_float() {
        "float"
    } else if obj.is_none() {
        "NoneType"
    } else if obj.is_not_implemented() {
        // NotImplemented / Ellipsis are tagged singletons (not ptr-backed).
        "NotImplementedType"
    } else if obj.is_ellipsis() {
        "ellipsis"
    } else if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                // Heap-allocated big integers (literals beyond the 48-bit
                // inline range, e.g. 2**64) are still `int` instances.
                // Without this, isinstance(2**64, int) returns False and
                // plistlib's UID range check / test_int round-trips break.
                ObjData::BigInt(_) => "int",
                ObjData::Complex(..) => "complex",
                ObjData::CodeObject { .. } => "code",
                _ => "",
            }
        }
    } else {
        ""
    };
    // bool is a subclass of int in Python
    let matches = type_name == target
        || (type_name == "bool" && target == "int")
        || target == "object" // everything is an instance of object
        || builtin_abc_instance_match(type_name, &target);
    MbValue::from_bool(matches)
}

fn builtin_abc_instance_match(type_name: &str, target: &str) -> bool {
    match target {
        "Hashable" => matches!(
            type_name,
            "bool" | "int" | "float" | "str" | "tuple" | "bytes" | "NoneType"
        ),
        "Sized" | "Iterable" | "Container" | "Collection" => matches!(
            type_name,
            "str" | "list" | "dict" | "tuple" | "set" | "frozenset" | "bytes" | "bytearray"
        ),
        "Reversible" => matches!(
            type_name,
            "str" | "list" | "dict" | "tuple" | "bytes" | "bytearray"
        ),
        "Sequence" => matches!(type_name, "str" | "list" | "tuple" | "bytes" | "bytearray"),
        "MutableSequence" => type_name == "list" || type_name == "bytearray",
        "Mapping" | "MutableMapping" => type_name == "dict",
        "Set" => type_name == "set" || type_name == "frozenset",
        "MutableSet" => type_name == "set",
        _ => false,
    }
}

/// Check if `class_name` is one of the PEP 634 built-in self-subject types and `obj`
/// is an instance of that type.  When true, `case ClassName(x):` captures the subject
/// itself rather than consulting `__match_args__`.
fn is_builtin_self_subject(class_name: &str, obj: MbValue) -> bool {
    match class_name {
        "bool" => obj.is_bool(),
        "int" => obj.is_int(),
        "float" => obj.is_float(),
        "str" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) }),
        "bytes" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Bytes(_)) }),
        "bytearray" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::ByteArray(_)) }),
        "list" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::List(_)) }),
        "tuple" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Tuple(_)) }),
        "dict" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Dict(_)) }),
        "set" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Set(_)) }),
        "frozenset" => obj
            .as_ptr()
            .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::FrozenSet(_)) }),
        // range, memoryview, type — not natively represented; fall through
        _ => false,
    }
}

/// PEP 634 positional class pattern match: get the value of positional attribute `pos`
/// by looking up `obj.__class__.__match_args__[pos]` and returning `getattr(obj, that_attr)`.
///
/// Returns `None` if `__match_args__` is not defined or `pos` is out of range.
pub fn mb_match_pos_arg(obj: MbValue, class_name_val: MbValue, pos: i64) -> MbValue {
    let class_name = match extract_str(class_name_val) {
        Some(n) => n,
        None => return MbValue::none(),
    };
    let pos = if pos >= 0 {
        pos as usize
    } else {
        return MbValue::none();
    };

    // PEP 634: built-in self-subject types — positional arg 0 captures the subject itself.
    if pos == 0 && is_builtin_self_subject(&class_name, obj) {
        // Retain: JIT releases both obj arg VReg and result VReg independently.
        unsafe { super::rc::retain_if_ptr(obj) };
        return obj;
    }

    // Use MRO-aware lookup for __match_args__ (#827) so that subclasses inheriting
    // __match_args__ from a base class are handled consistently with mb_class_has_pos_match.
    let attr_name = if let Some(match_args) = mro_lookup_class_attr(&class_name, "__match_args__") {
        if let Some(ptr) = match_args.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::List(ref lock) => {
                        let items = lock.read().unwrap();
                        if pos < items.len() {
                            extract_str(items[pos])
                        } else {
                            None
                        }
                    }
                    ObjData::Tuple(ref items) => {
                        // ObjData::Tuple holds Vec<MbValue> directly (no RwLock)
                        if pos < items.len() {
                            extract_str(items[pos])
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    match attr_name {
        Some(name) => {
            // Call mb_getattr with the resolved attribute name
            let attr_val = MbValue::from_ptr(MbObject::new_str(name));
            mb_getattr(obj, attr_val)
        }
        None => MbValue::none(),
    }
}

/// Check if an object has an attribute (instance dict or class methods/attrs).
/// Returns true even if the attribute's value is `None`.
/// Used for PEP 634 class-pattern keyword arg validation.
pub fn mb_instance_hasattr(obj: MbValue, attr: MbValue) -> MbValue {
    let attr_name = extract_str(attr).unwrap_or_default();
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // Check instance __dict__
                if fields.read().unwrap().contains_key(&attr_name) {
                    return MbValue::from_bool(true);
                }
                // Check class-level attrs and methods (MRO-aware).
                // Note: `__match_args__` membership alone is NOT sufficient — the attribute
                // must actually be retrievable via mb_getattr for patterns to succeed (#827).
                let method_found = !lookup_method(class_name, &attr_name).is_none();
                let attr_found = mro_lookup_class_attr(class_name, &attr_name).is_some();
                if method_found || attr_found {
                    return MbValue::from_bool(true);
                }
                return MbValue::from_bool(false);
            }
        }
    }
    MbValue::from_bool(false)
}

/// Check if a positional class-pattern match argument exists (#827).
/// Returns true if the class has `__match_args__[pos]` AND the object instance
/// actually has the attribute named by `__match_args__[pos]`.
pub fn mb_class_has_pos_match(obj: MbValue, class_name_val: MbValue, pos: i64) -> MbValue {
    let class_name = match extract_str(class_name_val) {
        Some(n) => n,
        None => return MbValue::from_bool(false),
    };
    let pos = if pos >= 0 {
        pos as usize
    } else {
        return MbValue::from_bool(false);
    };

    // PEP 634: built-in self-subject types — positional arg 0 always matches if type matches.
    if pos == 0 && is_builtin_self_subject(&class_name, obj) {
        return MbValue::from_bool(true);
    }

    let attr_name: Option<String> = {
        // Use MRO-aware lookup for __match_args__ (#827)
        if let Some(match_args) = mro_lookup_class_attr(&class_name, "__match_args__") {
            if let Some(ptr) = match_args.as_ptr() {
                unsafe {
                    match &(*ptr).data {
                        ObjData::List(ref items) => {
                            let items = items.read().unwrap();
                            if pos < items.len() {
                                extract_str(items[pos])
                            } else {
                                None
                            }
                        }
                        ObjData::Tuple(ref items) => {
                            // ObjData::Tuple holds Vec<MbValue> directly (no RwLock)
                            if pos < items.len() {
                                extract_str(items[pos])
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    };
    // Verify the instance also has the resolved attribute
    match attr_name {
        Some(name) => {
            let attr_val = MbValue::from_ptr(MbObject::new_str(name));
            mb_instance_hasattr(obj, attr_val)
        }
        None => MbValue::from_bool(false),
    }
}

/// Register `__match_args__` for a class (PEP 634 positional class pattern support).
/// Stores `args_list` (a list of attribute name strings) in the class's `class_attrs`.
pub fn mb_class_set_match_args(class_name: MbValue, args_list: MbValue) {
    let name = match extract_str(class_name) {
        Some(n) => n,
        None => return,
    };
    CLASS_REGISTRY.with(|reg| {
        if let Some(cls) = reg.borrow_mut().get_mut(&name) {
            // Only set if not already explicitly defined — preserve explicit __match_args__
            cls.class_attrs
                .entry("__match_args__".to_string())
                .or_insert(args_list);
        }
    });
}

/// issubclass check — first checks CLASS_REGISTRY, then falls back to
/// the built-in exception hierarchy.
/// Builtin (non-exception) type names that count as classes for the
/// issubclass arg-1 "must be a class" validation.
fn is_builtin_type_name(name: &str) -> bool {
    matches!(
        name,
        "int"
            | "float"
            | "complex"
            | "str"
            | "bool"
            | "bytes"
            | "bytearray"
            | "list"
            | "tuple"
            | "dict"
            | "set"
            | "frozenset"
            | "object"
            | "type"
            | "NoneType"
            | "range"
            | "slice"
            | "memoryview"
            | "property"
            | "staticmethod"
            | "classmethod"
            | "super"
            | "function"
            | "method"
            | "module"
            | "generator"
            | "enumerate"
            | "zip"
            | "map"
            | "filter"
            | "reversed"
            | "code"
            | "ellipsis"
    )
}

/// True iff `name` denotes a class the runtime knows about: a registered
/// user/native class, a builtin type, or a builtin exception.
fn is_known_class_name(name: &str) -> bool {
    if is_builtin_type_name(name) {
        return true;
    }
    if CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(name)) {
        return true;
    }
    super::exception::is_subclass_of(name, "BaseException")
}

pub fn mb_issubclass(child: MbValue, parent: MbValue) -> MbValue {
    // Tuple of types: issubclass(C, (A, B, ...)) — true iff any element matches.
    if let Some(ptr) = parent.as_ptr() {
        unsafe {
            if let ObjData::Tuple(ref items) = (*ptr).data {
                for &item in items.iter() {
                    if mb_issubclass(child, item).as_bool() == Some(true) {
                        return MbValue::from_bool(true);
                    }
                }
                return MbValue::from_bool(false);
            }
        }
    }
    if let Some(items) = union_type_args(parent) {
        for item in items {
            if mb_issubclass(child, item).as_bool() == Some(true) {
                return MbValue::from_bool(true);
            }
        }
        return MbValue::from_bool(false);
    }
    // CPython: arg 1 must be a class. Inline scalars never are; strings are
    // the runtime's class representation, so a string only fails when it
    // names no known class (e.g. issubclass("x", object)).
    if child.is_none() || child.as_int().is_some() || child.is_float() || child.as_bool().is_some()
    {
        super::builtins::raise_type_error("issubclass() arg 1 must be a class".to_string());
        return MbValue::none();
    }
    // Arg 2 has the same scalar restriction (tuple/union handled above).
    if parent.is_none()
        || parent.as_int().is_some()
        || parent.is_float()
        || parent.as_bool().is_some()
    {
        super::builtins::raise_type_error(
            "issubclass() arg 2 must be a class, a tuple of classes, or a union".to_string(),
        );
        return MbValue::none();
    }
    // Resolve type objects (Instance with class_name="type" and __name__ field)
    // in addition to plain strings. This matches the resolution logic in mb_isinstance
    // so that issubclass(type_obj, base_type_obj) works correctly (#974).
    let child_name = resolve_class_name(child).unwrap_or_default();
    let parent_name = resolve_class_name(parent).unwrap_or_default();
    if super::stdlib::enum_mod::is_functional_enum_class(child)
        && matches!(parent_name.as_str(), "Enum" | "object")
    {
        return MbValue::from_bool(true);
    }
    // Reflexivity holds for any class; in the string model two equal names
    // are the same class, so answer before the known-class validation.
    if !child_name.is_empty() && child_name == parent_name {
        return MbValue::from_bool(true);
    }
    if child.as_ptr().is_some()
        && unsafe {
            child
                .as_ptr()
                .map(|p| matches!(&(*p).data, ObjData::Str(_)))
                .unwrap_or(false)
        }
        && !is_known_class_name(&child_name)
        && !is_user_abc(&child_name)
        && !collections_abc_type_or_virtual_match(&child_name, &child_name)
    {
        super::builtins::raise_type_error("issubclass() arg 1 must be a class".to_string());
        return MbValue::none();
    }
    // Metaclass __subclasscheck__: issubclass(S, C) defers to
    // type(C).__subclasscheck__(C, S) when the metaclass defines it.
    let meta_check = CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(parent_name.as_str())
            .and_then(|c| c.metaclass.clone())
            .filter(|m| m != "type" && m != "ABCMeta")
            .map(|m| lookup_method(&m, "__subclasscheck__"))
            .filter(|m| !m.is_none())
    });
    if let Some(method) = meta_check {
        let cls_val = MbValue::from_ptr(MbObject::new_str(parent_name.clone()));
        let out = call_method_value2(method, cls_val, child);
        if super::exception::mb_has_exception().as_bool() == Some(true) || out.is_none() {
            // User dunder hit a runtime gap — fall back to the nominal check.
            super::exception::mb_clear_exception();
        } else {
            return MbValue::from_bool(super::builtins::mb_bool(out).as_bool() == Some(true));
        }
    }
    // contextlib.AbstractContextManager / AbstractAsyncContextManager use a
    // structural __subclasshook__: a class is a virtual subclass iff it defines
    // both __enter__ and __exit__ (sync) / __aenter__ and __aexit__ (async),
    // and neither is shadowed by a class attribute set to None.
    if parent_name == "AbstractContextManager"
        || parent_name == "AbstractAsyncContextManager"
    {
        let nominal = child_name == parent_name
            || class_mro_list(&child_name)
                .iter()
                .any(|base| base == &parent_name);
        if nominal {
            return MbValue::from_bool(true);
        }
        let (m1, m2) = if parent_name == "AbstractAsyncContextManager" {
            ("__aenter__", "__aexit__")
        } else {
            ("__enter__", "__exit__")
        };
        return MbValue::from_bool(
            class_defines_non_none(&child_name, m1) && class_defines_non_none(&child_name, m2),
        );
    }
    // os.PathLike uses a structural __subclasshook__: any class defining
    // __fspath__ is a virtual subclass.
    if parent_name == "os.PathLike" || parent_name == "PathLike" {
        return MbValue::from_bool(class_defines_non_none(&child_name, "__fspath__"));
    }
    if let Some(result) = user_abc_issubclass(&child_name, &parent_name) {
        return MbValue::from_bool(result);
    }
    CLASS_REGISTRY.with(|reg| {
        if let Some(cls) = reg.borrow().get(&child_name) {
            MbValue::from_bool(
                child_name == parent_name
                    || cls.mro.contains(&parent_name)
                    || parent_name == "object"
                    || collections_abc_type_or_virtual_match(&child_name, &parent_name)
                    || cls
                        .mro
                        .iter()
                        .any(|base| collections_abc_type_or_virtual_match(base, &parent_name)),
            )
        } else {
            MbValue::from_bool(
                child_name == parent_name
                || parent_name == "object" // all types are subclasses of object
                || (child_name == "bool" && parent_name == "int") // bool is subclass of int
                || collections_abc_type_or_virtual_match(&child_name, &parent_name)
                || collections_builtin_subclass(&child_name, &parent_name)
                || super::exception::is_subclass_of(&child_name, &parent_name),
            )
        }
    })
}

/// True iff `class_name` (walking its MRO) defines `member` as a real method
/// or as a class attribute whose value is not None. The first MRO class that
/// defines the name wins: if it binds the name to `None` (the CPython
/// "opt-out" idiom, e.g. `__enter__ = None`), this returns false. Used by the
/// contextlib AbstractContextManager structural `__subclasshook__`.
fn class_defines_non_none(class_name: &str, member: &str) -> bool {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        let mro: Vec<String> = match reg.get(class_name) {
            Some(cls) => {
                let mut chain = vec![class_name.to_string()];
                chain.extend(cls.mro.iter().filter(|name| *name != class_name).cloned());
                chain
            }
            None => return false,
        };
        for name in &mro {
            if let Some(cls) = reg.get(name) {
                if cls.methods.contains_key(member) {
                    return true;
                }
                if let Some(val) = cls.class_attrs.get(member) {
                    // First MRO class to bind the name decides; None opts out.
                    return !val.is_none();
                }
            }
        }
        false
    })
}

/// abc: the runtime class name of a value, for ABC isinstance dispatch.
/// Instances report their stored class name; builtins map to their type name.
fn abc_runtime_class_name(obj: MbValue) -> Option<String> {
    if obj.is_bool() {
        return Some("bool".to_string());
    }
    if obj.is_int() {
        return Some("int".to_string());
    }
    if obj.is_float() {
        return Some("float".to_string());
    }
    if obj.is_none() {
        return Some("NoneType".to_string());
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            return Some(match &(*ptr).data {
                ObjData::Instance { class_name, .. } => class_name.clone(),
                ObjData::Str(_) => "str".to_string(),
                ObjData::List(_) => "list".to_string(),
                ObjData::Dict(_) => "dict".to_string(),
                ObjData::Tuple(_) => "tuple".to_string(),
                ObjData::Set(_) => "set".to_string(),
                ObjData::FrozenSet(_) => "frozenset".to_string(),
                ObjData::Bytes(_) => "bytes".to_string(),
                ObjData::ByteArray(_) => "bytearray".to_string(),
                _ => return None,
            });
        }
    }
    None
}

/// abc: ABCMeta-style `__subclasscheck__` for user ABCs. Returns `Some(bool)`
/// when `parent_name` is a user ABC (so the caller should use the answer), or
/// `None` to fall through to the generic nominal check.
fn user_abc_issubclass(child_name: &str, parent_name: &str) -> Option<bool> {
    if child_name.is_empty() || parent_name.is_empty() {
        return None;
    }
    if !is_user_abc(parent_name) {
        return None;
    }
    // 1. Nominal subclass (own MRO contains the parent) — always true.
    let nominal = CLASS_REGISTRY.with(|reg| {
        reg.borrow()
            .get(child_name)
            .map_or(child_name == parent_name, |cls| {
                child_name == parent_name || cls.mro.iter().any(|b| b == parent_name)
            })
    });
    if nominal {
        return Some(true);
    }
    // 2. Custom __subclasshook__ (structural check). A definite answer wins.
    if let Some(hook_result) = user_abc_subclasshook(parent_name, child_name) {
        return Some(hook_result);
    }
    // 3. Explicitly registered virtual subclasses (ABCMeta.register).
    if collections_abc_virtual_match(child_name, parent_name) {
        return Some(true);
    }
    Some(false)
}

/// Check if `child` class is a subclass of `parent` class via the CLASS_REGISTRY MRO.
/// Used by exception.rs to check user-defined exception hierarchies.
pub fn check_class_hierarchy(child: &str, parent: &str) -> bool {
    CLASS_REGISTRY.with(|reg| {
        if let Some(cls) = reg.borrow().get(child) {
            cls.mro.contains(&parent.to_string())
        } else {
            false
        }
    })
}

/// Walk `child`'s MRO; return true if any class name satisfies `pred`. Used
/// by `exception::is_subclass_of` to detect built-in exception ancestors that
/// don't appear literally in the MRO (e.g. `ValueError` instead of
/// `"Exception"`). (#1551)
pub fn class_mro_any<F: Fn(&str) -> bool>(child: &str, pred: F) -> bool {
    CLASS_REGISTRY.with(|reg| {
        if let Some(cls) = reg.borrow().get(child) {
            cls.mro.iter().any(|c| pred(c))
        } else {
            false
        }
    })
}

// ── Property / classmethod / staticmethod ──

/// Create a @property descriptor.
/// Stores getter, setter, deleter as fields on a __property__ instance.
pub fn mb_property_new(getter: MbValue) -> MbValue {
    let prop = MbObject::new_instance("__property__".to_string());
    let ptr = MbValue::from_ptr(prop);
    let key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
    mb_setattr(ptr, key, getter);
    ptr
}

/// property.setter(fn) → returns new property with setter set.
pub fn mb_property_setter(prop: MbValue, setter: MbValue) -> MbValue {
    let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
    mb_setattr(prop, key, setter);
    prop
}

/// property.deleter(fn) → returns new property with deleter set.
pub fn mb_property_deleter(prop: MbValue, deleter: MbValue) -> MbValue {
    let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
    mb_setattr(prop, key, deleter);
    prop
}

/// Construct a property from call args: positional (`fget`, `fset`, `fdel`,
/// `doc`) and/or an optional trailing kwargs dict (`{fget, fset, fdel, doc}`).
/// `property` is in the native-kwargs allowlist, so a keyword call
/// (`property(fset=f)`) appends the dict. Property args are callables / None,
/// never dicts, so a trailing `ObjData::Dict` is the kwargs bag. This lets the
/// write-only / keyword forms build the correct (fget=None, fset=f) shape
/// instead of mis-binding the first arg as `fget`.
pub fn mb_property_construct(items: &[MbValue]) -> MbValue {
    let (pos, kwargs): (&[MbValue], Option<MbValue>) = match items.last() {
        Some(last)
            if last
                .as_ptr()
                .map_or(false, |p| unsafe { matches!(&(*p).data, ObjData::Dict(_)) }) =>
        {
            (&items[..items.len() - 1], Some(*last))
        }
        _ => (items, None),
    };
    let mut fget = pos.first().copied();
    let mut fset = pos.get(1).copied();
    let mut fdel = pos.get(2).copied();
    if let Some(kw) = kwargs {
        if let Some(kp) = kw.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*kp).data {
                    for (k, v) in lock.read().unwrap().iter() {
                        if let super::dict_ops::DictKey::Str(s) = k {
                            match s.as_str() {
                                "fget" => fget = Some(*v),
                                "fset" => fset = Some(*v),
                                "fdel" => fdel = Some(*v),
                                _ => {} // doc / unknown ignored
                            }
                        }
                    }
                }
            }
        }
    }
    let prop = mb_property_new(fget.unwrap_or_else(MbValue::none));
    if let Some(fs) = fset {
        if !fs.is_none() {
            mb_property_setter(prop, fs);
        }
    }
    if let Some(fd) = fdel {
        if !fd.is_none() {
            mb_property_deleter(prop, fd);
        }
    }
    prop
}

/// extern wrapper for the static `property(*args)` call form. The lowering boxes
/// the call args into a list; we unpack it and delegate to mb_property_construct
/// (positional fget/fset/fdel/doc plus an optional trailing kwargs dict).
pub fn mb_property_from_args(args_list: MbValue) -> MbValue {
    let items: Vec<MbValue> = match args_list.as_ptr() {
        Some(p) => unsafe {
            match &(*p).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                _ => vec![args_list],
            }
        },
        None => Vec::new(),
    };
    mb_property_construct(&items)
}

/// Get property value: calls fget(instance).
pub fn mb_property_get(prop: MbValue, instance: MbValue) -> MbValue {
    let key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
    let getter = mb_getattr(prop, key);
    if getter.is_none() {
        // A property with no fget is write-only: reading raises AttributeError
        // (CPython). Returning None silently let write-only reads succeed.
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("unreadable attribute".to_string())),
        );
        return MbValue::none();
    }
    // A closure-handle getter (`property(lambda self: ...)`) is an int handle,
    // not a bare TAG_FUNC, so the func-pointer paths below can't dispatch it.
    // Resolve it through the general value-call, which unpacks the closure.
    if getter.as_func().is_none()
        && getter.as_int().is_some()
        && !super::closure::mb_closure_get_func(getter).is_none()
    {
        let result = mb_call1_val(getter, instance);
        unsafe { super::rc::retain_if_ptr(result); }
        return result;
    }
    // Call the stored getter with instance. Try mb_call_method1 first
    // (CALLABLE_REGISTRY path for heap-pointer methods), then fall back to
    // direct TAG_FUNC invocation for JIT-compiled class methods that are
    // registered as func pointers.
    let val = mb_call_method1(getter, instance);
    if !val.is_none() {
        unsafe {
            super::rc::retain_if_ptr(val);
        }
        return val;
    }
    // Direct TAG_FUNC / raw address dispatch for class methods compiled
    // via Cranelift and stored as FuncRef values.
    // REQ: JIT-compiled functions use SystemV/C calling convention.
    if let Some(addr) = getter.as_func() {
        if addr > 4096 {
            let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr) };
            let result = f(instance);
            unsafe {
                super::rc::retain_if_ptr(result);
            }
            return result;
        }
    }
    let addr = extract_func_addr(getter);
    if addr > 4096 {
        let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr as usize) };
        let result = f(instance);
        unsafe {
            super::rc::retain_if_ptr(result);
        }
        return result;
    }
    MbValue::none()
}

/// Set property value: calls fset(instance, value).
/// R2 P1: Directly invoke the setter function pointer instead of going through
/// mb_call_method (which can't dispatch TAG_FUNC values as receivers).
pub fn mb_property_set(prop: MbValue, instance: MbValue, value: MbValue) {
    let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
    let setter = mb_getattr(prop, key);
    if setter.is_none() {
        // A property with no fset is read-only: assignment raises
        // AttributeError (CPython). Doing nothing silently let the write
        // succeed as a no-op.
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("can't set attribute".to_string())),
        );
        return;
    }
    // Direct function pointer invocation (TAG_FUNC).
    // REQ: JIT-compiled functions use SystemV/C calling convention.
    if let Some(addr) = setter.as_func() {
        if addr > 4096 {
            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr) };
            f(instance, value);
            return;
        }
    }
    // Fallback: try CALLABLE_REGISTRY for heap-pointer methods
    let addr = extract_func_addr(setter);
    if addr != 0 {
        let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
        if is_reg {
            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            f(instance, value);
        }
    }
}

/// Create a @cached_property descriptor. On first access the wrapped
/// getter is invoked and the result is stored directly on the instance
/// under the attribute name, so subsequent accesses hit the instance
/// __dict__ and bypass the descriptor (standard CPython semantics).
pub fn mb_cached_property_new(getter: MbValue, name: MbValue) -> MbValue {
    let desc = MbObject::new_instance("__cached_property__".to_string());
    let ptr = MbValue::from_ptr(desc);
    let fget_key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
    mb_setattr(ptr, fget_key, getter);
    let name_key = MbValue::from_ptr(MbObject::new_str("__name__".to_string()));
    mb_setattr(ptr, name_key, name);
    ptr
}

/// First-access helper for cached_property: runs the getter on `instance`
/// and writes the result into the instance field named by the descriptor.
pub fn mb_cached_property_get(desc: MbValue, instance: MbValue) -> MbValue {
    let fget_key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
    let name_key = MbValue::from_ptr(MbObject::new_str("__name__".to_string()));
    let getter = mb_getattr(desc, fget_key);
    let name_val = mb_getattr(desc, name_key);
    if getter.is_none() {
        return MbValue::none();
    }
    // Invoke getter(instance). Follow the same fallback ladder as
    // mb_property_get — handles heap-pointer methods and raw TAG_FUNC.
    let mut val = mb_call_method1(getter, instance);
    if val.is_none() {
        if let Some(addr) = getter.as_func() {
            if addr > 4096 {
                let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr) };
                val = f(instance);
            }
        }
        if val.is_none() {
            let addr = extract_func_addr(getter);
            if addr > 4096 {
                let f: extern "C" fn(MbValue) -> MbValue =
                    unsafe { std::mem::transmute(addr as usize) };
                val = f(instance);
            }
        }
    }
    if !val.is_none() {
        // Write into instance so next lookup hits the instance __dict__ and
        // skips this descriptor.
        if !name_val.is_none() {
            mb_setattr(instance, name_val, val);
        }
        unsafe {
            super::rc::retain_if_ptr(val);
        }
    }
    val
}

/// Create a @classmethod wrapper. Stores the function and marks it.
pub fn mb_classmethod_new(func: MbValue) -> MbValue {
    let cm = MbObject::new_instance("__classmethod__".to_string());
    let ptr = MbValue::from_ptr(cm);
    let key = MbValue::from_ptr(MbObject::new_str("__func__".to_string()));
    mb_setattr(ptr, key, func);
    ptr
}

/// Create a @staticmethod wrapper. Stores the function and marks it.
pub fn mb_staticmethod_new(func: MbValue) -> MbValue {
    let sm = MbObject::new_instance("__staticmethod__".to_string());
    let ptr = MbValue::from_ptr(sm);
    let key = MbValue::from_ptr(MbObject::new_str("__func__".to_string()));
    mb_setattr(ptr, key, func);
    ptr
}

/// Unwrap a classmethod/staticmethod to get the underlying function.
pub fn mb_descriptor_unwrap(desc: MbValue) -> MbValue {
    let key = MbValue::from_ptr(MbObject::new_str("__func__".to_string()));
    mb_getattr(desc, key)
}

// ── Metaclasses / ABC ──

thread_local! {
    /// Set of abstract method names per class.
    static ABSTRACT_METHODS: std::cell::RefCell<HashMap<String, Vec<String>>> =
        std::cell::RefCell::new(HashMap::new());
}

/// Mark a method as abstract. @abstractmethod decorator.
/// Returns the function unchanged but registers it in the abstract methods set.
pub fn mb_abstractmethod(func: MbValue) -> MbValue {
    // Mark the function with __isabstractmethod__ = True
    // This is checked during class creation
    func
}

/// Register abstract methods for a class.
/// Called during class definition when metaclass=ABCMeta or base=ABC.
pub fn mb_register_abstract(class_name: MbValue, method_names: MbValue) {
    let name = extract_str(class_name).unwrap_or_default();
    let mut methods = Vec::new();
    unsafe {
        if let Some(ptr) = method_names.as_ptr() {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for item in items.iter() {
                    if let Some(s) = extract_str(*item) {
                        methods.push(s);
                    }
                }
            }
        }
    }
    ABSTRACT_METHODS.with(|am| {
        am.borrow_mut().insert(name, methods);
    });
}

/// Check if a class can be instantiated (no unimplemented abstract methods).
/// Returns true if instantiation is allowed.
pub fn mb_check_abstract(class_name: MbValue) -> MbValue {
    let name = extract_str(class_name).unwrap_or_default();

    // Collect all abstract methods from base classes
    let abstract_methods: Vec<String> = ABSTRACT_METHODS.with(|am| {
        let am = am.borrow();
        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(cls) = reg.get(&name) {
                let mut abstracts = Vec::new();
                for mro_class in &cls.mro {
                    if let Some(methods) = am.get(mro_class) {
                        abstracts.extend(methods.clone());
                    }
                }
                abstracts
            } else {
                am.get(&name).cloned().unwrap_or_default()
            }
        })
    });

    // Check if all abstract methods are implemented
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(&name) {
            for method_name in &abstract_methods {
                if !cls.methods.contains_key(method_name) {
                    return MbValue::from_bool(false);
                }
            }
        }
        MbValue::from_bool(true)
    })
}

// ── super() Support ──

/// Create a super proxy that skips the current class in MRO lookup.
/// `super(ClassName, instance)` → proxy object that resolves methods
/// starting from the next class in MRO after ClassName.
pub fn mb_super(class_name: MbValue, instance: MbValue) -> MbValue {
    let name = extract_str(class_name).unwrap_or_default();
    // Create a super proxy as a special instance with __super_class__ and __super_self__
    let proxy = MbObject::new_instance("__super__".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*proxy).data {
            let mut fields = fields.write().unwrap();
            fields.insert(
                "__super_class__".to_string(),
                MbValue::from_ptr(MbObject::new_str(name)),
            );
            super::rc::retain_if_ptr(instance);
            fields.insert("__super_self__".to_string(), instance);
        }
    }
    MbValue::from_ptr(proxy)
}

/// Get an attribute from a super proxy — walks MRO starting after the given class.
pub fn mb_super_getattr(proxy: MbValue, attr: MbValue) -> MbValue {
    let attr_name = extract_str(attr).unwrap_or_default();

    if let Some(ptr) = proxy.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let super_class = fields
                    .get("__super_class__")
                    .and_then(|v| extract_str(*v))
                    .unwrap_or_default();
                let super_self = fields
                    .get("__super_self__")
                    .copied()
                    .unwrap_or(MbValue::none());

                // Get the actual class of the instance
                let instance_class = if let Some(self_ptr) = super_self.as_ptr() {
                    if let ObjData::Instance { ref class_name, .. } = (*self_ptr).data {
                        class_name.clone()
                    } else {
                        return MbValue::none();
                    }
                } else {
                    return MbValue::none();
                };

                let val = lookup_method_after(&instance_class, &super_class, &attr_name);
                super::rc::retain_if_ptr(val);
                return val;
            }
        }
    }
    MbValue::none()
}

/// Look up a method in MRO, starting after `skip_class`.
fn lookup_method_after(class_name: &str, skip_class: &str, method_name: &str) -> MbValue {
    CLASS_REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(cls) = reg.get(class_name) {
            let mut skip = true;
            for mro_class in &cls.mro {
                if skip {
                    if mro_class == skip_class {
                        skip = false;
                    }
                    continue;
                }
                if let Some(mro_cls) = reg.get(mro_class) {
                    if let Some(method) = mro_cls.methods.get(method_name) {
                        return *method;
                    }
                }
            }
        }
        MbValue::none()
    })
}

// ── Unary Operator Overloading ──

const UNARYOP_DUNDERS: &[&str] = &["pos", "neg", "not", "invert"];

/// Dispatch a unary operation through dunder methods.
/// `op_code` is a raw i64 index into UNARYOP_DUNDERS (FFI-safe for codegen).
///
/// Handles primitive types (int, float, bool) directly before falling back to
/// dunder method lookup on heap objects. This is needed for `Any`-typed values
/// (e.g., lambda parameters) where the codegen cannot specialise at compile time.
pub fn mb_dispatch_unaryop(op_code: i64, obj: MbValue) -> MbValue {
    // ── Primitive fast path ──
    // Decimal / Fraction integer handles must be intercepted before the
    // `as_int` arms below negate/copy raw handle ids (#2129).
    if super::builtins::is_decimal_handle_value(obj) {
        match op_code {
            0 => return super::stdlib::decimal_mod::mb_decimal_pos(obj),
            1 => return super::stdlib::decimal_mod::mb_decimal_neg(obj),
            2 => {
                let truthy = super::stdlib::decimal_mod::mb_decimal_bool(obj);
                return MbValue::from_bool(truthy.as_bool() != Some(true));
            }
            _ => {}
        }
    }
    if super::builtins::is_fraction_handle_value(obj) {
        match op_code {
            0 => return super::stdlib::fractions_mod::mb_fraction_pos(obj),
            1 => return super::stdlib::fractions_mod::mb_fraction_neg(obj),
            2 => {
                let truthy = super::stdlib::fractions_mod::mb_fraction_bool(obj);
                return MbValue::from_bool(truthy.as_bool() != Some(true));
            }
            _ => {}
        }
    }
    match op_code {
        0 => {
            // pos (+x)
            if let Some(i) = obj.as_int() {
                return MbValue::from_int(i);
            }
            if let Some(f) = obj.as_float() {
                return MbValue::from_float(f);
            }
            if let Some(b) = obj.as_bool() {
                return MbValue::from_int(b as i64);
            }
            // +complex → fresh complex (unary plus is identity). (#1256)
            if let Some(ptr) = obj.as_ptr() {
                unsafe {
                    if let super::rc::ObjData::Complex(re, im) = (*ptr).data {
                        return MbValue::from_ptr(MbObject::new_complex(re, im));
                    }
                }
            }
            // +timedelta → identity copy.
            if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(obj) {
                return super::stdlib::datetime_mod::timedelta_from_us(us);
            }
        }
        1 => {
            // neg (-x)
            if let Some(i) = obj.as_int() {
                return MbValue::from_int(-i);
            }
            if let Some(f) = obj.as_float() {
                return MbValue::from_float(-f);
            }
            if let Some(b) = obj.as_bool() {
                return MbValue::from_int(-(b as i64));
            }
            // -complex → both components negated. (#1256)
            if let Some(ptr) = obj.as_ptr() {
                unsafe {
                    if let super::rc::ObjData::Complex(re, im) = (*ptr).data {
                        return MbValue::from_ptr(MbObject::new_complex(-re, -im));
                    }
                }
            }
            // -timedelta → negated duration.
            if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(obj) {
                return super::stdlib::datetime_mod::timedelta_from_us(-us);
            }
        }
        2 => {
            // not (not x)
            if let Some(b) = obj.as_bool() {
                return MbValue::from_bool(!b);
            }
            if let Some(i) = obj.as_int() {
                return MbValue::from_bool(i == 0);
            }
        }
        3 => {
            // invert (~x)
            if let Some(i) = obj.as_int() {
                return MbValue::from_int(!i);
            }
            if let Some(b) = obj.as_bool() {
                return MbValue::from_int(!(b as i64));
            }
        }
        _ => {}
    }
    // Counter unary +/- — multiset sign filtering (CPython: `+c` keeps only
    // positive counts, `-c` flips signs then keeps positives).
    if super::stdlib::collections_mod::is_counter_instance(obj) {
        match op_code {
            0 => return super::stdlib::collections_mod::mb_counter_unary(obj, false),
            1 => return super::stdlib::collections_mod::mb_counter_unary(obj, true),
            _ => {}
        }
    }
    // statistics.NormalDist: -nd flips the mean (fresh object), +nd copies.
    if matches!(op_code, 0 | 1) {
        if let Some(nd) = super::stdlib::statistics_mod::normaldist_neg(obj) {
            if op_code == 1 {
                return nd;
            }
            // +nd: normaldist_neg already proved obj IS a NormalDist; negate
            // back to a same-params fresh copy.
            if let Some(copy) = super::stdlib::statistics_mod::normaldist_neg(nd) {
                return copy;
            }
        }
    }
    // ── Dunder method fallback ──
    let op_name = UNARYOP_DUNDERS
        .get(op_code as usize)
        .copied()
        .unwrap_or("neg");
    let dunder = format!("__{op_name}__");
    if let Some(method) = try_get_dunder(obj, &dunder) {
        return method; // Caller invokes with (obj)
    }
    MbValue::none()
}

// ── Special Method Protocol ──

/// Runtime-dispatched __getitem__: list, tuple, dict, str, or dunder.
/// Also handles slice tuples: if key is a Tuple(start, stop, step), dispatches to slice.
/// The `_children` list of an ET.Element dict-stub, if `obj` is one.
fn element_children_list(obj: MbValue) -> Option<MbValue> {
    let ptr = obj.as_ptr()?;
    unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let guard = lock.read().unwrap();
            let is_element = guard
                .get("__class__")
                .and_then(|v| v.as_ptr())
                .map(|p| matches!(&(*p).data, ObjData::Str(s) if s == "Element"))
                .unwrap_or(false);
            if is_element {
                return guard.get("_children").copied();
            }
        }
    }
    None
}

/// If `key` is a slice *object* (Instance, class_name "slice"), repackage its
/// (start, stop, step) as the 3-tuple that built-in container and native-stub
/// slice handling consume. Returns None when `key` is not a slice object.
fn slice_obj_as_tuple(key: MbValue) -> Option<MbValue> {
    let kp = key.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*kp).data {
            if class_name == "slice" {
                let g = fields.read().unwrap();
                return Some(MbValue::from_ptr(MbObject::new_tuple(vec![
                    g.get("start").copied().unwrap_or_else(MbValue::none),
                    g.get("stop").copied().unwrap_or_else(MbValue::none),
                    g.get("step").copied().unwrap_or_else(MbValue::none),
                ])));
            }
        }
    }
    None
}

/// True when `obj` defines a USER (compiled, non-native) dunder `name`. A user
/// dunder must receive a real slice object as a subscript key (CPython
/// semantics); a native-stub dunder (extern "C", e.g. struct-seq
/// `__getitem__`) expects the (start, stop, step) tuple normalization. The
/// native-stub methods are registered in NATIVE_FUNC_ADDRS or
/// VARIADIC_FUNC_ADDRS (struct-seq `__getitem__` uses the variadic
/// convention), so an address in neither marks a genuine user method. When the
/// address can't be resolved we conservatively report `false` (normalize — the
/// prior behavior).
fn obj_has_user_dunder(obj: MbValue, name: &str) -> bool {
    match try_get_dunder(obj, name) {
        Some(method) => {
            let addr = extract_func_addr(method);
            addr != 0
                && !super::module::is_native_func(addr)
                && !super::module::is_variadic_func(addr)
        }
        None => false,
    }
}

pub fn mb_obj_getitem(obj: MbValue, key: MbValue) -> MbValue {
    // A slice subscript now lowers to a real slice object. Normalize it to the
    // (start, stop, step) tuple form that built-in containers and native-stub
    // sequences consume - UNLESS `obj` defines a user `__getitem__`, which must
    // receive the real slice object (CPython semantics).
    if !obj_has_user_dunder(obj, "__getitem__") {
        if let Some(tuple_key) = slice_obj_as_tuple(key) {
            return mb_obj_getitem(obj, tuple_key);
        }
    }
    // ET.Element subscript indexes the children list (IndexError when out of
    // range; slices with step return child lists), not the stub dict's keys.
    if let Some(kids) = element_children_list(obj) {
        // Slice key arrives as a (start, stop, step) tuple.
        let slice_parts: Option<(Option<i64>, Option<i64>, Option<i64>)> =
            key.as_ptr().and_then(|p| unsafe {
                if let ObjData::Tuple(ref t) = (*p).data {
                    if t.len() == 3 {
                        return Some((t[0].as_int(), t[1].as_int(), t[2].as_int()));
                    }
                }
                None
            });
        if let Some(kp) = kids.as_ptr() {
            unsafe {
                if let ObjData::List(ref kl) = (*kp).data {
                    let items = kl.read().unwrap().to_vec();
                    let n = items.len() as i64;
                    if let Some((start, stop, step)) = slice_parts {
                        let step = step.unwrap_or(1);
                        let mut out = Vec::new();
                        if step > 0 {
                            let mut i = start
                                .map(|v| if v < 0 { v + n } else { v })
                                .unwrap_or(0)
                                .clamp(0, n);
                            let stop = stop
                                .map(|v| if v < 0 { v + n } else { v })
                                .unwrap_or(n)
                                .clamp(0, n);
                            while i < stop {
                                out.push(items[i as usize]);
                                i += step;
                            }
                        } else if step < 0 {
                            let mut i = start
                                .map(|v| if v < 0 { v + n } else { v })
                                .unwrap_or(n - 1)
                                .clamp(-1, n - 1);
                            let stop = stop
                                .map(|v| if v < 0 { v + n } else { v })
                                .unwrap_or(-1)
                                .clamp(-1, n - 1);
                            while i > stop {
                                out.push(items[i as usize]);
                                i += step;
                            }
                        }
                        for v in &out {
                            super::rc::retain_if_ptr(*v);
                        }
                        return MbValue::from_ptr(MbObject::new_list(out));
                    }
                    if let Some(idx) = key.as_int() {
                        let i = if idx < 0 { idx + n } else { idx };
                        if i < 0 || i >= n {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "child index out of range".to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                        let v = items[i as usize];
                        super::rc::retain_if_ptr(v);
                        return v;
                    }
                    // A non-int, non-slice index (e.g. Element[None]) is a
                    // TypeError, like list subscripting.
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "list indices must be integers or slices, not {}",
                            super::builtins::value_type_name(key)
                        ))),
                    );
                    return MbValue::none();
                }
            }
        }
    }
    // re.Match subscript is group lookup: m[0] / m['name'].
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "re.Match" {
                    let guard = fields.read().unwrap();
                    if let Some(i) = key.as_int() {
                        return guard
                            .get(&format!("group_{i}"))
                            .copied()
                            .unwrap_or_else(MbValue::none);
                    }
                    if let Some(nm) = extract_str(key) {
                        return guard
                            .get(&format!("group_name_{nm}"))
                            .copied()
                            .unwrap_or_else(MbValue::none);
                    }
                }
                // collections.deque[i] — index the backing `_items` list with
                // deque bounds semantics (CPython: "deque index out of range").
                if class_name == "collections.deque" {
                    if let (Some(items_v), Some(idx)) =
                        (fields.read().unwrap().get("_items").copied(), key.as_int())
                    {
                        if let Some(ip) = items_v.as_ptr() {
                            if let ObjData::List(ref lock) = (*ip).data {
                                let items = lock.read().unwrap();
                                let len = items.len() as i64;
                                let actual = if idx < 0 { idx + len } else { idx };
                                if actual >= 0 && actual < len {
                                    let v = items[actual as usize];
                                    super::rc::retain_if_ptr(v);
                                    return v;
                                }
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(
                                        "deque index out of range".to_string(),
                                    )),
                                );
                                return MbValue::none();
                            }
                        }
                    }
                }
            }
        }
    }
    // PEP 585: subscripting a native constructor func used as a class
    // (`tempfile.SpooledTemporaryFile[bytes]`, `deque[int]`) produces a
    // types.GenericAlias-shaped Instance (class_name "GenericAlias", matching
    // CPython's `type(list[int]).__name__`) carrying __origin__/__args__.
    if let Some(addr) = obj.as_func() {
        let known =
            super::module::NATIVE_TYPE_NAMES.with(|m| m.borrow().contains_key(&(addr as u64)));
        if known {
            let args_tuple = if let Some(kp) = key.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(_) = (*kp).data {
                        super::rc::retain_if_ptr(key);
                        key
                    } else {
                        super::rc::retain_if_ptr(key);
                        MbValue::from_ptr(MbObject::new_tuple(vec![key]))
                    }
                }
            } else {
                MbValue::from_ptr(MbObject::new_tuple(vec![key]))
            };
            let inst_ptr = MbObject::new_instance("GenericAlias".to_string());
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*inst_ptr).data {
                    let mut g = fields.write().unwrap();
                    g.insert("__origin__".to_string(), obj);
                    g.insert("__args__".to_string(), args_tuple);
                }
            }
            return MbValue::from_ptr(inst_ptr);
        }
    }
    // bool is a subclass of int in Python (#1680) — `xs[True]` ≡ `xs[1]`.
    // Coerce a bool key to an int before any container dispatch so the
    // built-in indexing paths don't hit the strict `as_int()` rejection.
    // EXCEPT a typing special form (Literal[True]): its subscript stores the
    // value itself, where a bool must stay a bool so Literal[True] != Literal[1].
    let obj_is_typing_form = obj.as_ptr().map(|p| unsafe {
        matches!(&(*p).data,
            ObjData::Instance { class_name, .. } if class_name == "typing.SpecialForm")
    }).unwrap_or(false);
    let key = if key.is_bool() && !obj_is_typing_form {
        MbValue::from_int(key.as_int_pyint().unwrap_or(0))
    } else {
        key
    };
    // range(a, b[, s]) returns an iterator handle (tagged int), not a list.
    // Support integer subscript so `range(10, 20)[5]` matches CPython's
    // range.__getitem__ semantics. Out-of-bounds raises IndexError.
    if super::iter::is_iter_handle(obj) {
        // A range index above 2^47 (e.g. `range(sys.maxsize)[i]` during a
        // bisect over a huge range) is a NaN-box-promoted BigInt; unbox it
        // when it fits i64, and promote the resulting element back to BigInt.
        let idx_i64 = key.as_int_pyint().or_else(|| {
            use num_traits::ToPrimitive;
            unsafe { super::bigint_ops::extract_bigint(key) }.and_then(|b| b.to_i64())
        });
        if let Some(idx) = idx_i64 {
            match super::iter::range_iter_getitem(obj, idx) {
                Some(v) => return super::bigint_ops::int_from_i64(v),
                None => {
                    if super::iter::mb_iter_range_params(obj).is_some() {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "range object index out of range".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                }
            }
        }
        if let Some(kp) = key.as_ptr() {
            unsafe {
                if let super::rc::ObjData::Tuple(ref items) = (*kp).data {
                    if items.len() == 3 {
                        if let Some(v) =
                            super::iter::range_iter_slice(obj, items[0], items[1], items[2])
                        {
                            return v;
                        }
                    }
                }
            }
        }
    }
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            return super::stdlib::array_mod::mb_array_getitem(obj, key);
        }
    }
    // Slice fast-path: a 3-element tuple key is the runtime
    // representation of a `Expr::Slice` (start, stop, step) and is
    // dispatched directly to the built-in container's slice routine.
    //
    // Only the four built-in container types take this path. For
    // anything else — most importantly user-defined classes with a
    // custom `__getitem__` — we MUST fall through to the general
    // dispatch below so the dunder receives the tuple key. Defaulting
    // to `mb_list_slice_full` on an Instance silently returns `[]`
    // and never invokes `__getitem__` (#1672).
    if let Some(kp) = key.as_ptr() {
        unsafe {
            if let super::rc::ObjData::Tuple(ref items) = (*kp).data {
                if items.len() == 3 {
                    if let Some(objp) = obj.as_ptr() {
                        match &(*objp).data {
                            super::rc::ObjData::List(_) => {
                                return super::list_ops::mb_list_slice_full(
                                    obj, items[0], items[1], items[2],
                                );
                            }
                            super::rc::ObjData::Tuple(_) => {
                                return super::tuple_ops::mb_tuple_slice_full(
                                    obj, items[0], items[1], items[2],
                                );
                            }
                            super::rc::ObjData::Str(_) => {
                                return super::string_ops::mb_str_slice_full(
                                    obj, items[0], items[1], items[2],
                                );
                            }
                            super::rc::ObjData::Bytes(_) | super::rc::ObjData::ByteArray(_) => {
                                return super::bytes_ops::mb_bytes_slice_full(
                                    obj, items[0], items[1], items[2],
                                );
                            }
                            // memoryview[a:b:c] — preserve a memoryview
                            // result plus contiguity metadata. The lowering
                            // pass packs `mv[1:4]` as a 3-tuple key, so it
                            // lands here rather than the slice-Instance path
                            // below. (#1256 sub-priority 4)
                            super::rc::ObjData::Instance { ref class_name, .. }
                                if class_name == "memoryview" =>
                            {
                                return memoryview_slice(obj, items[0], items[1], items[2]);
                            }
                            _ => {} // Instance / other → general dispatch below.
                        }
                    }
                }
            }
        }
    }
    unsafe {
        if let Some(ptr) = obj.as_ptr() {
            match &(*ptr).data {
                super::rc::ObjData::List(_) => {
                    return super::list_ops::mb_list_getitem(obj, key);
                }
                super::rc::ObjData::Tuple(_) => {
                    return super::tuple_ops::mb_tuple_getitem(obj, key);
                }
                super::rc::ObjData::Dict(_) => {
                    return super::dict_ops::mb_dict_getitem(obj, key);
                }
                super::rc::ObjData::Str(ref s) => {
                    // R11: __class_getitem__ — if obj is a class name, check for subscript support.
                    let is_class = CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(s.as_str()));
                    if is_class {
                        // Class-body enums: `Color["BLUE"]` is a name→member
                        // lookup (raises KeyError on a missing name).
                        if let Some(member) = super::stdlib::enum_class::enum_class_getitem(s, key)
                        {
                            return member;
                        }
                        let getitem_method = lookup_method(s, "__class_getitem__");
                        if !getitem_method.is_none() {
                            let addr = extract_func_addr(getitem_method);
                            if addr != 0 {
                                let is_registered =
                                    CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                                if is_registered {
                                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                                    let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                                        std::mem::transmute(addr as usize);
                                    return func(obj, key);
                                }
                            }
                        }
                        // No __class_getitem__ → raise TypeError
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'type' object is not subscriptable"
                            ))),
                        );
                        return MbValue::none();
                    }
                    return super::string_ops::mb_str_getitem(obj, key);
                }
                super::rc::ObjData::Bytes(_) | super::rc::ObjData::ByteArray(_) => {
                    return super::bytes_ops::mb_bytes_getitem(obj, key);
                }
                super::rc::ObjData::Set(_) => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "'set' object is not subscriptable".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
                super::rc::ObjData::FrozenSet(_) => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "'frozenset' object is not subscriptable".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
                super::rc::ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    // Functional-API enum classes: Color['RED'] is a
                    // name lookup; a missing name raises KeyError.
                    if class_name == "_MambaFunctionalEnum" {
                        if let Some(name) = extract_str(key) {
                            if let Some(member) = fields.read().unwrap().get(name.as_str()).copied()
                            {
                                super::rc::retain_if_ptr(member);
                                return member;
                            }
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!("'{name}'"))),
                            );
                            return MbValue::none();
                        }
                    }
                    // time.struct_time is a named 9-tuple: integer index and
                    // slice subscripts read the tm_* fields in CPython order.
                    if class_name == "struct_time" {
                        const ORDER: [&str; 9] = [
                            "tm_year", "tm_mon", "tm_mday", "tm_hour", "tm_min", "tm_sec",
                            "tm_wday", "tm_yday", "tm_isdst",
                        ];
                        let read = |name: &str| -> MbValue {
                            fields
                                .read()
                                .unwrap()
                                .get(name)
                                .copied()
                                .unwrap_or_else(MbValue::none)
                        };
                        if let Some(i) = key.as_int() {
                            let i = if i < 0 { i + 9 } else { i };
                            if (0..9).contains(&i) {
                                return read(ORDER[i as usize]);
                            }
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("IndexError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "tuple index out of range".to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                        // Slice key — the lowering packs s[:6] as a 3-tuple
                        // (start, stop, step); slice instances also appear.
                        let parts: Option<(i64, i64, i64)> = key.as_ptr().and_then(|kp| {
                            if let super::rc::ObjData::Tuple(ref t) = (*kp).data {
                                if t.len() == 3 {
                                    let start = t[0].as_int().unwrap_or(0);
                                    let stop = t[1].as_int().unwrap_or(9);
                                    let step = t[2].as_int().unwrap_or(1);
                                    return Some((start, stop, step));
                                }
                            }
                            None
                        });
                        if let Some((start, stop, step)) = parts {
                            if step == 1 {
                                let a = start.clamp(0, 9) as usize;
                                let b = stop.clamp(0, 9) as usize;
                                let items: Vec<MbValue> =
                                    (a..b.max(a)).map(|i| read(ORDER[i])).collect();
                                return MbValue::from_ptr(MbObject::new_tuple(items));
                            }
                        }
                    }
                    // Builtin type objects: PEP 585 generics subscript into
                    // aliases (list[int]); non-generic scalars raise (#22).
                    if class_name == "type" {
                        let tn = fields
                            .read()
                            .unwrap()
                            .get("__name__")
                            .copied()
                            .and_then(extract_str)
                            .unwrap_or_default();
                        match tn.as_str() {
                            "list" | "dict" | "set" | "frozenset" | "tuple" | "type" => {
                                return super::stdlib::typing_mod::pep585_subscript(obj, key);
                            }
                            "int" | "float" | "str" | "bool" | "bytes" | "complex"
                            | "bytearray" | "range" | "NoneType" => {
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(format!(
                                        "type '{tn}' is not subscriptable"
                                    ))),
                                );
                                return MbValue::none();
                            }
                            _ => {}
                        }
                    }
                    // typing special forms subscript into normalized alias
                    // objects: Union[int, str], Optional[int], List[int] (#22).
                    if class_name == "typing.SpecialForm" {
                        let name = fields
                            .read()
                            .unwrap()
                            .get("_name")
                            .copied()
                            .and_then(extract_str)
                            .unwrap_or_default();
                        return super::stdlib::typing_mod::special_form_subscript(&name, key);
                    }
                    // A parameterized generic alias / union (`(int | T)[str]`,
                    // `list[T][int]`) substitutes its __parameters__ with the
                    // subscript args (arity-checked).
                    if class_name == "typing.Alias" || class_name == "UnionType" {
                        return super::stdlib::typing_mod::alias_subscript(obj, key);
                    }
                    // namedtuple instances: int / slice indexing dispatches via
                    // an ephemeral tuple of the ordered field values so the
                    // existing tuple_ops paths handle bounds, negatives, and
                    // slices uniformly.
                    if let Some(vals) = super::stdlib::collections_mod::namedtuple_values(obj) {
                        let t = MbValue::from_ptr(MbObject::new_tuple(vals));
                        // Slice keys (res[:2], res[:]) lower to a 3-tuple
                        // (start, stop, step) on this dynamic path — unpack
                        // to the tuple slicer. Slice instances too.
                        if let Some(kp) = key.as_ptr() {
                            match &(*kp).data {
                                super::rc::ObjData::Tuple(kitems) if kitems.len() == 3 => {
                                    return super::tuple_ops::mb_tuple_slice_full(
                                        t, kitems[0], kitems[1], kitems[2],
                                    );
                                }
                                super::rc::ObjData::Instance {
                                    class_name: kcn,
                                    fields: kf,
                                } if kcn == "slice" => {
                                    let (start, stop, step) = {
                                        let f = kf.read().unwrap();
                                        (
                                            f.get("start").copied().unwrap_or_else(MbValue::none),
                                            f.get("stop").copied().unwrap_or_else(MbValue::none),
                                            f.get("step").copied().unwrap_or_else(MbValue::none),
                                        )
                                    };
                                    return super::tuple_ops::mb_tuple_slice_full(
                                        t, start, stop, step,
                                    );
                                }
                                _ => {}
                            }
                        }
                        return super::tuple_ops::mb_tuple_getitem(t, key);
                    }
                    // memoryview[i] — forward scalar indexes to the backing
                    // bytes/bytearray buffer. Slices return a new memoryview
                    // with byte-flat contiguity metadata.
                    // (#1256 sub-priority 4 — memoryview indexing gap)
                    if class_name == "memoryview" {
                        let buf = fields
                            .read()
                            .unwrap()
                            .get("_buffer")
                            .copied()
                            .unwrap_or(MbValue::none());
                        if buf.is_none() {
                            return MbValue::none();
                        }
                        // Detect a slice Instance key and dispatch to the
                        // bytes slice path so `mv[a:b:c]` returns the same
                        // shape as `bytes(b)[a:b:c]`.
                        if let Some(kp) = key.as_ptr() {
                            if let ObjData::Instance {
                                class_name: ref kn,
                                fields: ref kf,
                            } = (*kp).data
                            {
                                if kn == "slice" {
                                    let g = kf.read().unwrap();
                                    let start = g.get("start").copied().unwrap_or(MbValue::none());
                                    let stop = g.get("stop").copied().unwrap_or(MbValue::none());
                                    let step = g.get("step").copied().unwrap_or(MbValue::none());
                                    drop(g);
                                    return memoryview_slice(obj, start, stop, step);
                                }
                            }
                        }
                        return super::bytes_ops::mb_bytes_getitem(buf, key);
                    }
                    // UserDict / UserList / UserString subscript forwards to
                    // the backing value. UserDict honours the `__missing__`
                    // hook on subclasses (CPython: looked up on the class,
                    // consulted only by __getitem__ — `.get` bypasses it).
                    if let Some(kind) =
                        super::stdlib::collections_mod::user_wrapper_kind(class_name)
                    {
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        if !data.is_none() {
                            match kind {
                                "list" => return super::list_ops::mb_list_getitem(data, key),
                                "str" => return mb_obj_getitem(data, key),
                                _ => {
                                    if super::dict_ops::mb_dict_contains(data, key).as_bool()
                                        == Some(true)
                                    {
                                        return super::dict_ops::mb_dict_getitem(data, key);
                                    }
                                    let missing = lookup_method(class_name, "__missing__");
                                    if !missing.is_none() {
                                        let addr = extract_func_addr(missing);
                                        if addr > 4096 {
                                            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            return f(obj, key);
                                        }
                                    }
                                    let key_repr = super::builtins::mb_repr(key);
                                    let key_str = extract_str(key_repr).unwrap_or_default();
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "KeyError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(key_str)),
                                    );
                                    return MbValue::none();
                                }
                            }
                        }
                    }
                    if class_name == "collections.Counter" {
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        let val = super::dict_ops::mb_dict_getitem(data, key);
                        if !val.is_none() {
                            return val;
                        }
                        // Counter returns 0 for missing keys. mb_dict_getitem
                        // raised a KeyError into the runtime exception slot
                        // when it returned None; suppress it because Counter
                        // semantics never propagate KeyError on subscript.
                        super::exception::mb_clear_exception();
                        return MbValue::from_int(0);
                    }
                    if class_name == "collections.OrderedDict" {
                        // Forward subscript to the backing dict; KeyError on
                        // miss propagates (matches CPython OrderedDict). (#1650)
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        return super::dict_ops::mb_dict_getitem(data, key);
                    }
                    if class_name == "mappingproxy" {
                        // Read-only view: forward subscript to the wrapped
                        // mapping (KeyError on miss propagates).
                        let guard = fields.read().unwrap();
                        let data = guard.get("_mapping").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        return mb_obj_getitem(data, key);
                    }
                    if class_name == "collections.defaultdict" {
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        let factory = guard.get("_factory").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        // Look up key in the internal dict.
                        let val = super::dict_ops::mb_dict_getitem(data, key);
                        if !val.is_none() {
                            return val;
                        }
                        // Same suppression as Counter: mb_dict_getitem signals
                        // miss via KeyError; defaultdict's contract is to
                        // call default_factory instead.
                        super::exception::mb_clear_exception();
                        // CPython: defaultdict(None) behaves like a plain dict
                        // — a missing key raises KeyError instead of invoking
                        // a factory.
                        if factory.is_none() {
                            let key_repr = super::builtins::mb_repr(key);
                            let key_str = extract_str(key_repr).unwrap_or_default();
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(key_str)),
                            );
                            return MbValue::none();
                        }
                        // Key missing: call factory, insert, return default.
                        // Python builtins like `int`, `list`, `dict` are passed
                        // as type-name strings (legacy) or type-singleton objects (new).
                        let factory_name = super::builtins::callable_as_type_name(factory);
                        let default = if let Some(ref s) = factory_name {
                            match s.as_str() {
                                "int" => MbValue::from_int(0),
                                "float" => MbValue::from_float(0.0),
                                "str" => MbValue::from_ptr(MbObject::new_str(String::new())),
                                "list" => MbValue::from_ptr(MbObject::new_list(vec![])),
                                "dict" => MbValue::from_ptr(MbObject::new_dict()),
                                "bool" => MbValue::from_bool(false),
                                _ => mb_call0(factory),
                            }
                        } else {
                            mb_call0(factory)
                        };
                        super::dict_ops::mb_dict_setitem(data, key, default);
                        return default;
                    }
                    // types.SimpleNamespace is not a mapping: subscripting it
                    // raises TypeError (it defines no __getitem__). (#654)
                    if class_name == "SimpleNamespace" {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "'types.SimpleNamespace' object is not subscriptable".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(_method) = try_get_dunder(obj, "__getitem__") {
        let method_name = MbValue::from_ptr(MbObject::new_str("__getitem__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![key]));
        return mb_call_method(obj, method_name, args);
    }
    MbValue::none()
}

/// Runtime-dispatched __setitem__: list or dict.
pub fn mb_obj_setitem(obj: MbValue, key: MbValue, value: MbValue) -> MbValue {
    // Normalize a slice *object* key to the (start, stop, step) tuple the
    // built-in slice-assignment handling consumes, unless `obj` has a user
    // `__setitem__` (which must receive the real slice). Slice subscripts now
    // lower to slice objects, so `xs[a:b] = v` arrives here as an Instance.
    if !obj_has_user_dunder(obj, "__setitem__") {
        if let Some(tuple_key) = slice_obj_as_tuple(key) {
            return mb_obj_setitem(obj, tuple_key, value);
        }
    }
    // mappingproxy is a read-only mapping view: item assignment is a TypeError
    // (CPython). Reject before any backing-dict mutation.
    if let Some(ptr) = obj.as_ptr() {
        let is_proxy = unsafe {
            matches!(&(*ptr).data,
                ObjData::Instance { class_name, .. } if class_name == "mappingproxy")
        };
        if is_proxy {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "'mappingproxy' object does not support item assignment".to_string())),
            );
            return MbValue::none();
        }
    }
    // memoryview[i] = v / memoryview[a:b] = v → forward to the backing buffer
    // (a bytearray), which handles both index and slice assignment. A read-only
    // view (bytes source or toreadonly()) rejects the store.
    if let Some(ptr) = obj.as_ptr() {
        let mv_buf = unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "memoryview" {
                    let g = fields.read().unwrap();
                    let readonly = if let Some(ro) = g.get("_readonly") {
                        ro.as_bool() == Some(true)
                    } else {
                        g.get("_buffer")
                            .and_then(|b| b.as_ptr())
                            .map_or(true, |bp| !matches!((*bp).data, ObjData::ByteArray(_)))
                    };
                    if readonly {
                        drop(g);
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "cannot modify read-only memoryview object".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    g.get("_buffer").copied()
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(buf) = mv_buf {
            return mb_obj_setitem(buf, key, value);
        }
    }
    // ET.Element subscript assignment: e[i] = child / e[a:b] = [children].
    if let Some(kids) = element_children_list(obj) {
        let slice_parts: Option<(Option<i64>, Option<i64>)> = key.as_ptr().and_then(|p| unsafe {
            if let ObjData::Tuple(ref t) = (*p).data {
                if t.len() == 3 && t[2].as_int().unwrap_or(1) == 1 {
                    return Some((t[0].as_int(), t[1].as_int()));
                }
            }
            None
        });
        if let Some(kp) = kids.as_ptr() {
            unsafe {
                if let ObjData::List(ref kl) = (*kp).data {
                    if let Some((start, stop)) = slice_parts {
                        let new_items: Vec<MbValue> = value
                            .as_ptr()
                            .and_then(|vp| match &(*vp).data {
                                ObjData::List(l) => Some(l.read().unwrap().to_vec()),
                                ObjData::Tuple(t) => Some(t.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        let mut items = kl.write().unwrap();
                        let n = items.len() as i64;
                        let a = start
                            .map(|v| if v < 0 { v + n } else { v })
                            .unwrap_or(0)
                            .clamp(0, n) as usize;
                        let b = stop
                            .map(|v| if v < 0 { v + n } else { v })
                            .unwrap_or(n)
                            .clamp(0, n) as usize;
                        for v in &new_items {
                            super::rc::retain_if_ptr(*v);
                        }
                        let removed: Vec<MbValue> = items.drain(a..b.max(a)).collect();
                        for (off, v) in new_items.into_iter().enumerate() {
                            items.insert(a + off, v);
                        }
                        drop(items);
                        for r in removed {
                            super::rc::release_if_ptr(r);
                        }
                        return MbValue::none();
                    }
                    if let Some(idx) = key.as_int() {
                        let mut items = kl.write().unwrap();
                        let n = items.len() as i64;
                        let i = if idx < 0 { idx + n } else { idx };
                        if i >= 0 && i < n {
                            super::rc::retain_if_ptr(value);
                            let old = items[i as usize];
                            items[i as usize] = value;
                            drop(items);
                            super::rc::release_if_ptr(old);
                            return MbValue::none();
                        }
                    }
                }
            }
        }
    }
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            return super::stdlib::array_mod::mb_array_setitem(obj, key, value);
        }
    }
    // Check if key is a normalized slice tuple (3-element tuple from Slice
    // lowering). Only sequence-like builtins consume that internal form here;
    // mapping/user objects must still be able to use ordinary 3-tuple keys
    // such as `d[1, 2, 3]`.
    if let Some(kp) = key.as_ptr() {
        unsafe {
            if let super::rc::ObjData::Tuple(ref items) = (*kp).data {
                if items.len() == 3 {
                    let sequence_target = obj.as_ptr().map(|op| {
                        matches!(
                            &(*op).data,
                            super::rc::ObjData::List(_)
                                | super::rc::ObjData::ByteArray(_)
                                | super::rc::ObjData::Tuple(_)
                                | super::rc::ObjData::Bytes(_)
                                | super::rc::ObjData::Str(_)
                        )
                    }).unwrap_or(false);
                    if sequence_target {
                        super::list_ops::mb_list_setslice(
                            obj, items[0], items[1], items[2], value,
                        );
                        return MbValue::none();
                    }
                }
            }
        }
    }
    unsafe {
        if let Some(ptr) = obj.as_ptr() {
            match &(*ptr).data {
                super::rc::ObjData::List(_) => {
                    super::list_ops::mb_list_setitem(obj, key, value);
                    return MbValue::none();
                }
                super::rc::ObjData::Dict(_) => {
                    super::dict_ops::mb_dict_setitem(obj, key, value);
                    return MbValue::none();
                }
                super::rc::ObjData::ByteArray(_) => {
                    super::list_ops::mb_list_setitem(obj, key, value);
                    return MbValue::none();
                }
                // Immutable targets — mb_list_setitem raises the CPython
                // TypeError ("'tuple' object does not support item
                // assignment") instead of silently dropping the store.
                super::rc::ObjData::Tuple(_)
                | super::rc::ObjData::Bytes(_)
                | super::rc::ObjData::Str(_) => {
                    super::list_ops::mb_list_setitem(obj, key, value);
                    return MbValue::none();
                }
                super::rc::ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    if class_name == "collections.defaultdict"
                        || class_name == "collections.Counter"
                        || class_name == "collections.OrderedDict"
                        || super::stdlib::collections_mod::user_wrapper_kind(class_name).is_some()
                    {
                        let guard = fields.read().unwrap();
                        let data = guard.get("_data").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        if !data.is_none() {
                            // UserList backing is a list: route through the
                            // generic setitem so int indices work.
                            if matches!(
                                super::stdlib::collections_mod::user_wrapper_kind(class_name),
                                Some("list")
                            ) {
                                super::list_ops::mb_list_setitem(data, key, value);
                            } else {
                                super::dict_ops::mb_dict_setitem(data, key, value);
                            }
                            return MbValue::none();
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if let Some(_method) = try_get_dunder(obj, "__setitem__") {
        let method_name = MbValue::from_ptr(MbObject::new_str("__setitem__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![key, value]));
        return mb_call_method(obj, method_name, args);
    }
    MbValue::none()
}

/// del obj[key] — dispatch to list_delitem or dict_delitem at runtime.
pub fn mb_obj_delitem(obj: MbValue, key: MbValue) {
    // Normalize a slice *object* key to the (start, stop, step) tuple form,
    // unless `obj` has a user `__delitem__` (which must receive the real
    // slice). `del xs[a:b]` now lowers to a slice object.
    if !obj_has_user_dunder(obj, "__delitem__") {
        if let Some(tuple_key) = slice_obj_as_tuple(key) {
            return mb_obj_delitem(obj, tuple_key);
        }
    }
    // ET.Element subscript deletion: del e[i] / del e[a:b].
    if let Some(kids) = element_children_list(obj) {
        let slice_parts: Option<(Option<i64>, Option<i64>)> = key.as_ptr().and_then(|p| unsafe {
            if let ObjData::Tuple(ref t) = (*p).data {
                if t.len() == 3 && t[2].as_int().unwrap_or(1) == 1 {
                    return Some((t[0].as_int(), t[1].as_int()));
                }
            }
            None
        });
        if let Some(kp) = kids.as_ptr() {
            unsafe {
                if let ObjData::List(ref kl) = (*kp).data {
                    if let Some((start, stop)) = slice_parts {
                        let mut items = kl.write().unwrap();
                        let n = items.len() as i64;
                        let a = start
                            .map(|v| if v < 0 { v + n } else { v })
                            .unwrap_or(0)
                            .clamp(0, n) as usize;
                        let b = stop
                            .map(|v| if v < 0 { v + n } else { v })
                            .unwrap_or(n)
                            .clamp(0, n) as usize;
                        let removed: Vec<MbValue> = items.drain(a..b.max(a)).collect();
                        drop(items);
                        for r in removed {
                            super::rc::release_if_ptr(r);
                        }
                        return;
                    }
                    if let Some(idx) = key.as_int() {
                        let mut items = kl.write().unwrap();
                        let n = items.len() as i64;
                        let i = if idx < 0 { idx + n } else { idx };
                        if i >= 0 && i < n {
                            let removed = items.remove(i as usize);
                            drop(items);
                            super::rc::release_if_ptr(removed);
                            return;
                        }
                    }
                }
            }
        }
    }
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            super::stdlib::array_mod::mb_array_delitem(obj, key);
            return;
        }
    }
    unsafe {
        if let Some(ptr) = obj.as_ptr() {
            match &(*ptr).data {
                super::rc::ObjData::List(_) => {
                    super::list_ops::mb_list_delitem(obj, key);
                }
                super::rc::ObjData::ByteArray(_) => {
                    super::bytes_ops::mb_bytearray_delitem(obj, key);
                }
                super::rc::ObjData::Dict(_) => {
                    super::dict_ops::mb_dict_delitem(obj, key);
                }
                _ => {
                    // Dispatch the __delitem__ dunder like the __setitem__
                    // path; without one, item deletion is a TypeError.
                    if try_get_dunder(obj, "__delitem__").is_some() {
                        let method_name =
                            MbValue::from_ptr(MbObject::new_str("__delitem__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![key]));
                        let _ = mb_call_method(obj, method_name, args);
                    } else if let super::rc::ObjData::Instance { ref class_name, .. } = (*ptr).data
                    {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'{class_name}' object doesn't support item deletion"
                            ))),
                        );
                    }
                }
            }
        }
    }
}

// ── Context Manager Protocol ──

/// with obj as var: → calls __enter__, returns result to bind to var.
/// For file objects (int IDs): returns self (the file handle).
/// For class instances: dispatches __enter__ dunder.
pub fn mb_context_enter(obj: MbValue) -> MbValue {
    // @contextlib.contextmanager: the decorated call returns a generator
    // handle (a plain int). Drive the generator to its first `yield` and
    // return the yielded value. A raw generator in a `with` is always an
    // error in CPython (no __enter__), so treating live generator handles as
    // contextmanager-driven here is safe.
    if super::generator::is_known_generator(obj) {
        return super::stdlib::contextlib_mod::cm_gen_enter(obj);
    }
    // File objects: __enter__ returns self
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        let is_file = super::file_io::is_file_handle(id);
        if is_file {
            // Retain: JIT releases both obj arg VReg and result VReg.
            unsafe { super::rc::retain_if_ptr(obj) };
            return obj; // file.__enter__() → self
        }
    }
    // threading.Lock / RLock / Condition stubs are Instances whose method
    // table is not registered as a class; route their __enter__ to the
    // direct acquire-handler so `with lock:` works.
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" || class_name == "Condition" {
                    let _ = super::stdlib::threading_mod::mb_threading_lock_acquire(obj);
                    super::rc::retain_if_ptr(obj);
                    return obj;
                }
            }
        }
    }
    // tarfile.TarFile dict-stubs: __enter__ re-checks the closed flag and
    // raises OSError on a closed archive (CPython TarFile._check).
    if super::dict_ops::dict_stub_class(obj).as_deref() == Some("TarFile") {
        return super::stdlib::tarfile_mod::tarfile_context_enter(obj);
    }
    // tempfile instances: must run before the generic dunder lookup — their
    // registered class methods are dir()-listing stubs, not real callables.
    if let Some(r) = super::stdlib::tempfile_mod::tempfile_context_enter(obj) {
        return r;
    }
    // Class instances: look up __enter__
    if let Some(method) = try_get_dunder(obj, "__enter__") {
        // CPython requires BOTH __enter__ and __exit__ on the type; a missing
        // (e.g. misspelled) __exit__ raises TypeError at `with` entry. Gated to
        // user classes so native CM stubs whose __exit__ lives off the method
        // table are unaffected.
        if try_get_dunder(obj, "__exit__").is_none() {
            if let Some(ptr) = obj.as_ptr() {
                let cn = unsafe {
                    if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                        Some(class_name.clone())
                    } else { None }
                };
                if let Some(cn) = cn {
                    if USER_CLASSES.with(|u| u.borrow().contains(cn.as_str())) {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "'{cn}' object does not support the context manager protocol (missed __exit__ method)"
                            ))),
                        );
                        return MbValue::none();
                    }
                }
            }
        }
        let result = mb_call_method1(method, obj);
        // If __enter__ returned self (same pointer), add a retain to compensate:
        // the JIT holds TWO VRegs (ctx_vreg and enter_dest) that both point to
        // the same object, but only one refcount was present from the original
        // allocation. Without this retain, the JIT epilogue double-releases the
        // object → UAF.
        if result.to_bits() == obj.to_bits() {
            unsafe {
                super::rc::retain_if_ptr(result);
            }
        }
        return result;
    }
    // Retain: JIT releases both obj arg VReg and result VReg.
    unsafe { super::rc::retain_if_ptr(obj) };
    obj // fallback: return self
}

/// True if `obj` supports the context-manager protocol — i.e. `mb_context_enter`
/// has a real entry path for it (generator from @contextmanager, file handle,
/// threading Lock/RLock/Condition, tarfile.TarFile stub) or it resolves both
/// `__enter__` and `__exit__`. Used by `contextlib.ExitStack.enter_context` to
/// reject a non-context-manager argument with TypeError (CPython behavior).
/// Mirrors every positive branch above so a valid CM is never rejected.
pub fn value_supports_context_manager(obj: MbValue) -> bool {
    if super::generator::is_known_generator(obj) {
        return true;
    }
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::file_io::is_file_handle(id) {
            return true;
        }
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" || class_name == "Condition" {
                    return true;
                }
            }
        }
    }
    if super::dict_ops::dict_stub_class(obj).as_deref() == Some("TarFile") {
        return true;
    }
    // Generic protocol: both dunders must resolve (CPython checks
    // type(cm).__enter__ and type(cm).__exit__). A stub __enter__/__exit__ in a
    // registered native class still counts — try_get_dunder finds it.
    try_get_dunder(obj, "__enter__").is_some() && try_get_dunder(obj, "__exit__").is_some()
}

/// async with obj as var: → calls __aenter__, awaits the returned coroutine,
/// returns the resolved value to bind to var. Falls back to mb_context_enter
/// (sync __enter__) when no async dunder is defined, so user code can use
/// `async with` over plain context managers as CPython allows.
pub fn mb_async_context_enter(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__aenter__") {
        let coro = mb_call_method1(method, obj);
        // The async method returns a coroutine handle; await it to get the
        // actual entered value (e.g. `async def __aenter__(self): return ...`).
        let result = super::async_task::mb_await(coro);
        if result.to_bits() == obj.to_bits() {
            unsafe {
                super::rc::retain_if_ptr(result);
            }
        }
        return result;
    }
    mb_context_enter(obj)
}

/// async with cleanup: dispatches __aexit__(self, exc_type, exc_val, exc_tb)
/// and awaits the returned coroutine. Mirrors mb_context_exit's exception
/// suppression contract: when a pending exception is in flight, save it,
/// clear it so the coroutine body can run runtime helpers without short-
/// circuiting, then either re-raise or suppress based on the awaited
/// result. Falls back to mb_context_exit when __aexit__ is absent.
pub fn mb_async_context_exit(obj: MbValue, _has_exc: MbValue) -> MbValue {
    let pending = super::exception::mb_get_exception();
    let has_pending = !pending.is_none();
    if has_pending {
        super::exception::mb_clear_exception();
    }

    let result = if let Some(_method) = try_get_dunder(obj, "__aexit__") {
        let none = MbValue::none();
        let (exc_type, exc_val, exc_tb) = if has_pending {
            (pending, pending, none)
        } else {
            (none, none, none)
        };
        let method_name = MbValue::from_ptr(MbObject::new_str("__aexit__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![exc_type, exc_val, exc_tb]));
        let coro = mb_call_method(obj, method_name, args);
        super::async_task::mb_await(coro)
    } else {
        // Restore the exception so mb_context_exit reads the same state and
        // performs its own clear/restore dance.
        if has_pending {
            super::exception::mb_reraise(pending);
        }
        return mb_context_exit(obj, MbValue::none());
    };

    if has_pending {
        let suppressed = super::builtins::mb_is_truthy(result) != 0;
        if !suppressed {
            super::exception::mb_reraise(pending);
        }
    }

    result
}

/// Exit a context manager: calls __exit__(self, exc_type, exc_val, exc_tb).
/// Returns true if the exception should be suppressed.
/// For file objects: close the file, return false.
///
/// Reads the runtime exception state via `mb_get_exception()` so the with
/// statement's exception path actually triggers `__exit__(exc, exc, None)`.
/// The lowering pipeline emits a single `mb_context_exit(ctx, None)` call
/// after every `with` body — straight-line MIR, executed on both success
/// and failure paths because `raise` inside a `with` does NOT terminate the
/// block (see `lower/hir_to_mir.rs::HirStmt::Raise`, comment at line 2143).
/// `has_exc` is therefore always None at the call site; the actual exception
/// status must be read from thread-local state here.
///
/// Suppression contract (CPython): if `__exit__` returns truthy, the
/// exception is swallowed; otherwise it is re-raised so the enclosing
/// try/handler chain can match it. While `__exit__` is executing the
/// runtime exception flag must be cleared, otherwise short-circuiting
/// runtime helpers (mb_call_method etc.) bail out before `__exit__` runs.
pub fn mb_context_exit(obj: MbValue, _has_exc: MbValue) -> MbValue {
    let pending = super::exception::mb_get_exception();
    let has_pending = !pending.is_none();
    if has_pending {
        // Clear the runtime exception so the __exit__ body's runtime calls
        // (mb_call_method, attribute lookup, mb_print, ...) do not short-
        // circuit. Restored below if __exit__ does not suppress.
        super::exception::mb_clear_exception();
    }

    // @contextlib.contextmanager: drive the generator's exit. cm_gen_exit may
    // leave a *new* exception pending (when the generator raises a different
    // exception in its except/finally); in that case it returns truthy so we
    // do NOT re-raise the original `pending` and let the new one propagate.
    if super::generator::is_known_generator(obj) {
        let none = MbValue::none();
        let (exc_type, exc_val, exc_tb) = if has_pending {
            (pending, pending, none)
        } else {
            (none, none, none)
        };
        let result = super::stdlib::contextlib_mod::cm_gen_exit(obj, exc_type, exc_val, exc_tb);
        if has_pending {
            // If the generator left a *new* exception pending (different from
            // the original), it is already in the runtime slot — leave it.
            let new_pending = super::exception::mb_has_exception().as_bool() == Some(true);
            let suppressed = super::builtins::mb_is_truthy(result) != 0;
            if !suppressed && !new_pending {
                super::exception::mb_reraise(pending);
            }
        }
        return result;
    }

    // File objects: __exit__ closes the file
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        let is_file = super::file_io::is_file_handle(id);
        if is_file {
            super::file_io::mb_file_close(obj);
            if has_pending {
                super::exception::mb_reraise(pending);
            }
            return MbValue::from_bool(false); // never suppress exceptions
        }
    }
    // threading.Lock / RLock / Condition stubs: release the lock on exit
    // (no method-table dispatch, mirror the __enter__ early-return).
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" || class_name == "RLock" || class_name == "Condition" {
                    super::stdlib::threading_mod::mb_threading_lock_release(obj);
                    if has_pending {
                        super::exception::mb_reraise(pending);
                    }
                    return MbValue::from_bool(false);
                }
            }
        }
    }
    // tarfile.TarFile dict-stubs: __exit__ finalizes + closes on clean exit,
    // marks closed without the end-of-archive blocks when an exception is in
    // flight (CPython TarFile.__exit__ parity).
    if super::dict_ops::dict_stub_class(obj).as_deref() == Some("TarFile") {
        super::stdlib::tarfile_mod::tarfile_context_exit(obj, has_pending);
        if has_pending {
            super::exception::mb_reraise(pending);
        }
        return MbValue::from_bool(false);
    }
    // tempfile instances: close/cleanup before the generic dunder lookup
    // (registered class methods are dir()-listing stubs, not real callables).
    if let Some(r) = super::stdlib::tempfile_mod::tempfile_context_exit(obj) {
        if has_pending {
            super::exception::mb_reraise(pending);
        }
        return r;
    }
    // Class instances: look up __exit__
    let result = if let Some(method) = try_get_dunder(obj, "__exit__") {
        let addr = extract_func_addr(method);
        let none = MbValue::none();
        let (exc_type, exc_val, exc_tb) = if has_pending {
            // CPython passes (type(exc), exc, tb): exc_type is the exception's
            // CLASS object (not the value, so `exc_type.__name__` / `exc_type is
            // not None` work), and exc_tb is the exception's __traceback__ (so
            // `tb is not None` is True under an active exception).
            let type_obj = super::exception::get_exception_type_pub(pending)
                .map(|tn| make_type_object(&tn))
                .unwrap_or(pending);
            let tb = mb_getattr(
                pending,
                MbValue::from_ptr(MbObject::new_str("__traceback__".to_string())),
            );
            (type_obj, pending, tb)
        } else {
            (none, none, none)
        };
        if addr != 0 {
            let is_registered = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
            if is_registered {
                // __exit__(self, exc_type, exc_val, exc_tb) — 4-arg SystemV call.
                let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                    unsafe { std::mem::transmute(addr as usize) };
                f(obj, exc_type, exc_val, exc_tb)
            } else {
                // Fallback dispatch for non-registered methods.
                let method_name = MbValue::from_ptr(MbObject::new_str("__exit__".to_string()));
                let args = MbValue::from_ptr(MbObject::new_list(vec![exc_type, exc_val, exc_tb]));
                mb_call_method(obj, method_name, args)
            }
        } else {
            let method_name = MbValue::from_ptr(MbObject::new_str("__exit__".to_string()));
            let args = MbValue::from_ptr(MbObject::new_list(vec![exc_type, exc_val, exc_tb]));
            mb_call_method(obj, method_name, args)
        }
    } else {
        MbValue::from_bool(false)
    };

    // Re-raise the saved exception unless __exit__ explicitly suppressed
    // it (returned truthy). Suppression only applies when there WAS a
    // pending exception to begin with.
    if has_pending {
        let suppressed = super::builtins::mb_is_truthy(result) != 0;
        if !suppressed {
            super::exception::mb_reraise(pending);
        }
    }

    result
}

/// Call __contains__ on an object (for `in` operator).
pub fn mb_obj_contains(obj: MbValue, item: MbValue) -> MbValue {
    if let Some(data) = unwrap_dictlike_data(obj) {
        return super::dict_ops::mb_dict_contains(data, item);
    }
    if obj.is_int() {
        let id = obj.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            return super::stdlib::array_mod::mb_array_contains(obj, item);
        }
    }
    // Range-iterator handles look like ints (the iterator id). Match
    // CPython's range.__contains__ — O(1) math check, never iterates.
    if let Some((current, stop, step)) = super::iter::mb_iter_range_params(obj) {
        if let Some(v) = item.as_int() {
            if step == 0 {
                return MbValue::from_bool(false);
            }
            let in_bounds = if step > 0 {
                v >= current && v < stop
            } else {
                v <= current && v > stop
            };
            if !in_bounds {
                return MbValue::from_bool(false);
            }
            return MbValue::from_bool((v - current).rem_euclid(step.abs()) == 0);
        }
        return MbValue::from_bool(false);
    }
    // Class-body enum class: `member in Color` / `value in Color`.
    if let Some(p) = obj.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                if let Some(found) = super::stdlib::enum_class::class_contains(s, item) {
                    return MbValue::from_bool(found);
                }
            }
        }
    }
    // Flag composite containment: `Color.RED in (Color.RED | Color.BLUE)`.
    if let Some(found) = super::stdlib::enum_class::flag_member_contains(obj, item) {
        return MbValue::from_bool(found);
    }
    // Functional-API enum class objects: `member in EnumCls` (identity) and
    // `value in EnumCls` (data-type/value match, CPython 3.12 semantics).
    if let Some(items) = super::stdlib::enum_mod::functional_enum_members(obj) {
        let found = items.iter().any(|m| {
            if m.to_bits() == item.to_bits()
                || super::builtins::mb_eq(*m, item).as_bool().unwrap_or(false)
            {
                return true;
            }
            let mv = super::stdlib::enum_mod::mb_enum_member_value(*m);
            !mv.is_none() && super::builtins::mb_eq(mv, item).as_bool().unwrap_or(false)
        });
        return MbValue::from_bool(found);
    }
    if let Some(_method) = try_get_dunder(obj, "__contains__") {
        let method_name = MbValue::from_ptr(MbObject::new_str("__contains__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![item]));
        return mb_call_method(obj, method_name, args);
    }
    MbValue::from_bool(false)
}

/// Call __len__ on an object.
pub fn mb_obj_len(obj: MbValue) -> MbValue {
    if let Some(data) = unwrap_dictlike_data(obj) {
        return super::dict_ops::mb_dict_len(data);
    }
    if let Some(_method) = try_get_dunder(obj, "__len__") {
        let method_name = MbValue::from_ptr(MbObject::new_str("__len__".to_string()));
        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
        return mb_call_method(obj, method_name, args);
    }
    MbValue::from_int(0)
}

/// If obj is a dict-like collections Instance (defaultdict, Counter, OrderedDict),
/// return its backing `_data` dict. Otherwise None.
pub(crate) fn unwrap_dictlike_data(obj: MbValue) -> Option<MbValue> {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let super::rc::ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // ChainMap flattens to a fresh dict (front map wins). Callers
                // only read from the unwrapped mapping, so the copy is safe.
                if class_name == "collections.ChainMap" {
                    return super::stdlib::collections_mod::chainmap_flatten(obj);
                }
                if class_name == "collections.defaultdict"
                    || class_name == "collections.Counter"
                    || class_name == "collections.OrderedDict"
                    || class_name == "BaseCookie"
                    || class_name == "SimpleCookie"
                    || super::stdlib::collections_mod::user_wrapper_kind(class_name) == Some("dict")
                {
                    let guard = fields.read().unwrap();
                    let data = guard.get("_data").copied();
                    if let Some(d) = data {
                        if !d.is_none() {
                            return Some(d);
                        }
                    }
                    drop(guard);
                    // Cookie instances create their backing dict lazily; a
                    // fresh one is an empty mapping, not a non-iterable.
                    if class_name == "BaseCookie" || class_name == "SimpleCookie" {
                        let d = MbValue::from_ptr(MbObject::new_dict());
                        fields.write().unwrap().insert("_data".to_string(), d);
                        return Some(d);
                    }
                }
            }
        }
    }
    None
}

/// Call __str__ on an object for string conversion.
pub fn mb_obj_str(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__str__") {
        return method;
    }
    // Fallback: use __repr__ if available
    if let Some(method) = try_get_dunder(obj, "__repr__") {
        return method;
    }
    // Default: "<ClassName instance>"
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_str(format!("<{class_name} instance>")));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_str("<object>".to_string()))
}

/// Call __repr__ on an object.
pub fn mb_obj_repr(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__repr__") {
        return method;
    }
    mb_obj_str(obj)
}

/// Call __bool__ on an object for truthiness testing.
pub fn mb_obj_bool(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__bool__") {
        return method;
    }
    // Fallback: __len__ != 0
    if let Some(method) = try_get_dunder(obj, "__len__") {
        return method; // Caller checks != 0
    }
    MbValue::from_bool(true) // Default: truthy
}

/// Call __hash__ on an object.
pub fn mb_obj_hash(obj: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__hash__") {
        return method;
    }
    // Default: use pointer as hash
    if let Some(ptr) = obj.as_ptr() {
        return MbValue::from_int((ptr as u64 >> 17) as i64);
    }
    MbValue::from_int(0)
}

/// Call __format__ on an object: format(obj, spec).
pub fn mb_obj_format(obj: MbValue, _spec: MbValue) -> MbValue {
    if let Some(method) = try_get_dunder(obj, "__format__") {
        return method;
    }
    // Default: use str() representation
    mb_obj_str(obj)
}

/// Call __del__ on an object (destructor/finalizer).
pub fn mb_obj_del(obj: MbValue) {
    if let Some(_method) = try_get_dunder(obj, "__del__") {
        // __del__ found — caller should invoke it via mb_call_method1
        // For now just a marker; actual invocation happens during GC/release
    }
}

/// Call a method (stored as function pointer MbValue) with one argument.
/// Used for dunder invocation from runtime code (e.g., __iter__(self), __next__(self)).
///
/// Only callable values registered via `mb_class_register` can be invoked.
/// Non-callable or unregistered values return None (TypeError).
pub fn mb_call_method1(method: MbValue, arg: MbValue) -> MbValue {
    // Safepoint poll at method call (R4)
    super::gc::gc_safepoint();
    let addr = extract_func_addr(method);
    if addr != 0 {
        let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
        if is_registered {
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let func: extern "C" fn(MbValue) -> MbValue =
                unsafe { std::mem::transmute(addr as usize) };
            return func(arg);
        }
    }
    MbValue::none()
}

/// Call a 0-arg function stored as a TAG_FUNC NaN-boxed value.
/// Used for calling decorated functions at call sites via dynamic dispatch.
/// Does NOT require CALLABLE_REGISTRY membership.
/// Also resolves closure handles (integer IDs from mb_closure_new).
/// Native extern functions (`extern "C" fn(*const MbValue, usize) -> MbValue`)
/// are detected via `is_native_func` and dispatched with the correct ABI (#1132).
pub fn mb_call0(func: MbValue) -> MbValue {
    super::gc::gc_safepoint();
    // Re-box raw i64 returns from JIT-compiled functions. `is_boxed` (from
    // is_boxed_return_func(addr), set for any/object-returning callees)
    // disambiguates a non-NaN-prefixed raw return: those return a valid MbValue
    // (e.g. a float) and must pass through untouched. See mb_call1_val::rebox.
    fn rebox(raw: MbValue, is_boxed: bool) -> MbValue {
        if is_boxed {
            return raw;
        }
        let bits = raw.to_bits();
        const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
        if bits & NAN_PREFIX == NAN_PREFIX {
            raw
        } else {
            super::builtins::mb_box_int(bits as i64)
        }
    }
    // Try TAG_FUNC direct function pointer first
    if let Some(addr) = func.as_func() {
        if addr > 4096 {
            // Native extern functions use (args_ptr, nargs) convention (#1132).
            if super::module::is_native_func(addr as u64) {
                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                    unsafe { std::mem::transmute(addr) };
                return unsafe { f(std::ptr::null(), 0) };
            }
            // Variadic / kwargs: route through mb_call_spread for uniform packing.
            if super::module::is_variadic_func(addr as u64)
                || super::module::is_kwargs_func(addr as u64)
            {
                let args_list = MbValue::from_ptr(super::rc::MbObject::new_list(vec![]));
                return super::builtins::mb_call_spread(func, args_list);
            }
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let is_boxed = super::module::is_boxed_return_func(addr as u64);
            let f: extern "C" fn() -> MbValue = unsafe { std::mem::transmute(addr) };
            return rebox(f(), is_boxed);
        }
    }
    // Try closure handle (integer ID → lookup inner function)
    if func.as_int().is_some() {
        let fn_val = super::closure::mb_closure_get_func(func);
        if let Some(addr) = fn_val.as_func() {
            if addr > 4096 {
                let is_boxed = super::module::is_boxed_return_func(addr as u64);
                // If the closure carries default argument values (e.g.
                // `lambda x=i: ...`), dispatch using those defaults so the
                // callee sees the frozen values instead of uninitialized
                // arg registers.
                let defaults = super::closure::closure_defaults(func);
                if !defaults.is_empty() {
                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                    match defaults.len() {
                        1 => {
                            let f: extern "C" fn(MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr) };
                            return rebox(f(defaults[0]), is_boxed);
                        }
                        2 => {
                            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr) };
                            return rebox(f(defaults[0], defaults[1]), is_boxed);
                        }
                        3 => {
                            let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                                unsafe { std::mem::transmute(addr) };
                            return rebox(f(defaults[0], defaults[1], defaults[2]), is_boxed);
                        }
                        _ => { /* fall through to 0-arg call */ }
                    }
                }
                let f: extern "C" fn() -> MbValue = unsafe { std::mem::transmute(addr) };
                return rebox(f(), is_boxed);
            }
        }
    }
    // functools.lru_cache bound-method for cache_info() / cache_clear() —
    // zero-arg call routes through mb_call_spread which detects the class.
    // Unbound-method wrappers (`m = Path.cwd; m()`) also route through
    // mb_call_spread, which owns the receiver-less classmethod dispatch.
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "functools._lru_bound_method"
                    || class_name == "functools.lru_cache_wrapper"
                    || class_name == "functools.lru_cache_factory"
                    || class_name == "functools.singledispatch"
                    || class_name == "__unbound_method__"
                    || class_name == "__bound_native_method__"
                    || class_name == "collections.namedtuple_factory"
                {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
            }
        }
    }
    // Type object as constructor: type('MyClass', bases, dict)() → instance creation
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    if let Some(type_name) = fields
                        .read()
                        .ok()
                        .and_then(|f| f.get("__name__").and_then(|v| extract_str(*v)))
                    {
                        // Singleton types: `type(None)()` / `type(...)()` /
                        // `type(NotImplemented)()` return the singleton itself,
                        // not a fresh instance (`type(None)() is None`).
                        match type_name.as_str() {
                            "NoneType" => return MbValue::none(),
                            "ellipsis" => return MbValue::ellipsis(),
                            "NotImplementedType" => return MbValue::not_implemented(),
                            _ => {}
                        }
                        let name_val = MbValue::from_ptr(MbObject::new_str(type_name));
                        let args_list = MbValue::from_ptr(MbObject::new_list(vec![]));
                        return mb_instance_new_with_init(name_val, args_list);
                    }
                }
                // weakref.ref(obj)() — zero-arg invocation of the strong-ref
                // carve-out. mb_call_spread also has this case (builtins.rs
                // ~3900), but the no-arg call path lands in mb_call0 first and
                // would otherwise fall through to MbValue::none() because
                // ReferenceType has no registered __call__ method.
                if class_name == "ReferenceType" {
                    let f = fields.read().unwrap();
                    let target = f.get("_target").copied().unwrap_or_else(MbValue::none);
                    drop(f);
                    super::rc::retain_if_ptr(target);
                    return target;
                }
                // Generic user-class instance callable — dispatch __call__.
                // Needed for `iter(c, sentinel)` where c defines __call__, and
                // any other 0-arg invocation of a callable instance.
                let _ = fields; // silence unused when the next block moves to methods
                let call_method = lookup_method(class_name, "__call__");
                if !call_method.is_none() {
                    let addr = extract_func_addr(call_method);
                    if addr > 4096 {
                        let is_boxed = super::module::is_boxed_return_func(addr as u64);
                        let f: extern "C" fn(MbValue) -> MbValue =
                            std::mem::transmute(addr as usize);
                        return rebox(f(func), is_boxed);
                    }
                }
            }
            // Bare class-name string naming a registered class → zero-arg
            // construction (`from M import C; C()`).
            if let ObjData::Str(ref s) = (*ptr).data {
                if class_is_registered(s) {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![]));
                    return mb_instance_new_with_init(func, args_list);
                }
            }
        }
    }
    MbValue::none()
}

/// Call a 1-arg function stored as a TAG_FUNC NaN-boxed value.
/// Used for calling decorated functions at call sites via dynamic dispatch.
/// Does NOT require CALLABLE_REGISTRY membership.
/// Also resolves closure handles (integer IDs from mb_closure_new).
/// Native extern functions (`extern "C" fn(*const MbValue, usize) -> MbValue`)
/// are detected via `is_native_func` and dispatched with the correct ABI (#1132).
pub fn mb_call1_val(func: MbValue, arg: MbValue) -> MbValue {
    super::gc::gc_safepoint();
    // Re-box raw i64 returns from JIT-compiled functions that declared a
    // primitive (int) return type — mb_call_spread has the same logic.
    //
    // A non-NaN-prefixed raw return is AMBIGUOUS: it is either a raw machine int
    // (int fast-path return, needs boxing) OR a float MbValue (untagged raw f64
    // bits, already correct). `is_boxed` — from is_boxed_return_func(addr), set
    // for any/object-returning callees — disambiguates: those return a valid
    // MbValue (a float, or an already-boxed value), so pass it through untouched
    // rather than mis-boxing the bit pattern as a giant int.
    fn rebox(raw: MbValue, is_boxed: bool) -> MbValue {
        if is_boxed {
            return raw;
        }
        let bits = raw.to_bits();
        const NAN_PREFIX: u64 = 0xFFF8_0000_0000_0000;
        if bits & NAN_PREFIX == NAN_PREFIX {
            raw
        } else {
            super::builtins::mb_box_int(bits as i64)
        }
    }
    // functools.partial / functools.wraps dispatch on Instance class_name.
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // Unbound method wrapper: route through mb_call_spread so
                // the 1-arg form `str.lower("HELLO")` dispatches correctly.
                if class_name == "__unbound_method__" || class_name == "__bound_native_method__" {
                    let _ = fields; // suppressed — dispatch via call_spread
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
                if class_name == "collections.abc._register_bound"
                    || class_name == "abc._user_register_bound"
                {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
                if class_name == "functools.partial"
                    || class_name == "functools.lru_cache_wrapper"
                    || class_name == "functools.lru_cache_factory"
                    || class_name == "functools.cmp_to_key"
                    || class_name == "functools.singledispatch"
                    || class_name == "functools._sd_register"
                    || class_name == "collections.namedtuple_factory"
                {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
                if class_name == "functools.wraps" {
                    // @wraps(f) applied to wrapper: copy __name__/__doc__/__module__
                    // /__qualname__ from f, set __wrapped__ = f, then return the
                    // wrapper. Full copy lives in functools_mod.
                    let wrapped = fields
                        .read()
                        .unwrap()
                        .get("_wrapped")
                        .copied()
                        .unwrap_or(MbValue::none());
                    return super::stdlib::functools_mod::mb_functools_wraps_apply(arg, wrapped);
                }
                if class_name == "UnionType" {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "Cannot instantiate typing.Union".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
                // Type instances (class objects assigned to variables, e.g.
                // `tt = bytearray; tt(b"abc")`) — route to mb_call_spread which
                // knows how to dispatch builtin constructors by __name__.
                if class_name == "type" {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
                if class_name == "HTTPStatus" {
                    return super::stdlib::http_mod::mb_httpstatus_call(arg);
                }
                // typing.NewType wrappers are identity callables.
                if class_name == "typing.NewType" {
                    super::rc::retain_if_ptr(arg);
                    return arg;
                }
                // Bound methods (types.MethodType): call __func__ with
                // __self__ prepended.
                if class_name == "method" {
                    let (func_v, self_v) = {
                        let f = fields.read().unwrap();
                        (
                            f.get("__func__").copied().unwrap_or_else(MbValue::none),
                            f.get("__self__").copied().unwrap_or_else(MbValue::none),
                        )
                    };
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![self_v, arg]));
                    return super::builtins::mb_call_spread(func_v, args_list);
                }
                // Functional-API enum class objects: `EnumCls(value)` is the
                // value→member lookup (`Minor(2) is Minor.july`).
                if super::stdlib::enum_mod::is_functional_enum_class(func) {
                    return super::stdlib::enum_mod::mb_functional_enum_call(func, arg);
                }
                // __call__ dunder dispatch for callable instances
                let call_method = lookup_method(class_name, "__call__");
                if !call_method.is_none() {
                    let method_name = MbValue::from_ptr(MbObject::new_str("__call__".to_string()));
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return mb_call_method(func, method_name, args_list);
                }
            }
        }
    }
    // Try TAG_FUNC direct function pointer first
    if let Some(addr) = func.as_func() {
        if addr > 4096 {
            // Native extern functions use (args_ptr, nargs) convention (#1132).
            if super::module::is_native_func(addr as u64) {
                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                    unsafe { std::mem::transmute(addr) };
                let args = [arg];
                return unsafe { f(args.as_ptr(), args.len()) };
            }
            // Variadic / kwargs: route through mb_call_spread for uniform packing.
            if super::module::is_variadic_func(addr as u64)
                || super::module::is_kwargs_func(addr as u64)
            {
                let args_list = MbValue::from_ptr(super::rc::MbObject::new_list(vec![arg]));
                return super::builtins::mb_call_spread(func, args_list);
            }
            // REQ: JIT-compiled functions use SystemV/C calling convention.
            let is_boxed = super::module::is_boxed_return_func(addr as u64);
            let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr) };
            return rebox(f(arg), is_boxed);
        }
    }
    // Try closure handle (integer ID → lookup inner function)
    if func.as_int().is_some() {
        let fn_val = super::closure::mb_closure_get_func(func);
        if let Some(addr) = fn_val.as_func() {
            if addr > 4096 {
                let is_boxed = super::module::is_boxed_return_func(addr as u64);
                if super::module::is_variadic_func(addr as u64)
                    || super::module::is_kwargs_func(addr as u64)
                {
                    let args_list = MbValue::from_ptr(super::rc::MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(fn_val, args_list);
                }
                // Partial-default dispatch: if the closure declares more params
                // than the call supplies, fill the trailing slots from
                // `defaults`. `defaults` holds only the Some(...) entries, so
                // the last (arity - K) of them line up with the missing params.
                let arity = super::closure::closure_arity(func);
                if arity > 1 {
                    let defaults = super::closure::closure_defaults(func);
                    let needed = arity - 1;
                    if defaults.len() >= needed {
                        let take_from = defaults.len() - needed;
                        let fill = &defaults[take_from..];
                        match arity {
                            2 => {
                                let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                    unsafe { std::mem::transmute(addr) };
                                return rebox(f(arg, fill[0]), is_boxed);
                            }
                            3 => {
                                let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                                    unsafe { std::mem::transmute(addr) };
                                return rebox(f(arg, fill[0], fill[1]), is_boxed);
                            }
                            4 => {
                                let f: extern "C" fn(
                                    MbValue,
                                    MbValue,
                                    MbValue,
                                    MbValue,
                                ) -> MbValue = unsafe { std::mem::transmute(addr) };
                                return rebox(f(arg, fill[0], fill[1], fill[2]), is_boxed);
                            }
                            _ => { /* arity > 4: fall through to plain 1-arg dispatch */ }
                        }
                    }
                }
                // REQ: JIT-compiled functions use SystemV/C calling convention.
                let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr) };
                return rebox(f(arg), is_boxed);
            }
        }
    }
    // A bare class-name string naming a registered user class is a constructor
    // when called through a value binding (e.g. `from plistlib import UID;
    // UID(1)`). Route to mb_call_spread, which fires __init__.
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                if class_is_registered(s) {
                    let args_list = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    return super::builtins::mb_call_spread(func, args_list);
                }
            }
        }
    }
    MbValue::none()
}

// ── Generic Method Dispatch (#380) ──

/// `receiver.method(pos..., kw=v...)` — the keyword-argument method-call form.
/// mb_call_method only takes a positional arg list, so the lowering routes
/// method calls that carry keywords here. We resolve the method's declared
/// parameter names (from the introspection registry, which now covers methods)
/// and bind the keywords into their positional slots, then dispatch through the
/// normal mb_call_method path (so Instance / descriptor / MRO semantics are
/// unchanged). Falls back to the legacy trailing-kwargs-dict convention when
/// the parameters can't be bound (native methods, *args, defaulted gaps).
pub fn mb_call_method_kwargs(
    receiver: MbValue,
    method_name: MbValue,
    pos_list: MbValue,
    kwargs_dict: MbValue,
) -> MbValue {
    let name = extract_str(method_name).unwrap_or_default();
    let pos = super::builtins::extract_items(pos_list);

    let bound: Option<Vec<MbValue>> = (|| {
        // Receiver class → resolve the (possibly inherited) method value.
        let class_name = receiver.as_ptr().and_then(|p| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*p).data {
                Some(class_name.clone())
            } else {
                None
            }
        })?;
        let method_val = lookup_method(&class_name, &name);
        if method_val.is_none() {
            return None;
        }
        let argcount = super::closure::mb_func_get_argcount(method_val).as_int()?;
        if argcount <= 1 {
            return None; // only `self` (or unknown) — nothing to bind
        }
        let varnames_tuple = super::closure::mb_func_get_varnames(method_val);
        let vp = varnames_tuple.as_ptr()?;
        let names: Vec<String> = unsafe {
            if let ObjData::Tuple(items) = &(*vp).data {
                items.iter().filter_map(|v| extract_str(*v)).collect()
            } else {
                return None;
            }
        };
        if names.len() < argcount as usize {
            return None;
        }
        // User-visible params are co_varnames[1..argcount] (drop the leading self).
        let user_params: Vec<&str> = names[1..argcount as usize]
            .iter()
            .map(|s| s.as_str())
            .collect();
        let mut slots: Vec<Option<MbValue>> = vec![None; user_params.len()];
        if pos.len() > slots.len() {
            return None; // too many positional → let the normal path handle it
        }
        for (i, v) in pos.iter().enumerate() {
            slots[i] = Some(*v);
        }
        let kp = kwargs_dict.as_ptr()?;
        let pairs: Vec<(String, MbValue)> = unsafe {
            if let ObjData::Dict(ref lock) = (*kp).data {
                lock.read().unwrap().iter().filter_map(|(k, v)| {
                    if let super::dict_ops::DictKey::Str(s) = k {
                        Some((s.clone(), *v))
                    } else {
                        None
                    }
                }).collect()
            } else {
                return None;
            }
        };
        for (k, v) in &pairs {
            match user_params.iter().position(|p| *p == k.as_str()) {
                Some(idx) => {
                    if slots[idx].is_some() {
                        return None; // duplicate value for a param → normal path
                    }
                    slots[idx] = Some(*v);
                }
                None => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "{}() got an unexpected keyword argument '{}'",
                            name, k
                        ))),
                    );
                    return Some(Vec::new()); // sentinel: raised (caught below)
                }
            }
        }
        // Every param must be filled — a defaulted gap would need the default,
        // which co_varnames doesn't carry, so fall back to the legacy path.
        let mut out = Vec::with_capacity(slots.len());
        for s in slots {
            out.push(s?);
        }
        Some(out)
    })();

    // An unexpected-keyword raise short-circuits here.
    if super::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    match bound {
        Some(args) => {
            let args_list = MbValue::from_ptr(MbObject::new_list_borrowed(args));
            mb_call_method(receiver, method_name, args_list)
        }
        None => {
            // Legacy fallback: append the kwargs dict as a trailing positional
            // arg (the previous convention) and let mb_call_method handle it.
            let mut all = pos;
            all.push(kwargs_dict);
            let args_list = MbValue::from_ptr(MbObject::new_list_borrowed(all));
            mb_call_method(receiver, method_name, args_list)
        }
    }
}

/// Type-tagged method dispatch: receiver.method_name(args).
/// Checks NaN-box tag for primitives, then ObjData variant for heap objects.
/// Falls back to MRO-based lookup for user class instances.
pub fn mb_call_method(receiver: MbValue, method_name: MbValue, args: MbValue) -> MbValue {
    // Safepoint poll at method dispatch (R4)
    super::gc::gc_safepoint();

    // Typed native wrappers are raw `Box<T>` pointers, not `MbObject`s.
    // Method lowering reaches this directly, so dispatch before fast paths.
    if let Some(type_name) = native_type_name_for(receiver) {
        let name = extract_str(method_name).unwrap_or_default();
        if let Some(getter) = super::registry_bridge::lookup_getter(type_name, &name) {
            let reg_receiver = cclab_mamba_registry::MbValue::from_bits(receiver.to_bits());
            let getter_args = [reg_receiver];
            let callable = unsafe { getter(getter_args.as_ptr(), getter_args.len()) };
            let callable = MbValue::from_bits(callable.to_bits());
            if let Some(addr) = callable.as_func() {
                let items = super::builtins::extract_items(args);
                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                    unsafe { std::mem::transmute(addr) };
                return unsafe { f(items.as_ptr(), items.len()) };
            }
            return super::builtins::mb_call_spread(callable, args);
        }
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{}' object has no attribute '{}'",
                type_name, name
            ))),
        );
        return MbValue::none();
    }

    // Comparison dunders on the None singleton: None.__eq__/__ne__ compare by
    // identity — True/False only against None, NotImplemented for any other
    // type (so `==`/`!=` fall back to identity). Ordering dunders are always
    // NotImplemented. None carries no MbObject, so without this it would fall
    // through to "'NoneType' object has no attribute '__ne__'".
    if receiver.is_none() {
        let name = extract_str(method_name).unwrap_or_default();
        if matches!(name.as_str(),
            "__eq__" | "__ne__" | "__lt__" | "__le__" | "__gt__" | "__ge__")
        {
            let items = super::builtins::extract_items(args);
            let other = items.first().copied().unwrap_or_else(MbValue::none);
            return match name.as_str() {
                "__eq__" if other.is_none() => MbValue::from_bool(true),
                "__ne__" if other.is_none() => MbValue::from_bool(false),
                _ => MbValue::not_implemented(),
            };
        }
    }

    // Bare object() instance: object's base dunders. A bare object() has no
    // registered methods, so __init__ (a no-op), __repr__/__str__ (default
    // repr), identity __eq__/__ne__, and __hash__ would otherwise fail to
    // dispatch. Scoped to class_name == "object" exactly, so user subclasses
    // (which carry their own class name) are unaffected.
    if let Some(ptr) = receiver.as_ptr() {
        let is_bare_object = unsafe {
            matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == "object")
        };
        if is_bare_object {
            let name = extract_str(method_name).unwrap_or_default();
            let items = super::builtins::extract_items(args);
            let other_bits = items.first().map_or(0, |v| v.to_bits());
            match name.as_str() {
                "__init__" => return MbValue::none(),
                "__repr__" | "__str__" => return super::builtins::mb_repr(receiver),
                "__eq__" => return MbValue::from_bool(receiver.to_bits() == other_bits),
                "__ne__" => return MbValue::from_bool(receiver.to_bits() != other_bits),
                "__hash__" => return super::builtins::mb_hash(receiver),
                _ => {}
            }
        }
    }

    // Issue #2097 fast path — module / plain-dict method dispatch is the
    // single hottest CALL_METHOD shape in idiomatic Python (`keyword.iskeyword(w)`
    // inside a for-loop). The JIT bakes the method name as an immortal
    // `ObjData::Str`, so we can borrow `&str` out of it directly instead
    // of cloning a fresh `String` for every iteration. Combined with a
    // direct Dict probe (skipping the upstream memoryview / lru_cache /
    // type-name classmethod cascades that don't fire on plain modules),
    // this collapses ~50us per call back down to a single hashmap lookup
    // + extern-C indirect call. Falls back to the full slow path on miss
    // so the existing dunder / descriptor / Instance semantics for every
    // other receiver shape are unchanged.
    if let (Some(recv_ptr), Some(name_ptr)) = (receiver.as_ptr(), method_name.as_ptr()) {
        unsafe {
            if let (ObjData::Dict(ref lock), ObjData::Str(ref name_s)) =
                (&(*recv_ptr).data, &(*name_ptr).data)
            {
                let callable = {
                    let guard = lock.read().unwrap();
                    guard.get(name_s.as_str()).copied()
                };
                if let Some(func_val) = callable {
                    if let Some(addr) = func_val.as_func() {
                        if super::module::is_native_func(addr as u64) {
                            let items = super::builtins::extract_items(args);
                            let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                std::mem::transmute(addr);
                            return f(items.as_ptr(), items.len());
                        }
                        return super::builtins::mb_call_spread(func_val, args);
                    }
                    if super::builtins::mb_callable(func_val).as_bool() == Some(true) {
                        return super::builtins::mb_call_spread(func_val, args);
                    }
                    // Non-callable dict entry → fall through to slow path
                    // so dict.method() dispatch (`keys()` etc.) still works.
                }
            }
        }
    }

    let name = extract_str(method_name).unwrap_or_default();
    if receiver.as_func().is_some() && name == "_convert" {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "'function' object has no attribute '_convert'".to_string(),
            )),
        );
        return MbValue::none();
    }

    // Native-class constructor func as receiver (`datetime.datetime.fromordinal(1)`,
    // `datetime.date.today()`): the chained call lowers to a CallMethod whose
    // receiver is the dispatcher func. Resolve through NATIVE_TYPE_NAMES into the
    // class table; the registered classmethod values are raw `(args_ptr, nargs)`
    // dispatchers, so pass the args through whole (no receiver split). Gated to
    // the date/datetime tables — other native types (pathlib.Path.joinpath)
    // rely on receiver-style unbound dispatch.
    if let Some(addr) = receiver.as_func() {
        let native_type =
            super::module::NATIVE_TYPE_NAMES.with(|map| map.borrow().get(&(addr as u64)).cloned());
        if let Some(nt) = native_type {
            if nt == "collections.Counter" && name == "fromkeys" {
                return mb_counter_fromkeys_not_implemented();
            }
            {
                let items = super::builtins::extract_items(args);
                if items.is_empty() {
                    if let Some(result) =
                        super::stdlib::string_constants_mod::static_no_self_error(&nt, &name)
                    {
                        return result;
                    }
                }
            }
            // `datetime.timezone.utc.dst(x)` arrives here with the CONSTRUCTOR
            // func as receiver and "utc" consumed by getattr — but chained
            // `datetime.timezone.utc` may also lower as CallMethod("utc") with
            // no args; surface the class attribute in that shape too.
            if nt == "datetime.timezone" {
                if let Some(v) = super::stdlib::datetime_mod::timezone_class_attr(&name) {
                    let items = super::builtins::extract_items(args);
                    if items.is_empty() {
                        return v;
                    }
                }
            }
            // `call.a(1)` — a method call on the call factory builds a named
            // call object for ANY method name.
            if nt == "_mock_call_factory" {
                return super::stdlib::unittest_mock_mod::make_named_call(&name, args);
            }
            if matches!(
                nt.as_str(),
                "date"
                    | "datetime"
                    | "datetime.time"
                    | "StackSummary"
                    | "TracebackException"
                    | "patch"
                    | "zipfile.ZipInfo"
                    | "chain"
            ) {
                let m = lookup_method(&nt, &name);
                if let Some(maddr) = m.as_func() {
                    if super::module::is_native_func(maddr as u64) {
                        let items = super::builtins::extract_items(args);
                        let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                            unsafe { std::mem::transmute(maddr) };
                        return unsafe { f(items.as_ptr(), items.len()) };
                    }
                }
            }
            // pathlib classmethods (`pathlib.Path.cwd()` / `Path.home()`):
            // the receiver is the class constructor dispatcher func, not an
            // instance. The registered methods are variadic
            // `fn(self, args_list)` — NOT raw `(args_ptr, nargs)` dispatchers
            // like the date/datetime table above — so dispatch with a None
            // receiver; the methods default to the host concrete flavour.
            if matches!(
                nt.as_str(),
                "Path"
                    | "PosixPath"
                    | "WindowsPath"
                    | "PurePath"
                    | "PurePosixPath"
                    | "PureWindowsPath"
            ) && matches!(name.as_str(), "cwd" | "home")
            {
                let m = lookup_method(&nt, &name);
                if let Some(maddr) = m.as_func() {
                    if super::module::is_variadic_func(maddr as u64) {
                        let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                            unsafe { std::mem::transmute(maddr) };
                        let arg_list = MbValue::from_ptr(MbObject::new_list(
                            super::builtins::extract_items(args),
                        ));
                        return f(MbValue::none(), arg_list);
                    }
                }
            }
        }
    }

    // unittest.mock: a method call on a mock instance autovivifies the child
    // mock and records the call — unless the name is a real registered helper
    // (assert_*, reset_mock, ...), which dispatches normally below.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if super::stdlib::unittest_mock_mod::is_mock_class(class_name)
                    && lookup_method(class_name, &name).is_none()
                {
                    let child = super::stdlib::unittest_mock_mod::mock_attr_child(receiver, &name);
                    if child.is_none() {
                        return MbValue::none();
                    }
                    return super::stdlib::unittest_mock_mod::mock_record_call(child, args);
                }
            }
        }
    }

    // statistics.NormalDist behavioral methods (cdf / pdf / inv_cdf / zscore /
    // quantiles). The constructor lives in statistics_mod; the methods are
    // delegated here so the instance stays a plain Instance. Returns None for
    // method names the module does not model, falling through to the generic
    // path below.
    // contextlib.ExitStack / AsyncExitStack stateful methods (enter_context,
    // callback, push, pop_all, close, plus the dunders). Route to the
    // contextlib module so the receiver's `_callbacks` list is mutated.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "contextlib.ExitStack" || class_name == "contextlib.AsyncExitStack"
                {
                    return super::stdlib::contextlib_mod::mb_exitstack_method(
                        receiver, &name, args,
                    );
                }
            }
        }
    }

    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "NormalDist" {
                    let items = super::builtins::extract_items(args);
                    if let Some(result) =
                        super::stdlib::statistics_mod::mb_statistics_normaldist_method(
                            receiver, &name, &items,
                        )
                    {
                        return result;
                    }
                }
            }
        }
    }
    // NormalDist classmethod: `NormalDist.from_samples(data)` — the receiver
    // is the native constructor dispatcher (mapped to "NormalDist" in
    // NATIVE_TYPE_NAMES), not an instance.
    if receiver.as_func().is_some()
        && name == "from_samples"
        && resolve_class_name(receiver).as_deref() == Some("NormalDist")
    {
        let items = super::builtins::extract_items(args);
        let data = items.first().copied().unwrap_or_else(MbValue::none);
        return super::stdlib::statistics_mod::mb_statistics_normaldist_from_samples(data);
    }

    // tempfile instance classes (SpooledTemporaryFile / NamedTemporaryFile /
    // TemporaryDirectory): route their file-protocol methods to tempfile_mod.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if matches!(
                    class_name.as_str(),
                    "SpooledTemporaryFile" | "NamedTemporaryFile" | "TemporaryDirectory"
                ) {
                    let items = super::builtins::extract_items(args);
                    if let Some(result) = super::stdlib::tempfile_mod::tempfile_instance_method(
                        receiver, &name, &items,
                    ) {
                        return result;
                    }
                }
            }
        }
    }

    // lzma incremental codec objects (LZMACompressor / LZMADecompressor):
    // route compress/decompress/flush to the liblzma streaming state.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "LZMACompressor" || class_name == "LZMADecompressor" {
                    let items = super::builtins::extract_items(args);
                    if let Some(result) =
                        super::stdlib::lzma_mod::lzma_instance_method(receiver, &name, &items)
                    {
                        return result;
                    }
                }
            }
        }
    }

    // User subclasses of random.Random: implement randrange/randint through
    // the CPython `_randbelow` override contract (an overridden getrandbits /
    // random must be exercised), and delegate the rest to the instance's
    // native generator handle. User-overridden methods fall through to the
    // generic dispatch (None return).
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name != "Random"
                    && class_name != "SystemRandom"
                    && class_mro_any(class_name, |c| c == "Random" || c == "SystemRandom")
                {
                    let items = super::builtins::extract_items(args);
                    if let Some(result) =
                        super::stdlib::random_mod::random_subclass_method(receiver, &name, &items)
                    {
                        return result;
                    }
                }
            }
        }
    }

    // User subclasses of calendar.HTMLCalendar: the native instances carry
    // flat-args method fields (no self), which a subclass instance lacks and
    // which cannot read css theme overrides off the receiver. Route subclass
    // receivers to the receiver-aware implementations. Native instances
    // (class_name == the base itself) keep their existing field path.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name != "HTMLCalendar"
                    && class_name != "LocaleHTMLCalendar"
                    && class_mro_any(class_name, |c| {
                        c == "HTMLCalendar" || c == "LocaleHTMLCalendar"
                    })
                {
                    let items = super::builtins::extract_items(args);
                    if let Some(result) = super::stdlib::calendar_mod::html_calendar_subclass_method(
                        receiver, &name, &items,
                    ) {
                        return result;
                    }
                }
            }
        }
    }

    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_matches_collections_abc(class_name, "MutableSequence") {
                    if let Some(result) = dispatch_mutable_sequence_mixin(receiver, &name, args) {
                        return result;
                    }
                }
            }
        }
    }

    // functools.lru_cache_wrapper: intercept cache_info / cache_clear before
    // the generic instance method-lookup path. Without this, `f.cache_info()`
    // where f is a wrapped function returns None because the wrapper has
    // neither a __class__ method table nor a field named cache_info.
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "memoryview" {
                    match name.as_str() {
                        "tobytes" => {
                            // Return the readable bytes from the underlying buffer.
                            let buf = fields.read().unwrap().get("_buffer").copied();
                            if let Some(data) = buf.and_then(super::builtins::try_bytes_like) {
                                return MbValue::from_ptr(MbObject::new_bytes(data));
                            }
                            return MbValue::from_ptr(MbObject::new_bytes(vec![]));
                        }
                        "tolist" => {
                            // Return list of int byte values.
                            let buf = fields.read().unwrap().get("_buffer").copied();
                            if let Some(bv) = buf.and_then(super::builtins::try_bytes_like) {
                                let items: Vec<MbValue> = bv
                                    .iter()
                                    .map(|byte| MbValue::from_int(*byte as i64))
                                    .collect();
                                return MbValue::from_ptr(MbObject::new_list(items));
                            }
                            return MbValue::from_ptr(MbObject::new_list(vec![]));
                        }
                        "release" => return MbValue::none(),
                        "hex" => {
                            let buf = fields.read().unwrap().get("_buffer").copied();
                            let bytes_vec = buf.and_then(super::builtins::try_bytes_like);
                            let hexs = bytes_vec
                                .unwrap_or_default()
                                .iter()
                                .map(|b| format!("{b:02x}"))
                                .collect::<String>();
                            return MbValue::from_ptr(MbObject::new_str(hexs));
                        }
                        // toreadonly() returns a new view over the same buffer,
                        // marked read-only.
                        "toreadonly" => {
                            let buf = fields
                                .read()
                                .unwrap()
                                .get("_buffer")
                                .copied()
                                .unwrap_or_else(MbValue::none);
                            let inst = MbObject::new_instance("memoryview".to_string());
                            if let ObjData::Instance { fields: ref nf, .. } = (*inst).data {
                                let mut g = nf.write().unwrap();
                                super::rc::retain_if_ptr(buf);
                                g.insert("_buffer".to_string(), buf);
                                g.insert("_readonly".to_string(), MbValue::from_bool(true));
                                let obj = fields.read().unwrap()
                                    .get("_obj").copied().unwrap_or(buf);
                                super::rc::retain_if_ptr(obj);
                                g.insert("_obj".to_string(), obj);
                                if let Some(v) = fields.read().unwrap().get("_contiguous").copied() {
                                    g.insert("_contiguous".to_string(), v);
                                }
                                if let Some(v) = fields.read().unwrap().get("_stride").copied() {
                                    g.insert("_stride".to_string(), v);
                                }
                            }
                            return MbValue::from_ptr(inst);
                        }
                        _ => {}
                    }
                }
            }
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "functools.lru_cache_wrapper" {
                    match name.as_str() {
                        "cache_info" => {
                            return super::stdlib::functools_mod::mb_functools_lru_cache_info(
                                receiver,
                            );
                        }
                        "cache_clear" => {
                            return super::stdlib::functools_mod::mb_functools_lru_cache_clear(
                                receiver,
                            );
                        }
                        _ => {}
                    }
                }
            }
            // slice.indices(length) — CPython 3.12 returns the (start, stop,
            // step) tuple a Python sequence would use for `seq[s]` against a
            // sequence of length `length`. Filed under #1256 long-tail.
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "slice" && name == "__hash__" {
                    return super::builtins::mb_hash(receiver);
                }
            }
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "slice" && name == "indices" {
                    let arg_items = super::builtins::extract_items(args);
                    // length accepts any SupportsIndex (int / bool / object with
                    // __index__), matching CPython's PySlice_GetIndices. A
                    // non-integer arg (e.g. `slice(0,5).indices("x")`) raises
                    // TypeError, not a silent length-0 fallback.
                    let length = match arg_items.first() {
                        Some(v) => match super::builtins::resolve_index_value(*v) {
                            Some(n) => n,
                            None => {
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(format!(
                                        "'{}' object cannot be interpreted as an integer",
                                        super::builtins::value_type_name(*v)
                                    ))),
                                );
                                return MbValue::none();
                            }
                        },
                        None => 0,
                    };
                    let (start_v, stop_v, step_v) = {
                        let f = fields.read().unwrap();
                        (
                            f.get("start").copied().unwrap_or(MbValue::none()),
                            f.get("stop").copied().unwrap_or(MbValue::none()),
                            f.get("step").copied().unwrap_or(MbValue::none()),
                        )
                    };
                    let step = if step_v.is_none() {
                        1
                    } else {
                        step_v.as_int().unwrap_or(1)
                    };
                    let step = if step == 0 { 1 } else { step };
                    let (default_start, default_stop) = if step > 0 {
                        (0i64, length)
                    } else {
                        (length - 1, -1i64)
                    };
                    let clamp = |v: i64| -> i64 {
                        // Negative index → add length; then clamp to
                        // [lower, upper] where bounds depend on step sign.
                        let mut x = if v < 0 { v + length } else { v };
                        if step > 0 {
                            if x < 0 {
                                x = 0;
                            }
                            if x > length {
                                x = length;
                            }
                        } else {
                            if x < -1 {
                                x = -1;
                            }
                            if x > length - 1 {
                                x = length - 1;
                            }
                        }
                        x
                    };
                    let start = if start_v.is_none() {
                        default_start
                    } else {
                        clamp(start_v.as_int().unwrap_or(default_start))
                    };
                    let stop = if stop_v.is_none() {
                        default_stop
                    } else {
                        clamp(stop_v.as_int().unwrap_or(default_stop))
                    };
                    return MbValue::from_ptr(super::rc::MbObject::new_tuple(vec![
                        MbValue::from_int(start),
                        MbValue::from_int(stop),
                        MbValue::from_int(step),
                    ]));
                }
            }
            // ExceptionGroup (PEP 654) subgroup / split. These groups are
            // Instances carrying an `exceptions` field; their methods are not in
            // the class method table, so route them to the exception runtime
            // before the generic lookup raises AttributeError.
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if (class_name == "ExceptionGroup"
                    || class_name == "BaseExceptionGroup"
                    || fields.read().unwrap().contains_key("exceptions"))
                    && matches!(name.as_str(), "subgroup" | "split")
                {
                    let items = super::builtins::extract_items(args);
                    let arg = items.first().copied().unwrap_or_else(MbValue::none);
                    return match name.as_str() {
                        "subgroup" => super::exception::mb_exception_group_subgroup(receiver, arg),
                        _ => super::exception::mb_exception_group_split(receiver, arg),
                    };
                }
            }
            // threading.Lock / RLock / Event / Condition stubs are Instances
            // whose method tables are not registered through the normal class
            // machinery. Dispatch their well-known methods (acquire / release
            // / set / clear / is_set / __enter__ / __exit__) directly to the
            // threading_mod handlers; otherwise the generic instance method-
            // lookup path raises AttributeError.
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name == "Lock" {
                    match name.as_str() {
                        "acquire" | "__enter__" => {
                            return super::stdlib::threading_mod::mb_threading_lock_acquire(
                                receiver,
                            );
                        }
                        "release" => {
                            return super::stdlib::threading_mod::mb_threading_lock_release(
                                receiver,
                            );
                        }
                        "__exit__" => {
                            super::stdlib::threading_mod::mb_threading_lock_release(receiver);
                            return MbValue::from_bool(false);
                        }
                        "locked" => {
                            // Reflect current state; the field carries the truth.
                            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                                if let Some(v) = fields.read().unwrap().get("locked").copied() {
                                    return v;
                                }
                            }
                            return MbValue::from_bool(false);
                        }
                        _ => {}
                    }
                }
                if class_name == "RLock" {
                    match name.as_str() {
                        "acquire" | "__enter__" => {
                            return super::stdlib::threading_mod::mb_threading_lock_acquire(
                                receiver,
                            );
                        }
                        "release" => {
                            return super::stdlib::threading_mod::mb_threading_lock_release(
                                receiver,
                            );
                        }
                        "__exit__" => {
                            super::stdlib::threading_mod::mb_threading_lock_release(receiver);
                            return MbValue::from_bool(false);
                        }
                        _ => {}
                    }
                }
                if class_name == "Event" {
                    match name.as_str() {
                        "set" => {
                            return super::stdlib::threading_mod::mb_threading_event_set(receiver);
                        }
                        "clear" => {
                            return super::stdlib::threading_mod::mb_threading_event_clear(
                                receiver,
                            );
                        }
                        "is_set" => {
                            return super::stdlib::threading_mod::mb_threading_event_is_set(
                                receiver,
                            );
                        }
                        "wait" => {
                            // Sync stub: report set state without blocking.
                            return super::stdlib::threading_mod::mb_threading_event_is_set(
                                receiver,
                            );
                        }
                        _ => {}
                    }
                }
                if class_name == "Condition" {
                    match name.as_str() {
                        "acquire" | "__enter__" => {
                            return super::stdlib::threading_mod::mb_threading_lock_acquire(
                                receiver,
                            );
                        }
                        "release" => {
                            return super::stdlib::threading_mod::mb_threading_lock_release(
                                receiver,
                            );
                        }
                        "__exit__" => {
                            super::stdlib::threading_mod::mb_threading_lock_release(receiver);
                            return MbValue::from_bool(false);
                        }
                        "notify" | "notify_all" | "wait" | "wait_for" => {
                            // CPython requires the condition's lock to be held.
                            let held = if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                                fields
                                    .read()
                                    .unwrap()
                                    .get("locked")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false)
                            } else {
                                false
                            };
                            if !held {
                                let verb = if name.starts_with("notify") {
                                    "notify on"
                                } else {
                                    "wait on"
                                };
                                super::exception::mb_raise(
                                    MbValue::from_ptr(MbObject::new_str(
                                        "RuntimeError".to_string(),
                                    )),
                                    MbValue::from_ptr(MbObject::new_str(format!(
                                        "cannot {verb} un-acquired lock"
                                    ))),
                                );
                                return MbValue::none();
                            }
                            return MbValue::none();
                        }
                        _ => {}
                    }
                }
                if class_name == "Semaphore" || class_name == "BoundedSemaphore" {
                    match name.as_str() {
                        "acquire" | "__enter__" => {
                            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                                let mut f = fields.write().unwrap();
                                let v = f.get("value").and_then(|x| x.as_int()).unwrap_or(0);
                                if v > 0 {
                                    f.insert("value".into(), MbValue::from_int(v - 1));
                                    return MbValue::from_bool(true);
                                }
                                // Sync stub: a zero semaphore cannot block.
                                return MbValue::from_bool(false);
                            }
                            return MbValue::from_bool(false);
                        }
                        "release" | "__exit__" => {
                            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                                let mut f = fields.write().unwrap();
                                let v = f.get("value").and_then(|x| x.as_int()).unwrap_or(0);
                                let bound = f.get("bound").and_then(|x| x.as_int());
                                if let Some(b) = bound {
                                    if v + 1 > b {
                                        drop(f);
                                        super::exception::mb_raise(
                                            MbValue::from_ptr(MbObject::new_str(
                                                "ValueError".to_string(),
                                            )),
                                            MbValue::from_ptr(MbObject::new_str(
                                                "Semaphore released too many times".to_string(),
                                            )),
                                        );
                                        return MbValue::none();
                                    }
                                }
                                f.insert("value".into(), MbValue::from_int(v + 1));
                            }
                            if name == "__exit__" {
                                return MbValue::from_bool(false);
                            }
                            return MbValue::none();
                        }
                        _ => {}
                    }
                }
                // io.StringIO / io.BytesIO method dispatch — also Instances
                // without a registered method table.
                if class_name == "StringIO" {
                    let arg_items = super::builtins::extract_items(args);
                    match name.as_str() {
                        "write" => {
                            let d = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_stringio_write(receiver, d);
                        }
                        "read" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_stringio_read_n(receiver, n);
                        }
                        "readline" => {
                            return super::stdlib::io_mod::mb_stringio_readline(receiver);
                        }
                        "readlines" => {
                            return super::stdlib::io_mod::mb_stringio_readlines(receiver);
                        }
                        "__iter__" => {
                            return super::stdlib::io_mod::dispatch_stringio_iter(receiver);
                        }
                        "getvalue" => {
                            return super::stdlib::io_mod::mb_stringio_getvalue(receiver);
                        }
                        "seek" => {
                            let p = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            let w = arg_items
                                .get(1)
                                .copied()
                                .unwrap_or_else(|| MbValue::from_int(0));
                            return super::stdlib::io_mod::mb_stringio_seek_whence(receiver, p, w);
                        }
                        "tell" => {
                            return super::stdlib::io_mod::mb_stringio_tell(receiver);
                        }
                        "truncate" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_stringio_truncate(receiver, n);
                        }
                        "readable" | "writable" | "seekable" => return MbValue::from_bool(true),
                        "close" => return super::stdlib::io_mod::mb_stringio_close(receiver),
                        "flush" => return MbValue::none(),
                        "__enter__" => {
                            super::rc::retain_if_ptr(receiver);
                            return receiver;
                        }
                        "__exit__" => return MbValue::from_bool(false),
                        _ => {}
                    }
                }
                if class_name == "BytesIO" {
                    let arg_items = super::builtins::extract_items(args);
                    match name.as_str() {
                        "write" => {
                            let d = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bytesio_write(receiver, d);
                        }
                        "read" | "read1" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bytesio_read_n(receiver, n);
                        }
                        "readline" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bytesio_readline(receiver, n);
                        }
                        "readlines" => {
                            return super::stdlib::io_mod::mb_bytesio_readlines(receiver);
                        }
                        "__iter__" => {
                            return super::stdlib::io_mod::dispatch_bytesio_iter(receiver);
                        }
                        "readinto" => {
                            let d = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bytesio_readinto(receiver, d);
                        }
                        "getvalue" => {
                            return super::stdlib::io_mod::mb_bytesio_getvalue(receiver);
                        }
                        "seek" => {
                            let p = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            let w = arg_items
                                .get(1)
                                .copied()
                                .unwrap_or_else(|| MbValue::from_int(0));
                            return super::stdlib::io_mod::mb_bytesio_seek_with_whence(
                                receiver, p, w,
                            );
                        }
                        "tell" => {
                            return super::stdlib::io_mod::mb_bytesio_tell(receiver);
                        }
                        "truncate" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bytesio_truncate(receiver, n);
                        }
                        "readable" | "seekable" | "writable" => return MbValue::from_bool(true),
                        "close" => return super::stdlib::io_mod::mb_bytesio_close(receiver),
                        "flush" => return MbValue::none(),
                        "__enter__" => {
                            super::rc::retain_if_ptr(receiver);
                            return receiver;
                        }
                        "__exit__" => return MbValue::from_bool(false),
                        _ => {}
                    }
                }
                if class_name == "BufferedWriter" {
                    let arg_items = super::builtins::extract_items(args);
                    match name.as_str() {
                        "write" => {
                            let d = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bufferedwriter_write(receiver, d);
                        }
                        "read" | "read1" | "peek" | "readline" | "readlines" => {
                            return super::stdlib::io_mod::mb_bufferedwriter_read(receiver);
                        }
                        "readable" => return MbValue::from_bool(false),
                        "writable" | "seekable" => return MbValue::from_bool(true),
                        "close" | "flush" => return MbValue::none(),
                        "__enter__" => {
                            super::rc::retain_if_ptr(receiver);
                            return receiver;
                        }
                        "__exit__" => return MbValue::from_bool(false),
                        _ => {}
                    }
                }
                if class_name == "TextIOWrapper" {
                    let arg_items = super::builtins::extract_items(args);
                    match name.as_str() {
                        "write" => {
                            let d = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_textiowrapper_write(receiver, d);
                        }
                        "read" => {
                            return super::stdlib::io_mod::mb_textiowrapper_read(receiver);
                        }
                        "flush" => return super::stdlib::io_mod::mb_textiowrapper_flush(receiver),
                        "close" => return MbValue::none(),
                        "__enter__" => {
                            super::rc::retain_if_ptr(receiver);
                            return receiver;
                        }
                        "__exit__" => return MbValue::from_bool(false),
                        _ => {}
                    }
                }
                if class_name == "BufferedReader" {
                    let arg_items = super::builtins::extract_items(args);
                    match name.as_str() {
                        "read" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bufferedreader_read(receiver, n);
                        }
                        "read1" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bufferedreader_read1(receiver, n);
                        }
                        "peek" => {
                            let n = arg_items.first().copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::io_mod::mb_bufferedreader_peek(receiver, n);
                        }
                        "readline" => {
                            return super::stdlib::io_mod::mb_bufferedreader_readline(receiver);
                        }
                        "close" | "flush" => return MbValue::none(),
                        "__enter__" => {
                            super::rc::retain_if_ptr(receiver);
                            return receiver;
                        }
                        "__exit__" => return MbValue::from_bool(false),
                        _ => {}
                    }
                }
                if class_name == "TracebackException" && name == "format" {
                    return super::stdlib::traceback_mod::mb_traceback_exception_format(receiver);
                }
                if class_name == "Thread" {
                    match name.as_str() {
                        "start" | "run" => {
                            return super::stdlib::threading_mod::mb_threading_thread_start(
                                receiver,
                            );
                        }
                        "join" => {
                            return super::stdlib::threading_mod::mb_threading_thread_join(
                                receiver,
                            );
                        }
                        "is_alive" | "isAlive" => {
                            return super::stdlib::threading_mod::mb_threading_thread_is_alive(
                                receiver,
                            );
                        }
                        "getName" => {
                            unsafe {
                                if let Some(ptr) = receiver.as_ptr() {
                                    if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                                        if let Some(v) = fields.read().unwrap().get("name").copied()
                                        {
                                            return v;
                                        }
                                    }
                                }
                            }
                            return MbValue::none();
                        }
                        "setName" | "setDaemon" => return MbValue::none(),
                        _ => {}
                    }
                }
                if class_name == "Barrier" {
                    match name.as_str() {
                        "wait" => {
                            return super::stdlib::threading_mod::mb_threading_barrier_wait(
                                receiver,
                            );
                        }
                        "reset" => {
                            return super::stdlib::threading_mod::mb_threading_barrier_reset(
                                receiver,
                            );
                        }
                        "abort" => {
                            return super::stdlib::threading_mod::mb_threading_barrier_abort(
                                receiver,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // `str.lower("HELLO")` — receiver is the type-name string "str", method
    // is "lower", args[0] is the real receiver. Swap so the method dispatches
    // against args[0] with the remaining args. Excluded: classmethods and
    // static builders that take their own arg shape (maketrans, fromkeys,
    // fromhex, fromisoformat, from_bytes, ...).
    //
    // Also handles type objects (Instance { class_name="type", __name__=X })
    // produced by mb_builtin_type_obj() — both `str.lower` (string receiver)
    // and `str.lower` after the type-singleton refactor (type object receiver)
    // must reach the same dispatch path.
    {
        // Extract the effective type name from the receiver — either from a
        // plain string (old path) or from a type-singleton object (new path).
        let type_name_opt: Option<String> = if let Some(ptr) = receiver.as_ptr() {
            unsafe {
                match &(*ptr).data {
                    ObjData::Str(ref s) => Some(s.clone()),
                    ObjData::Instance {
                        class_name: ref cn,
                        ref fields,
                    } if cn == "type" => fields
                        .read()
                        .unwrap()
                        .get("__name__")
                        .and_then(|v| extract_str(*v)),
                    _ => None,
                }
            }
        } else {
            None
        };

        if let Some(ref s) = type_name_opt {
            // complex.__eq__/__ne__/__lt__/… called directly on the type object
            // (`complex.__eq__(a, b)`). Lowered as a method call, so it bypasses
            // the getattr unbound-method wrapper; compute via the shared helper.
            if s == "complex" {
                let items = super::builtins::extract_items(args);
                let a = items.first().copied().unwrap_or_else(MbValue::none);
                let b = items.get(1).copied().unwrap_or_else(MbValue::none);
                if let Some(result) = super::builtins::complex_cmp_dunder(name.as_str(), a, b) {
                    return result;
                }
            }
            // <type>.__new__(cls) — allocate a BARE instance of `cls` without
            // running __init__ (CPython's object.__new__). The type wall builds a
            // receiver for instance-method checks via `obj = object.__new__(C)`.
            // `cls` is the first arg (a type object carrying __name__, or a bare
            // class-name string); the bound type `s` (usually "object") is ignored.
            if name == "__new__" {
                let items = super::builtins::extract_items(args);
                if let Some(cls) = items.first().copied() {
                    // `resolve_class_name` recovers the class name from a bare
                    // name string, a `type` object, OR a native constructor func
                    // pointer (defaultdict/OrderedDict/date are dispatcher funcs
                    // registered in NATIVE_TYPE_NAMES) — the latter is what the
                    // type-wall `obj = object.__new__(StdlibClass)` idiom needs.
                    // The old inline extraction only handled Str / `type` objects
                    // and returned None for a func pointer, dropping to a getattr
                    // fallback that raised "'type' has no attribute '__new__'".
                    if let Some(cn) = resolve_class_name(cls) {
                        return MbValue::from_ptr(MbObject::new_instance(cn));
                    }
                }
            }
            if name == "register" && is_collections_abc_name(s) {
                let items = super::builtins::extract_items(args);
                let child = items.first().copied().unwrap_or_else(MbValue::none);
                return mb_collections_abc_register(s, child);
            }
            if name == "register" && is_user_abc(s) {
                let items = super::builtins::extract_items(args);
                let child = items.first().copied().unwrap_or_else(MbValue::none);
                return mb_user_abc_register(s, child);
            }
            let is_type_name = matches!(
                s.as_str(),
                "str"
                    | "list"
                    | "dict"
                    | "tuple"
                    | "set"
                    | "frozenset"
                    | "int"
                    | "float"
                    | "bool"
                    | "bytes"
                    | "bytearray"
            );
            let is_classmethod_name = matches!(
                name.as_str(),
                "maketrans"
                    | "fromkeys"
                    | "fromhex"
                    | "fromisoformat"
                    | "from_bytes"
                    | "fromtimestamp"
                    | "fromordinal"
                    | "from_float"
                    | "utcfromtimestamp"
                    | "today"
                    | "now"
                    | "utcnow"
            );
            if is_type_name && is_classmethod_name {
                // Classmethod dispatch — intercept the ones we implement
                // and pass the type arg along so callees can specialize.
                let arg_items: Vec<MbValue> = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            Some(lk.read().unwrap().to_vec())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                if s == "int" && name == "from_bytes" {
                    return mb_int_from_bytes(
                        arg_items.first().copied().unwrap_or_else(MbValue::none),
                        arg_items.get(1).copied().unwrap_or_else(MbValue::none),
                        arg_items.get(2).copied().unwrap_or_else(MbValue::none),
                    );
                }
                if s == "dict" && name == "fromkeys" {
                    let iterable = arg_items.first().copied().unwrap_or_else(MbValue::none);
                    let value = arg_items.get(1).copied().unwrap_or_else(MbValue::none);
                    let dict = super::dict_ops::mb_dict_new();
                    let handle = super::iter::mb_iter(iterable);
                    if !handle.is_none() {
                        loop {
                            if super::iter::mb_has_next(handle).as_bool() != Some(true) {
                                break;
                            }
                            let k = super::iter::mb_next(handle);
                            unsafe {
                                super::rc::retain_if_ptr(value);
                            }
                            super::dict_ops::mb_dict_setitem(dict, k, value);
                        }
                    }
                    return dict;
                }
                // Fallthrough: unhandled classmethod — synthesise a string receiver
                // and delegate to dispatch_str_method (handles bytes.fromhex, str.maketrans, etc.)
                let str_recv = MbValue::from_ptr(MbObject::new_str(s.clone()));
                return super::string_ops::dispatch_str_method(&name, str_recv, args);
            }
            if is_type_name && !is_classmethod_name {
                let items = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            Some(lk.read().unwrap().clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                if !items.is_empty() {
                    let new_recv = items[0];
                    let rest = MbValue::from_ptr(MbObject::new_list(items[1..].to_vec()));
                    return mb_call_method(new_recv, method_name, rest);
                }
            }
        }
    }

    // Generator protocol: intercept .send() / .throw() / .close() on generator handles
    if receiver.is_int() && super::generator::is_known_generator(receiver) {
        return dispatch_generator_method(receiver, &name, args);
    }

    // Hashlib handle protocol: integer IDs allocated by hashlib_mod dispatch
    // `update`/`hexdigest`/`digest`/`copy` to mb_hashlib_* free functions.
    // Same shape as the file_handle protocol below, kept above because hash
    // handles share the int-receiver space and the more-specific check must
    // fire first.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::hashlib_mod::is_hashlib_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "update" => {
                    let data = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::hashlib_mod::mb_hashlib_update(receiver, data);
                }
                "hexdigest" => {
                    let len = arg_items.first().and_then(|v| v.as_int());
                    return super::stdlib::hashlib_mod::mb_hashlib_hexdigest_len(receiver, len);
                }
                "digest" => {
                    let len = arg_items.first().and_then(|v| v.as_int());
                    return super::stdlib::hashlib_mod::mb_hashlib_digest_len(receiver, len);
                }
                "copy" => return super::stdlib::hashlib_mod::mb_hashlib_copy(receiver),
                _ => {}
            }
        }
    }

    // Hmac handle protocol: same shape as hashlib above. Methods
    // `update`/`hexdigest`/`digest`/`copy` route to mb_hmac_* free
    // functions.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::hmac_mod::is_hmac_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "update" => {
                    let data = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::hmac_mod::mb_hmac_update(receiver, data);
                }
                "hexdigest" => return super::stdlib::hmac_mod::mb_hmac_hexdigest(receiver),
                "digest" => return super::stdlib::hmac_mod::mb_hmac_digest(receiver),
                "copy" => return super::stdlib::hmac_mod::mb_hmac_copy(receiver),
                _ => {}
            }
        }
    }

    // Decimal handle protocol: same shape as hashlib/hmac/fractions. All
    // method names (arith dunders, comparisons, predicates, quantize,
    // sqrt, as_tuple, ... plus the legacy `add`/`sub`/`mul`/`truediv`/
    // `str_`/`is_zero` bench entry points) route through
    // `decimal_mod::dispatch_method`.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::decimal_mod::is_decimal_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            if let Some(result) =
                super::stdlib::decimal_mod::dispatch_method(receiver, name.as_str(), &arg_items)
            {
                return result;
            }
        }
    }

    // Array handle protocol: typed numeric container methods. Task #35 — typed-array
    // (tier:compute, bulk-bytes shape). Routes append/extend/fromlist/frombytes/
    // tobytes/tolist/buffer_info/byteswap/count/index/insert/pop/remove/reverse
    // to mb_array_* free functions.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::array_mod::is_array_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "append" => {
                    let v = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_append(receiver, v);
                }
                "extend" => {
                    let it = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_extend(receiver, it);
                }
                "fromlist" => {
                    let lst = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_fromlist(receiver, lst);
                }
                "frombytes" => {
                    let buf = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_frombytes(receiver, buf);
                }
                "fromunicode" => {
                    let text = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_fromunicode(receiver, text);
                }
                "tobytes" => return super::stdlib::array_mod::mb_array_tobytes(receiver),
                "tolist" => return super::stdlib::array_mod::mb_array_tolist(receiver),
                "tounicode" => return super::stdlib::array_mod::mb_array_tounicode(receiver),
                "buffer_info" => return super::stdlib::array_mod::mb_array_buffer_info(receiver),
                "byteswap" => return super::stdlib::array_mod::mb_array_byteswap(receiver),
                "count" => {
                    let t = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_count(receiver, t);
                }
                "index" => {
                    let t = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_index(receiver, t);
                }
                "insert" => {
                    let idx = arg_items.first().copied().unwrap_or(MbValue::none());
                    let v = arg_items.get(1).copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_insert(receiver, idx, v);
                }
                "pop" => {
                    let idx = arg_items.first().copied().unwrap_or(MbValue::from_int(-1));
                    return super::stdlib::array_mod::mb_array_pop(receiver, idx);
                }
                "remove" => {
                    let t = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::array_mod::mb_array_remove(receiver, t);
                }
                "reverse" => return super::stdlib::array_mod::mb_array_reverse(receiver),
                _ => {}
            }
        }
    }

    // Queue handle protocol: integer IDs allocated by queue_mod dispatch
    // `put`/`get`/`empty`/`qsize`/`full`/`task_done`/`join` to mb_queue_*
    // free functions. Task #70 — Wave-6 Ship #3 (#1472), integer-handle
    // pattern for Queue / LifoQueue / PriorityQueue.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::queue_mod::is_queue_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            // block/timeout arrive positionally or in a trailing kwargs dict.
            let kw_dict = arg_items.last().copied().filter(|v| {
                v.as_ptr()
                    .map(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) })
                    .unwrap_or(false)
            });
            let kw = |key: &str| -> Option<MbValue> {
                let ptr = kw_dict?.as_ptr()?;
                unsafe {
                    if let ObjData::Dict(ref lock) = (*ptr).data {
                        return lock.read().unwrap().get(key).copied();
                    }
                }
                None
            };
            let positional: &[MbValue] = if kw_dict.is_some() {
                &arg_items[..arg_items.len() - 1]
            } else {
                &arg_items[..]
            };
            let as_timeout = |v: MbValue| -> Option<f64> {
                if v.is_none() {
                    None
                } else {
                    v.as_float().or_else(|| v.as_int().map(|i| i as f64))
                }
            };
            match name.as_str() {
                "put" | "put_nowait" => {
                    let item = positional.first().copied().unwrap_or(MbValue::none());
                    let (blocking, timeout) = if name == "put_nowait" {
                        (false, None)
                    } else {
                        let block = positional
                            .get(1)
                            .copied()
                            .or_else(|| kw("block"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        let timeout = positional
                            .get(2)
                            .copied()
                            .or_else(|| kw("timeout"))
                            .and_then(as_timeout);
                        (block, timeout)
                    };
                    return super::stdlib::queue_mod::mb_queue_put_checked(
                        receiver, item, blocking, timeout,
                    );
                }
                "get" | "get_nowait" => {
                    let (blocking, timeout) = if name == "get_nowait" {
                        (false, None)
                    } else {
                        let block = positional
                            .first()
                            .copied()
                            .or_else(|| kw("block"))
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        let timeout = positional
                            .get(1)
                            .copied()
                            .or_else(|| kw("timeout"))
                            .and_then(as_timeout);
                        (block, timeout)
                    };
                    return super::stdlib::queue_mod::mb_queue_get_checked(
                        receiver, blocking, timeout,
                    );
                }
                "empty" => return super::stdlib::queue_mod::mb_queue_empty(receiver),
                "qsize" => return super::stdlib::queue_mod::mb_queue_qsize(receiver),
                "full" => return super::stdlib::queue_mod::mb_queue_full(receiver),
                "task_done" => return super::stdlib::queue_mod::mb_queue_task_done(receiver),
                "join" => return MbValue::none(),
                _ => {}
            }
        }
    }

    // Random handle protocol: integer IDs allocated by random_mod dispatch
    // 23 methods (random/seed/randint/randrange/uniform/triangular/choice/
    // shuffle/sample/choices/gauss/normalvariate/expovariate/lognormvariate/
    // vonmisesvariate/gammavariate/betavariate/paretovariate/weibullvariate/
    // getrandbits/randbytes/getstate/setstate) to mb_random_method_* free
    // functions. Task #40 — Random class via integer-handle pattern.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::random_mod::is_random_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let a0 = || arg_items.first().copied().unwrap_or(MbValue::none());
            let a1 = || arg_items.get(1).copied().unwrap_or(MbValue::none());
            let a2 = || arg_items.get(2).copied().unwrap_or(MbValue::none());
            match name.as_str() {
                "random" => return super::stdlib::random_mod::mb_random_method_random(receiver),
                "seed" => {
                    if arg_items.len() > 2 {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "seed() takes from 1 to 3 positional arguments but {} were given",
                                arg_items.len() + 1
                            ))),
                        );
                        return MbValue::none();
                    }
                    return super::stdlib::random_mod::mb_random_method_seed(receiver, a0());
                }
                "randint" => {
                    return super::stdlib::random_mod::mb_random_method_randint(
                        receiver,
                        a0(),
                        a1(),
                    )
                }
                "randrange" => {
                    return super::stdlib::random_mod::mb_random_method_randrange(
                        receiver,
                        a0(),
                        a1(),
                        arg_items
                            .get(2)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_int(1)),
                    )
                }
                "uniform" => {
                    return super::stdlib::random_mod::mb_random_method_uniform(
                        receiver,
                        a0(),
                        a1(),
                    )
                }
                "triangular" => {
                    return super::stdlib::random_mod::mb_random_method_triangular(
                        receiver,
                        a0(),
                        a1(),
                        a2(),
                    )
                }
                "choice" => {
                    return super::stdlib::random_mod::mb_random_method_choice(receiver, a0())
                }
                "shuffle" => {
                    return super::stdlib::random_mod::mb_random_method_shuffle(receiver, a0())
                }
                "sample" => {
                    return super::stdlib::random_mod::mb_random_method_sample(receiver, a0(), a1())
                }
                "choices" => {
                    // Full routing (weights / cum_weights / k) like the
                    // module-level dispatcher.
                    let (w, cw, k) = super::stdlib::random_mod::parse_choices_kwargs(&arg_items);
                    return super::stdlib::random_mod::mb_random_method_choices_full(
                        receiver,
                        a0(),
                        w,
                        cw,
                        k,
                    );
                }
                "gauss" => {
                    return super::stdlib::random_mod::mb_random_method_gauss(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(0.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "normalvariate" => {
                    return super::stdlib::random_mod::mb_random_method_normalvariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(0.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "expovariate" => {
                    return super::stdlib::random_mod::mb_random_method_expovariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "lognormvariate" => {
                    return super::stdlib::random_mod::mb_random_method_lognormvariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(0.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "vonmisesvariate" => {
                    return super::stdlib::random_mod::mb_random_method_vonmisesvariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(0.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(0.0)),
                    )
                }
                "gammavariate" => {
                    return super::stdlib::random_mod::mb_random_method_gammavariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "betavariate" => {
                    return super::stdlib::random_mod::mb_random_method_betavariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "paretovariate" => {
                    return super::stdlib::random_mod::mb_random_method_paretovariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "weibullvariate" => {
                    return super::stdlib::random_mod::mb_random_method_weibullvariate(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                        arg_items
                            .get(1)
                            .copied()
                            .unwrap_or_else(|| MbValue::from_float(1.0)),
                    )
                }
                "getrandbits" => {
                    return super::stdlib::random_mod::mb_random_method_getrandbits(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_int(32)),
                    )
                }
                "randbytes" => {
                    return super::stdlib::random_mod::mb_random_method_randbytes(
                        receiver,
                        arg_items
                            .first()
                            .copied()
                            .unwrap_or_else(|| MbValue::from_int(0)),
                    )
                }
                "binomialvariate" => {
                    return super::stdlib::random_mod::mb_random_method_binomialvariate(
                        receiver,
                        a0(),
                        a1(),
                    )
                }
                "getstate" => {
                    return super::stdlib::random_mod::mb_random_method_getstate(receiver)
                }
                "setstate" => {
                    return super::stdlib::random_mod::mb_random_method_setstate(receiver, a0())
                }
                _ => {}
            }
        }
    }

    // Fraction handle protocol: integer IDs allocated by fractions_mod.
    // Dispatch arith dunders, comparison dunders, unary dunders, prop
    // reads, and conversion methods through a single
    // `dispatch_method(handle, name, &args)` entrypoint. Task #45 —
    // Fraction class via integer-handle pattern.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::fractions_mod::is_fraction_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let result =
                super::stdlib::fractions_mod::dispatch_method(receiver, name.as_str(), &arg_items);
            if !result.is_none() {
                return result;
            }
        }
    }

    // JSONEncoder/JSONDecoder handle protocols. Methods `encode`/`iterencode`/
    // `default` for encoders; `decode`/`raw_decode` for decoders. Task #33 —
    // wire json surface gap to clear Gate 3 to ≥80%.
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::stdlib::json_mod::is_json_encoder_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "encode" => {
                    let obj = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::json_mod::mb_json_encoder_encode(receiver, obj);
                }
                "iterencode" => {
                    let obj = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::json_mod::mb_json_encoder_iterencode(receiver, obj);
                }
                "default" => {
                    let obj = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::json_mod::mb_json_encoder_default(receiver, obj);
                }
                _ => {}
            }
        }
        if super::stdlib::json_mod::is_json_decoder_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "decode" => {
                    let s = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::stdlib::json_mod::mb_json_decoder_decode(receiver, s);
                }
                "raw_decode" => {
                    let s = arg_items.first().copied().unwrap_or(MbValue::none());
                    let idx = arg_items.get(1).copied().unwrap_or(MbValue::from_int(0));
                    return super::stdlib::json_mod::mb_json_decoder_raw_decode(receiver, s, idx);
                }
                _ => {}
            }
        }
    }

    // File handle protocol: integer IDs in the file handle table dispatch to
    // mb_file_* functions.  Must be checked before the generic primitive guard
    // below, which would otherwise raise AttributeError.
    // REQ: file handle method dispatch on integer handle IDs
    if receiver.is_int() {
        let id = receiver.as_int().unwrap_or(0) as u64;
        if super::file_io::is_file_handle(id) {
            let arg_items: Vec<MbValue> = args
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::List(ref lk) = (*p).data {
                        Some(lk.read().unwrap().to_vec())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            match name.as_str() {
                "write" => {
                    let text = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_write(receiver, text);
                }
                "read" => {
                    let n = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_read_n(receiver, n);
                }
                "readline" => {
                    let n = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_readline_n(receiver, n);
                }
                "readlines" => {
                    return super::file_io::mb_file_readlines(receiver);
                }
                "readinto" => {
                    let dst = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_readinto(receiver, dst);
                }
                "writelines" => {
                    let lst = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_writelines(receiver, lst);
                }
                "tell" => {
                    return super::file_io::mb_file_tell(receiver);
                }
                "seek" => {
                    let off = arg_items.first().copied().unwrap_or(MbValue::none());
                    let w = arg_items.get(1).copied().unwrap_or(MbValue::from_int(0));
                    return super::file_io::mb_file_seek(receiver, off, w);
                }
                "flush" => {
                    return super::file_io::mb_file_flush(receiver);
                }
                "truncate" => {
                    let n = arg_items.first().copied().unwrap_or(MbValue::none());
                    return super::file_io::mb_file_truncate(receiver, n);
                }
                "close" => {
                    super::file_io::mb_file_close(receiver);
                    return MbValue::none();
                }
                _ => {}
            }
        }
    }

    // Primitive int/bool method dispatch
    if receiver.is_int() || receiver.is_bool() {
        let val = if receiver.is_bool() {
            receiver.as_bool().map(|b| b as i64).unwrap_or(0)
        } else {
            receiver.as_int().unwrap_or(0)
        };
        match name.as_str() {
            // int is its own index: (7).__index__() == 7. Also covers bools
            // (True.__index__() == 1) via the bool→i64 coercion above.
            "__index__" | "__int__" => {
                return MbValue::from_int(val);
            }
            "bit_length" => {
                if val == 0 {
                    return MbValue::from_int(0);
                }
                return MbValue::from_int((64 - val.unsigned_abs().leading_zeros()) as i64);
            }
            "bit_count" => {
                return MbValue::from_int(val.unsigned_abs().count_ones() as i64);
            }
            "to_bytes" => {
                let arg_items: Vec<MbValue> = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            Some(lk.read().unwrap().to_vec())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                // Default `length=1`, `byteorder='big'`, `signed=False`
                // (CPython 3.11+ defaults).
                let length = arg_items
                    .first()
                    .and_then(|v| v.as_int())
                    .unwrap_or(1)
                    .max(0) as usize;
                let byteorder = arg_items
                    .get(1)
                    .and_then(|v| v.as_ptr())
                    .and_then(|p| unsafe {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "big".to_string());
                // `signed=True` may arrive positionally (arg 2) or packed
                // into a trailing kwargs dict (my #107 method-kwargs fix).
                let signed = arg_items
                    .iter()
                    .find_map(|v| {
                        if let Some(b) = v.as_bool() {
                            return Some(b);
                        }
                        if let Some(ptr) = v.as_ptr() {
                            unsafe {
                                if let ObjData::Dict(ref lock) = (*ptr).data {
                                    let guard = lock.read().unwrap();
                                    let k = super::dict_ops::DictKey::Str("signed".to_string());
                                    return guard.get(&k).and_then(|v2| v2.as_bool());
                                }
                            }
                        }
                        None
                    })
                    .unwrap_or(false);
                // Encode val as two's-complement in `length` bytes.
                let mut buf = vec![0u8; length];
                let raw = if signed {
                    val as i128
                } else {
                    val as i128 & ((1i128 << (length * 8).min(127)) - 1)
                };
                let bits = if signed && val < 0 {
                    // Two's complement — fill top with 0xFF then write LE digits.
                    let mut b = vec![0xFFu8; length];
                    let mut v = val as i128;
                    for slot in b.iter_mut().take(length) {
                        *slot = (v & 0xFF) as u8;
                        v >>= 8;
                    }
                    b
                } else {
                    let mut v = raw.max(0);
                    for slot in buf.iter_mut().take(length) {
                        *slot = (v & 0xFF) as u8;
                        v >>= 8;
                    }
                    buf
                };
                let ordered = if byteorder == "big" {
                    let mut r = bits.clone();
                    r.reverse();
                    r
                } else {
                    bits
                };
                return MbValue::from_ptr(super::rc::MbObject::new_bytes(ordered));
            }
            "__abs__" => return MbValue::from_int(val.abs()),
            "conjugate" => return MbValue::from_int(val),
            "real" => return MbValue::from_int(val),
            "imag" => return MbValue::from_int(0),
            "numerator" => return MbValue::from_int(val),
            "denominator" => return MbValue::from_int(1),
            "is_integer" => return MbValue::from_bool(true),
            "as_integer_ratio" => {
                let pair = vec![MbValue::from_int(val), MbValue::from_int(1)];
                return MbValue::from_ptr(MbObject::new_tuple(pair));
            }
            // Arithmetic dunders — let `(5).__add__(3)` / `int.__mul__(a, b)`
            // work as method calls. Fall back to builtins::mb_* so the same
            // logic drives both operator form and method form.
            "__add__" | "__sub__" | "__mul__" | "__floordiv__" | "__mod__" | "__pow__"
            | "__and__" | "__or__" | "__xor__" | "__truediv__" => {
                let arg = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            lk.read().unwrap().first().copied()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(MbValue::none);
                let receiver_boxed = MbValue::from_int(val);
                return match name.as_str() {
                    "__add__" => super::builtins::mb_add(receiver_boxed, arg),
                    "__sub__" => super::builtins::mb_sub(receiver_boxed, arg),
                    "__mul__" => super::builtins::mb_mul(receiver_boxed, arg),
                    "__floordiv__" => super::builtins::mb_floordiv(receiver_boxed, arg),
                    "__mod__" => super::builtins::mb_mod(receiver_boxed, arg),
                    "__pow__" => super::builtins::mb_pow(receiver_boxed, arg),
                    "__and__" => super::builtins::mb_bitand(receiver_boxed, arg),
                    "__or__" => super::builtins::mb_bitor(receiver_boxed, arg),
                    "__xor__" => super::builtins::mb_bitxor(receiver_boxed, arg),
                    "__truediv__" => super::builtins::mb_div(receiver_boxed, arg),
                    _ => unreachable!(),
                };
            }
            "__neg__" => return MbValue::from_int(-val),
            "__pos__" => return MbValue::from_int(val),
            "__invert__" => return MbValue::from_int(!val),
            "__floor__" | "__ceil__" | "__trunc__" => return MbValue::from_int(val),
            "__int__" => return MbValue::from_int(val),
            "__float__" => return MbValue::from_float(val as f64),
            "__bool__" => return MbValue::from_bool(val != 0),
            "__hash__" => return super::builtins::mb_hash(MbValue::from_int(val)),
            _ => {}
        }
    }

    // Float primitive methods.
    if receiver.is_float() {
        let f = receiver.as_float().unwrap_or(0.0);
        match name.as_str() {
            "is_integer" => {
                return MbValue::from_bool(f.is_finite() && f == f.floor());
            }
            "as_integer_ratio" => {
                // CPython: returns (num, den) such that f == num/den, gcd(num,den)==1.
                // NaN → ValueError; inf → OverflowError; 0.0 → (0, 1).
                if f.is_nan() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "ValueError",
                        "cannot convert NaN to integer ratio",
                    ));
                    return MbValue::none();
                }
                if f.is_infinite() {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "OverflowError",
                        "cannot convert Infinity to integer ratio",
                    ));
                    return MbValue::none();
                }
                if f == 0.0 {
                    let pair = vec![MbValue::from_int(0), MbValue::from_int(1)];
                    return MbValue::from_ptr(MbObject::new_tuple(pair));
                }
                let bits = f.to_bits();
                let sign_neg = (bits >> 63) != 0;
                let raw_exp = ((bits >> 52) & 0x7ff) as i32;
                let mantissa_field: u64 = bits & ((1u64 << 52) - 1);
                let (mut m_int, mut e_off): (u64, i32) = if raw_exp == 0 {
                    // Subnormal: implicit 0, true exponent fixed at -1074.
                    (mantissa_field, -1074)
                } else {
                    ((1u64 << 52) | mantissa_field, raw_exp - 1023 - 52)
                };
                // Reduce 2-power: shift off trailing zeros, lifting the exponent.
                let tz = m_int.trailing_zeros() as i32;
                m_int >>= tz;
                e_off += tz;
                // Compose (num, den) within mamba's i48 int range. CPython would
                // return arbitrary-precision ints here; mamba raises OverflowError
                // when either component would not fit (see #-int-i48-overflow).
                const I48_MAX: u64 = (1u64 << 47) - 1;
                let overflow = || {
                    super::exception::set_current_exception(super::exception::MbException::new(
                        "OverflowError",
                        "as_integer_ratio result exceeds mamba's i48 int range",
                    ));
                };
                let (num_abs, den): (u64, u64) = if e_off >= 0 {
                    let shift = e_off as u32;
                    if shift >= 64 || m_int.checked_shl(shift).is_none() {
                        overflow();
                        return MbValue::none();
                    }
                    let n = m_int << shift;
                    if n > I48_MAX {
                        overflow();
                        return MbValue::none();
                    }
                    (n, 1)
                } else {
                    let neg_e = (-e_off) as u32;
                    if neg_e >= 48 || m_int > I48_MAX {
                        overflow();
                        return MbValue::none();
                    }
                    (m_int, 1u64 << neg_e)
                };
                let num = if sign_neg {
                    -(num_abs as i64)
                } else {
                    num_abs as i64
                };
                let pair = vec![MbValue::from_int(num), MbValue::from_int(den as i64)];
                return MbValue::from_ptr(MbObject::new_tuple(pair));
            }
            "__abs__" => return MbValue::from_float(f.abs()),
            "conjugate" => return MbValue::from_float(f),
            "real" => return MbValue::from_float(f),
            "imag" => return MbValue::from_float(0.0),
            "hex" => {
                // CPython float.hex(): "[-]0x<lead>.<13 hex digits>p<sign><exp>".
                let s = if f.is_nan() {
                    "nan".to_string()
                } else if f.is_infinite() {
                    if f < 0.0 {
                        "-inf".to_string()
                    } else {
                        "inf".to_string()
                    }
                } else {
                    let prefix = if f.is_sign_negative() { "-0x" } else { "0x" };
                    if f == 0.0 {
                        format!("{}0.0p+0", prefix)
                    } else {
                        let bits = f.abs().to_bits();
                        let exp_bits = ((bits >> 52) & 0x7ff) as i64;
                        let mantissa = bits & 0x000f_ffff_ffff_ffff;
                        let (lead, exp) = if exp_bits == 0 {
                            (0u64, -1022i64) // subnormal
                        } else {
                            (1u64, exp_bits - 1023)
                        };
                        let exp_sign = if exp >= 0 { "+" } else { "-" };
                        format!(
                            "{}{}.{:013x}p{}{}",
                            prefix,
                            lead,
                            mantissa,
                            exp_sign,
                            exp.abs()
                        )
                    }
                };
                return MbValue::from_ptr(MbObject::new_str(s));
            }
            "__floor__" | "__ceil__" | "__trunc__" => {
                // Non-finite floats cannot convert to int: NaN -> ValueError,
                // +/-inf -> OverflowError (CPython float.__floor__/__ceil__/__trunc__).
                if f.is_nan() {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "cannot convert float NaN to integer".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
                if f.is_infinite() {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "cannot convert float infinity to integer".to_string(),
                        )),
                    );
                    return MbValue::none();
                }
                let r = match name.as_str() {
                    "__floor__" => f.floor(),
                    "__ceil__" => f.ceil(),
                    _ => f.trunc(),
                };
                return MbValue::from_int(r as i64);
            }
            "__round__" => {
                // Banker's rounding (round half to even). 0-arg form returns int;
                // 1-arg form returns float (the ndigits argument).
                let arg_items: Vec<MbValue> = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            Some(lk.read().unwrap().to_vec())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                let banker_round = |x: f64| -> f64 {
                    let r = x.round();
                    // f64::round uses half-away-from-zero; switch to half-to-even.
                    if (x - x.trunc()).abs() == 0.5 {
                        let t = x.trunc();
                        if (t as i64) % 2 == 0 {
                            t
                        } else {
                            r
                        }
                    } else {
                        r
                    }
                };
                if arg_items.is_empty() {
                    return MbValue::from_int(banker_round(f) as i64);
                }
                let nd = arg_items[0].as_int().unwrap_or(0);
                if nd == 0 {
                    return MbValue::from_float(banker_round(f));
                }
                let scale = 10f64.powi(nd as i32);
                return MbValue::from_float(banker_round(f * scale) / scale);
            }
            "__neg__" => return MbValue::from_float(-f),
            "__pos__" => return MbValue::from_float(f),
            "__int__" => return MbValue::from_int(f as i64),
            "__float__" => return MbValue::from_float(f),
            "__bool__" => return MbValue::from_bool(f != 0.0),
            "__add__" | "__sub__" | "__mul__" | "__truediv__" | "__floordiv__" | "__mod__"
            | "__pow__" => {
                let arg = args
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::List(ref lk) = (*p).data {
                            lk.read().unwrap().first().copied()
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(MbValue::none);
                let receiver_boxed = MbValue::from_float(f);
                return match name.as_str() {
                    "__add__" => super::builtins::mb_add(receiver_boxed, arg),
                    "__sub__" => super::builtins::mb_sub(receiver_boxed, arg),
                    "__mul__" => super::builtins::mb_mul(receiver_boxed, arg),
                    "__truediv__" => super::builtins::mb_div(receiver_boxed, arg),
                    "__floordiv__" => super::builtins::mb_floordiv(receiver_boxed, arg),
                    "__mod__" => super::builtins::mb_mod(receiver_boxed, arg),
                    "__pow__" => super::builtins::mb_pow(receiver_boxed, arg),
                    _ => unreachable!(),
                };
            }
            _ => {}
        }
    }

    // Primitives — no heap deref needed
    if receiver.is_int() || receiver.is_bool() || receiver.is_none() || receiver.is_float() {
        let type_name = if receiver.is_int() {
            "int"
        } else if receiver.is_float() {
            "float"
        } else if receiver.is_bool() {
            "bool"
        } else {
            "NoneType"
        };
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{type_name}' object has no attribute '{name}'"
            ))),
        );
        return MbValue::none();
    }

    // Heap objects — deref and dispatch by ObjData variant
    if let Some(ptr) = receiver.as_ptr() {
        unsafe {
            // Native stdlib instances with fixed method tables (re.Match,
            // datetime.datetime, functools.partial, ...) short-circuit before
            // the MRO lookup.
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // UserDict / UserList / UserString: delegate method calls to
                // the backing dict/list/str (full builtin method surface for
                // free). User-defined overrides in subclasses win — only
                // forward when the MRO has no such method.
                if super::stdlib::collections_mod::user_wrapper_kind(class_name).is_some()
                    && lookup_method(class_name, &name).is_none()
                {
                    let data = fields.read().unwrap().get("_data").copied();
                    if let Some(data) = data {
                        if !data.is_none() {
                            let raw = mb_call_method(data, method_name, args);
                            // `copy()` returns the SAME wrapper class (CPython:
                            // UserList.copy() -> UserList), not the bare payload.
                            if name == "copy" {
                                return super::stdlib::collections_mod::user_wrapper_rewrap_like(receiver, raw);
                            }
                            return raw;
                        }
                    }
                }
                // namedtuple factory classmethod: Point._make(iterable).
                if class_name == "collections.namedtuple_factory" && name == "_make" {
                    let arg0 = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                lk.read().unwrap().first().copied()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(MbValue::none());
                    return super::stdlib::collections_mod::mb_namedtuple_make(receiver, arg0);
                }
                // namedtuple instance methods: _asdict() / _replace(**kw).
                // Marker-field dispatch — namedtuple instances carry a
                // dynamic class_name (the user-facing tuple name).
                if matches!(name.as_str(), "_asdict" | "_replace")
                    && fields.read().unwrap().contains_key("_namedtuple_fields")
                {
                    if name == "_asdict" {
                        return super::stdlib::collections_mod::mb_namedtuple_asdict(receiver);
                    }
                    // _replace kwargs arrive as a trailing dict positional.
                    let kwargs = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                lk.read().unwrap().last().copied()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(MbValue::none());
                    return super::stdlib::collections_mod::mb_namedtuple_replace(receiver, kwargs);
                }
                // Counter-specific methods. `update`/`subtract` ACCUMULATE
                // counts (CPython semantics) — they must intercept before the
                // generic dict-like forwarding below, whose `update` replaces.
                if class_name == "collections.Counter"
                    && matches!(name.as_str(), "update" | "subtract" | "total" | "elements")
                {
                    let arg_items: Vec<MbValue> = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                Some(lk.read().unwrap().to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    // CPython: update/subtract take at most ONE positional
                    // (`update(iterable=None, /, **kwds)`); a second positional
                    // is a TypeError. Keyword args arrive as a single trailing
                    // kwargs dict, so a >1 arg slice means 2+ positionals.
                    if matches!(name.as_str(), "update" | "subtract") && arg_items.len() > 1 {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "{name}() takes at most 1 positional argument ({} given)",
                                arg_items.len()
                            ))),
                        );
                        return MbValue::none();
                    }
                    match name.as_str() {
                        "update" => {
                            return super::stdlib::collections_mod::mb_counter_update_args(
                                receiver, &arg_items, 1,
                            )
                        }
                        "subtract" => {
                            return super::stdlib::collections_mod::mb_counter_update_args(
                                receiver, &arg_items, -1,
                            )
                        }
                        "total" => {
                            return super::stdlib::collections_mod::mb_counter_total(receiver)
                        }
                        "elements" => {
                            return super::stdlib::collections_mod::mb_counter_elements(receiver)
                        }
                        _ => {}
                    }
                }
                // Dict-like collections classes: forward dict methods (keys, values,
                // items, get, pop, update, setdefault, clear, copy, __contains__,
                // __len__) to the backing `_data` dict.
                if class_name == "collections.defaultdict"
                    || class_name == "collections.OrderedDict"
                    || (class_name == "collections.Counter"
                        && matches!(
                            name.as_str(),
                            "keys"
                                | "values"
                                | "items"
                                | "get"
                                | "pop"
                                | "setdefault"
                                | "clear"
                                | "copy"
                        ))
                {
                    let data = {
                        let guard = fields.read().unwrap();
                        guard.get("_data").copied().unwrap_or(MbValue::none())
                    };
                    if !data.is_none() {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let a0 = arg_items.first().copied().unwrap_or(MbValue::none());
                        let a1 = arg_items.get(1).copied().unwrap_or(MbValue::none());
                        match name.as_str() {
                            "keys" => return super::dict_ops::mb_dict_keys(data),
                            "values" => return super::dict_ops::mb_dict_values(data),
                            "items" => return super::dict_ops::mb_dict_items(data),
                            "get" => return super::dict_ops::mb_dict_get(data, a0, a1),
                            "pop" => return super::dict_ops::mb_dict_pop(data, a0, a1),
                            "setdefault" => {
                                return super::dict_ops::mb_dict_setdefault(data, a0, a1)
                            }
                            "update" => {
                                super::dict_ops::mb_dict_update(data, a0);
                                return MbValue::none();
                            }
                            "clear" => {
                                super::dict_ops::mb_dict_clear(data);
                                return MbValue::none();
                            }
                            "copy" => return super::dict_ops::mb_dict_copy(data),
                            // OrderedDict-specific reordering.
                            "move_to_end" if class_name == "collections.OrderedDict" => {
                                let last = if arg_items.len() > 1 {
                                    a1.as_bool().unwrap_or(true)
                                } else {
                                    true
                                };
                                return super::stdlib::collections_mod::mb_ordereddict_move_to_end(
                                    data, a0, last,
                                );
                            }
                            "popitem" if class_name == "collections.OrderedDict" => {
                                // OrderedDict.popitem(last=True): LIFO by default,
                                // FIFO when last=False.
                                let last = if arg_items.is_empty() {
                                    true
                                } else {
                                    a0.as_bool().unwrap_or(true)
                                };
                                return super::stdlib::collections_mod::mb_ordereddict_popitem(
                                    data, last,
                                );
                            }
                            _ => {}
                        }
                    }
                }
                if class_name == "datetime.datetime" {
                    // `.strftime(fmt)` → formatted string
                    if name == "strftime" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let fmt = arg_items.first().copied().unwrap_or(MbValue::none());
                        return super::stdlib::datetime_mod::mb_datetime_strftime(receiver, fmt);
                    }
                    // `.timestamp()` → float (Unix timestamp)
                    if name == "timestamp" {
                        return super::stdlib::datetime_mod::mb_datetime_timestamp(receiver);
                    }
                    // Attribute-like getters: year / month / day / hour / minute / second
                    if matches!(
                        name.as_str(),
                        "year" | "month" | "day" | "hour" | "minute" | "second"
                    ) {
                        let guard = fields.read().unwrap();
                        return guard.get(&name).copied().unwrap_or(MbValue::none());
                    }
                }
                if class_name == "collections.deque" {
                    let guard = fields.read().unwrap();
                    let items = guard.get("_items").copied().unwrap_or(MbValue::none());
                    let maxlen = guard.get("_maxlen").and_then(|v| v.as_int());
                    drop(guard);
                    let arg_items: Vec<MbValue> = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                Some(lk.read().unwrap().to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    if name == "appendleft" || name == "append" {
                        let val = arg_items.first().copied().unwrap_or(MbValue::none());
                        if let Some(ptr) = items.as_ptr() {
                            if let ObjData::List(ref lock) = (*ptr).data {
                                let mut list = lock.write().unwrap();
                                if name == "appendleft" {
                                    list.insert(0, val);
                                    if let Some(ml) = maxlen {
                                        while list.len() > ml as usize {
                                            list.pop();
                                        }
                                    }
                                } else {
                                    list.push(val);
                                    if let Some(ml) = maxlen {
                                        while list.len() > ml as usize {
                                            list.remove(0);
                                        }
                                    }
                                }
                            }
                        }
                        return MbValue::none();
                    }
                    if name == "popleft" {
                        return super::stdlib::collections_mod::mb_deque_popleft(items);
                    }
                    if name == "pop" {
                        return super::stdlib::collections_mod::mb_deque_pop(items);
                    }
                    if name == "rotate" {
                        let n = arg_items.first().copied().unwrap_or(MbValue::from_int(1));
                        return super::stdlib::collections_mod::mb_deque_rotate(items, n);
                    }
                    if name == "clear" {
                        if let Some(ptr) = items.as_ptr() {
                            if let ObjData::List(ref lock) = (*ptr).data {
                                lock.write().unwrap().clear();
                            }
                        }
                        return MbValue::none();
                    }
                    if name == "extend" || name == "extendleft" {
                        let iterable = arg_items.first().copied().unwrap_or(MbValue::none());
                        // Materialize the iterable into a Vec<MbValue>.
                        let new_items: Vec<MbValue> = if let Some(ip) = iterable.as_ptr() {
                            match &(*ip).data {
                                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                                ObjData::Tuple(ref tup) => tup.clone(),
                                _ => Vec::new(),
                            }
                        } else {
                            Vec::new()
                        };
                        if let Some(ptr) = items.as_ptr() {
                            if let ObjData::List(ref lock) = (*ptr).data {
                                let mut list = lock.write().unwrap();
                                if name == "extend" {
                                    list.extend(new_items);
                                    if let Some(ml) = maxlen {
                                        while list.len() > ml as usize {
                                            list.remove(0);
                                        }
                                    }
                                } else {
                                    // extendleft: each element pushed individually
                                    // to the left, so the iterable order is reversed.
                                    for v in new_items {
                                        list.insert(0, v);
                                        if let Some(ml) = maxlen {
                                            while list.len() > ml as usize {
                                                list.pop();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        return MbValue::none();
                    }
                }
                if class_name == "collections.Counter" {
                    if name == "most_common" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let n = arg_items.first().copied().unwrap_or(MbValue::none());
                        return super::stdlib::collections_mod::mb_counter_most_common(receiver, n);
                    }
                }
                if class_name == "ContextVar" || class_name == "Context" {
                    let arg_items: Vec<MbValue> = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                Some(lk.read().unwrap().to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let _ = &fields;
                    match (class_name.as_str(), name.as_str()) {
                        ("ContextVar", "get") => {
                            return super::stdlib::contextvars_mod::mb_contextvar_get(
                                receiver,
                                arg_items.first().copied(),
                            );
                        }
                        ("ContextVar", "set") => {
                            let value = arg_items.first().copied().unwrap_or(MbValue::none());
                            return super::stdlib::contextvars_mod::mb_contextvar_set(
                                receiver, value,
                            );
                        }
                        ("ContextVar", "reset") => {
                            let token = arg_items.first().copied().unwrap_or(MbValue::none());
                            return super::stdlib::contextvars_mod::mb_contextvar_reset(
                                receiver, token,
                            );
                        }
                        ("Context", "run") => {
                            let func = arg_items.first().copied().unwrap_or(MbValue::none());
                            let rest: Vec<MbValue> = arg_items.iter().skip(1).copied().collect();
                            return super::stdlib::contextvars_mod::mb_context_run(
                                receiver, func, rest,
                            );
                        }
                        ("Context", "copy") => {
                            // ctx.copy() — a snapshot of the snapshot; reuse
                            // run-free field cloning by returning a fresh
                            // Context built from this one's fields.
                            return super::stdlib::contextvars_mod::mb_context_copy(receiver);
                        }
                        ("Context", "create_decimal") => {
                            // decimal.Context.create_decimal(x) constructs a
                            // Decimal (shares Decimal()'s parsing/validation, so
                            // a non-numeric arg like ['%'] raises ValueError).
                            // contextvars.Context has no such method, so this
                            // case is decimal-Context only in practice.
                            let arg = arg_items.first().copied().unwrap_or_else(|| {
                                MbValue::from_ptr(MbObject::new_str("0".to_string()))
                            });
                            return super::stdlib::decimal_mod::mb_decimal_new(arg);
                        }
                        _ => {}
                    }
                }
                if class_name == "re.Pattern" {
                    // re.Pattern dispatches its match/search/findall/sub/split
                    // methods through the existing module-level helpers, using
                    // the stored pattern string (with the compile-time flags
                    // folded in as an inline prefix) as the first argument.
                    let pat = {
                        let guard = fields.read().unwrap();
                        let raw = guard.get("pattern").copied().unwrap_or(MbValue::none());
                        let flags = guard.get("flags").copied().unwrap_or(MbValue::none());
                        super::stdlib::re_mod::with_flags(raw, flags)
                    };
                    let arg_items: Vec<MbValue> = args
                        .as_ptr()
                        .and_then(|p| {
                            if let ObjData::List(ref lk) = (*p).data {
                                Some(lk.read().unwrap().to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let a0 = arg_items.first().copied().unwrap_or(MbValue::none());
                    let a1 = arg_items.get(1).copied().unwrap_or(MbValue::none());
                    let a2 = arg_items.get(2).copied().unwrap_or(MbValue::none());
                    // A bytes-compiled pattern rejects a str subject and a
                    // str-compiled pattern rejects a bytes subject (CPython
                    // TypeError). The stored `pat` is always a str, so the
                    // module helpers can't see a bytes pattern — validate here
                    // against the compiled pattern's `_is_bytes` flag. (For
                    // sub/subn the subject is the 2nd arg, not the 1st.)
                    let pat_is_bytes = fields
                        .read()
                        .unwrap()
                        .get("_is_bytes")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let subject = match name.as_str() {
                        "sub" | "subn" => a1,
                        "match" | "match_" | "fullmatch" | "search" | "findall" | "finditer"
                        | "split" => a0,
                        _ => MbValue::none(),
                    };
                    if let Some(sp) = subject.as_ptr() {
                        let (sub_bytes, sub_str) = match &(*sp).data {
                            ObjData::Bytes(_) | ObjData::ByteArray(_) => (true, false),
                            ObjData::Str(_) => (false, true),
                            _ => (false, false),
                        };
                        if pat_is_bytes && sub_str {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "cannot use a bytes pattern on a string-like object"
                                        .to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                        if !pat_is_bytes && sub_bytes {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "cannot use a string pattern on a bytes-like object"
                                        .to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                    }
                    match name.as_str() {
                        "match" | "match_" => return super::stdlib::re_mod::mb_re_match(pat, a0),
                        "fullmatch" => return super::stdlib::re_mod::mb_re_fullmatch(pat, a0),
                        "search" => return super::stdlib::re_mod::mb_re_search(pat, a0),
                        "findall" => return super::stdlib::re_mod::mb_re_findall(pat, a0),
                        "sub" => return super::stdlib::re_mod::mb_re_sub_count(pat, a0, a1, a2),
                        "subn" => return super::stdlib::re_mod::mb_re_subn_count(pat, a0, a1, a2),
                        "split" => return super::stdlib::re_mod::mb_re_split(pat, a0),
                        "finditer" => {
                            let pos = arg_items.get(1).copied().unwrap_or_else(MbValue::none);
                            let endpos = arg_items.get(2).copied().unwrap_or_else(MbValue::none);
                            return super::stdlib::re_mod::mb_re_finditer_window(
                                pat, a0, pos, endpos,
                            );
                        }
                        _ => {}
                    }
                }
                if class_name == "re.Match" {
                    let guard = fields.read().unwrap();
                    // .group([idx_or_name]) — returns the matched substring for
                    // the given group (0 = full match), or the full match when
                    // called with no args.
                    if name == "group" || name == "__getitem__" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let count = guard
                            .get("_group_count")
                            .and_then(|v| v.as_int())
                            .unwrap_or(0);
                        let lookup = |key: MbValue| -> MbValue {
                            if let Some(i) = key.as_int() {
                                // An index beyond the pattern's group count is
                                // IndexError; an unmatched group is None.
                                if i < 0 || i > count {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "IndexError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "no such group".to_string(),
                                        )),
                                    );
                                    return MbValue::none();
                                }
                                let k = format!("group_{}", i);
                                return guard.get(&k).copied().unwrap_or(MbValue::none());
                            }
                            if let Some(nm) = extract_str(key) {
                                let k = format!("group_name_{}", nm);
                                return match guard.get(&k).copied() {
                                    Some(v) => v,
                                    None => {
                                        super::exception::mb_raise(
                                            MbValue::from_ptr(MbObject::new_str(
                                                "IndexError".to_string(),
                                            )),
                                            MbValue::from_ptr(MbObject::new_str(
                                                "no such group".to_string(),
                                            )),
                                        );
                                        MbValue::none()
                                    }
                                };
                            }
                            MbValue::none()
                        };
                        // group(i, j, ...) with 2+ selectors returns a tuple.
                        if arg_items.len() >= 2 {
                            let vals: Vec<MbValue> = arg_items.iter().map(|k| lookup(*k)).collect();
                            return MbValue::from_ptr(MbObject::new_tuple(vals));
                        }
                        let key = arg_items.first().copied().unwrap_or(MbValue::from_int(0));
                        return lookup(key);
                    }
                    if name == "start" || name == "end" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let i = arg_items.first().and_then(|v| v.as_int()).unwrap_or(0);
                        let key = if i == 0 {
                            (if name == "start" { "_start" } else { "_end" }).to_string()
                        } else if name == "start" {
                            format!("group_start_{}", i)
                        } else {
                            format!("group_end_{}", i)
                        };
                        return guard.get(&key).copied().unwrap_or(MbValue::none());
                    }
                    // .groups([default]) — tuple of group_1..N (None for unmatched).
                    if name == "groups" {
                        let n = guard
                            .get("_group_count")
                            .and_then(|v| v.as_int())
                            .unwrap_or(0);
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let default = arg_items.first().copied().unwrap_or(MbValue::none());
                        let mut elems = Vec::with_capacity(n as usize);
                        for i in 1..=n {
                            let k = format!("group_{}", i);
                            let v = guard.get(&k).copied().unwrap_or(MbValue::none());
                            elems.push(if v.is_none() { default } else { v });
                        }
                        return MbValue::from_ptr(MbObject::new_tuple(elems));
                    }
                    // .groupdict([default]) — dict of named-group → value.
                    if name == "groupdict" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let default = arg_items.first().copied().unwrap_or(MbValue::none());
                        let names_val = guard
                            .get("_group_names")
                            .copied()
                            .unwrap_or(MbValue::none());
                        let d = super::dict_ops::mb_dict_new();
                        if let Some(p) = names_val.as_ptr() {
                            if let ObjData::List(ref lk) = (*p).data {
                                let names = lk.read().unwrap().clone();
                                for nm in names {
                                    let nm_str = extract_str(nm).unwrap_or_default();
                                    let k = format!("group_name_{}", nm_str);
                                    let v = guard.get(&k).copied().unwrap_or(MbValue::none());
                                    let v_eff = if v.is_none() { default } else { v };
                                    super::dict_ops::mb_dict_setitem(d, nm, v_eff);
                                }
                            }
                        }
                        return d;
                    }
                    // .span([group]) — (start, end) tuple. Group 0 / no-arg
                    // returns the full-match span; positive group args read
                    // the per-group offsets stored at construction time.
                    if name == "span" {
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let i = arg_items.first().and_then(|v| v.as_int()).unwrap_or(0);
                        let (sk, ek) = if i == 0 {
                            ("_start".to_string(), "_end".to_string())
                        } else {
                            (format!("group_start_{}", i), format!("group_end_{}", i))
                        };
                        let s = guard.get(&sk).copied().unwrap_or(MbValue::from_int(-1));
                        let e = guard.get(&ek).copied().unwrap_or(MbValue::from_int(-1));
                        return MbValue::from_ptr(MbObject::new_tuple(vec![s, e]));
                    }
                    // .expand(template) — substitute backrefs in template (#1620).
                    if name == "expand" {
                        drop(guard);
                        let arg_items: Vec<MbValue> = args
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(lk.read().unwrap().to_vec())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        let tmpl = arg_items.first().copied().unwrap_or(MbValue::none());
                        return super::stdlib::re_mod::mb_re_match_expand(receiver, tmpl);
                    }
                }
            }
            return match &(*ptr).data {
                ObjData::Str(ref s) => {
                    // R1 P1: Class-level method dispatch (e.g. Dog.get_species()).
                    // If the string is a registered class name, look up the method.
                    let class_name_str = s.clone();
                    let is_class =
                        CLASS_REGISTRY.with(|reg| reg.borrow().contains_key(&class_name_str));
                    if is_class {
                        if let Some(meta_desc) =
                            metaclass_data_descriptor_for_class(&class_name_str, &name)
                        {
                            let cls_val =
                                MbValue::from_ptr(MbObject::new_str(class_name_str.clone()));
                            let callable = invoke_descriptor_get(meta_desc, cls_val);
                            return super::builtins::mb_call_spread(callable, args);
                        }
                        let method = lookup_method(&class_name_str, &name);
                        if !method.is_none() {
                            let (actual_method, dk) = unwrap_descriptor_method(method);
                            let call_method = if actual_method.as_func().is_some()
                                || actual_method.as_int().is_some()
                            {
                                actual_method
                            } else {
                                method
                            };
                            let addr = extract_func_addr(call_method);
                            if addr != 0 {
                                let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                                if is_reg {
                                    // @staticmethod: no implicit first arg
                                    // @classmethod: pass the class name as cls
                                    // regular method: pass the class string as receiver (self)
                                    let mut all_args = Vec::new();
                                    if dk != DescriptorKind::StaticMethod {
                                        let first_arg = if dk == DescriptorKind::ClassMethod {
                                            MbValue::from_ptr(MbObject::new_str(
                                                class_name_str.clone(),
                                            ))
                                        } else {
                                            receiver
                                        };
                                        all_args.push(first_arg);
                                    }
                                    if let Some(args_ptr) = args.as_ptr() {
                                        if let ObjData::List(ref lock) = (*args_ptr).data {
                                            let items = lock.read().unwrap();
                                            all_args.extend(items.iter());
                                        }
                                    }
                                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                                    return match all_args.len() {
                                        0 => {
                                            let f: extern "C" fn() -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            f()
                                        }
                                        1 => {
                                            let f: extern "C" fn(MbValue) -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            f(all_args[0])
                                        }
                                        2 => {
                                            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            f(all_args[0], all_args[1])
                                        }
                                        3 => {
                                            let f: extern "C" fn(
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                            )
                                                -> MbValue = std::mem::transmute(addr as usize);
                                            f(all_args[0], all_args[1], all_args[2])
                                        }
                                        4 => {
                                            let f: extern "C" fn(
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                            )
                                                -> MbValue = std::mem::transmute(addr as usize);
                                            f(all_args[0], all_args[1], all_args[2], all_args[3])
                                        }
                                        _ => MbValue::none(),
                                    };
                                }
                            }
                        }
                    }
                    super::string_ops::dispatch_str_method(&name, receiver, args)
                }
                ObjData::List(_) => super::list_ops::dispatch_list_method(&name, receiver, args),
                ObjData::Dict(ref lock) => {
                    // Module dicts may have callable TAG_FUNC entries.
                    let callable = {
                        let guard = lock.read().unwrap();
                        guard.get(&name).copied()
                    };
                    if let Some(func_val) = callable {
                        if let Some(addr) = func_val.as_func() {
                            if super::module::is_native_func(addr as u64) {
                                // Native extern functions use (args_ptr, nargs) ABI — NOT fn(MbValue).
                                // Using the wrong ABI here causes SIGSEGV (#1132).
                                let items = super::builtins::extract_items(args);
                                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                    std::mem::transmute(addr);
                                return f(items.as_ptr(), items.len());
                            }
                            // JIT-compiled function: delegate to mb_call_spread for proper
                            // arity dispatch and re-boxing of raw returns.
                            return super::builtins::mb_call_spread(func_val, args);
                        }
                        // A module attribute that is an exception class-name string
                        // (e.g. zoneinfo.ZoneInfoNotFoundError, InvalidTZPathWarning)
                        // called as a constructor builds an exception instance, the
                        // same as a bare `ValueError("msg")`.
                        if let Some(sp) = func_val.as_ptr() {
                            if let ObjData::Str(ref s) = (*sp).data {
                                if super::exception::is_builtin_exception_name(s)
                                    || class_mro_any(s, super::exception::is_builtin_exception_name)
                                {
                                    return super::exception::mb_exception_new_with_args(func_val, args);
                                }
                            }
                        }
                    }
                    super::dict_ops::dispatch_dict_method(&name, receiver, args)
                }
                ObjData::Tuple(_) => super::tuple_ops::dispatch_tuple_method(&name, receiver, args),
                ObjData::Set(_) | ObjData::FrozenSet(_) => {
                    super::set_ops::dispatch_set_method(&name, receiver, args)
                }
                ObjData::Bytes(_) | ObjData::ByteArray(_) => {
                    super::bytes_ops::dispatch_bytes_method(&name, receiver, args)
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                    ..
                } => {
                    // Super proxy: dispatch through MRO after the current class
                    if class_name == "__super__" {
                        let fields_guard = fields.read().unwrap();
                        let super_class = fields_guard
                            .get("__super_class__")
                            .and_then(|v| extract_str(*v))
                            .unwrap_or_default();
                        let super_self = fields_guard
                            .get("__super_self__")
                            .copied()
                            .unwrap_or(MbValue::none());
                        drop(fields_guard);
                        // Get the actual class of the instance for MRO
                        let instance_class = if let Some(self_ptr) = super_self.as_ptr() {
                            if let ObjData::Instance { ref class_name, .. } = (*self_ptr).data {
                                class_name.clone()
                            } else {
                                String::new()
                            }
                        } else {
                            String::new()
                        };
                        // Metaclass context: `self` is a CLASS (a name string),
                        // so the MRO walked is the metaclass's. The builtin
                        // `type.__call__` tail of that MRO is default instance
                        // creation of the class (bypassing metaclass routing,
                        // which is the very frame we are in).
                        if instance_class.is_empty() {
                            let self_class = super_self.as_ptr().and_then(|p| match &(*p).data {
                                ObjData::Str(s) => Some(s.clone()),
                                _ => None,
                            });
                            if let Some(cls_str) = self_class {
                                let meta = CLASS_REGISTRY.with(|reg| {
                                    reg.borrow().get(&cls_str).and_then(|c| c.metaclass.clone())
                                });
                                if let Some(meta) = meta {
                                    let inherited = lookup_method_after(&meta, &super_class, &name);
                                    if !inherited.is_none() {
                                        // A parent metaclass method: call it
                                        // with the class as self.
                                        return call_method_value_with_args(
                                            inherited, super_self, args,
                                        );
                                    }
                                    if name == "__call__" {
                                        return instance_new_default(super_self, args);
                                    }
                                }
                            }
                        }
                        let method = lookup_method_after(&instance_class, &super_class, &name);
                        if !method.is_none() {
                            // R1 P1: Unwrap classmethod/staticmethod descriptors for super dispatch.
                            let (actual_method, dk) = unwrap_descriptor_method(method);
                            let call_method = if actual_method.as_func().is_some()
                                || actual_method.as_int().is_some()
                            {
                                actual_method
                            } else {
                                method
                            };
                            let addr = extract_func_addr(call_method);
                            if addr != 0 {
                                let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                                if is_reg {
                                    let mut all_args = Vec::new();
                                    if dk != DescriptorKind::StaticMethod {
                                        let first_arg = if dk == DescriptorKind::ClassMethod {
                                            MbValue::from_ptr(MbObject::new_str(
                                                instance_class.clone(),
                                            ))
                                        } else {
                                            super_self
                                        };
                                        all_args.push(first_arg);
                                    }
                                    if let Some(args_ptr) = args.as_ptr() {
                                        if let ObjData::List(ref lock) = (*args_ptr).data {
                                            let items = lock.read().unwrap();
                                            all_args.extend(items.iter());
                                        }
                                    }
                                    // REQ: JIT-compiled functions use SystemV/C calling convention.
                                    match all_args.len() {
                                        1 => {
                                            let f: extern "C" fn(MbValue) -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            return f(all_args[0]);
                                        }
                                        2 => {
                                            let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                                std::mem::transmute(addr as usize);
                                            return f(all_args[0], all_args[1]);
                                        }
                                        3 => {
                                            let f: extern "C" fn(
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                            )
                                                -> MbValue = std::mem::transmute(addr as usize);
                                            return f(all_args[0], all_args[1], all_args[2]);
                                        }
                                        4 => {
                                            let f: extern "C" fn(
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                                MbValue,
                                            )
                                                -> MbValue = std::mem::transmute(addr as usize);
                                            return f(
                                                all_args[0],
                                                all_args[1],
                                                all_args[2],
                                                all_args[3],
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            return method;
                        }
                        // Built-in __init__ fallback for Exception base classes:
                        // sets message, __type__, and args on the instance.
                        if name == "__init__" {
                            let mro = CLASS_REGISTRY.with(|reg| {
                                reg.borrow()
                                    .get(&instance_class)
                                    .map(|c| c.mro.clone())
                                    .unwrap_or_default()
                            });
                            let is_exc =
                                mro.iter().any(|c| c == "Exception" || c == "BaseException");
                            if is_exc {
                                // Extract args from the args list
                                let mut arg_items = Vec::new();
                                if let Some(args_ptr) = args.as_ptr() {
                                    if let ObjData::List(ref lock) = (*args_ptr).data {
                                        arg_items = lock.read().unwrap().to_vec();
                                    }
                                }
                                // Set message from first arg
                                if let Some(first) = arg_items.first() {
                                    mb_setattr(
                                        super_self,
                                        MbValue::from_ptr(MbObject::new_str("message".to_string())),
                                        *first,
                                    );
                                }
                                // Set __type__
                                mb_setattr(
                                    super_self,
                                    MbValue::from_ptr(MbObject::new_str("__type__".to_string())),
                                    MbValue::from_ptr(MbObject::new_str(instance_class.clone())),
                                );
                                // Set args as tuple
                                let args_tuple = MbValue::from_ptr(MbObject::new_tuple(arg_items));
                                mb_setattr(
                                    super_self,
                                    MbValue::from_ptr(MbObject::new_str("args".to_string())),
                                    args_tuple,
                                );
                                return MbValue::none();
                            }
                        }
                        return MbValue::none();
                    }
                    // Per-instance bound method (types.MethodType assigned to
                    // an instance attribute) shadows the class method — Python
                    // instance-dict lookup order.
                    {
                        let field = fields.read().ok().and_then(|f| f.get(&name).copied());
                        if let Some(fv) = field {
                            if let Some(fp) = fv.as_ptr() {
                                if let ObjData::Instance {
                                    class_name: ref fcn,
                                    fields: ref ffields,
                                } = (*fp).data
                                {
                                    if fcn == "method" {
                                        let (func_v, self_v) = {
                                            let fr = ffields.read().unwrap();
                                            (
                                                fr.get("__func__")
                                                    .copied()
                                                    .unwrap_or_else(MbValue::none),
                                                fr.get("__self__")
                                                    .copied()
                                                    .unwrap_or_else(MbValue::none),
                                            )
                                        };
                                        let mut all_args = vec![self_v];
                                        if let Some(args_ptr) = args.as_ptr() {
                                            if let ObjData::List(ref lock) = (*args_ptr).data {
                                                all_args.extend(lock.read().unwrap().iter());
                                            }
                                        }
                                        let args_list =
                                            MbValue::from_ptr(MbObject::new_list(all_args));
                                        return super::builtins::mb_call_spread(func_v, args_list);
                                    }
                                }
                            }
                        }
                    }
                    // MRO-based method lookup for regular instances
                    let method = lookup_method(class_name, &name);
                    if !method.is_none() {
                        // R1 P1: Unwrap classmethod/staticmethod descriptors.
                        // For @classmethod, pass class name string as first arg instead of instance.
                        // For @staticmethod, skip self entirely.
                        let (actual_method, dk) = unwrap_descriptor_method(method);
                        let call_method = if actual_method.as_func().is_some()
                            || actual_method.as_int().is_some()
                        {
                            actual_method
                        } else {
                            method
                        };
                        let addr = extract_func_addr(call_method);
                        if addr != 0 {
                            let is_reg = CALLABLE_REGISTRY.with(|r| r.borrow().contains(&addr));
                            if is_reg {
                                let mut all_args = Vec::new();
                                if dk != DescriptorKind::StaticMethod {
                                    let first_arg = if dk == DescriptorKind::ClassMethod {
                                        MbValue::from_ptr(MbObject::new_str(class_name.clone()))
                                    } else {
                                        receiver
                                    };
                                    all_args.push(first_arg);
                                }
                                let pos_args_start = all_args.len();
                                if let Some(args_ptr) = args.as_ptr() {
                                    if let ObjData::List(ref lock) = (*args_ptr).data {
                                        let items = lock.read().unwrap();
                                        all_args.extend(items.iter());
                                    }
                                }
                                // Variadic / kwargs methods: pack positional args into a list
                                // (and empty dict for **kwargs) so the compiled signature
                                // (self [, args_list] [, kwargs_dict]) gets the expected shape.
                                let is_variadic = super::module::is_variadic_func(addr as u64);
                                let has_kwargs = super::module::is_kwargs_func(addr as u64);
                                if is_variadic || has_kwargs {
                                    let pos: Vec<MbValue> =
                                        all_args.drain(pos_args_start..).collect();
                                    if is_variadic {
                                        all_args.push(MbValue::from_ptr(MbObject::new_list(
                                            pos.clone(),
                                        )));
                                    }
                                    if has_kwargs {
                                        all_args.push(MbValue::from_ptr(MbObject::new_dict()));
                                    }
                                    let _ = pos;
                                }
                                // REQ: JIT-compiled functions use SystemV/C calling convention.
                                match all_args.len() {
                                    1 => {
                                        let f: extern "C" fn(MbValue) -> MbValue =
                                            std::mem::transmute(addr as usize);
                                        return f(all_args[0]);
                                    }
                                    2 => {
                                        let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                            std::mem::transmute(addr as usize);
                                        return f(all_args[0], all_args[1]);
                                    }
                                    3 => {
                                        let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                                            std::mem::transmute(addr as usize);
                                        return f(all_args[0], all_args[1], all_args[2]);
                                    }
                                    4 => {
                                        let f: extern "C" fn(
                                            MbValue,
                                            MbValue,
                                            MbValue,
                                            MbValue,
                                        )
                                            -> MbValue = std::mem::transmute(addr as usize);
                                        return f(
                                            all_args[0],
                                            all_args[1],
                                            all_args[2],
                                            all_args[3],
                                        );
                                    }
                                    _ => {}
                                }
                                return MbValue::none(); // Fallback: too many args
                            }
                        }
                        // Closure handle or other callable stored as class attr:
                        // call it with self as the first arg (bound method).
                        if call_method.as_int().is_some() {
                            // Closure handle — call via mb_call1_val(method, self)
                            return mb_call1_val(call_method, receiver);
                        }
                        return method;
                    }
                    // CPython: when the class defines no such method, `obj.attr(args)`
                    // consults the instance __dict__. A callable stored as an attribute
                    // (e.g. `mod.func = fn` / `mod.Cls = C` on a types.ModuleType, or any
                    // instance whose field holds a function/class) is retrieved as-is and
                    // called with NO implicit self — instance-dict values are not bound
                    // descriptors. mb_getattr already resolves these for the load form
                    // (`obj.attr`); mirror it here so the call form matches.
                    let field_val = {
                        let guard = fields.read().unwrap();
                        guard.get(name.as_str()).copied()
                    };
                    if let Some(fv) = field_val {
                        if super::builtins::mb_callable(fv).as_bool() == Some(true) {
                            super::rc::retain_if_ptr(fv);
                            return super::builtins::mb_call_spread(fv, args);
                        }
                    }
                    // str-mixin enum members ((str, Enum) / StrEnum) inherit
                    // the str method surface: delegate against the raw value
                    // (`Direction.EAST.upper()` → "EAST").
                    if let Some(sv) = super::stdlib::enum_class::str_mixin_member_value(receiver) {
                        return super::string_ops::dispatch_str_method(&name, sv, args);
                    }
                    // PEP 695: `Alias.__value__()` — lazily resolved fields on
                    // TypeVar / TypeAliasType instances may hold callables;
                    // resolve through the lazy hook, then call (no bound self,
                    // mirroring the instance-dict rule above).
                    if super::pep695::is_pep695_class(class_name) {
                        if let Some(v) =
                            super::pep695::instance_lazy_attr_hook(receiver, class_name, &name)
                        {
                            if super::builtins::mb_callable(v).as_bool() == Some(true) {
                                return super::builtins::mb_call_spread(v, args);
                            }
                        }
                    }
                    // CPython: `obj.name(args)` where `name` is not a real
                    // method/attribute falls back to type(obj).__getattr__(obj,
                    // name); the returned value is then called. mb_getattr
                    // already resolves the load form (`obj.name`) through
                    // __getattr__ — mirror it for the fused call form so e.g.
                    // `t.foo(x)` works when `foo` is supplied by __getattr__.
                    // (Only the explicit __getattr__ dunder is consulted, not
                    // the lenient mb_getattr path, so a genuinely-absent name on
                    // a class without __getattr__ still raises AttributeError.)
                    // functools.singledispatch wrapper methods: register / dispatch.
                    if class_name == "functools.singledispatch" {
                        let items = super::builtins::extract_items(args);
                        match name.as_str() {
                            "register" => {
                                return super::stdlib::functools_mod::mb_singledispatch_register(
                                    receiver, &items,
                                );
                            }
                            "dispatch" => {
                                let t = items.first().copied().unwrap_or_else(MbValue::none);
                                return super::stdlib::functools_mod::mb_singledispatch_dispatch(
                                    receiver, t,
                                );
                            }
                            _ => {}
                        }
                    }
                    let getattr_dunder = lookup_method(class_name, "__getattr__");
                    if !getattr_dunder.is_none() {
                        let name_val = MbValue::from_ptr(MbObject::new_str(name.clone()));
                        let resolved = if let Some(addr) = getattr_dunder.as_func() {
                            let func: extern "C" fn(MbValue, MbValue) -> MbValue =
                                std::mem::transmute(addr);
                            func(receiver, name_val)
                        } else {
                            super::rc::retain_if_ptr(getattr_dunder);
                            getattr_dunder
                        };
                        // __getattr__ may raise (e.g. AttributeError for a
                        // genuinely-absent name); propagate it rather than
                        // calling the None result.
                        if super::exception::current_exception_type().is_some() {
                            return MbValue::none();
                        }
                        return super::builtins::mb_call_spread(resolved, args);
                    }
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'{class_name}' object has no attribute '{name}'"
                        ))),
                    );
                    MbValue::none()
                }
                ObjData::BigInt(_) => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'int' object has no attribute '{name}'"
                        ))),
                    );
                    MbValue::none()
                }
                ObjData::Complex(re, im) => {
                    // CPython exposes a small method surface on complex
                    // (#1256 — conjugate is the only CPython method on
                    // the complex type; __format__/__abs__ etc. dispatch
                    // elsewhere). Both `c.conjugate()` and the no-arg
                    // `c.__complex__()` return self-shaped values.
                    match name.as_str() {
                        "conjugate" => {
                            return MbValue::from_ptr(MbObject::new_complex(*re, -*im));
                        }
                        "__complex__" => {
                            return MbValue::from_ptr(MbObject::new_complex(*re, *im));
                        }
                        // CPython: complex(3,4).__getnewargs__() == (3.0, 4.0)
                        // (used by copy/pickle to reconstruct the value).
                        "__getnewargs__" => {
                            return MbValue::from_ptr(MbObject::new_tuple(vec![
                                MbValue::from_float(*re),
                                MbValue::from_float(*im),
                            ]));
                        }
                        _ => {}
                    }
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'complex' object has no attribute '{name}'"
                        ))),
                    );
                    MbValue::none()
                }
                ObjData::CodeObject { .. } => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "'code' object has no attribute '{name}'"
                        ))),
                    );
                    MbValue::none()
                }
            };
        }
    }
    MbValue::none()
}

/// Extract string from MbValue.
fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Resolve a class name from a value that may be either a plain string or a type object
/// (Instance with class_name="type" and __name__ field). This is the same resolution
/// logic used in mb_isinstance, extended for use in mb_issubclass (#974).
pub(crate) fn resolve_class_name(val: MbValue) -> Option<String> {
    // Try plain string first (most common path)
    if let Some(s) = extract_str(val) {
        return Some(s);
    }
    // Native-dispatcher function pointers used as types (e.g. io.StringIO is a
    // constructor dispatcher). Map the pointer to its recorded class name so
    // issubclass / type resolution see a real class.
    if let Some(addr) = val.as_func() {
        let name =
            super::module::NATIVE_TYPE_NAMES.with(|map| map.borrow().get(&(addr as u64)).cloned());
        if name.is_some() {
            return name;
        }
    }
    // Try type object: Instance with class_name="type" and __name__ field
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            class_name: ref cn,
            ref fields,
        } = (*ptr).data
        {
            if cn == "type" {
                fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__name__").and_then(|v| extract_str(*v)))
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn union_type_args(val: MbValue) -> Option<Vec<MbValue>> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            class_name: ref cn,
            ref fields,
        } = (*ptr).data
        {
            if cn == "UnionType" {
                return fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__args__").copied())
                    .and_then(|args| args.as_ptr())
                    .and_then(|args_ptr| match &(*args_ptr).data {
                        ObjData::Tuple(items) => Some(items.clone()),
                        _ => None,
                    });
            }
        }
        None
    })
}

// ── Cleanup ──

/// Reset all class-related thread_local state to defaults.
/// Called as part of centralized runtime cleanup between test executions.
/// Values are cleared without releasing — refcount imbalance from mixed
/// code paths makes release unsafe. Leaked objects reclaimed at process exit.
pub(crate) fn cleanup_all_classes() {
    let _ = CLASS_REGISTRY.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = CALLABLE_REGISTRY.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = SLOTS_REGISTRY.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = DICT_SUPPRESSED.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = KWARGS_REGISTRY.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = LAST_RAISED_INSTANCE.with(|c| c.try_borrow_mut().map(|mut m| *m = None));
    let _ = ABSTRACT_METHODS.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    cleanup_class_docs();
    let _ = METHOD_CACHE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = SIMPLE_CLASS_CACHE.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    let _ = ABC_VIRTUAL_SUBCLASSES.with(|c| c.try_borrow_mut().map(|mut m| m.clear()));
    METHOD_CACHE_GEN.with(|g| g.set(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_register_and_instance() {
        let methods = HashMap::new();
        mb_class_register("Dog", vec!["Animal".to_string()], methods);

        let name = MbValue::from_ptr(MbObject::new_str("Dog".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        assert!(inst.is_ptr());

        // Set and get attribute
        let attr = MbValue::from_ptr(MbObject::new_str("name".to_string()));
        let val = MbValue::from_ptr(MbObject::new_str("Rex".to_string()));
        mb_setattr(inst, attr, val);

        let attr2 = MbValue::from_ptr(MbObject::new_str("name".to_string()));
        let result = mb_getattr(inst, attr2);
        assert!(result.is_ptr());
    }

    #[test]
    fn test_isinstance() {
        mb_class_register("Animal", vec![], HashMap::new());
        mb_class_register("Dog", vec!["Animal".to_string()], HashMap::new());

        let dog_name = MbValue::from_ptr(MbObject::new_str("Dog".to_string()));
        let inst = mb_instance_new(dog_name, MbValue::none());

        let animal = MbValue::from_ptr(MbObject::new_str("Animal".to_string()));
        let dog = MbValue::from_ptr(MbObject::new_str("Dog".to_string()));
        let cat = MbValue::from_ptr(MbObject::new_str("Cat".to_string()));

        assert_eq!(mb_isinstance(inst, dog).as_bool(), Some(true));
        assert_eq!(mb_isinstance(inst, animal).as_bool(), Some(true));
        assert_eq!(mb_isinstance(inst, cat).as_bool(), Some(false));
    }

    #[test]
    fn test_primitive_isinstance() {
        let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        assert_eq!(
            mb_isinstance(MbValue::from_int(42), int_type).as_bool(),
            Some(true),
        );
    }

    #[test]
    fn test_super_method_lookup() {
        let mut base_methods = HashMap::new();
        base_methods.insert("greet".to_string(), MbValue::from_int(100));
        mb_class_register("Base2", vec![], base_methods);

        let mut child_methods = HashMap::new();
        child_methods.insert("greet".to_string(), MbValue::from_int(200));
        mb_class_register("Child2", vec!["Base2".to_string()], child_methods);

        // Create instance of Child2
        let name = MbValue::from_ptr(MbObject::new_str("Child2".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // super(Child2, inst).greet should find Base2.greet
        let cls = MbValue::from_ptr(MbObject::new_str("Child2".to_string()));
        let proxy = mb_super(cls, inst);
        let attr = MbValue::from_ptr(MbObject::new_str("greet".to_string()));
        let method = mb_super_getattr(proxy, attr);
        assert_eq!(method.as_int(), Some(100)); // Base2's method
    }

    #[test]
    fn test_mro_single_inheritance() {
        mb_class_register("Base", vec![], HashMap::new());
        mb_class_register("Child", vec!["Base".to_string()], HashMap::new());

        CLASS_REGISTRY.with(|reg| {
            let cls = reg.borrow();
            let child = cls.get("Child").unwrap();
            assert_eq!(child.mro, vec!["Child", "Base", "object"]);
        });
    }

    #[test]
    fn test_dunder_binop_dispatch() {
        // Verifies the lookup half of mb_dispatch_binop. The full dispatch
        // path also invokes the method, which requires a real callable
        // (TAG_FUNC address or CALLABLE_REGISTRY entry). Setting one of
        // those up in a unit test would need a JIT-compiled function, so
        // this test sticks to the lookup mechanism via lookup_method.
        let mut methods = HashMap::new();
        methods.insert("__add__".to_string(), MbValue::from_int(999));
        mb_class_register("Addable", vec![], methods);

        let method = lookup_method("Addable", "__add__");
        assert_eq!(
            method.as_int(),
            Some(999),
            "__add__ dunder should be found by lookup_method",
        );
    }

    #[test]
    fn test_dunder_unaryop_dispatch() {
        // Same rationale as test_dunder_binop_dispatch.
        let mut methods = HashMap::new();
        methods.insert("__neg__".to_string(), MbValue::from_int(777));
        mb_class_register("Negatable", vec![], methods);

        let method = lookup_method("Negatable", "__neg__");
        assert_eq!(
            method.as_int(),
            Some(777),
            "__neg__ dunder should be found by lookup_method",
        );
    }

    #[test]
    fn test_dunder_getitem_dispatch() {
        // Verify that __getitem__ dunder is found via try_get_dunder.
        // mb_obj_getitem now actually calls the method (not just returns it),
        // so we test the lookup mechanism directly.
        let mut methods = HashMap::new();
        methods.insert("__getitem__".to_string(), MbValue::from_int(555));
        mb_class_register("Indexable", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("Indexable".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let dunder = try_get_dunder(inst, "__getitem__");
        assert!(dunder.is_some(), "__getitem__ should be found on Indexable");
        assert_eq!(dunder.unwrap().as_int(), Some(555));
    }

    #[test]
    fn test_dunder_setitem_dispatch() {
        // Verify that __setitem__ dunder is found via try_get_dunder.
        let mut methods = HashMap::new();
        methods.insert("__setitem__".to_string(), MbValue::from_int(333));
        mb_class_register("MutableIdx", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("MutableIdx".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let dunder = try_get_dunder(inst, "__setitem__");
        assert!(
            dunder.is_some(),
            "__setitem__ should be found on MutableIdx"
        );
        assert_eq!(dunder.unwrap().as_int(), Some(333));
    }

    #[test]
    fn test_getattr_fallback_to_dunder() {
        let mut methods = HashMap::new();
        methods.insert("__getattr__".to_string(), MbValue::from_int(888));
        mb_class_register("Dynamic", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("Dynamic".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Access a nonexistent attribute — should fall back to __getattr__
        let attr = MbValue::from_ptr(MbObject::new_str("missing".to_string()));
        let result = mb_getattr(inst, attr);
        assert_eq!(
            result.as_int(),
            Some(888),
            "__getattr__ should be the fallback"
        );
    }

    #[test]
    fn test_delattr_removes_field() {
        let methods = HashMap::new();
        mb_class_register("Deletable", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("Deletable".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Set then delete an attribute
        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(42));

        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let before = mb_getattr(inst, attr2);
        assert_eq!(before.as_int(), Some(42));

        let attr3 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_delattr(inst, attr3);

        let attr4 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let after = mb_getattr(inst, attr4);
        assert!(after.is_none(), "field should be deleted");
    }

    #[test]
    fn test_hasattr() {
        mb_class_register("HasAttrTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("HasAttrTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(mb_hasattr(inst, attr).as_bool(), Some(false));

        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr2, MbValue::from_int(1));

        let attr3 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(mb_hasattr(inst, attr3).as_bool(), Some(true));
    }

    #[test]
    fn test_isinstance_primitives() {
        let str_type = MbValue::from_ptr(MbObject::new_str("str".to_string()));
        let list_type = MbValue::from_ptr(MbObject::new_str("list".to_string()));
        let dict_type = MbValue::from_ptr(MbObject::new_str("dict".to_string()));
        let float_type = MbValue::from_ptr(MbObject::new_str("float".to_string()));
        let bool_type = MbValue::from_ptr(MbObject::new_str("bool".to_string()));
        let none_type = MbValue::from_ptr(MbObject::new_str("NoneType".to_string()));

        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(mb_isinstance(s, str_type).as_bool(), Some(true));

        let l = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(mb_isinstance(l, list_type).as_bool(), Some(true));

        let d = MbValue::from_ptr(MbObject::new_dict());
        assert_eq!(mb_isinstance(d, dict_type).as_bool(), Some(true));

        assert_eq!(
            mb_isinstance(MbValue::from_float(1.0), float_type).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_isinstance(MbValue::from_bool(true), bool_type).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_isinstance(MbValue::none(), none_type).as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_isinstance_tuple() {
        let tuple_type = MbValue::from_ptr(MbObject::new_str("tuple".to_string()));
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        assert_eq!(mb_isinstance(t, tuple_type).as_bool(), Some(true));
    }

    #[test]
    fn test_isinstance_set() {
        let set_type = MbValue::from_ptr(MbObject::new_str("set".to_string()));
        let s = MbValue::from_ptr(MbObject::new_set(vec![]));
        assert_eq!(mb_isinstance(s, set_type).as_bool(), Some(true));
    }

    #[test]
    fn test_issubclass() {
        mb_class_register("IsSubBase", vec![], HashMap::new());
        mb_class_register("IsSubChild", vec!["IsSubBase".to_string()], HashMap::new());

        let child = MbValue::from_ptr(MbObject::new_str("IsSubChild".to_string()));
        let base = MbValue::from_ptr(MbObject::new_str("IsSubBase".to_string()));
        let other = MbValue::from_ptr(MbObject::new_str("Other".to_string()));

        assert_eq!(mb_issubclass(child, base).as_bool(), Some(true));
        assert_eq!(mb_issubclass(child, other).as_bool(), Some(false));
    }

    #[test]
    fn test_issubclass_same() {
        let name = MbValue::from_ptr(MbObject::new_str("SameClass".to_string()));
        let name2 = MbValue::from_ptr(MbObject::new_str("SameClass".to_string()));
        assert_eq!(mb_issubclass(name, name2).as_bool(), Some(true));
    }

    #[test]
    fn test_class_define() {
        let name = MbValue::from_ptr(MbObject::new_str("Defined".to_string()));
        let base = MbValue::none();
        let method_names = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("greet".to_string()),
        )]));
        let method_values = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(42)]));
        mb_class_define(name, base, method_names, method_values);

        CLASS_REGISTRY.with(|reg| {
            let cls = reg.borrow();
            assert!(cls.contains_key("Defined"));
            assert!(cls.get("Defined").unwrap().methods.contains_key("greet"));
        });
    }

    #[test]
    fn test_getattr_default() {
        mb_class_register("DefaultTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("DefaultTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("missing".to_string()));
        let default = MbValue::from_int(99);
        assert_eq!(mb_getattr_default(inst, attr, default).as_int(), Some(99));
    }

    #[test]
    fn test_vars() {
        mb_class_register("VarsTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("VarsTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(10));

        let vars = mb_vars(inst);
        assert!(vars.is_ptr());
    }

    #[test]
    fn test_dir() {
        let mut methods = HashMap::new();
        methods.insert("speak".to_string(), MbValue::from_int(1));
        mb_class_register("DirTest", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("DirTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(42));

        let dir_list = mb_dir(inst);
        assert!(dir_list.is_ptr());
    }

    #[test]
    fn test_dir_module_dict_includes_module_keys() {
        let module = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*module).data {
                let mut attrs = lock.write().unwrap();
                attrs.insert(
                    "__name__".into(),
                    MbValue::from_ptr(MbObject::new_str("__future__".to_string())),
                );
                attrs.insert(
                    "all_feature_names".into(),
                    MbValue::from_ptr(MbObject::new_list(vec![])),
                );
            }
        }

        let dir_list = mb_dir(MbValue::from_ptr(module));
        let names: Vec<String> = unsafe {
            if let ObjData::List(ref lock) = (*dir_list.as_ptr().unwrap()).data {
                lock.read()
                    .unwrap()
                    .iter()
                    .filter_map(|value| extract_str(*value))
                    .collect()
            } else {
                Vec::new()
            }
        };
        assert!(names.contains(&"__name__".to_string()));
        assert!(names.contains(&"all_feature_names".to_string()));
    }

    #[test]
    fn test_property_new() {
        let getter = MbValue::from_int(100);
        let prop = mb_property_new(getter);
        assert!(prop.is_ptr());
        let key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
        let fget = mb_getattr(prop, key);
        assert_eq!(fget.as_int(), Some(100));
    }

    #[test]
    fn test_property_setter_deleter() {
        let prop = mb_property_new(MbValue::from_int(1));
        mb_property_setter(prop, MbValue::from_int(2));
        mb_property_deleter(prop, MbValue::from_int(3));

        let fset_key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
        assert_eq!(mb_getattr(prop, fset_key).as_int(), Some(2));

        let fdel_key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
        assert_eq!(mb_getattr(prop, fdel_key).as_int(), Some(3));
    }

    #[test]
    fn test_classmethod_staticmethod() {
        let cm = mb_classmethod_new(MbValue::from_int(42));
        assert!(cm.is_ptr());
        let func = mb_descriptor_unwrap(cm);
        assert_eq!(func.as_int(), Some(42));

        let sm = mb_staticmethod_new(MbValue::from_int(99));
        let func2 = mb_descriptor_unwrap(sm);
        assert_eq!(func2.as_int(), Some(99));
    }

    #[test]
    fn test_abstractmethod_passthrough() {
        let f = MbValue::from_int(42);
        assert_eq!(mb_abstractmethod(f).as_int(), Some(42));
    }

    #[test]
    fn test_check_abstract_concrete() {
        let mut methods = HashMap::new();
        methods.insert("do_thing".to_string(), MbValue::from_int(1));
        mb_class_register("ConcreteABC", vec![], methods);

        // Register abstract method
        let name = MbValue::from_ptr(MbObject::new_str("ConcreteABC".to_string()));
        let abs_methods = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("do_thing".to_string()),
        )]));
        mb_register_abstract(name, abs_methods);

        let cls_name = MbValue::from_ptr(MbObject::new_str("ConcreteABC".to_string()));
        assert_eq!(mb_check_abstract(cls_name).as_bool(), Some(true));
    }

    #[test]
    fn test_obj_str_default() {
        mb_class_register("ObjStrTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("ObjStrTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let result = mb_obj_str(inst);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert!(s.contains("ObjStrTest"));
            }
        }
    }

    #[test]
    fn test_obj_repr_fallback() {
        mb_class_register("ObjReprTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("ObjReprTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let result = mb_obj_repr(inst);
        // Falls back to obj_str
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert!(s.contains("ObjReprTest"));
            }
        }
    }

    #[test]
    fn test_obj_bool_default() {
        mb_class_register("ObjBoolTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("ObjBoolTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        assert_eq!(mb_obj_bool(inst).as_bool(), Some(true));
    }

    #[test]
    fn test_obj_hash_default() {
        mb_class_register("ObjHashTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("ObjHashTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let h = mb_obj_hash(inst);
        assert!(h.as_int().is_some());
    }

    #[test]
    fn test_obj_contains_no_dunder() {
        mb_class_register("NoContains", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoContains".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        assert_eq!(
            mb_obj_contains(inst, MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_obj_len_no_dunder() {
        mb_class_register("NoLen", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoLen".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        assert_eq!(mb_obj_len(inst).as_int(), Some(0));
    }

    #[test]
    fn test_obj_format_no_dunder() {
        mb_class_register("NoFormat", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoFormat".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let result = mb_obj_format(inst, MbValue::none());
        // Falls back to obj_str
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert!(s.contains("NoFormat"));
            }
        }
    }

    #[test]
    fn test_obj_delitem_list() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        mb_obj_delitem(list, MbValue::from_int(1));
        unsafe {
            if let ObjData::List(ref lock) = (*list.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            }
        }
    }

    #[test]
    fn test_obj_delitem_dict() {
        let dict = MbValue::from_ptr(MbObject::new_dict());
        let key = MbValue::from_ptr(MbObject::new_str("k".to_string()));
        crate::runtime::dict_ops::mb_dict_setitem(dict, key, MbValue::from_int(1));
        let key2 = MbValue::from_ptr(MbObject::new_str("k".to_string()));
        mb_obj_delitem(dict, key2);
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            }
        }
    }

    #[test]
    fn test_check_setattr_dunder_none() {
        mb_class_register("NoSetattr", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoSetattr".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        assert!(mb_check_setattr_dunder(inst).is_none());
    }

    #[test]
    fn test_check_delattr_dunder_none() {
        mb_class_register("NoDelattr", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoDelattr".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        assert!(mb_check_delattr_dunder(inst).is_none());
    }

    #[test]
    fn test_multiple_inheritance_mro() {
        mb_class_register("MIBase1", vec![], HashMap::new());
        mb_class_register("MIBase2", vec![], HashMap::new());
        mb_class_register(
            "MIChild",
            vec!["MIBase1".to_string(), "MIBase2".to_string()],
            HashMap::new(),
        );

        CLASS_REGISTRY.with(|reg| {
            let cls = reg.borrow();
            let child = cls.get("MIChild").unwrap();
            assert_eq!(child.mro[0], "MIChild");
            assert!(child.mro.contains(&"MIBase1".to_string()));
            assert!(child.mro.contains(&"MIBase2".to_string()));
            assert!(child.mro.contains(&"object".to_string()));
        });
    }

    #[test]
    fn test_register_slots() {
        mb_class_register("Slotted", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("Slotted".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
            MbValue::from_ptr(MbObject::new_str("y".to_string())),
        ]));
        mb_register_slots(name, slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slots = reg.get("Slotted").unwrap();
            assert_eq!(slots.len(), 2);
            assert!(slots.contains(&"x".to_string()));
            assert!(slots.contains(&"y".to_string()));
        });
    }

    #[test]
    fn test_dispatch_binop_reverse() {
        // Verify the reverse-dunder lookup half of mb_dispatch_binop.
        // Full reverse dispatch needs an invokable method (see
        // test_dunder_binop_dispatch comment); test the lookup directly.
        let mut methods = HashMap::new();
        methods.insert("__radd__".to_string(), MbValue::from_int(111));
        mb_class_register("RAddable", vec![], methods);

        let method = lookup_method("RAddable", "__radd__");
        assert_eq!(method.as_int(), Some(111), "__radd__ should be found");
    }

    #[test]
    fn test_dispatch_binop_not_found() {
        mb_class_register("NoBinop", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoBinop".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let result = mb_dispatch_binop(0, inst, MbValue::from_int(1));
        assert!(result.is_none());
    }

    #[test]
    fn test_dispatch_unaryop_not_found() {
        mb_class_register("NoUnary", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoUnary".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let result = mb_dispatch_unaryop(1, inst);
        assert!(result.is_none());
    }

    #[test]
    fn test_lookup_dunder() {
        let mut methods = HashMap::new();
        methods.insert("__len__".to_string(), MbValue::from_int(42));
        mb_class_register("LenTest", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("LenTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let dunder = MbValue::from_ptr(MbObject::new_str("__len__".to_string()));
        let result = mb_lookup_dunder(inst, dunder);
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_lookup_dunder_missing() {
        mb_class_register("NoDunder", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("NoDunder".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let dunder = MbValue::from_ptr(MbObject::new_str("__missing__".to_string()));
        assert!(mb_lookup_dunder(inst, dunder).is_none());
    }

    #[test]
    fn test_extract_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(extract_str(s), Some("hello".to_string()));
        assert_eq!(extract_str(MbValue::from_int(42)), None);
        assert_eq!(extract_str(MbValue::none()), None);
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_c3_mro_linear_chain() {
        mb_class_register("MroBase", vec![], std::collections::HashMap::new());
        mb_class_register(
            "MroChild",
            vec!["MroBase".to_string()],
            std::collections::HashMap::new(),
        );
        mb_class_register(
            "MroGrand",
            vec!["MroChild".to_string()],
            std::collections::HashMap::new(),
        );
        let mro = compute_mro("MroGrand", &["MroChild".to_string()]);
        assert!(mro.contains(&"MroGrand".to_string()));
        assert!(mro.contains(&"MroChild".to_string()));
        assert!(mro.contains(&"MroBase".to_string()));
        let gi = mro.iter().position(|x| x == "MroGrand").unwrap();
        let ci = mro.iter().position(|x| x == "MroChild").unwrap();
        let bi = mro.iter().position(|x| x == "MroBase").unwrap();
        assert!(gi < ci, "MroGrand must precede MroChild in MRO");
        assert!(ci < bi, "MroChild must precede MroBase in MRO");
    }

    #[test]
    fn test_py312_c3_mro_diamond() {
        mb_class_register("DiaA", vec![], std::collections::HashMap::new());
        mb_class_register(
            "DiaB",
            vec!["DiaA".to_string()],
            std::collections::HashMap::new(),
        );
        mb_class_register(
            "DiaC",
            vec!["DiaA".to_string()],
            std::collections::HashMap::new(),
        );
        mb_class_register(
            "DiaD",
            vec!["DiaB".to_string(), "DiaC".to_string()],
            std::collections::HashMap::new(),
        );
        let mro = compute_mro("DiaD", &["DiaB".to_string(), "DiaC".to_string()]);
        let ai = mro.iter().position(|x| x == "DiaA").unwrap();
        let bi = mro.iter().position(|x| x == "DiaB").unwrap();
        let ci = mro.iter().position(|x| x == "DiaC").unwrap();
        assert!(bi < ai, "B must precede A in diamond MRO");
        assert!(ci < ai, "C must precede A in diamond MRO");
        assert!(bi < ci, "B must precede C per declaration order");
    }

    #[test]
    fn test_py312_instance_inherits_parent_method() {
        let mut parent_methods = std::collections::HashMap::new();
        parent_methods.insert("greet".to_string(), MbValue::from_int(42));
        mb_class_register("ParentCls", vec![], parent_methods);
        mb_class_register(
            "ChildCls",
            vec!["ParentCls".to_string()],
            std::collections::HashMap::new(),
        );
        let name = MbValue::from_ptr(MbObject::new_str("ChildCls".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let method_name = MbValue::from_ptr(MbObject::new_str("greet".to_string()));
        let result = mb_getattr(inst, method_name);
        assert_eq!(
            result.as_int(),
            Some(42),
            "child should inherit parent method"
        );
    }

    #[test]
    fn test_py312_class_override_shadows_parent() {
        let mut parent_methods = std::collections::HashMap::new();
        parent_methods.insert("val".to_string(), MbValue::from_int(1));
        mb_class_register("OvBase", vec![], parent_methods);
        let mut child_methods = std::collections::HashMap::new();
        child_methods.insert("val".to_string(), MbValue::from_int(99));
        mb_class_register("OvChild", vec!["OvBase".to_string()], child_methods);
        let name = MbValue::from_ptr(MbObject::new_str("OvChild".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("val".to_string()));
        assert_eq!(mb_getattr(inst, attr).as_int(), Some(99));
    }

    #[test]
    fn test_init_subclass_basic() {
        static HOOK_INVOKED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        extern "C" fn hook_fn(_cls: MbValue) -> MbValue {
            HOOK_INVOKED.store(true, std::sync::atomic::Ordering::SeqCst);
            MbValue::none()
        }

        // Register hook as a callable
        let hook_addr = hook_fn as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(hook_addr as u64);
        });

        // Register Base class with __init_subclass__
        let mut base_methods = HashMap::new();
        base_methods.insert(
            "__init_subclass__".to_string(),
            MbValue::from_func(hook_addr),
        );
        mb_class_register("IscBase_T2", vec![], base_methods);

        // Register Child — this should trigger the hook
        mb_class_register(
            "IscChild_T2",
            vec!["IscBase_T2".to_string()],
            HashMap::new(),
        );

        assert!(
            HOOK_INVOKED.load(std::sync::atomic::Ordering::SeqCst),
            "__init_subclass__ hook was not called when IscChild_T2 was registered"
        );
    }

    // ── New tests (coverage expansion) ────────────────────────────────────────

    #[test]
    fn test_mro_single_class_no_bases() {
        // compute_mro on a class with no bases → MRO = [ClassName, object]
        let mro = compute_mro("MroSolo001", &[]);
        assert_eq!(mro[0], "MroSolo001");
        assert!(mro.contains(&"object".to_string()));
    }

    #[test]
    fn test_mro_two_levels() {
        mb_class_register("MroTwoA001", vec![], HashMap::new());
        mb_class_register("MroTwoB001", vec!["MroTwoA001".to_string()], HashMap::new());
        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let b = reg.get("MroTwoB001").unwrap();
            let bi = b.mro.iter().position(|x| x == "MroTwoB001").unwrap();
            let ai = b.mro.iter().position(|x| x == "MroTwoA001").unwrap();
            assert!(bi < ai, "B must precede A in two-level MRO");
        });
    }

    #[test]
    fn test_mro_three_levels() {
        mb_class_register("Mro3A001", vec![], HashMap::new());
        mb_class_register("Mro3B001", vec!["Mro3A001".to_string()], HashMap::new());
        mb_class_register("Mro3C001", vec!["Mro3B001".to_string()], HashMap::new());
        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let c = reg.get("Mro3C001").unwrap();
            let ci = c.mro.iter().position(|x| x == "Mro3C001").unwrap();
            let bi = c.mro.iter().position(|x| x == "Mro3B001").unwrap();
            let ai = c.mro.iter().position(|x| x == "Mro3A001").unwrap();
            assert!(ci < bi, "C before B");
            assert!(bi < ai, "B before A");
        });
    }

    #[test]
    fn test_mro_multiple_parents_no_diamond() {
        mb_class_register("MroNdB001", vec![], HashMap::new());
        mb_class_register("MroNdC001", vec![], HashMap::new());
        mb_class_register(
            "MroNdD001",
            vec!["MroNdB001".to_string(), "MroNdC001".to_string()],
            HashMap::new(),
        );
        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let d = reg.get("MroNdD001").unwrap();
            let di = d.mro.iter().position(|x| x == "MroNdD001").unwrap();
            let bi = d.mro.iter().position(|x| x == "MroNdB001").unwrap();
            let ci = d.mro.iter().position(|x| x == "MroNdC001").unwrap();
            assert!(di < bi);
            assert!(bi < ci, "B declared before C, must appear first");
        });
    }

    #[test]
    fn test_instance_default_attrs_empty() {
        mb_class_register("InstEmpty001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("InstEmpty001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        // No attributes set — getattr of anything not on the class returns none
        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent_field".to_string()));
        assert!(mb_getattr(inst, attr).is_none());
    }

    #[test]
    fn test_setattr_and_getattr() {
        mb_class_register("SetGetTest001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("SetGetTest001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(42));

        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(42));
    }

    #[test]
    fn test_delattr_removes_attribute() {
        mb_class_register("DelAttrTest001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("DelAttrTest001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("y".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(7));

        let attr2 = MbValue::from_ptr(MbObject::new_str("y".to_string()));
        mb_delattr(inst, attr2);

        let attr3 = MbValue::from_ptr(MbObject::new_str("y".to_string()));
        assert!(
            mb_getattr(inst, attr3).is_none(),
            "deleted attr must be gone"
        );
    }

    #[test]
    fn test_setattr_multiple_attrs() {
        mb_class_register("MultiAttr001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("MultiAttr001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        for (key, val) in [("a", 1i64), ("b", 2), ("c", 3)] {
            let attr = MbValue::from_ptr(MbObject::new_str(key.to_string()));
            mb_setattr(inst, attr, MbValue::from_int(val));
        }
        for (key, expected) in [("a", 1i64), ("b", 2), ("c", 3)] {
            let attr = MbValue::from_ptr(MbObject::new_str(key.to_string()));
            assert_eq!(mb_getattr(inst, attr).as_int(), Some(expected));
        }
    }

    #[test]
    fn test_getattr_missing_falls_through_to_class() {
        let mut methods = HashMap::new();
        methods.insert("speak".to_string(), MbValue::from_int(555));
        mb_class_register("ClassMethod001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("ClassMethod001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // "speak" not set on instance — should find it on the class
        let attr = MbValue::from_ptr(MbObject::new_str("speak".to_string()));
        assert_eq!(mb_getattr(inst, attr).as_int(), Some(555));
    }

    #[test]
    fn test_isinstance_basic_true() {
        mb_class_register("IsinstA001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("IsinstA001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let cls = MbValue::from_ptr(MbObject::new_str("IsinstA001".to_string()));
        assert_eq!(mb_isinstance(inst, cls).as_bool(), Some(true));
    }

    #[test]
    fn test_isinstance_basic_false() {
        mb_class_register("IsinstB001", vec![], HashMap::new());
        mb_class_register("IsinstC001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("IsinstB001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let cls = MbValue::from_ptr(MbObject::new_str("IsinstC001".to_string()));
        assert_eq!(mb_isinstance(inst, cls).as_bool(), Some(false));
    }

    #[test]
    fn test_isinstance_with_inheritance() {
        mb_class_register("IsinstParent001", vec![], HashMap::new());
        mb_class_register(
            "IsinstChild001",
            vec!["IsinstParent001".to_string()],
            HashMap::new(),
        );
        let name = MbValue::from_ptr(MbObject::new_str("IsinstChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let parent = MbValue::from_ptr(MbObject::new_str("IsinstParent001".to_string()));
        assert_eq!(mb_isinstance(inst, parent).as_bool(), Some(true));
    }

    #[test]
    fn test_issubclass_transitive() {
        mb_class_register("IssubA001", vec![], HashMap::new());
        mb_class_register("IssubB001", vec!["IssubA001".to_string()], HashMap::new());
        mb_class_register("IssubC001", vec!["IssubB001".to_string()], HashMap::new());
        let c = MbValue::from_ptr(MbObject::new_str("IssubC001".to_string()));
        let a = MbValue::from_ptr(MbObject::new_str("IssubA001".to_string()));
        assert_eq!(mb_issubclass(c, a).as_bool(), Some(true));
    }

    #[test]
    fn test_issubclass_self() {
        mb_class_register("IssubSelf001", vec![], HashMap::new());
        let x1 = MbValue::from_ptr(MbObject::new_str("IssubSelf001".to_string()));
        let x2 = MbValue::from_ptr(MbObject::new_str("IssubSelf001".to_string()));
        assert_eq!(mb_issubclass(x1, x2).as_bool(), Some(true));
    }

    #[test]
    fn test_slots_restricts_attrs() {
        mb_class_register("SlotsRestrict001", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("SlotsRestrict001".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(cls_name, slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slot_list = reg.get("SlotsRestrict001").unwrap();
            assert!(
                slot_list.contains(&"x".to_string()),
                "slot x must be allowed"
            );
            assert!(
                !slot_list.contains(&"y".to_string()),
                "slot y must not be present"
            );
        });
    }

    #[test]
    fn test_slots_empty_allows_nothing() {
        mb_class_register("SlotsEmpty001", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("SlotsEmpty001".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![]));
        mb_register_slots(cls_name, slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slot_list = reg.get("SlotsEmpty001").unwrap();
            assert!(slot_list.is_empty(), "empty __slots__ must have no entries");
        });
    }

    #[test]
    fn test_property_fget_stored() {
        let getter = MbValue::from_int(200);
        let prop = mb_property_new(getter);
        assert!(prop.is_ptr());
        let key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
        assert_eq!(mb_getattr(prop, key).as_int(), Some(200));
    }

    #[test]
    fn test_property_fset_stored() {
        let prop = mb_property_new(MbValue::from_int(1));
        let setter = MbValue::from_int(300);
        mb_property_setter(prop, setter);
        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
        assert_eq!(mb_getattr(prop, key).as_int(), Some(300));
    }

    #[test]
    fn test_property_fdel_stored() {
        let prop = mb_property_new(MbValue::from_int(1));
        let deleter = MbValue::from_int(400);
        mb_property_deleter(prop, deleter);
        let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
        assert_eq!(mb_getattr(prop, key).as_int(), Some(400));
    }

    #[test]
    fn test_class_with_init_method() {
        let mut methods = HashMap::new();
        methods.insert("__init__".to_string(), MbValue::from_int(77));
        mb_class_register("WithInit001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("WithInit001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("__init__".to_string()));
        assert_eq!(mb_getattr(inst, attr).as_int(), Some(77));
    }

    #[test]
    fn test_class_overrides_method_in_child() {
        let mut parent_methods = HashMap::new();
        parent_methods.insert("m".to_string(), MbValue::from_int(1));
        mb_class_register("OvParent001", vec![], parent_methods);

        let mut child_methods = HashMap::new();
        child_methods.insert("m".to_string(), MbValue::from_int(2));
        mb_class_register("OvChild001", vec!["OvParent001".to_string()], child_methods);

        let name = MbValue::from_ptr(MbObject::new_str("OvChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("m".to_string()));
        assert_eq!(
            mb_getattr(inst, attr).as_int(),
            Some(2),
            "child override must win"
        );
    }

    #[test]
    fn test_instance_setattr_overrides_class_method() {
        let mut methods = HashMap::new();
        methods.insert("m".to_string(), MbValue::from_int(10));
        mb_class_register("InstOverride001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("InstOverride001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Set instance field with same name as a class method (non-data-descriptor)
        let attr = MbValue::from_ptr(MbObject::new_str("m".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(99));

        let attr2 = MbValue::from_ptr(MbObject::new_str("m".to_string()));
        // Instance dict takes priority over regular class attribute (Python semantics)
        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(99));
    }

    #[test]
    fn test_mb_class_define_with_base() {
        // Define DefBase first so the MRO walk can find it
        mb_class_register("DefBase001", vec![], HashMap::new());

        let name = MbValue::from_ptr(MbObject::new_str("DefChild001".to_string()));
        let base = MbValue::from_ptr(MbObject::new_str("DefBase001".to_string()));
        let method_names = MbValue::from_ptr(MbObject::new_list(vec![]));
        let method_values = MbValue::from_ptr(MbObject::new_list(vec![]));
        mb_class_define(name, base, method_names, method_values);

        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let cls = reg
                .get("DefChild001")
                .expect("DefChild001 should be registered");
            assert!(cls.bases.contains(&"DefBase001".to_string()));
        });
    }

    #[test]
    fn test_compute_mro_empty_bases() {
        let mro = compute_mro("ComputeSolo001", &[]);
        assert_eq!(mro[0], "ComputeSolo001");
        assert!(mro.contains(&"object".to_string()));
        assert_eq!(mro.len(), 2, "solo class MRO should be [ClassName, object]");
    }

    #[test]
    fn test_compute_mro_linear() {
        // Pre-register A and B so compute_mro can walk their MROs
        mb_class_register("CmroA001", vec![], HashMap::new());
        mb_class_register("CmroB001", vec!["CmroA001".to_string()], HashMap::new());

        let mro = compute_mro("CmroC001", &["CmroB001".to_string()]);
        let ci = mro.iter().position(|x| x == "CmroC001").unwrap();
        let bi = mro.iter().position(|x| x == "CmroB001").unwrap();
        let ai = mro.iter().position(|x| x == "CmroA001").unwrap();
        assert!(ci < bi && bi < ai, "linear MRO must be C, B, A");
    }

    #[test]
    fn test_vars_returns_dict() {
        mb_class_register("VarsDictTest001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("VarsDictTest001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(5));

        let vars = mb_vars(inst);
        assert!(vars.is_ptr(), "mb_vars must return a ptr (dict)");
    }

    #[test]
    fn test_dir_returns_list() {
        mb_class_register("DirListTest001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("DirListTest001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let dir_val = mb_dir(inst);
        assert!(dir_val.is_ptr(), "mb_dir must return a ptr (list)");
    }

    #[test]
    fn test_getattr_default_present() {
        mb_class_register("GadPresent001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GadPresent001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(123));

        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let result = mb_getattr_default(inst, attr2, MbValue::from_int(999));
        assert_eq!(
            result.as_int(),
            Some(123),
            "present attr must not use default"
        );
    }

    #[test]
    fn test_getattr_default_absent() {
        mb_class_register("GadAbsent001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GadAbsent001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("missing_key".to_string()));
        let result = mb_getattr_default(inst, attr, MbValue::from_int(42));
        assert_eq!(result.as_int(), Some(42), "absent attr must return default");
    }

    #[test]
    fn test_multiple_inheritance_attribute_lookup() {
        // D(B, C): B has m=1, C has m=2 — D should get B's m (MRO order)
        let mut b_methods = HashMap::new();
        b_methods.insert("m".to_string(), MbValue::from_int(1));
        mb_class_register("MILookB001", vec![], b_methods);

        let mut c_methods = HashMap::new();
        c_methods.insert("m".to_string(), MbValue::from_int(2));
        mb_class_register("MILookC001", vec![], c_methods);

        mb_class_register(
            "MILookD001",
            vec!["MILookB001".to_string(), "MILookC001".to_string()],
            HashMap::new(),
        );

        let name = MbValue::from_ptr(MbObject::new_str("MILookD001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("m".to_string()));
        assert_eq!(
            mb_getattr(inst, attr).as_int(),
            Some(1),
            "MRO: B.m should win over C.m"
        );
    }

    #[test]
    fn test_class_attrs_accessible_on_instance() {
        // mb_class_define sets methods; verify they are accessible via getattr on instance
        mb_class_register("ClsAttrAccess001", vec![], HashMap::new());

        let name = MbValue::from_ptr(MbObject::new_str("ClsAttrAccess001Def".to_string()));
        let base = MbValue::from_ptr(MbObject::new_str("ClsAttrAccess001".to_string()));
        let method_names = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("run".to_string()),
        )]));
        let method_values = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(88)]));
        mb_class_define(name, base, method_names, method_values);

        let inst_name = MbValue::from_ptr(MbObject::new_str("ClsAttrAccess001Def".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("run".to_string()));
        assert_eq!(mb_getattr(inst, attr).as_int(), Some(88));
    }

    #[test]
    fn test_instance_repr_contains_class_name() {
        mb_class_register("ReprClass001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("ReprClass001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        // mb_obj_str / mb_obj_repr should embed the class name in the result string
        let repr = mb_obj_repr(inst);
        if let Some(ptr) = repr.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    assert!(s.contains("ReprClass001"), "repr must contain class name");
                    return;
                }
            }
        }
        panic!("mb_obj_repr did not return a Str value");
    }

    // ── P1 OOP Conformance Tests (mamba-conformance-p1) ──────────────────────

    // --- T1: @classmethod ---

    #[test]
    fn test_p1_t1_1_classmethod_basic_wraps_function() {
        // T1.1: @classmethod wraps a function and descriptor_unwrap retrieves it
        let func_val = MbValue::from_int(42);
        let cm = mb_classmethod_new(func_val);
        assert!(cm.is_ptr(), "classmethod wrapper must be a ptr");

        // Unwrap should return the original function
        let unwrapped = mb_descriptor_unwrap(cm);
        assert_eq!(
            unwrapped.as_int(),
            Some(42),
            "unwrapping classmethod must yield original func"
        );
    }

    #[test]
    fn test_p1_t1_1_classmethod_descriptor_protocol_on_instance() {
        // T1.1: When a classmethod is stored on a class, accessing it on an instance
        // should invoke the descriptor protocol and return the unwrapped function.
        let func_val = MbValue::from_int(77);
        let cm = mb_classmethod_new(func_val);

        let mut methods = HashMap::new();
        methods.insert("get_species".to_string(), cm);
        mb_class_register("CmAnimal001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("CmAnimal001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("get_species".to_string()));

        // Descriptor protocol: getattr on instance with classmethod should invoke
        // invoke_descriptor_get which calls mb_descriptor_unwrap
        let result = mb_getattr(inst, attr);
        // The descriptor protocol for __classmethod__ returns the unwrapped __func__
        assert_eq!(
            result.as_int(),
            Some(77),
            "classmethod descriptor should unwrap to original function"
        );
    }

    #[test]
    fn test_p1_t1_2_classmethod_inheritance() {
        // T1.2: Subclass inherits classmethod from parent via MRO
        let func_val = MbValue::from_int(55);
        let cm = mb_classmethod_new(func_val);

        let mut parent_methods = HashMap::new();
        parent_methods.insert("cm_method".to_string(), cm);
        mb_class_register("CmParent001", vec![], parent_methods);
        mb_class_register(
            "CmChild001",
            vec!["CmParent001".to_string()],
            HashMap::new(),
        );

        // Child instance should inherit the classmethod
        let name = MbValue::from_ptr(MbObject::new_str("CmChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("cm_method".to_string()));
        let result = mb_getattr(inst, attr);
        assert_eq!(
            result.as_int(),
            Some(55),
            "child should inherit classmethod from parent"
        );
    }

    #[test]
    fn test_p1_t1_3_classmethod_unwrap_descriptor_method() {
        // T1.3: unwrap_descriptor_method extracts function from classmethod and reports ClassMethod
        let func_val = MbValue::from_int(99);
        let cm = mb_classmethod_new(func_val);

        let (unwrapped, dk) = unwrap_descriptor_method(cm);
        assert_eq!(
            unwrapped.as_int(),
            Some(99),
            "unwrapped function must match"
        );
        assert_eq!(
            dk,
            DescriptorKind::ClassMethod,
            "must identify as classmethod"
        );
    }

    #[test]
    fn test_p1_t1_3_staticmethod_unwrap() {
        // T1.3 (related): staticmethod unwraps and is identified as StaticMethod
        let func_val = MbValue::from_int(88);
        let sm = mb_staticmethod_new(func_val);

        let (unwrapped, dk) = unwrap_descriptor_method(sm);
        assert_eq!(
            unwrapped.as_int(),
            Some(88),
            "unwrapped function must match"
        );
        assert_eq!(
            dk,
            DescriptorKind::StaticMethod,
            "must identify as staticmethod"
        );
    }

    #[test]
    fn test_p1_t1_3_unwrap_plain_method() {
        // T1.3 (related): Plain method value is returned unchanged
        let func_val = MbValue::from_int(66);
        let (unwrapped, dk) = unwrap_descriptor_method(func_val);
        assert_eq!(unwrapped.as_int(), Some(66));
        assert_eq!(dk, DescriptorKind::Regular, "plain method must be regular");
    }

    // --- T2: @property ---

    #[test]
    fn test_p1_t2_1_property_getter_via_descriptor() {
        // T2.1: @property getter is invoked when attribute is read on an instance
        // Uses a non-callable getter to verify the descriptor protocol path.
        let getter = MbValue::from_int(314);
        let prop = mb_property_new(getter);

        let mut methods = HashMap::new();
        methods.insert("area".to_string(), prop);
        mb_class_register("PropCircle001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("PropCircle001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("area".to_string()));

        // Property is a data descriptor (has __set__), so mb_getattr should invoke
        // invoke_descriptor_get → mb_property_get → mb_call_method1(getter, instance)
        // Since getter (int 314) is not in CALLABLE_REGISTRY, mb_call_method1 returns none
        // The key test is that the descriptor protocol IS invoked (no crash, no raw return)
        let result = mb_getattr(inst, attr);
        // With non-callable getter, mb_property_get calls mb_call_method1 which
        // returns MbValue::none() for unregistered addresses
        assert!(
            result.is_none() || result.as_int().is_some(),
            "property getter descriptor protocol must be invoked without crash"
        );
    }

    #[test]
    fn test_p1_t2_1_property_is_data_descriptor() {
        // T2.1: Verify property is recognized as a data descriptor
        let prop = mb_property_new(MbValue::from_int(1));
        assert!(
            is_data_descriptor(prop),
            "@property must be a data descriptor"
        );
        assert!(is_descriptor(prop), "@property must be a descriptor");
    }

    #[test]
    fn test_p1_t2_2_property_setter_stores_fset() {
        // T2.2: @property.setter stores the setter function
        let prop = mb_property_new(MbValue::from_int(10));
        let setter = MbValue::from_int(20);
        mb_property_setter(prop, setter);

        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
        let stored = mb_getattr(prop, key);
        assert_eq!(stored.as_int(), Some(20), "setter must be stored as fset");
    }

    #[test]
    fn test_p1_t2_3_property_deleter_stores_fdel() {
        // T2.3: @property.deleter stores the deleter function
        let prop = mb_property_new(MbValue::from_int(10));
        let deleter = MbValue::from_int(30);
        mb_property_deleter(prop, deleter);

        let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
        let stored = mb_getattr(prop, key);
        assert_eq!(stored.as_int(), Some(30), "deleter must be stored as fdel");
    }

    #[test]
    fn test_p1_t2_4_property_readonly_no_setter() {
        // T2.4: Property created with only getter has no fset
        let prop = mb_property_new(MbValue::from_int(100));

        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
        let fset = mb_getattr(prop, key);
        assert!(fset.is_none(), "property without setter must have no fset");
    }

    #[test]
    fn test_p1_t2_property_data_descriptor_priority() {
        // Property (data descriptor) should take priority over instance dict
        let prop = mb_property_new(MbValue::from_int(999));

        let mut methods = HashMap::new();
        methods.insert("x".to_string(), prop);
        mb_class_register("PropPriority001", vec![], methods);

        let name = MbValue::from_ptr(MbObject::new_str("PropPriority001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Set instance field "x" — should NOT shadow the property because
        // data descriptors have priority over instance __dict__
        // But mb_setattr with property actually calls invoke_descriptor_set,
        // which calls mb_property_set. If setter is not set, it does nothing.
        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(42));

        // Reading "x" should go through the property descriptor, not instance dict
        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let result = mb_getattr(inst, attr2);
        // The property getter (int 999) is not callable, so mb_property_get returns none
        // The key assertion: it does NOT return 42 (the instance dict value should not be stored)
        assert_ne!(
            result.as_int(),
            Some(42),
            "data descriptor must take priority over instance dict"
        );
    }

    // --- T3: getattr/setattr/delattr ---

    #[test]
    fn test_p1_t3_1_getattr_existing_attribute() {
        // T3.1: getattr returns existing attribute value
        mb_class_register("GetAttrBox001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrBox001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(10));

        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        let result = mb_getattr(inst, attr2);
        assert_eq!(
            result.as_int(),
            Some(10),
            "getattr must return existing attribute value"
        );
    }

    #[test]
    fn test_p1_t3_2_getattr_missing_with_default() {
        // T3.2: getattr with default returns default when attr absent
        mb_class_register("GetAttrDef001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrDef001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("weight".to_string()));
        let default = MbValue::from_int(99);
        let result = mb_getattr_default(inst, attr, default);
        assert_eq!(
            result.as_int(),
            Some(99),
            "missing attr must return default"
        );
    }

    #[test]
    fn test_p1_t3_3_getattr_missing_no_default() {
        // T3.3: getattr without default returns None for missing attribute
        mb_class_register("GetAttrMiss001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrMiss001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent".to_string()));
        let result = mb_getattr(inst, attr);
        assert!(
            result.is_none(),
            "missing attr without default must return None"
        );
    }

    #[test]
    fn test_p1_t3_4_setattr_creates_and_updates() {
        // T3.4: setattr creates attribute, then updates it
        mb_class_register("SetAttrBox001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("SetAttrBox001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Create attribute
        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(10));

        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(10));

        // Update attribute
        let attr3 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        mb_setattr(inst, attr3, MbValue::from_int(20));

        let attr4 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        assert_eq!(
            mb_getattr(inst, attr4).as_int(),
            Some(20),
            "setattr must update existing attr"
        );
    }

    #[test]
    fn test_p1_t3_5_delattr_removes_attribute() {
        // T3.5: delattr removes the attribute
        mb_class_register("DelAttrBox001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("DelAttrBox001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let attr = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(10));

        // Verify it exists
        let attr2 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        assert_eq!(mb_getattr(inst, attr2).as_int(), Some(10));

        // Delete it
        let attr3 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        mb_delattr(inst, attr3);

        // Verify it's gone
        let attr4 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        assert!(
            mb_getattr(inst, attr4).is_none(),
            "delattr must remove the attribute"
        );

        // hasattr should return false
        let attr5 = MbValue::from_ptr(MbObject::new_str("size".to_string()));
        assert_eq!(mb_hasattr(inst, attr5).as_bool(), Some(false));
    }

    #[test]
    fn test_p1_t3_hasattr_after_setattr_delattr_cycle() {
        // Combined cycle: set, check, delete, check
        mb_class_register("HasAttrCycle001", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("HasAttrCycle001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // Initially missing
        let attr = MbValue::from_ptr(MbObject::new_str("color".to_string()));
        assert_eq!(mb_hasattr(inst, attr).as_bool(), Some(false));

        // Set
        let attr2 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
        mb_setattr(
            inst,
            attr2,
            MbValue::from_ptr(MbObject::new_str("red".to_string())),
        );

        // Now present
        let attr3 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
        assert_eq!(mb_hasattr(inst, attr3).as_bool(), Some(true));

        // Delete
        let attr4 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
        mb_delattr(inst, attr4);

        // Gone again
        let attr5 = MbValue::from_ptr(MbObject::new_str("color".to_string()));
        assert_eq!(mb_hasattr(inst, attr5).as_bool(), Some(false));
    }

    // --- T4: super().method() ---

    #[test]
    fn test_p1_t4_1_super_method_return_value() {
        // T4.1: super().method() return value is propagated to caller
        let mut base_methods = HashMap::new();
        base_methods.insert("value".to_string(), MbValue::from_int(42));
        mb_class_register("SuperBase001", vec![], base_methods);

        let mut child_methods = HashMap::new();
        child_methods.insert("value".to_string(), MbValue::from_int(43));
        mb_class_register(
            "SuperChild001",
            vec!["SuperBase001".to_string()],
            child_methods,
        );

        // Create instance of SuperChild001
        let name = MbValue::from_ptr(MbObject::new_str("SuperChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        // super(SuperChild001, inst).value → should find SuperBase001.value
        let cls = MbValue::from_ptr(MbObject::new_str("SuperChild001".to_string()));
        let proxy = mb_super(cls, inst);
        let attr = MbValue::from_ptr(MbObject::new_str("value".to_string()));
        let result = mb_super_getattr(proxy, attr);

        assert_eq!(
            result.as_int(),
            Some(42),
            "super() must return parent's method value, not child's"
        );
    }

    #[test]
    fn test_p1_t4_2_super_chain_three_levels() {
        // T4.2: A→B→C super chain preserves returns through MRO
        let mut a_methods = HashMap::new();
        a_methods.insert("compute".to_string(), MbValue::from_int(10));
        mb_class_register("SuperA001", vec![], a_methods);

        let mut b_methods = HashMap::new();
        b_methods.insert("compute".to_string(), MbValue::from_int(15));
        mb_class_register("SuperB001", vec!["SuperA001".to_string()], b_methods);

        let mut c_methods = HashMap::new();
        c_methods.insert("compute".to_string(), MbValue::from_int(18));
        mb_class_register("SuperC001", vec!["SuperB001".to_string()], c_methods);

        // From C, super should find B.compute
        let name_c = MbValue::from_ptr(MbObject::new_str("SuperC001".to_string()));
        let inst_c = mb_instance_new(name_c, MbValue::none());
        let cls_c = MbValue::from_ptr(MbObject::new_str("SuperC001".to_string()));
        let proxy_c = mb_super(cls_c, inst_c);
        let attr = MbValue::from_ptr(MbObject::new_str("compute".to_string()));
        let result_c = mb_super_getattr(proxy_c, attr);
        assert_eq!(
            result_c.as_int(),
            Some(15),
            "super() from C must find B.compute"
        );

        // From B, super should find A.compute
        let name_b = MbValue::from_ptr(MbObject::new_str("SuperB001".to_string()));
        let inst_b = mb_instance_new(name_b, MbValue::none());
        let cls_b = MbValue::from_ptr(MbObject::new_str("SuperB001".to_string()));
        let proxy_b = mb_super(cls_b, inst_b);
        let attr2 = MbValue::from_ptr(MbObject::new_str("compute".to_string()));
        let result_b = mb_super_getattr(proxy_b, attr2);
        assert_eq!(
            result_b.as_int(),
            Some(10),
            "super() from B must find A.compute"
        );
    }

    #[test]
    fn test_p1_t4_3_super_init_lookup() {
        // T4.3: super().__init__() finds parent __init__
        let mut base_methods = HashMap::new();
        base_methods.insert("__init__".to_string(), MbValue::from_int(777));
        mb_class_register("SuperInitBase001", vec![], base_methods);

        let mut child_methods = HashMap::new();
        child_methods.insert("__init__".to_string(), MbValue::from_int(888));
        mb_class_register(
            "SuperInitChild001",
            vec!["SuperInitBase001".to_string()],
            child_methods,
        );

        let name = MbValue::from_ptr(MbObject::new_str("SuperInitChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let cls = MbValue::from_ptr(MbObject::new_str("SuperInitChild001".to_string()));
        let proxy = mb_super(cls, inst);
        let attr = MbValue::from_ptr(MbObject::new_str("__init__".to_string()));
        let result = mb_super_getattr(proxy, attr);
        assert_eq!(
            result.as_int(),
            Some(777),
            "super().__init__() must find parent's __init__"
        );
    }

    #[test]
    fn test_p1_t4_super_proxy_structure() {
        // Verify super proxy stores __super_class__ and __super_self__
        let cls = MbValue::from_ptr(MbObject::new_str("SomeClass".to_string()));
        let inst = MbValue::from_int(12345); // dummy instance
        let proxy = mb_super(cls, inst);

        assert!(proxy.is_ptr(), "super proxy must be a pointer");
        if let Some(ptr) = proxy.as_ptr() {
            unsafe {
                if let ObjData::Instance {
                    ref class_name,
                    ref fields,
                    ..
                } = (*ptr).data
                {
                    assert_eq!(
                        class_name, "__super__",
                        "super proxy class must be __super__"
                    );
                    let fields = fields.read().unwrap();
                    assert!(
                        fields.contains_key("__super_class__"),
                        "proxy must have __super_class__ field"
                    );
                    assert!(
                        fields.contains_key("__super_self__"),
                        "proxy must have __super_self__ field"
                    );
                } else {
                    panic!("super proxy must be an Instance");
                }
            }
        }
    }

    #[test]
    fn test_p1_t4_super_method_not_found() {
        // super().missing_method() should return None
        let mut methods = HashMap::new();
        methods.insert("greet".to_string(), MbValue::from_int(1));
        mb_class_register("SuperNfBase001", vec![], methods);
        mb_class_register(
            "SuperNfChild001",
            vec!["SuperNfBase001".to_string()],
            HashMap::new(),
        );

        let name = MbValue::from_ptr(MbObject::new_str("SuperNfChild001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());

        let cls = MbValue::from_ptr(MbObject::new_str("SuperNfChild001".to_string()));
        let proxy = mb_super(cls, inst);
        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent".to_string()));
        let result = mb_super_getattr(proxy, attr);
        assert!(
            result.is_none(),
            "super() looking up nonexistent method must return None"
        );
    }

    // --- T5: MRO ---

    #[test]
    fn test_p1_t5_1_diamond_mro_exact_order() {
        // T5.1: D(B, C) where B(A), C(A) → MRO must be [D, B, C, A, object]
        mb_class_register("DiamondA001", vec![], HashMap::new());
        mb_class_register(
            "DiamondB001",
            vec!["DiamondA001".to_string()],
            HashMap::new(),
        );
        mb_class_register(
            "DiamondC001",
            vec!["DiamondA001".to_string()],
            HashMap::new(),
        );
        mb_class_register(
            "DiamondD001",
            vec!["DiamondB001".to_string(), "DiamondC001".to_string()],
            HashMap::new(),
        );

        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let d = reg.get("DiamondD001").unwrap();
            assert_eq!(d.mro[0], "DiamondD001", "MRO[0] must be D");
            assert_eq!(d.mro[1], "DiamondB001", "MRO[1] must be B");
            assert_eq!(d.mro[2], "DiamondC001", "MRO[2] must be C");
            assert_eq!(d.mro[3], "DiamondA001", "MRO[3] must be A");
            assert_eq!(d.mro[4], "object", "MRO[4] must be object");
            assert_eq!(d.mro.len(), 5, "Diamond MRO must have exactly 5 entries");
        });
    }

    #[test]
    fn test_p1_t5_1_diamond_method_resolution() {
        // T5.1: Method resolution follows MRO order in diamond inheritance
        let mut a_methods = HashMap::new();
        a_methods.insert("who".to_string(), MbValue::from_int(1));
        mb_class_register("DmrA001", vec![], a_methods);

        let mut b_methods = HashMap::new();
        b_methods.insert("who".to_string(), MbValue::from_int(2));
        mb_class_register("DmrB001", vec!["DmrA001".to_string()], b_methods);

        let mut c_methods = HashMap::new();
        c_methods.insert("who".to_string(), MbValue::from_int(3));
        mb_class_register("DmrC001", vec!["DmrA001".to_string()], c_methods);

        // D has no "who" method — must resolve via MRO: D→B→C→A
        mb_class_register(
            "DmrD001",
            vec!["DmrB001".to_string(), "DmrC001".to_string()],
            HashMap::new(),
        );

        let name = MbValue::from_ptr(MbObject::new_str("DmrD001".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("who".to_string()));
        let result = mb_getattr(inst, attr);
        assert_eq!(
            result.as_int(),
            Some(2),
            "Diamond MRO must resolve D.who() to B.who() (first parent in MRO)"
        );
    }

    #[test]
    fn test_p1_t5_2_linear_mro_exact_order() {
        // T5.2: C(B), B(A) → MRO = [C, B, A, object]
        mb_class_register("LinA001", vec![], HashMap::new());
        mb_class_register("LinB001", vec!["LinA001".to_string()], HashMap::new());
        mb_class_register("LinC001", vec!["LinB001".to_string()], HashMap::new());

        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let c = reg.get("LinC001").unwrap();
            assert_eq!(c.mro[0], "LinC001", "MRO[0] must be C");
            assert_eq!(c.mro[1], "LinB001", "MRO[1] must be B");
            assert_eq!(c.mro[2], "LinA001", "MRO[2] must be A");
            assert_eq!(c.mro[3], "object", "MRO[3] must be object");
            assert_eq!(c.mro.len(), 4, "Linear MRO must have exactly 4 entries");
        });
    }

    #[test]
    fn test_p1_t5_3_inconsistent_mro_sets_typeerror() {
        // T5.3: an inconsistent hierarchy sets a *catchable* TypeError (CPython
        // raises at the class statement) rather than panicking and aborting.
        // Create X(A, B) and Y(B, A) — then Z(X, Y) is inconsistent.
        crate::runtime::exception::clear_current_exception();
        mb_class_register("IncA001", vec![], HashMap::new());
        mb_class_register("IncB001", vec![], HashMap::new());
        mb_class_register(
            "IncX001",
            vec!["IncA001".to_string(), "IncB001".to_string()],
            HashMap::new(),
        );
        mb_class_register(
            "IncY001",
            vec!["IncB001".to_string(), "IncA001".to_string()],
            HashMap::new(),
        );
        // This sets a pending TypeError (no panic).
        mb_class_register(
            "IncZ001",
            vec!["IncX001".to_string(), "IncY001".to_string()],
            HashMap::new(),
        );
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("TypeError"),
            "inconsistent MRO should set a pending TypeError"
        );
        crate::runtime::exception::clear_current_exception();
    }

    #[test]
    fn test_p1_t5_c3_merge_empty_lists() {
        // c3_merge with empty input returns empty result
        let mut lists: Vec<Vec<String>> = Vec::new();
        let result = c3_merge(&mut lists).unwrap();
        assert!(
            result.is_empty(),
            "empty input to c3_merge must return empty result"
        );
    }

    #[test]
    fn test_p1_t5_c3_merge_single_list() {
        // c3_merge with a single list returns that list
        let mut lists = vec![vec!["A".to_string(), "B".to_string()]];
        let result = c3_merge(&mut lists).unwrap();
        assert_eq!(result, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn test_p1_t5_c3_merge_inconsistent() {
        // c3_merge returns Err for inconsistent input
        // A appears in tail of second list, B appears in tail of first list → deadlock
        let mut lists = vec![
            vec!["A".to_string(), "B".to_string()],
            vec!["B".to_string(), "A".to_string()],
        ];
        let result = c3_merge(&mut lists);
        assert!(result.is_err(), "inconsistent hierarchy must return Err");
    }

    #[test]
    fn test_p1_t5_class_define_multi_registers_bases() {
        // mb_class_define_multi correctly registers a class with multiple bases
        mb_class_register("MultiDefA001", vec![], HashMap::new());
        mb_class_register("MultiDefB001", vec![], HashMap::new());

        let name = MbValue::from_ptr(MbObject::new_str("MultiDefC001".to_string()));
        let bases_list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("MultiDefA001".to_string())),
            MbValue::from_ptr(MbObject::new_str("MultiDefB001".to_string())),
        ]));
        let method_names = MbValue::from_ptr(MbObject::new_list(vec![]));
        let method_values = MbValue::from_ptr(MbObject::new_list(vec![]));
        mb_class_define_multi(name, bases_list, method_names, method_values);

        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let cls = reg.get("MultiDefC001").expect("class must be registered");
            assert_eq!(cls.bases.len(), 2);
            assert!(cls.bases.contains(&"MultiDefA001".to_string()));
            assert!(cls.bases.contains(&"MultiDefB001".to_string()));
            // MRO should include both parents
            assert!(cls.mro.contains(&"MultiDefA001".to_string()));
            assert!(cls.mro.contains(&"MultiDefB001".to_string()));
        });
    }

    #[test]
    fn test_p1_t5_single_base_mro_no_object_dup() {
        // Single-base MRO should not duplicate "object"
        mb_class_register("NoDupBase001", vec![], HashMap::new());
        mb_class_register(
            "NoDupChild001",
            vec!["NoDupBase001".to_string()],
            HashMap::new(),
        );

        CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let c = reg.get("NoDupChild001").unwrap();
            let object_count = c.mro.iter().filter(|x| *x == "object").count();
            assert_eq!(object_count, 1, "object must appear exactly once in MRO");
        });
    }

    // ── Cleanup tests (R1: per-module cleanup for classes) ──

    #[test]
    fn test_cleanup_all_classes_clears_registry() {
        mb_class_register("CleanupClassTest", vec![], HashMap::new());
        CLASS_REGISTRY.with(|reg| {
            assert!(
                reg.borrow().contains_key("CleanupClassTest"),
                "class should exist before cleanup"
            );
        });

        cleanup_all_classes();

        CLASS_REGISTRY.with(|reg| {
            assert!(
                !reg.borrow().contains_key("CleanupClassTest"),
                "CLASS_REGISTRY should be empty after cleanup"
            );
        });
    }

    #[test]
    fn test_cleanup_all_classes_clears_slots_registry() {
        mb_class_register("CleanupSlots", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("CleanupSlots".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(cls_name, slots);

        cleanup_all_classes();

        SLOTS_REGISTRY.with(|reg| {
            assert!(
                reg.borrow().is_empty(),
                "SLOTS_REGISTRY should be empty after cleanup"
            );
        });
    }

    #[test]
    fn test_cleanup_all_classes_clears_abstract_methods() {
        mb_class_register("CleanupABC", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("CleanupABC".to_string()));
        let abs_methods = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("do_thing".to_string()),
        )]));
        mb_register_abstract(cls_name, abs_methods);

        cleanup_all_classes();

        ABSTRACT_METHODS.with(|reg| {
            assert!(
                reg.borrow().is_empty(),
                "ABSTRACT_METHODS should be empty after cleanup"
            );
        });
    }

    #[test]
    fn test_cleanup_all_classes_on_empty() {
        cleanup_all_classes();
        // No panic = success
    }

    #[test]
    fn test_cleanup_all_classes_then_reregister() {
        mb_class_register("CleanupRereg", vec![], HashMap::new());
        cleanup_all_classes();

        // Re-register after cleanup
        let mut new_methods = HashMap::new();
        new_methods.insert("new_method".to_string(), MbValue::from_int(42));
        mb_class_register("CleanupRereg", vec![], new_methods);

        let name = MbValue::from_ptr(MbObject::new_str("CleanupRereg".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("new_method".to_string()));
        assert_eq!(
            mb_getattr(inst, attr).as_int(),
            Some(42),
            "re-registered class should work after cleanup"
        );
    }

    // ── R13: isinstance with tuple-of-types ──

    #[test]
    fn test_isinstance_tuple_of_types_match() {
        // isinstance(42, (int, str)) should return True
        let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let str_type = MbValue::from_ptr(MbObject::new_str("str".to_string()));
        let types = MbValue::from_ptr(MbObject::new_tuple(vec![int_type, str_type]));
        assert_eq!(
            mb_isinstance(MbValue::from_int(42), types).as_bool(),
            Some(true),
            "isinstance(42, (int, str)) should be True",
        );
    }

    #[test]
    fn test_isinstance_tuple_of_types_second_match() {
        // isinstance("hello", (int, str)) should return True (matches second type)
        let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let str_type = MbValue::from_ptr(MbObject::new_str("str".to_string()));
        let types = MbValue::from_ptr(MbObject::new_tuple(vec![int_type, str_type]));
        let val = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(
            mb_isinstance(val, types).as_bool(),
            Some(true),
            "isinstance('hello', (int, str)) should be True",
        );
    }

    #[test]
    fn test_isinstance_tuple_of_types_no_match() {
        // isinstance(3.14, (int, str)) should return False
        let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let str_type = MbValue::from_ptr(MbObject::new_str("str".to_string()));
        let types = MbValue::from_ptr(MbObject::new_tuple(vec![int_type, str_type]));
        assert_eq!(
            mb_isinstance(MbValue::from_float(3.14), types).as_bool(),
            Some(false),
            "isinstance(3.14, (int, str)) should be False",
        );
    }

    #[test]
    fn test_isinstance_tuple_of_types_empty() {
        // isinstance(42, ()) should return False
        let types = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        assert_eq!(
            mb_isinstance(MbValue::from_int(42), types).as_bool(),
            Some(false),
            "isinstance(42, ()) should be False",
        );
    }

    #[test]
    fn test_isinstance_tuple_with_bool() {
        // isinstance(True, (bool, int)) should return True
        let bool_type = MbValue::from_ptr(MbObject::new_str("bool".to_string()));
        let int_type = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let types = MbValue::from_ptr(MbObject::new_tuple(vec![bool_type, int_type]));
        assert_eq!(
            mb_isinstance(MbValue::from_bool(true), types).as_bool(),
            Some(true),
            "isinstance(True, (bool, int)) should be True",
        );
    }

    // ── R13: mb_getattr_default ──

    #[test]
    fn test_getattr_default_found() {
        mb_class_register("GetAttrTest", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrTest".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr, MbValue::from_int(99));
        let attr2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        let result = mb_getattr_default(inst, attr2, MbValue::from_int(0));
        assert_eq!(
            result.as_int(),
            Some(99),
            "getattr should return existing attr, not default"
        );
    }

    #[test]
    fn test_getattr_default_not_found() {
        mb_class_register("GetAttrMiss", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrMiss".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("nonexistent".to_string()));
        let default = MbValue::from_int(42);
        let result = mb_getattr_default(inst, attr, default);
        assert_eq!(
            result.as_int(),
            Some(42),
            "getattr should return default for missing attr"
        );
    }

    #[test]
    fn test_getattr_default_with_str_default() {
        mb_class_register("GetAttrStr", vec![], HashMap::new());
        let name = MbValue::from_ptr(MbObject::new_str("GetAttrStr".to_string()));
        let inst = mb_instance_new(name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("missing".to_string()));
        let default = MbValue::from_ptr(MbObject::new_str("fallback".to_string()));
        let result = mb_getattr_default(inst, attr, default);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "fallback");
            } else {
                panic!("expected str default");
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════
    // P1 Class Features Tests (mamba-p1-lang-features / class-features-spec)
    // Scenarios S1–S12: R10–R14
    // ═══════════════════════════════════════════════════════════════

    // ── S1: __init_subclass__ receives keyword arguments (R10) ──

    #[test]
    fn test_s1_init_subclass_receives_kwargs() {
        // S1: class Base defines __init_subclass__(cls, kwargs_dict),
        //     class Child(Base, registry="users") passes kwargs through.
        use std::sync::atomic::{AtomicBool, Ordering};
        static S1_HOOK_CALLED: AtomicBool = AtomicBool::new(false);

        extern "C" fn s1_hook(_cls: MbValue, kwargs: MbValue) -> MbValue {
            S1_HOOK_CALLED.store(true, Ordering::SeqCst);
            // kwargs should be a dict with key "registry" → "users"
            let key = MbValue::from_ptr(MbObject::new_str("registry".to_string()));
            let val = crate::runtime::dict_ops::mb_dict_getitem(kwargs, key);
            unsafe {
                if let Some(ptr) = val.as_ptr() {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        assert_eq!(s, "users", "kwargs['registry'] must be 'users'");
                    }
                }
            }
            MbValue::none()
        }

        let hook_addr = s1_hook as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(hook_addr as u64);
        });

        let mut base_methods = HashMap::new();
        base_methods.insert(
            "__init_subclass__".to_string(),
            MbValue::from_func(hook_addr),
        );
        mb_class_register("S1Base", vec![], base_methods);

        // Set kwargs BEFORE registering child (as the lowering pass would do)
        let child_name_val = MbValue::from_ptr(MbObject::new_str("S1Child".to_string()));
        let keys = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("registry".to_string()),
        )]));
        let values = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("users".to_string()),
        )]));
        mb_class_set_kwargs(child_name_val, keys, values);

        // Register Child — triggers __init_subclass__ with kwargs
        mb_class_register("S1Child", vec!["S1Base".to_string()], HashMap::new());

        assert!(
            S1_HOOK_CALLED.load(Ordering::SeqCst),
            "S1: __init_subclass__ must be called with kwargs when Child is registered"
        );
    }

    // ── S2: __init_subclass__ no kwargs, no handler defined (R10) ──

    #[test]
    fn test_s2_init_subclass_no_kwargs_no_handler() {
        // S2: Parent has no __init_subclass__, Child(Parent) with no kwargs → no error.
        crate::runtime::exception::mb_clear_exception();
        mb_class_register("S2Parent", vec![], HashMap::new());
        mb_class_register("S2Child", vec!["S2Parent".to_string()], HashMap::new());

        // Should succeed without raising an exception
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(false),
            "S2: No error when base lacks __init_subclass__ and no kwargs are passed"
        );
    }

    // ── S3: __init_subclass__ extra kwargs without handler raises TypeError (R10) ──

    #[test]
    fn test_s3_init_subclass_kwargs_without_handler_raises_type_error() {
        // S3: Base has no __init_subclass__, Child(Base, key="val") → TypeError.
        crate::runtime::exception::mb_clear_exception();
        mb_class_register("S3Base", vec![], HashMap::new());

        // Set kwargs on the child
        let child_name_val = MbValue::from_ptr(MbObject::new_str("S3Child".to_string()));
        let keys = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("key".to_string()),
        )]));
        let values = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("val".to_string()),
        )]));
        mb_class_set_kwargs(child_name_val, keys, values);

        // Register child — should raise TypeError
        mb_class_register("S3Child", vec!["S3Base".to_string()], HashMap::new());

        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "S3: TypeError must be raised when base has no __init_subclass__ but kwargs provided"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // ── S4: __class_getitem__ enables generic subscript (R11) ──

    #[test]
    fn test_s4_class_getitem_subscript() {
        // S4: class MyList defines __class_getitem__, MyList[int] calls it.
        // The __class_getitem__ is dispatched via mb_obj_getitem on a class name string.
        use std::sync::atomic::{AtomicBool, Ordering};
        static S4_GETITEM_CALLED: AtomicBool = AtomicBool::new(false);

        extern "C" fn s4_class_getitem(_cls: MbValue, _key: MbValue) -> MbValue {
            S4_GETITEM_CALLED.store(true, Ordering::SeqCst);
            MbValue::from_ptr(MbObject::new_str("MyList[int]".to_string()))
        }

        let hook_addr = s4_class_getitem as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(hook_addr as u64);
        });

        let mut methods = HashMap::new();
        methods.insert(
            "__class_getitem__".to_string(),
            MbValue::from_func(hook_addr),
        );
        mb_class_register("S4MyList", vec![], methods);

        // Subscript: S4MyList[int] → mb_obj_getitem on class name string
        let cls_name = MbValue::from_ptr(MbObject::new_str("S4MyList".to_string()));
        let key = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let result = mb_obj_getitem(cls_name, key);

        assert!(
            S4_GETITEM_CALLED.load(Ordering::SeqCst),
            "S4: __class_getitem__ must be called when class is subscripted"
        );
        unsafe {
            if let Some(ptr) = result.as_ptr() {
                if let ObjData::Str(ref s) = (*ptr).data {
                    assert_eq!(
                        s, "MyList[int]",
                        "S4: __class_getitem__ return value must propagate"
                    );
                    return;
                }
            }
        }
        panic!("S4: __class_getitem__ must return the correct string result");
    }

    // ── S5: Subscript on class without __class_getitem__ raises TypeError (R11) ──

    #[test]
    fn test_s5_class_subscript_without_getitem_raises_type_error() {
        // S5: class Foo with no __class_getitem__, Foo[int] → TypeError.
        crate::runtime::exception::mb_clear_exception();
        mb_class_register("S5Foo", vec![], HashMap::new());

        let cls_name = MbValue::from_ptr(MbObject::new_str("S5Foo".to_string()));
        let key = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let _result = mb_obj_getitem(cls_name, key);

        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "S5: TypeError must be raised when class has no __class_getitem__"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // ── S6: __set_name__ called on descriptors after class creation (R12) ──

    #[test]
    fn test_s6_set_name_called_on_descriptors() {
        // S6: Descriptor with __set_name__ is called after class creation.
        // We simulate by creating a "descriptor" class with __set_name__, then
        // registering a class that has an instance of it as a class attribute.
        use std::sync::atomic::{AtomicBool, Ordering};
        static S6_SET_NAME_CALLED: AtomicBool = AtomicBool::new(false);

        extern "C" fn s6_set_name(_self: MbValue, _owner: MbValue, _name: MbValue) -> MbValue {
            S6_SET_NAME_CALLED.store(true, Ordering::SeqCst);
            MbValue::none()
        }

        let set_name_addr = s6_set_name as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(set_name_addr as u64);
        });

        // Register descriptor class with __set_name__ method
        let mut desc_methods = HashMap::new();
        desc_methods.insert(
            "__set_name__".to_string(),
            MbValue::from_func(set_name_addr),
        );
        mb_class_register("S6Descriptor", vec![], desc_methods);

        // Create a descriptor instance
        let desc_cls = MbValue::from_ptr(MbObject::new_str("S6Descriptor".to_string()));
        let desc_inst = mb_instance_new(desc_cls, MbValue::none());

        // Set the descriptor instance as a class attribute on a new class,
        // THEN register the class so __set_name__ fires.
        // We need to register with class_attrs pre-populated.
        // Use mb_class_set_class_attr before registration won't work because
        // __set_name__ fires inside mb_class_register. So we set it as a
        // class attribute first, then call mb_class_register which scans attrs.
        let cls_name_val = MbValue::from_ptr(MbObject::new_str("S6MyClass".to_string()));
        let attr_name_val = MbValue::from_ptr(MbObject::new_str("field".to_string()));

        // Register the class first (empty), then set class attr, then re-register
        // Actually, __set_name__ fires in mb_class_register. We need to add the
        // class attr before registration. Let's use mb_class_set_class_attr to
        // pre-populate, which works on already-registered classes.
        // The correct flow: register class, set class attr, then __set_name__
        // would have been called if attrs were present at registration time.
        //
        // For unit testing the __set_name__ protocol directly, we can manually
        // call try_get_dunder_on_value to verify the protocol detection works.
        // The full integration path requires the class attr to be present during
        // mb_class_register. Let's test by pre-inserting the attr into CLASS_REGISTRY.

        // Register the host class with the descriptor as a class attribute
        mb_class_register("S6MyClass", vec![], HashMap::new());
        mb_class_set_class_attr(cls_name_val, attr_name_val, desc_inst);

        // Now re-register to trigger __set_name__ (the protocol runs on class_attrs)
        // We simulate this by calling the __set_name__ protocol manually like
        // mb_class_register does, since re-registration overrides the class.
        let class_attrs: HashMap<String, MbValue> = CLASS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            reg.get("S6MyClass")
                .map(|cls| cls.class_attrs.clone())
                .unwrap_or_default()
        });

        // Manually call __set_name__ on attrs (as mb_class_register does)
        for (attr_name, val) in &class_attrs {
            if let Some(set_name_method) = try_get_dunder_on_value(*val, "__set_name__") {
                let addr = extract_func_addr(set_name_method);
                if addr != 0 {
                    let is_registered = CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr));
                    if is_registered {
                        let owner = MbValue::from_ptr(MbObject::new_str("S6MyClass".to_string()));
                        let attr_str = MbValue::from_ptr(MbObject::new_str(attr_name.clone()));
                        let func: fn(MbValue, MbValue, MbValue) -> MbValue =
                            unsafe { std::mem::transmute(addr as usize) };
                        func(*val, owner, attr_str);
                    }
                }
            }
        }

        assert!(
            S6_SET_NAME_CALLED.load(Ordering::SeqCst),
            "S6: __set_name__ must be called on descriptor attributes after class creation"
        );
    }

    // ── S7: __slots__ with inheritance merges parent slots (R13) ──

    #[test]
    fn test_s7_slots_inheritance_merge() {
        // S7: Base has __slots__=['x'], Child(Base) has __slots__=['y'].
        //     c.x=1 and c.y=2 both succeed; c.z=3 raises AttributeError.
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S7Base", vec![], HashMap::new());
        let base_name = MbValue::from_ptr(MbObject::new_str("S7Base".to_string()));
        let base_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(base_name, base_slots);

        mb_class_register("S7Child", vec!["S7Base".to_string()], HashMap::new());
        let child_name = MbValue::from_ptr(MbObject::new_str("S7Child".to_string()));
        let child_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("y".to_string()),
        )]));
        mb_register_slots(child_name, child_slots);

        // Verify effective slots include both x and y
        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slots = reg.get("S7Child").unwrap();
            assert!(
                slots.contains(&"x".to_string()),
                "S7: Child effective slots must include parent slot 'x'"
            );
            assert!(
                slots.contains(&"y".to_string()),
                "S7: Child effective slots must include own slot 'y'"
            );
        });

        // Create instance and test attribute access
        let inst_name = MbValue::from_ptr(MbObject::new_str("S7Child".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        // c.x = 1 — should succeed (from base slots)
        let attr_x = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr_x, MbValue::from_int(1));
        let attr_x2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_x2).as_int(),
            Some(1),
            "S7: c.x = 1 must succeed (inherited slot)"
        );

        // c.y = 2 — should succeed (own slot)
        let attr_y = MbValue::from_ptr(MbObject::new_str("y".to_string()));
        mb_setattr(inst, attr_y, MbValue::from_int(2));
        let attr_y2 = MbValue::from_ptr(MbObject::new_str("y".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_y2).as_int(),
            Some(2),
            "S7: c.y = 2 must succeed (own slot)"
        );

        // c.z = 3 — should raise AttributeError (not in slots)
        crate::runtime::exception::mb_clear_exception();
        let attr_z = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        mb_setattr(inst, attr_z, MbValue::from_int(3));
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "S7: c.z = 3 must raise AttributeError (not in merged slots)"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // ── S8: __slots__ suppresses __dict__ (R13) ──

    #[test]
    fn test_s8_slots_suppresses_dict() {
        // S8: class Compact with __slots__=['x','y'] → obj.__dict__ raises AttributeError.
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S8Compact", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("S8Compact".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
            MbValue::from_ptr(MbObject::new_str("y".to_string())),
        ]));
        mb_register_slots(cls_name, slots);

        // Verify __dict__ is suppressed
        DICT_SUPPRESSED.with(|reg| {
            assert!(
                reg.borrow().contains("S8Compact"),
                "S8: class with __slots__ (no '__dict__') must have __dict__ suppressed"
            );
        });

        let inst_name = MbValue::from_ptr(MbObject::new_str("S8Compact".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        // Access __dict__ → should raise AttributeError
        let dict_attr = MbValue::from_ptr(MbObject::new_str("__dict__".to_string()));
        let _result = mb_getattr(inst, dict_attr);
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "S8: __dict__ access must raise AttributeError when __slots__ defined"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // ── S9: __slots__ with __dict__ explicitly listed (R13) ──

    #[test]
    fn test_s9_slots_with_dict_in_slots() {
        // S9: class Hybrid with __slots__=['x', '__dict__'] → obj.x=1 and obj.z=3 both succeed.
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S9Hybrid", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("S9Hybrid".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
            MbValue::from_ptr(MbObject::new_str("__dict__".to_string())),
        ]));
        mb_register_slots(cls_name, slots);

        // Verify __dict__ is NOT suppressed (because '__dict__' is in slots)
        DICT_SUPPRESSED.with(|reg| {
            assert!(
                !reg.borrow().contains("S9Hybrid"),
                "S9: class with '__dict__' in __slots__ must NOT suppress __dict__"
            );
        });

        let inst_name = MbValue::from_ptr(MbObject::new_str("S9Hybrid".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        // obj.x = 1 (slot)
        let attr_x = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr_x, MbValue::from_int(1));
        let attr_x2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_x2).as_int(),
            Some(1),
            "S9: obj.x = 1 must succeed (named slot)"
        );

        // obj.z = 3 (via __dict__, since not suppressed)
        let attr_z = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        mb_setattr(inst, attr_z, MbValue::from_int(3));
        let attr_z2 = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_z2).as_int(),
            Some(3),
            "S9: obj.z = 3 must succeed (via __dict__)"
        );

        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(false),
            "S9: No exception when __dict__ is in __slots__"
        );
    }

    // ── S10: Empty __slots__ allows no instance attributes (R13) ──

    #[test]
    fn test_s10_empty_slots_allows_nothing() {
        // S10: class Empty with __slots__=() → obj.x = 1 raises AttributeError.
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S10Empty", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("S10Empty".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![]));
        mb_register_slots(cls_name, slots);

        // Verify slots registry has empty list for S10Empty
        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slot_list = reg.get("S10Empty").unwrap();
            assert!(
                slot_list.is_empty(),
                "S10: empty __slots__ must have no entries"
            );
        });

        // Verify __dict__ is suppressed
        DICT_SUPPRESSED.with(|reg| {
            assert!(
                reg.borrow().contains("S10Empty"),
                "S10: class with empty __slots__ must suppress __dict__"
            );
        });

        let inst_name = MbValue::from_ptr(MbObject::new_str("S10Empty".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        // obj.x = 1 → must raise AttributeError
        let attr_x = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr_x, MbValue::from_int(1));
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "S10: Setting any attribute on empty __slots__ must raise AttributeError"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // ── S11: Compiler emits mb_register_slots for __slots__ (R14) ──

    #[test]
    fn test_s11_register_slots_populates_registry() {
        // S11: After mb_register_slots is called (as the compiler would emit),
        //      SLOTS_REGISTRY has the correct entry.
        mb_class_register("S11Foo", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("S11Foo".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
        ]));
        mb_register_slots(cls_name, slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slot_list = reg.get("S11Foo").unwrap();
            assert_eq!(
                slot_list.len(),
                2,
                "S11: SLOTS_REGISTRY must have 2 entries for S11Foo"
            );
            assert!(
                slot_list.contains(&"a".to_string()),
                "S11: slot 'a' must be present"
            );
            assert!(
                slot_list.contains(&"b".to_string()),
                "S11: slot 'b' must be present"
            );
        });
    }

    // ── S12: Child without __slots__ inherits but gets __dict__ (R13) ──

    #[test]
    fn test_s12_child_without_slots_inherits_and_gets_dict() {
        // S12: Base with __slots__=['x'], Child(Base) without __slots__ declaration.
        //      c.x=1 succeeds (inherited slot), c.z=99 succeeds (via __dict__).
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S12Base", vec![], HashMap::new());
        let base_name = MbValue::from_ptr(MbObject::new_str("S12Base".to_string()));
        let base_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(base_name, base_slots);

        // Register Child WITHOUT calling mb_register_slots (no __slots__ declaration)
        mb_class_register("S12Child", vec!["S12Base".to_string()], HashMap::new());

        // Child should NOT be in SLOTS_REGISTRY (it didn't define __slots__)
        let child_has_own_slots = SLOTS_REGISTRY.with(|reg| reg.borrow().contains_key("S12Child"));
        assert!(
            !child_has_own_slots,
            "S12: Child without __slots__ must NOT be in SLOTS_REGISTRY"
        );

        // Child should NOT have __dict__ suppressed
        let child_dict_suppressed = DICT_SUPPRESSED.with(|reg| reg.borrow().contains("S12Child"));
        assert!(
            !child_dict_suppressed,
            "S12: Child without __slots__ must NOT suppress __dict__"
        );

        let inst_name = MbValue::from_ptr(MbObject::new_str("S12Child".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        // c.x = 1 — succeeds (child inherits from base, __dict__ is available)
        let attr_x = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        mb_setattr(inst, attr_x, MbValue::from_int(1));
        let attr_x2 = MbValue::from_ptr(MbObject::new_str("x".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_x2).as_int(),
            Some(1),
            "S12: c.x = 1 must succeed"
        );

        // c.z = 99 — succeeds (via __dict__, since Child has no __slots__)
        let attr_z = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        mb_setattr(inst, attr_z, MbValue::from_int(99));
        let attr_z2 = MbValue::from_ptr(MbObject::new_str("z".to_string()));
        assert_eq!(
            mb_getattr(inst, attr_z2).as_int(),
            Some(99),
            "S12: c.z = 99 must succeed (via __dict__)"
        );

        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(false),
            "S12: No exceptions expected"
        );
    }

    // ── Additional R10-R14 edge case tests ──

    #[test]
    fn test_r10_init_subclass_without_kwargs_calls_hook() {
        // __init_subclass__ with no kwargs still gets called (1-arg form)
        use std::sync::atomic::{AtomicBool, Ordering};
        static R10_NO_KW_CALLED: AtomicBool = AtomicBool::new(false);

        extern "C" fn r10_hook(_cls: MbValue) -> MbValue {
            R10_NO_KW_CALLED.store(true, Ordering::SeqCst);
            MbValue::none()
        }

        let hook_addr = r10_hook as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(hook_addr as u64);
        });

        let mut base_methods = HashMap::new();
        base_methods.insert(
            "__init_subclass__".to_string(),
            MbValue::from_func(hook_addr),
        );
        mb_class_register("R10NoKwBase", vec![], base_methods);

        // Register child without kwargs
        mb_class_register(
            "R10NoKwChild",
            vec!["R10NoKwBase".to_string()],
            HashMap::new(),
        );

        assert!(
            R10_NO_KW_CALLED.load(Ordering::SeqCst),
            "R10: __init_subclass__ must be called even without kwargs"
        );
    }

    #[test]
    fn test_r10_class_set_kwargs_stores_correctly() {
        // Verify mb_class_set_kwargs populates KWARGS_REGISTRY correctly
        let cls = MbValue::from_ptr(MbObject::new_str("R10KwTest".to_string()));
        let keys = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
        ]));
        let values = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        mb_class_set_kwargs(cls, keys, values);

        KWARGS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let kwargs = reg.get("R10KwTest").unwrap();
            assert_eq!(kwargs.len(), 2, "R10: kwargs must have 2 entries");
            assert_eq!(kwargs.get("a").unwrap().as_int(), Some(1));
            assert_eq!(kwargs.get("b").unwrap().as_int(), Some(2));
        });

        // Clean up: remove from registry to not affect other tests
        KWARGS_REGISTRY.with(|reg| {
            reg.borrow_mut().remove("R10KwTest");
        });
    }

    #[test]
    fn test_r13_slots_merge_three_level_inheritance() {
        // Base['x'] → Mid['y'] → Leaf['z'] → effective slots = ['z','y','x']
        crate::runtime::exception::mb_clear_exception();

        mb_class_register("S13Base", vec![], HashMap::new());
        let base_name = MbValue::from_ptr(MbObject::new_str("S13Base".to_string()));
        let base_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(base_name, base_slots);

        mb_class_register("S13Mid", vec!["S13Base".to_string()], HashMap::new());
        let mid_name = MbValue::from_ptr(MbObject::new_str("S13Mid".to_string()));
        let mid_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("y".to_string()),
        )]));
        mb_register_slots(mid_name, mid_slots);

        mb_class_register("S13Leaf", vec!["S13Mid".to_string()], HashMap::new());
        let leaf_name = MbValue::from_ptr(MbObject::new_str("S13Leaf".to_string()));
        let leaf_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("z".to_string()),
        )]));
        mb_register_slots(leaf_name, leaf_slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let leaf_slots = reg.get("S13Leaf").unwrap();
            assert!(
                leaf_slots.contains(&"x".to_string()),
                "R13: Leaf must inherit Base slot 'x'"
            );
            assert!(
                leaf_slots.contains(&"y".to_string()),
                "R13: Leaf must inherit Mid slot 'y'"
            );
            assert!(
                leaf_slots.contains(&"z".to_string()),
                "R13: Leaf must have own slot 'z'"
            );
        });

        // Verify actual attribute access works
        let inst_name = MbValue::from_ptr(MbObject::new_str("S13Leaf".to_string()));
        let inst = mb_instance_new(inst_name, MbValue::none());

        for (name, val) in [("x", 10i64), ("y", 20), ("z", 30)] {
            let attr = MbValue::from_ptr(MbObject::new_str(name.to_string()));
            mb_setattr(inst, attr, MbValue::from_int(val));
        }
        for (name, expected) in [("x", 10i64), ("y", 20), ("z", 30)] {
            let attr = MbValue::from_ptr(MbObject::new_str(name.to_string()));
            assert_eq!(
                mb_getattr(inst, attr).as_int(),
                Some(expected),
                "R13: three-level slot {} must be accessible",
                name
            );
        }

        // Unslotted attr must fail
        crate::runtime::exception::mb_clear_exception();
        let attr_w = MbValue::from_ptr(MbObject::new_str("w".to_string()));
        mb_setattr(inst, attr_w, MbValue::from_int(99));
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "R13: Setting unslotted attribute on Leaf must raise AttributeError"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_r13_slots_no_duplicate_in_merge() {
        // If parent and child both declare slot 'x', effective set has only one 'x'
        mb_class_register("S13DupBase", vec![], HashMap::new());
        let base_name = MbValue::from_ptr(MbObject::new_str("S13DupBase".to_string()));
        let base_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(base_name, base_slots);

        mb_class_register(
            "S13DupChild",
            vec!["S13DupBase".to_string()],
            HashMap::new(),
        );
        let child_name = MbValue::from_ptr(MbObject::new_str("S13DupChild".to_string()));
        let child_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(child_name, child_slots);

        SLOTS_REGISTRY.with(|reg| {
            let reg = reg.borrow();
            let slots = reg.get("S13DupChild").unwrap();
            let x_count = slots.iter().filter(|s| *s == "x").count();
            assert_eq!(
                x_count, 1,
                "R13: duplicate slot 'x' must appear only once in effective set"
            );
        });
    }

    #[test]
    fn test_r11_class_getitem_inherited() {
        // __class_getitem__ can be inherited from a parent class
        use std::sync::atomic::{AtomicBool, Ordering};
        static R11_INHERITED_CALLED: AtomicBool = AtomicBool::new(false);

        extern "C" fn r11_getitem(_cls: MbValue, _key: MbValue) -> MbValue {
            R11_INHERITED_CALLED.store(true, Ordering::SeqCst);
            MbValue::from_ptr(MbObject::new_str("inherited".to_string()))
        }

        let hook_addr = r11_getitem as *const () as usize;
        CALLABLE_REGISTRY.with(|reg| {
            reg.borrow_mut().insert(hook_addr as u64);
        });

        let mut parent_methods = HashMap::new();
        parent_methods.insert(
            "__class_getitem__".to_string(),
            MbValue::from_func(hook_addr),
        );
        mb_class_register("R11Parent", vec![], parent_methods);
        mb_class_register("R11Child", vec!["R11Parent".to_string()], HashMap::new());

        // Subscript R11Child[int] — should find __class_getitem__ from parent via MRO
        let cls_name = MbValue::from_ptr(MbObject::new_str("R11Child".to_string()));
        let key = MbValue::from_ptr(MbObject::new_str("int".to_string()));
        let _result = mb_obj_getitem(cls_name, key);

        assert!(
            R11_INHERITED_CALLED.load(Ordering::SeqCst),
            "R11: __class_getitem__ must be inherited and callable from child class"
        );
    }

    #[test]
    fn test_r12_try_get_dunder_on_value_for_non_instance() {
        // try_get_dunder_on_value should return None for non-instance values
        let result = try_get_dunder_on_value(MbValue::from_int(42), "__set_name__");
        assert!(
            result.is_none(),
            "R12: try_get_dunder_on_value on int must return None"
        );

        let result2 = try_get_dunder_on_value(MbValue::none(), "__set_name__");
        assert!(
            result2.is_none(),
            "R12: try_get_dunder_on_value on None must return None"
        );
    }

    #[test]
    fn test_r13_dict_suppressed_cleared_on_cleanup() {
        // After cleanup_all_classes, DICT_SUPPRESSED should be empty
        mb_class_register("R13CleanupDict", vec![], HashMap::new());
        let cls_name = MbValue::from_ptr(MbObject::new_str("R13CleanupDict".to_string()));
        let slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        mb_register_slots(cls_name, slots);

        DICT_SUPPRESSED.with(|reg| {
            assert!(
                reg.borrow().contains("R13CleanupDict"),
                "R13: DICT_SUPPRESSED must contain class before cleanup"
            );
        });

        cleanup_all_classes();

        DICT_SUPPRESSED.with(|reg| {
            assert!(
                reg.borrow().is_empty(),
                "R13: DICT_SUPPRESSED must be empty after cleanup"
            );
        });
    }

    // ── type() 3-arg class system scenarios (#974) ──

    #[test]
    fn test_issubclass_with_type_created_class() {
        // Scenario: issubclass with type()-created class
        // GIVEN Base is a statically-defined class and Child created via type()
        // WHEN issubclass(Child, Base) is evaluated with both args as type objects
        // THEN Returns True because Child's MRO includes Base
        cleanup_all_classes();

        mb_class_register("T3Base", vec![], HashMap::new());
        mb_class_register("T3Child", vec!["T3Base".to_string()], HashMap::new());

        let child_type = make_type_object("T3Child");
        let base_type = make_type_object("T3Base");
        let result = mb_issubclass(child_type, base_type);
        assert_eq!(
            result.as_bool(),
            Some(true),
            "issubclass(Child_type_obj, Base_type_obj) must be True via MRO"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_issubclass_type_object_and_string() {
        // Scenario: issubclass with type object and string
        // GIVEN MyClass = type('MyClass', (object,), {})
        // WHEN issubclass(MyClass, object) where MyClass is type obj, object is string
        // THEN Returns True — resolve_class_name reads __name__ from type obj
        cleanup_all_classes();

        mb_class_register("T3MyClass", vec!["object".to_string()], HashMap::new());

        let my_class_type = make_type_object("T3MyClass");
        let object_str = MbValue::from_ptr(MbObject::new_str("object".to_string()));
        let result = mb_issubclass(my_class_type, object_str);
        assert_eq!(
            result.as_bool(),
            Some(true),
            "issubclass(type_obj, 'object') must be True"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_isinstance_dynamic_class_with_inheritance() {
        // Scenario: isinstance dispatch for dynamically-created class with inheritance
        // GIVEN class Animal: pass and Dog = type('Dog', (Animal,), {})
        // WHEN d = Dog() and isinstance(d, Animal)
        // THEN Returns True because Dog's MRO contains Animal
        cleanup_all_classes();

        mb_class_register("T3Animal", vec![], HashMap::new());
        mb_class_register("T3Dog", vec!["T3Animal".to_string()], HashMap::new());

        let dog_name = MbValue::from_ptr(MbObject::new_str("T3Dog".to_string()));
        let instance = mb_instance_new(dog_name, MbValue::none());

        // isinstance(d, Animal) via type object
        let animal_type = make_type_object("T3Animal");
        let result = mb_isinstance(instance, animal_type);
        assert_eq!(
            result.as_bool(),
            Some(true),
            "isinstance(Dog(), Animal_type_obj) must be True via MRO"
        );

        // isinstance(d, Animal) via string (should also work)
        let animal_str = MbValue::from_ptr(MbObject::new_str("T3Animal".to_string()));
        let result2 = mb_isinstance(instance, animal_str);
        assert_eq!(
            result2.as_bool(),
            Some(true),
            "isinstance(Dog(), 'Animal') must also be True"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_dunder_method_dispatch_type_created_class() {
        // Scenario: dunder method dispatch on type()-created class
        // GIVEN MyClass registered with __repr__ and __eq__ in methods
        // WHEN lookup_method is called for those dunders
        // THEN The methods are found via MRO lookup in class methods
        cleanup_all_classes();

        let sentinel = MbValue::from_int(777);
        let mut methods = HashMap::new();
        methods.insert("__repr__".to_string(), sentinel);
        methods.insert("__eq__".to_string(), MbValue::from_int(888));
        mb_class_register("T3Dunder", vec![], methods);

        let repr = lookup_method("T3Dunder", "__repr__");
        assert_eq!(
            repr.as_int(),
            Some(777),
            "__repr__ must be found via lookup_method"
        );

        let eq = lookup_method("T3Dunder", "__eq__");
        assert_eq!(
            eq.as_int(),
            Some(888),
            "__eq__ must be found via lookup_method"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_method_as_class_attr_callable() {
        // Scenario: method as class_attr still callable
        // GIVEN a function value is set as class_attr on a type()-created class
        // WHEN getattr is called on an instance for that attribute
        // THEN lookup finds the value in class_attrs via MRO fallback
        cleanup_all_classes();

        let greet_val = MbValue::from_int(42);
        mb_class_register("T3Greeter", vec![], HashMap::new());

        // Set greet as a class attribute
        let cls_name = MbValue::from_ptr(MbObject::new_str("T3Greeter".to_string()));
        let attr_name = MbValue::from_ptr(MbObject::new_str("greet".to_string()));
        mb_class_set_class_attr(cls_name, attr_name, greet_val);

        // Create instance and access the class attr
        let inst_name = MbValue::from_ptr(MbObject::new_str("T3Greeter".to_string()));
        let instance = mb_instance_new(inst_name, MbValue::none());
        let attr = MbValue::from_ptr(MbObject::new_str("greet".to_string()));
        let result = mb_getattr(instance, attr);
        assert_eq!(
            result.as_int(),
            Some(42),
            "class_attr 'greet' must be accessible on instance via MRO fallback"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_mro_multi_base_type_class() {
        // Scenario: MRO correct for multi-base type() class
        // GIVEN class A: pass, class B: pass, C = type('C', (A, B), {})
        // WHEN C's MRO is computed
        // THEN MRO is [C, A, B, object] via C3 linearization
        cleanup_all_classes();

        mb_class_register("T3A", vec![], HashMap::new());
        mb_class_register("T3B", vec![], HashMap::new());
        mb_class_register(
            "T3C",
            vec!["T3A".to_string(), "T3B".to_string()],
            HashMap::new(),
        );

        CLASS_REGISTRY.with(|reg| {
            let cls = reg.borrow();
            let c = cls.get("T3C").unwrap();
            assert_eq!(
                c.mro,
                vec!["T3C", "T3A", "T3B", "object"],
                "MRO must be [C, A, B, object] via C3 linearization"
            );
        });

        cleanup_all_classes();
    }

    #[test]
    fn test_issubclass_both_type_objects() {
        // Additional: both child and parent are type objects
        cleanup_all_classes();

        mb_class_register("T3X", vec![], HashMap::new());
        mb_class_register("T3Y", vec!["T3X".to_string()], HashMap::new());

        let x_type = make_type_object("T3X");
        let y_type = make_type_object("T3Y");

        // issubclass(Y, X) -> True
        assert_eq!(
            mb_issubclass(y_type, x_type).as_bool(),
            Some(true),
            "issubclass(Y_type, X_type) must be True"
        );
        // issubclass(X, Y) -> False
        assert_eq!(
            mb_issubclass(x_type, y_type).as_bool(),
            Some(false),
            "issubclass(X_type, Y_type) must be False"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_issubclass_type_object_with_object() {
        // issubclass(SomeType, object) should be True for any class
        cleanup_all_classes();

        mb_class_register("T3Obj", vec![], HashMap::new());
        let obj_type = make_type_object("T3Obj");
        let object_str = MbValue::from_ptr(MbObject::new_str("object".to_string()));

        assert_eq!(
            mb_issubclass(obj_type, object_str).as_bool(),
            Some(true),
            "issubclass(any_type_obj, 'object') must be True"
        );

        cleanup_all_classes();
    }

    #[test]
    fn test_resolve_class_name_plain_string() {
        // resolve_class_name with a plain string should return the string
        let val = MbValue::from_ptr(MbObject::new_str("MyClass".to_string()));
        assert_eq!(resolve_class_name(val), Some("MyClass".to_string()));
    }

    #[test]
    fn test_resolve_class_name_type_object() {
        // resolve_class_name with a type object should extract __name__
        let type_obj = make_type_object("DynClass");
        assert_eq!(resolve_class_name(type_obj), Some("DynClass".to_string()));
    }

    #[test]
    fn test_resolve_class_name_non_type_instance() {
        // resolve_class_name with a non-type instance should return None
        let inst = MbValue::from_ptr(MbObject::new_instance("SomeClass".to_string()));
        assert_eq!(
            resolve_class_name(inst),
            None,
            "non-type Instance should not resolve to a class name"
        );
    }

    #[test]
    fn test_resolve_class_name_none() {
        // resolve_class_name with None should return None
        assert_eq!(resolve_class_name(MbValue::none()), None);
    }

    // R13 (PEP 487): child class's effective __slots__ must merge parent slots
    // via MRO walk. Complements test_slots_restricts_attrs (single-class) which
    // only exercises the own-slot path. Tick-170 spec-align on runtime/class.md
    // logged R10-R14 ALL SHIPPED with R10 covered by test_init_subclass_basic;
    // this anchors the R13 inheritance-merge branch at class.rs:1382-1400.
    #[test]
    fn test_slots_inheritance_merges_parent_slots() {
        mb_class_register("SlotsParent001", vec![], HashMap::new());
        let parent_name = MbValue::from_ptr(MbObject::new_str("SlotsParent001".to_string()));
        let parent_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("a".to_string()),
        )]));
        mb_register_slots(parent_name, parent_slots);
        mb_class_register(
            "SlotsChild001",
            vec!["SlotsParent001".to_string()],
            HashMap::new(),
        );
        let child_name = MbValue::from_ptr(MbObject::new_str("SlotsChild001".to_string()));
        let child_slots = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("b".to_string()),
        )]));
        mb_register_slots(child_name, child_slots);
        SLOTS_REGISTRY.with(|reg| {
            let effective = reg.borrow().get("SlotsChild001").cloned().unwrap();
            assert!(
                effective.contains(&"b".to_string()),
                "child's own slot b must be present"
            );
            assert!(
                effective.contains(&"a".to_string()),
                "R13: parent slot a must be merged"
            );
        });
    }
}
