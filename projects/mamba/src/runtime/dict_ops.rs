use super::rc::{MbObject, ObjData};
/// Dict operations for the Mamba runtime (#285) — thread-safe.
///
/// Implements Python-compatible dict methods. All mutable access goes
/// through RwLock guards for thread-safety.
use super::value::MbValue;

/// Type-preserving dict key. Distinguishes int from string keys so that
/// `d[1]` and `d["1"]` are distinct entries (matching CPython semantics).
#[derive(Debug)]
pub enum DictKey {
    Int(i64),
    /// Non-integral float key, stored as its raw IEEE-754 bits (`0.5`, `1.5`).
    /// Integral float values (`1.0`, `2.0`) are normalized to `Int` in
    /// `to_dict_key` so `{1: a, 1.0: b}` collapses to one entry and
    /// `hash(1.0) == hash(1)` — matching CPython numeric key equality. Two
    /// equal non-integral floats share identical bits, so bit-keying buckets
    /// them together correctly.
    Float(u64),
    Str(String),
    Bytes(Vec<u8>),
    Bool(bool),
    None,
    /// User-class instance key: `hash_val` comes from `__hash__`, `ptr` holds
    /// the instance so `__eq__` can be dispatched when buckets collide.
    /// `ptr` is retained on construction and released on Drop/Clone-to-None.
    Instance {
        hash_val: i64,
        ptr: usize,
        class_name: String,
    },
    /// Fallback for non-hashable heap objects we can't currently route through
    /// the dunder protocol — keyed by the raw NaN-boxed bits as a string so
    /// identity-based storage still works.
    Other(String),
    /// Function-pointer key — TAG_FUNC values keyed by the raw code address.
    /// Function objects are hashable in Python (identity-based), so a dict
    /// keyed on `f` must match `f` again on lookup. Storing the raw addr
    /// preserves identity and lets `dict_key_to_mbvalue` rebuild the
    /// `MbValue::from_func(addr)` for repr / iteration.
    Func(usize),
    /// Tuple key — hashed structurally via `mb_tuple_hash` so that two
    /// literal `(0, 1)` tuples bucket together. `ptr` retains the tuple
    /// object so element-wise `mb_tuple_eq` can break hash collisions.
    Tuple {
        hash_val: i64,
        ptr: usize,
    },
    /// FrozenSet key — hashed by content via `mb_hash` (order-independent),
    /// so two frozensets with equal elements (regardless of build order)
    /// bucket together. `ptr` retains the frozenset object so `mb_eq` can
    /// break hash collisions element-wise. Mirrors the Tuple variant.
    FrozenSet {
        hash_val: i64,
        ptr: usize,
    },
}

impl Clone for DictKey {
    fn clone(&self) -> Self {
        match self {
            DictKey::Int(i) => DictKey::Int(*i),
            DictKey::Float(b) => DictKey::Float(*b),
            DictKey::Str(s) => DictKey::Str(s.clone()),
            DictKey::Bytes(b) => DictKey::Bytes(b.clone()),
            DictKey::Bool(b) => DictKey::Bool(*b),
            DictKey::None => DictKey::None,
            DictKey::Instance {
                hash_val,
                ptr,
                class_name,
            } => {
                // Retain so the cloned key owns its own rc on the instance.
                let val = super::value::MbValue::from_ptr(*ptr as *mut super::rc::MbObject);
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                DictKey::Instance {
                    hash_val: *hash_val,
                    ptr: *ptr,
                    class_name: class_name.clone(),
                }
            }
            DictKey::Other(s) => DictKey::Other(s.clone()),
            DictKey::Func(addr) => DictKey::Func(*addr),
            DictKey::Tuple { hash_val, ptr } => {
                let val = super::value::MbValue::from_ptr(*ptr as *mut super::rc::MbObject);
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                DictKey::Tuple {
                    hash_val: *hash_val,
                    ptr: *ptr,
                }
            }
            DictKey::FrozenSet { hash_val, ptr } => {
                let val = super::value::MbValue::from_ptr(*ptr as *mut super::rc::MbObject);
                unsafe {
                    super::rc::retain_if_ptr(val);
                }
                DictKey::FrozenSet {
                    hash_val: *hash_val,
                    ptr: *ptr,
                }
            }
        }
    }
}

impl Drop for DictKey {
    fn drop(&mut self) {
        match self {
            DictKey::Instance { ptr, .. }
            | DictKey::Tuple { ptr, .. }
            | DictKey::FrozenSet { ptr, .. } => {
                let val = super::value::MbValue::from_ptr(*ptr as *mut super::rc::MbObject);
                unsafe {
                    super::rc::release_if_ptr(val);
                }
            }
            _ => {}
        }
    }
}

impl std::hash::Hash for DictKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            // Str hashes identically to `str` so that `Equivalent<DictKey> for str`
            // lookups hit the correct bucket without allocating a DictKey.
            DictKey::Str(s) => s.hash(state),
            // All other variants include the discriminant to avoid collisions
            // between e.g. Int(42) and Str("42").
            _ => {
                std::mem::discriminant(self).hash(state);
                match self {
                    DictKey::Int(i) => i.hash(state),
                    DictKey::Float(b) => b.hash(state),
                    DictKey::Bytes(b) => b.hash(state),
                    DictKey::Bool(b) => b.hash(state),
                    DictKey::None => {}
                    DictKey::Instance { hash_val, .. } => hash_val.hash(state),
                    DictKey::Other(s) => s.hash(state),
                    DictKey::Func(addr) => addr.hash(state),
                    DictKey::Tuple { hash_val, .. } => hash_val.hash(state),
                    DictKey::FrozenSet { hash_val, .. } => hash_val.hash(state),
                    DictKey::Str(_) => unreachable!(),
                }
            }
        }
    }
}

impl PartialEq for DictKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DictKey::Int(a), DictKey::Int(b)) => a == b,
            (DictKey::Float(a), DictKey::Float(b)) => a == b,
            (DictKey::Str(a), DictKey::Str(b)) => a == b,
            (DictKey::Bytes(a), DictKey::Bytes(b)) => a == b,
            (DictKey::Bool(a), DictKey::Bool(b)) => a == b,
            (DictKey::None, DictKey::None) => true,
            (
                DictKey::Instance {
                    hash_val: ha,
                    ptr: pa,
                    ..
                },
                DictKey::Instance {
                    hash_val: hb,
                    ptr: pb,
                    ..
                },
            ) => {
                if ha != hb {
                    return false;
                }
                if pa == pb {
                    return true;
                }
                // Hash collision with different pointers — dispatch __eq__.
                let a_val = super::value::MbValue::from_ptr(*pa as *mut super::rc::MbObject);
                let b_val = super::value::MbValue::from_ptr(*pb as *mut super::rc::MbObject);
                super::builtins::mb_eq(a_val, b_val)
                    .as_bool()
                    .unwrap_or(false)
            }
            (
                DictKey::Tuple {
                    hash_val: ha,
                    ptr: pa,
                },
                DictKey::Tuple {
                    hash_val: hb,
                    ptr: pb,
                },
            ) => {
                if ha != hb {
                    return false;
                }
                if pa == pb {
                    return true;
                }
                let a_val = super::value::MbValue::from_ptr(*pa as *mut super::rc::MbObject);
                let b_val = super::value::MbValue::from_ptr(*pb as *mut super::rc::MbObject);
                super::tuple_ops::mb_tuple_eq(a_val, b_val)
                    .as_bool()
                    .unwrap_or(false)
            }
            (
                DictKey::FrozenSet {
                    hash_val: ha,
                    ptr: pa,
                },
                DictKey::FrozenSet {
                    hash_val: hb,
                    ptr: pb,
                },
            ) => {
                if ha != hb {
                    return false;
                }
                if pa == pb {
                    return true;
                }
                // Hash collision with different objects — compare by content
                // (mb_eq routes through the FrozenSet set-equality arm).
                let a_val = super::value::MbValue::from_ptr(*pa as *mut super::rc::MbObject);
                let b_val = super::value::MbValue::from_ptr(*pb as *mut super::rc::MbObject);
                super::builtins::mb_eq(a_val, b_val)
                    .as_bool()
                    .unwrap_or(false)
            }
            (DictKey::Other(a), DictKey::Other(b)) => a == b,
            (DictKey::Func(a), DictKey::Func(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for DictKey {}

/// Allow `map.get("key")` on `IndexMap<DictKey, V>` — matches only `DictKey::Str`.
/// Hash compatibility: `DictKey::Str(s)` hashes identically to `s: str`.
impl indexmap::Equivalent<DictKey> for str {
    fn equivalent(&self, key: &DictKey) -> bool {
        matches!(key, DictKey::Str(s) if s.as_str() == self)
    }
}

/// Allow `map.get(&name)` on `IndexMap<DictKey, V>` where `name: String`.
impl indexmap::Equivalent<DictKey> for String {
    fn equivalent(&self, key: &DictKey) -> bool {
        matches!(key, DictKey::Str(s) if s == self)
    }
}

fn format_bytes_key(data: &[u8]) -> String {
    let has_single = data.contains(&b'\'');
    let has_double = data.contains(&b'"');
    let use_double = has_single && !has_double;
    let quote = if use_double { b'"' } else { b'\'' };
    let mut out = String::with_capacity(data.len() + 3);
    out.push('b');
    out.push(quote as char);
    for &b in data {
        match b {
            b'\\' => out.push_str("\\\\"),
            b'\n' => out.push_str("\\n"),
            b'\r' => out.push_str("\\r"),
            b'\t' => out.push_str("\\t"),
            c if c == quote => { out.push('\\'); out.push(c as char); }
            0x20..=0x7E => out.push(b as char),
            c => out.push_str(&format!("\\x{c:02x}")),
        }
    }
    out.push(quote as char);
    out
}

impl std::fmt::Display for DictKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictKey::Int(i) => write!(f, "{i}"),
            DictKey::Float(bits) => write!(f, "{}", dict_key_display(&DictKey::Float(*bits))),
            DictKey::Str(s) => write!(f, "{s}"),
            DictKey::Bytes(b) => write!(f, "{}", format_bytes_key(b)),
            DictKey::Bool(b) => write!(f, "{}", if *b { "True" } else { "False" }),
            DictKey::None => write!(f, "None"),
            DictKey::Instance {
                class_name, ptr, ..
            } => {
                write!(
                    f,
                    "<{class_name} at {:p}>",
                    *ptr as *const super::rc::MbObject
                )
            }
            DictKey::Other(s) => write!(f, "{s}"),
            DictKey::Func(addr) => write!(f, "<function at 0x{addr:x}>"),
            DictKey::Tuple { ptr, .. } | DictKey::FrozenSet { ptr, .. } => {
                let val = super::value::MbValue::from_ptr(*ptr as *mut super::rc::MbObject);
                let r = super::builtins::mb_repr(val);
                let s = r
                    .as_ptr()
                    .and_then(|p| unsafe {
                        if let ObjData::Str(ref s) = (*p).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default();
                write!(f, "{s}")
            }
        }
    }
}

impl From<String> for DictKey {
    fn from(s: String) -> Self {
        DictKey::Str(s)
    }
}

impl From<&str> for DictKey {
    fn from(s: &str) -> Self {
        DictKey::Str(s.to_string())
    }
}

impl DictKey {
    /// Borrow the inner string for `Str` and `Other` variants.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DictKey::Str(s) | DictKey::Other(s) => Some(s),
            _ => None,
        }
    }

    /// Returns true if this key starts with the given prefix (Str variant only).
    pub fn starts_with(&self, prefix: &str) -> bool {
        matches!(self, DictKey::Str(s) if s.starts_with(prefix))
    }

    /// Returns true if this key ends with the given suffix (Str variant only).
    pub fn ends_with(&self, suffix: &str) -> bool {
        matches!(self, DictKey::Str(s) if s.ends_with(suffix))
    }
}

/// Convert an MbValue to a DictKey for storage/lookup.
pub fn to_dict_key(val: MbValue) -> DictKey {
    // Check tagged types first (int, bool, none)
    if let Some(i) = val.as_int() {
        // Decimal/Fraction integer handles key by numeric value so
        // `{Fraction(6, 3): "a", 2: "b"}` collapses like CPython (#2129).
        // The range guard keeps plain-int keys to one compare.
        if (i as u64) >= super::integer_handle_registry::HANDLE_MIN_ID
            && (super::stdlib::decimal_mod::is_decimal_handle(i as u64)
                || super::stdlib::fractions_mod::is_fraction_handle(i as u64))
        {
            if let Some(iv) = super::stdlib::decimal_mod::mb_numeric_handle_integral_i64(val) {
                return DictKey::Int(iv);
            }
            if let Some(f) = super::stdlib::decimal_mod::mb_numeric_handle_exact_f64(val) {
                return to_dict_key(MbValue::from_float(f));
            }
        }
        return DictKey::Int(i);
    }
    if val.is_bool() {
        return DictKey::Bool(val.as_bool().unwrap_or(false));
    }
    if val.is_none() {
        return DictKey::None;
    }
    // Float keys. CPython hashes an integral float equal to the matching int
    // (`hash(1.0) == hash(1)`, `1.0 == 1`), so normalize integral floats to an
    // Int key — `{1: a, 1.0: b}` then collapses to one entry. Non-integral
    // floats (`0.5`, `1.5`) get a dedicated Float key bucketed by raw bits.
    // Checked before the ptr/fallback paths; ints/bools/ptrs are NaN-boxed so
    // `is_float()` is false for them and never reaches here.
    if let Some(f) = val.as_float() {
        if f.is_finite() && f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
            return DictKey::Int(f as i64);
        }
        return DictKey::Float(val.to_bits());
    }
    // TAG_FUNC values are hashable by code address (Python identity hash).
    if let Some(addr) = val.as_func() {
        return DictKey::Func(addr);
    }
    // Check pointer types (str, etc.)
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(ref s) => return DictKey::Str(s.clone()),
                ObjData::Bytes(ref b) => return DictKey::Bytes(b.clone()),
                ObjData::Tuple(_) => {
                    // Structural hash via mb_tuple_hash; retain the tuple object
                    // so element-wise eq can break collisions.
                    let h = super::tuple_ops::mb_tuple_hash(val).as_int().unwrap_or(0);
                    super::rc::retain_if_ptr(val);
                    return DictKey::Tuple {
                        hash_val: h,
                        ptr: ptr as usize,
                    };
                }
                ObjData::FrozenSet(_) => {
                    // Content hash via mb_hash (order-independent), so two equal
                    // frozensets bucket together regardless of build order;
                    // retain the object so mb_eq can break collisions.
                    let h = super::builtins::mb_hash(val).as_int().unwrap_or(0);
                    super::rc::retain_if_ptr(val);
                    return DictKey::FrozenSet {
                        hash_val: h,
                        ptr: ptr as usize,
                    };
                }
                ObjData::Instance { class_name, .. } => {
                    // Dispatch __hash__ to bucket by Python's hash; fall back to
                    // pointer identity when the class defines no __hash__.
                    let h = super::builtins::mb_hash(val)
                        .as_int()
                        .unwrap_or_else(|| (ptr as u64 >> 17) as i64);
                    super::rc::retain_if_ptr(val);
                    return DictKey::Instance {
                        hash_val: h,
                        ptr: ptr as usize,
                        class_name: class_name.clone(),
                    };
                }
                _ => {}
            }
        }
    }
    // Fallback: use display representation
    DictKey::Other(format!("{}", val.to_bits()))
}

/// Convert a DictKey back to an MbValue for iteration/display.
pub fn dict_key_to_mbvalue(key: &DictKey) -> MbValue {
    match key {
        DictKey::Int(i) => MbValue::from_int(*i),
        DictKey::Float(bits) => MbValue::from_float(f64::from_bits(*bits)),
        DictKey::Str(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        DictKey::Bytes(b) => MbValue::from_ptr(MbObject::new_bytes(b.clone())),
        DictKey::Bool(b) => MbValue::from_bool(*b),
        DictKey::None => MbValue::none(),
        DictKey::Instance { ptr, .. } => {
            let val = MbValue::from_ptr(*ptr as *mut MbObject);
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        }
        DictKey::Other(s) => MbValue::from_ptr(MbObject::new_str(s.clone())),
        DictKey::Func(addr) => MbValue::from_func(*addr),
        DictKey::Tuple { ptr, .. } | DictKey::FrozenSet { ptr, .. } => {
            let val = MbValue::from_ptr(*ptr as *mut MbObject);
            unsafe {
                super::rc::retain_if_ptr(val);
            }
            val
        }
    }
}

/// Return the raw key as its args[0] form for exception messages.
/// Unlike `dict_key_display`, this does NOT pre-quote strings — the
/// KeyError __str__ path adds the repr quoting when the exception is
/// printed or stringified, and adding quotes here would double up.
pub fn dict_key_raw_str(key: &DictKey) -> String {
    match key {
        DictKey::Int(i) => i.to_string(),
        DictKey::Str(s) => s.clone(),
        DictKey::Bytes(b) => format_bytes_key(b),
        DictKey::Bool(b) => {
            if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        DictKey::None => "None".to_string(),
        _ => dict_key_display(key),
    }
}

/// Format a DictKey for display (e.g., in repr).
pub fn dict_key_display(key: &DictKey) -> String {
    match key {
        DictKey::Int(i) => i.to_string(),
        DictKey::Float(bits) => {
            // Route through mb_repr so float keys render with Python's float
            // formatting (`0.5`, `1.5`, `1e+20`) identical to float values.
            let val = MbValue::from_float(f64::from_bits(*bits));
            super::builtins::mb_repr(val)
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        DictKey::Str(s) => format!("'{s}'"),
        DictKey::Bytes(b) => format_bytes_key(b),
        DictKey::Bool(b) => {
            if *b {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        DictKey::None => "None".to_string(),
        DictKey::Instance { ptr, .. }
        | DictKey::Tuple { ptr, .. }
        | DictKey::FrozenSet { ptr, .. } => {
            let val = MbValue::from_ptr(*ptr as *mut MbObject);
            super::builtins::mb_repr(val)
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        }
        DictKey::Other(s) => s.clone(),
        DictKey::Func(addr) => {
            // Reuse mb_repr to pick up the FUNC_NAMES-aware
            // `<function NAME at 0xADDR>` shape introduced in Fire 73.
            let val = MbValue::from_func(*addr);
            super::builtins::mb_repr(val)
                .as_ptr()
                .and_then(|p| unsafe {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| format!("<function at 0x{addr:x}>"))
        }
    }
}

#[allow(dead_code)]
fn new_str(s: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.to_string()))
}

// ── Creation ──

/// Create a new empty dict.
pub fn mb_dict_new() -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// dict(iterable_of_pairs) — build a dict from an iterable of (key, value) pairs.
/// Also accepts another dict (shallow copy), user-defined iterables (via __iter__),
/// and iterator handles from mb_iter/generators.
pub fn mb_dict_from_pairs(iterable: MbValue) -> MbValue {
    let dict = MbValue::from_ptr(MbObject::new_dict());

    // Collect pairs from whatever the input is, then apply uniformly.
    let pairs: Vec<MbValue> = if iterable.is_int() {
        // Wrap raw generator handles via mb_iter (idempotent for ITERATORS
        // entries) so generator exhaustion clears the runtime StopIteration
        // — see mb_list_from_iterable for the matching fix.
        let handle = super::iter::mb_iter(iterable);
        let mut items = Vec::new();
        loop {
            if super::iter::mb_has_next(handle).as_bool() != Some(true) {
                break;
            }
            items.push(super::iter::mb_next(handle));
        }
        items
    } else if let Some(ptr) = iterable.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
                ObjData::Tuple(ref items) => items.clone(),
                ObjData::Dict(ref src) => {
                    // dict(other_dict) — shallow copy.
                    let guard = src.read().unwrap();
                    for (k, &v) in guard.iter() {
                        let kv = dict_key_to_mbvalue(k);
                        super::rc::retain_if_ptr(v);
                        mb_dict_setitem(dict, kv, v);
                    }
                    return dict;
                }
                ObjData::Instance { .. } => {
                    // dict-like collections instances (defaultdict, Counter,
                    // OrderedDict) store their entries under a backing `_data`
                    // dict. Iterating them yields *keys* (not pairs), so the
                    // generic pair-fallback below would build {None: None}.
                    // Copy from `_data` directly. (#1632)
                    if let Some(backing) = super::class::unwrap_dictlike_data(iterable) {
                        if let Some(bp) = backing.as_ptr() {
                            if let ObjData::Dict(ref src) = (*bp).data {
                                let guard = src.read().unwrap();
                                for (k, &v) in guard.iter() {
                                    let kv = dict_key_to_mbvalue(k);
                                    super::rc::retain_if_ptr(v);
                                    mb_dict_setitem(dict, kv, v);
                                }
                                return dict;
                            }
                        }
                    }
                    let handle = super::iter::mb_iter(iterable);
                    if handle.is_none() {
                        return dict;
                    }
                    let mut items = Vec::new();
                    loop {
                        if super::iter::mb_has_next(handle).as_bool() != Some(true) {
                            break;
                        }
                        items.push(super::iter::mb_next(handle));
                    }
                    items
                }
                _ => return dict,
            }
        }
    } else {
        return dict;
    };

    for pair in pairs {
        let k = super::list_ops::mb_list_getitem(pair, MbValue::from_int(0));
        let v = super::list_ops::mb_list_getitem(pair, MbValue::from_int(1));
        mb_dict_setitem(dict, k, v);
    }
    dict
}

// ── Access ──

/// True when `key` is the runtime's 3-tuple slice representation.
fn is_slice_tuple(key: MbValue) -> bool {
    if let Some(kp) = key.as_ptr() {
        unsafe {
            if let ObjData::Tuple(ref items) = (*kp).data {
                return items.len() == 3;
            }
        }
    }
    false
}

/// dict[key] -> value  (raises KeyError if key not found)
pub fn mb_dict_getitem(dict: MbValue, key: MbValue) -> MbValue {
    // ET.Element stub: integer / slice subscripts read `_children`
    // (`e[0]`, `e[1:3]` → list of children) instead of dict keys.
    if key.as_int().is_some() || is_slice_tuple(key) {
        if let Some(children) = super::stdlib::xml_mod::element_stub_children(dict) {
            if let Some(kp) = key.as_ptr() {
                unsafe {
                    if let ObjData::Tuple(ref items) = (*kp).data {
                        if items.len() == 3 {
                            return super::list_ops::mb_list_slice_full(
                                children, items[0], items[1], items[2],
                            );
                        }
                    }
                }
            }
            return super::list_ops::mb_list_getitem(children, key);
        }
    }
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = match lock.try_read() {
                    Ok(g) => g,
                    Err(_) => lock.read().unwrap(),
                };
                if let Some(&v) = guard.get(&dk) {
                    super::rc::retain_if_ptr(v);
                    return v;
                }
                drop(guard);
                let key_repr = dict_key_raw_str(&dk);
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(key_repr)),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// dict.get(key, default) -> value
pub fn mb_dict_get(dict: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = match lock.try_read() {
                    Ok(g) => g,
                    Err(_) => lock.read().unwrap(),
                };
                if let Some(&val) = guard.get(&dk) {
                    super::rc::retain_if_ptr(val);
                    return val;
                }
                super::rc::retain_if_ptr(default);
                return default;
            }
        }
    }
    unsafe { super::rc::retain_if_ptr(default) };
    default
}

/// dict[key] = value
pub fn mb_dict_setitem(dict: MbValue, key: MbValue, value: MbValue) {
    // ET.Element stub: `e[i] = child` replaces the i-th child.
    if key.as_int().is_some() {
        if let Some(children) = super::stdlib::xml_mod::element_stub_children(dict) {
            super::list_ops::mb_list_setitem(children, key, value);
            return;
        }
    }
    // A mutable container can't be a mapping key (CPython: unhashable type).
    if let Some(tn) = super::set_ops::unhashable_type_name(key) {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!("unhashable type: '{tn}'"))),
        );
        return;
    }
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                super::rc::retain_if_ptr(value);
                let mut map = match lock.try_write() {
                    Ok(m) => m,
                    Err(_) => lock.write().unwrap(),
                };
                if let Some(existing) = map.get_mut(&dk) {
                    let old_val = *existing;
                    *existing = value;
                    super::rc::release_if_ptr(old_val);
                } else {
                    map.insert(dk, value);
                }
            }
        }
    }
}

/// del dict[key]
pub fn mb_dict_delitem(dict: MbValue, key: MbValue) {
    // ET.Element stub: `del e[i]` / `del e[a:b]` remove children.
    if key.as_int().is_some() || is_slice_tuple(key) {
        if let Some(children) = super::stdlib::xml_mod::element_stub_children(dict) {
            if key.as_int().is_some() {
                super::list_ops::mb_list_delitem(children, key);
                return;
            }
            // Slice deletion: normalize (start, stop, step) over the child
            // list and remove the selected indices in descending order.
            if let (Some(kp), Some(cp)) = (key.as_ptr(), children.as_ptr()) {
                unsafe {
                    if let (ObjData::Tuple(ref items), ObjData::List(ref lock)) =
                        (&(*kp).data, &(*cp).data)
                    {
                        let mut list = lock.write().unwrap();
                        let len = list.len() as i64;
                        let step = items[2].as_int().unwrap_or(1).max(1);
                        let clamp = |v: i64| -> i64 {
                            let v = if v < 0 { v + len } else { v };
                            v.clamp(0, len)
                        };
                        let start = clamp(items[0].as_int().unwrap_or(0));
                        let stop = clamp(items[1].as_int().unwrap_or(len));
                        let mut idx: Vec<i64> = Vec::new();
                        let mut p = start;
                        while p < stop {
                            idx.push(p);
                            p += step;
                        }
                        for &j in idx.iter().rev() {
                            if (j as usize) < list.len() {
                                let removed = list.remove(j as usize);
                                super::rc::release_if_ptr(removed);
                            }
                        }
                        return;
                    }
                }
            }
            return;
        }
    }
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.write().unwrap().shift_remove(&dk);
            }
        }
    }
}

// ── Query Methods ──

/// key in dict -> bool
pub fn mb_dict_contains(dict: MbValue, key: MbValue) -> MbValue {
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let guard = match lock.try_read() {
                    Ok(g) => g,
                    Err(_) => lock.read().unwrap(),
                };
                return MbValue::from_bool(guard.contains_key(&dk));
            }
        }
    }
    MbValue::from_bool(false)
}

/// Check if a value is a mapping (dict) — used for PEP 634 mapping pattern matching.
pub fn mb_is_mapping(val: MbValue) -> MbValue {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return MbValue::from_bool(matches!((*ptr).data, ObjData::Dict(_)));
        }
    }
    MbValue::from_bool(false)
}

/// len(dict) -> int
pub fn mb_dict_len(dict: MbValue) -> MbValue {
    // ET.Element stub: len(e) is the child count.
    if let Some(children) = super::stdlib::xml_mod::element_stub_children(dict) {
        if let Some(ptr) = children.as_ptr() {
            unsafe {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return MbValue::from_int(lock.read().unwrap().len() as i64);
                }
            }
        }
    }
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                return MbValue::from_int(lock.read().unwrap().len() as i64);
            }
        }
    }
    MbValue::from_int(0)
}

// ── Iteration Helpers ──

/// dict.keys() -> list of keys (preserving original types)
pub fn mb_dict_keys(dict: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let keys: Vec<MbValue> = map.keys().map(|k| dict_key_to_mbvalue(k)).collect();
                return MbValue::from_ptr(MbObject::new_list(keys));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

fn dict_view_class_kind(class_name: &str) -> Option<&'static str> {
    match class_name.rsplit('.').next().unwrap_or(class_name) {
        "dict_keys" | "KeysView" => Some("keys"),
        "dict_items" | "ItemsView" => Some("items"),
        "dict_values" | "ValuesView" => Some("values"),
        _ => None,
    }
}

fn dict_view_data(view: MbValue) -> Option<MbValue> {
    let ptr = view.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            dict_view_class_kind(class_name)?;
            return fields.read().unwrap().get("_data").copied();
        }
    }
    None
}

pub(crate) fn dict_view_kind(view: MbValue) -> Option<&'static str> {
    let ptr = view.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            return dict_view_class_kind(class_name);
        }
    }
    None
}

fn dict_view_make(dict: MbValue, class_name: &str) -> MbValue {
    unsafe { super::rc::retain_if_ptr(dict) };
    let view = MbValue::from_ptr(MbObject::new_instance(class_name.to_string()));
    if let Some(ptr) = view.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert("_data".to_string(), dict);
            }
        }
    }
    view
}

pub fn mb_dict_keys_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "dict_keys")
}

pub fn mb_dict_values_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "dict_values")
}

pub fn mb_dict_items_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "dict_items")
}

pub(crate) fn mb_dict_keys_abc_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "KeysView")
}

pub(crate) fn mb_dict_values_abc_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "ValuesView")
}

pub(crate) fn mb_dict_items_abc_view(dict: MbValue) -> MbValue {
    dict_view_make(dict, "ItemsView")
}

pub(crate) fn dict_view_elements(view: MbValue) -> Option<Vec<MbValue>> {
    let kind = dict_view_kind(view)?;
    let data = dict_view_data(view)?;
    let list = match kind {
        "keys" => mb_dict_keys(data),
        "items" => mb_dict_items(data),
        "values" => mb_dict_values(data),
        _ => return None,
    };
    Some(super::builtins::extract_items(list))
}

pub(crate) fn dict_view_len(view: MbValue) -> Option<i64> {
    let data = dict_view_data(view)?;
    mb_dict_len(data).as_int()
}

pub(crate) fn dict_view_is_setlike(view: MbValue) -> bool {
    matches!(dict_view_kind(view), Some("keys" | "items"))
}

pub(crate) fn dict_view_as_set(view: MbValue) -> Option<MbValue> {
    if !dict_view_is_setlike(view) {
        return None;
    }
    let items = dict_view_elements(view)?;
    let list = MbValue::from_ptr(MbObject::new_list_borrowed(items));
    Some(super::set_ops::mb_set_from_list(list))
}

fn dict_view_contains(view: MbValue, needle: MbValue) -> MbValue {
    let Some(kind) = dict_view_kind(view) else {
        return MbValue::from_bool(false);
    };
    let Some(data) = dict_view_data(view) else {
        return MbValue::from_bool(false);
    };
    if kind == "keys" {
        let result = mb_dict_contains(data, needle);
        if super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
        return result;
    }
    let haystack = match kind {
        "items" => mb_dict_items(data),
        "values" => mb_dict_values(data),
        _ => return MbValue::from_bool(false),
    };
    for item in super::builtins::extract_items(haystack) {
        if item.to_bits() == needle.to_bits()
            || super::builtins::mb_eq(item, needle).as_bool() == Some(true)
        {
            return MbValue::from_bool(true);
        }
        if super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
    }
    MbValue::from_bool(false)
}

fn is_set_or_frozenset(value: MbValue) -> bool {
    value.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Set(_) | ObjData::FrozenSet(_))
    })
}

fn reject_plain_non_iterable(value: MbValue) -> Option<MbValue> {
    let is_plain_non_iterable = value.is_none()
        || value.is_bool()
        || value.as_float().is_some()
        || value
            .as_int()
            .is_some_and(|i| {
                !super::iter::is_iter_handle(value)
                    && !super::file_io::is_file_handle(i as u64)
            });
    if !is_plain_non_iterable {
        return None;
    }
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "'{}' object is not iterable",
            super::builtins::value_type_name(value),
        ))),
    );
    Some(MbValue::none())
}

pub(crate) fn dict_view_eq(a: MbValue, b: MbValue) -> Option<bool> {
    let a_is_view = dict_view_kind(a).is_some();
    let b_is_view = dict_view_kind(b).is_some();
    if !a_is_view && !b_is_view {
        return None;
    }
    let a_is_setlike = dict_view_is_setlike(a);
    let b_is_setlike = dict_view_is_setlike(b);
    if a_is_setlike && (b_is_setlike || is_set_or_frozenset(b)) {
        let left = dict_view_as_set(a)?;
        let right = if b_is_setlike { dict_view_as_set(b)? } else { b };
        return Some(super::builtins::mb_eq(left, right).as_bool() == Some(true));
    }
    if b_is_setlike && is_set_or_frozenset(a) {
        let right = dict_view_as_set(b)?;
        return Some(super::builtins::mb_eq(a, right).as_bool() == Some(true));
    }
    Some(false)
}

pub(crate) fn dict_view_or(a: MbValue, b: MbValue) -> Option<MbValue> {
    if dict_view_is_setlike(a) {
        if let Some(result) = reject_plain_non_iterable(b) {
            return Some(result);
        }
        let left = dict_view_as_set(a)?;
        return Some(super::set_ops::mb_set_union(left, b));
    }
    if dict_view_is_setlike(b) {
        if let Some(result) = reject_plain_non_iterable(a) {
            return Some(result);
        }
        let right = dict_view_as_set(b)?;
        return Some(super::set_ops::mb_set_union(right, a));
    }
    None
}

pub(crate) fn dict_view_and(a: MbValue, b: MbValue) -> Option<MbValue> {
    if dict_view_is_setlike(a) {
        if let Some(result) = reject_plain_non_iterable(b) {
            return Some(result);
        }
        let left = dict_view_as_set(a)?;
        return Some(super::set_ops::mb_set_intersection(left, b));
    }
    if dict_view_is_setlike(b) {
        if let Some(result) = reject_plain_non_iterable(a) {
            return Some(result);
        }
        let right = dict_view_as_set(b)?;
        return Some(super::set_ops::mb_set_intersection(right, a));
    }
    None
}

pub(crate) fn dict_view_sub(a: MbValue, b: MbValue) -> Option<MbValue> {
    if dict_view_is_setlike(a) {
        if let Some(result) = reject_plain_non_iterable(b) {
            return Some(result);
        }
        let left = dict_view_as_set(a)?;
        return Some(super::set_ops::mb_set_difference(left, b));
    }
    if dict_view_is_setlike(b) && is_set_or_frozenset(a) {
        let right = dict_view_as_set(b)?;
        return Some(super::set_ops::mb_set_difference(a, right));
    }
    None
}

pub(crate) fn dict_view_xor(a: MbValue, b: MbValue) -> Option<MbValue> {
    if dict_view_is_setlike(a) {
        if let Some(result) = reject_plain_non_iterable(b) {
            return Some(result);
        }
        let left = dict_view_as_set(a)?;
        return Some(super::set_ops::mb_set_symmetric_difference(left, b));
    }
    if dict_view_is_setlike(b) {
        if let Some(result) = reject_plain_non_iterable(a) {
            return Some(result);
        }
        let right = dict_view_as_set(b)?;
        return Some(super::set_ops::mb_set_symmetric_difference(right, a));
    }
    None
}

pub(crate) fn dict_view_method(receiver: MbValue, name: &str, args: MbValue) -> Option<MbValue> {
    dict_view_kind(receiver)?;
    match name {
        "__contains__" => {
            let needle = super::builtins::extract_items(args)
                .first()
                .copied()
                .unwrap_or_else(MbValue::none);
            Some(dict_view_contains(receiver, needle))
        }
        "isdisjoint" if dict_view_is_setlike(receiver) => {
            let left = dict_view_as_set(receiver)?;
            let other = super::builtins::extract_items(args)
                .first()
                .copied()
                .unwrap_or_else(MbValue::none);
            Some(super::set_ops::mb_set_isdisjoint(left, other))
        }
        _ => None,
    }
}

pub(crate) fn mappingproxy_from_mapping(data: MbValue) -> MbValue {
    unsafe { super::rc::retain_if_ptr(data) };
    let proxy = MbValue::from_ptr(MbObject::new_instance("mappingproxy".to_string()));
    if let Some(ptr) = proxy.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                fields.write().unwrap().insert("_mapping".to_string(), data);
            }
        }
    }
    proxy
}

pub(crate) fn mappingproxy_mapping(proxy: MbValue) -> Option<MbValue> {
    let ptr = proxy.as_ptr()?;
    unsafe {
        if let ObjData::Instance { ref class_name, ref fields } = (*ptr).data {
            if class_name == "mappingproxy" {
                let data = fields.read().unwrap().get("_mapping").copied();
                return data.filter(|v| !v.is_none());
            }
        }
    }
    None
}

pub(crate) fn dict_view_mapping_proxy(view: MbValue) -> Option<MbValue> {
    dict_view_kind(view)?;
    let data = dict_view_data(view)?;
    let proxy = mappingproxy_from_mapping(data);
    Some(proxy)
}

/// dict.values() -> list of values
pub fn mb_dict_values(dict: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let values: Vec<MbValue> = map.values().copied().collect();
                // Values borrowed from the dict — retain.
                return MbValue::from_ptr(MbObject::new_list_borrowed(values));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

/// dict.items() -> list of (key, value) tuples
pub fn mb_dict_items(dict: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let items: Vec<MbValue> = map
                    .iter()
                    .map(|(k, v)| {
                        let key = dict_key_to_mbvalue(k);
                        // Value *v is borrowed from the dict — retain it.
                        // key is newly created (owned) — no retain needed.
                        super::rc::retain_if_ptr(*v);
                        MbValue::from_ptr(MbObject::new_tuple(vec![key, *v]))
                    })
                    .collect();
                return MbValue::from_ptr(MbObject::new_list(items));
            }
        }
    }
    MbValue::from_ptr(MbObject::new_list(Vec::new()))
}

// ── Mutation Methods ──

/// dict.pop(key, default) -> removed value or default
pub fn mb_dict_pop(dict: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                return lock.write().unwrap().shift_remove(&dk).unwrap_or(default);
            }
        }
    }
    default
}

/// dict.pop(key) without default — raises KeyError if key not found.
pub fn mb_dict_pop_no_default(dict: MbValue, key: MbValue) -> MbValue {
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                if let Some(v) = lock.write().unwrap().shift_remove(&dk) {
                    return v;
                }
                // Raise KeyError (CPython 3.12 format)
                let key_repr = dict_key_raw_str(&dk);
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(key_repr)),
                );
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// dict.setdefault(key, default) -> existing or newly set value
pub fn mb_dict_setdefault(dict: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let dk = to_dict_key(key);
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let val = *lock.write().unwrap().entry(dk).or_insert(default);
                super::rc::retain_if_ptr(val);
                return val;
            }
        }
    }
    default
}

/// dict.update(other) — merge other dict into this one
pub fn mb_dict_update(dict: MbValue, other: MbValue) {
    unsafe {
        let Some(dict_ptr) = dict.as_ptr() else {
            return;
        };
        let ObjData::Dict(ref dict_lock) = (*dict_ptr).data else {
            return;
        };

        // Collect pairs from `other` first to avoid holding two locks.
        let pairs: Vec<(DictKey, MbValue)> = if let Some(ptr) = other.as_ptr() {
            match &(*ptr).data {
                ObjData::Dict(ref lock) => lock
                    .read()
                    .unwrap()
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect(),
                ObjData::List(ref lock) => {
                    let items = lock.read().unwrap().clone();
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        if let Some(pair_ptr) = item.as_ptr() {
                            let pair = match &(*pair_ptr).data {
                                ObjData::Tuple(ref t) if t.len() == 2 => Some((t[0], t[1])),
                                ObjData::List(ref l) => {
                                    let l = l.read().unwrap();
                                    if l.len() == 2 {
                                        Some((l[0], l[1]))
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            if let Some((k, v)) = pair {
                                out.push((to_dict_key(k), v));
                                super::rc::retain_if_ptr(v);
                            }
                        }
                    }
                    out
                }
                ObjData::Tuple(ref items) => {
                    let mut out = Vec::with_capacity(items.len());
                    for item in items {
                        if let Some(pair_ptr) = item.as_ptr() {
                            let pair = match &(*pair_ptr).data {
                                ObjData::Tuple(ref t) if t.len() == 2 => Some((t[0], t[1])),
                                ObjData::List(ref l) => {
                                    let l = l.read().unwrap();
                                    if l.len() == 2 {
                                        Some((l[0], l[1]))
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            if let Some((k, v)) = pair {
                                out.push((to_dict_key(k), v));
                                super::rc::retain_if_ptr(v);
                            }
                        }
                    }
                    out
                }
                ObjData::Instance { .. } => {
                    // dict-like collections instances (defaultdict, Counter,
                    // OrderedDict) iterate as keys, not pairs. Pull pairs
                    // from the backing `_data` dict via the registered
                    // class fast-path. (#1634)
                    if let Some(backing) = super::class::unwrap_dictlike_data(other) {
                        if let Some(bp) = backing.as_ptr() {
                            if let ObjData::Dict(ref src) = (*bp).data {
                                src.read()
                                    .unwrap()
                                    .iter()
                                    .map(|(k, v)| {
                                        super::rc::retain_if_ptr(*v);
                                        (k.clone(), *v)
                                    })
                                    .collect()
                            } else {
                                return;
                            }
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        } else {
            return;
        };

        let mut map = dict_lock.write().unwrap();
        for (k, v) in pairs {
            map.insert(k, v);
        }
    }
}

/// dict.clear()
pub fn mb_dict_clear(dict: MbValue) {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                lock.write().unwrap().clear();
            }
        }
    }
}

/// dict.copy() -> shallow copy
pub fn mb_dict_copy(dict: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let cloned = lock.read().unwrap().clone();
                let obj = Box::new(MbObject {
                    header: super::rc::MbObjectHeader {
                        rc: std::sync::atomic::AtomicU32::new(1),
                        kind: super::rc::ObjKind::Dict,
                    },
                    data: ObjData::Dict(crate::runtime::rc::MbRwLock::new(cloned)),
                });
                return MbValue::from_ptr(Box::into_raw(obj));
            }
        }
    }
    mb_dict_new()
}

/// dict.popitem() -> (key, value) tuple; removes and returns the last
/// inserted (LIFO) item — matches CPython 3.7+ semantics.
/// `dict.fromkeys(iterable, value=None)` — build a new dict mapping each key
/// from `iterable` to `value`. Later duplicate keys keep the last (insertion)
/// position, matching CPython.
pub fn mb_dict_fromkeys(keys: MbValue, value: MbValue) -> MbValue {
    let out = mb_dict_new();
    let handle = super::iter::mb_iter(keys);
    if handle.is_none() {
        // A non-iterable keys argument is a TypeError (e.g. fromkeys(3)),
        // not an empty result.
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("object is not iterable".to_string())),
        );
        return MbValue::none();
    }
    loop {
        if super::iter::mb_has_next(handle).as_bool() != Some(true) {
            break;
        }
        let k = super::iter::mb_next(handle);
        if k.is_none() && super::iter::mb_has_next(handle).as_bool() != Some(true) {
            break;
        }
        unsafe {
            super::rc::retain_if_ptr(value);
        }
        mb_dict_setitem(out, k, value);
    }
    out
}

pub fn mb_dict_popitem(dict: MbValue) -> MbValue {
    unsafe {
        if let Some(ptr) = dict.as_ptr() {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let mut map = lock.write().unwrap();
                if let Some((k, v)) = map.pop() {
                    let key = dict_key_to_mbvalue(&k);
                    return MbValue::from_ptr(MbObject::new_tuple(vec![key, v]));
                }
            }
        }
    }
    // KeyError: 'popitem(): dictionary is empty'
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("KeyError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "popitem(): dictionary is empty".to_string(),
        )),
    );
    MbValue::none()
}

// ── Comparison ──

/// dict == dict -> bool (same keys and values)
pub fn mb_dict_eq(a: MbValue, b: MbValue) -> MbValue {
    // Delegate to the generic deep-equality entry point so dict values
    // (which may be heap objects like strings or nested containers) are
    // compared structurally, not by NaN-boxed bit pattern.
    super::builtins::mb_eq(a, b)
}

/// dict | dict -> merged dict (Python 3.9+)
pub fn mb_dict_merge(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        let ma = a.as_ptr().and_then(|p| {
            if let ObjData::Dict(ref lock) = (*p).data {
                Some(lock.read().unwrap().clone())
            } else {
                None
            }
        });
        let mb_map = b.as_ptr().and_then(|p| {
            if let ObjData::Dict(ref lock) = (*p).data {
                Some(lock.read().unwrap().clone())
            } else {
                None
            }
        });
        if let (Some(mut merged), Some(mb_map)) = (ma, mb_map) {
            for (k, v) in mb_map {
                merged.insert(k, v);
            }
            let obj = Box::new(MbObject {
                header: super::rc::MbObjectHeader {
                    rc: std::sync::atomic::AtomicU32::new(1),
                    kind: super::rc::ObjKind::Dict,
                },
                data: ObjData::Dict(crate::runtime::rc::MbRwLock::new(merged)),
            });
            return MbValue::from_ptr(Box::into_raw(obj));
        }
    }
    MbValue::none()
}

// ── PEP 584 Merge Operators ──

/// dict | dict -> new merged dict (Python 3.9+ PEP 584, `__or__`).
/// Creates a NEW dict: copies all pairs from `a`, then merges `b` (b wins on conflict).
/// Returns the `NotImplemented` singleton if either operand is not a dict, so
/// that the operator dispatcher (or `dict.__or__(other)` called directly) can
/// fall back / raise TypeError at the operator level — matching CPython, where
/// `dict.__or__` only accepts another mapping and otherwise returns
/// `NotImplemented` rather than eagerly raising.
// REQ: R7
pub fn mb_dict_or(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        let is_a_dict = a
            .as_ptr()
            .map_or(false, |p| matches!((*p).data, ObjData::Dict(_)));
        let is_b_dict = b
            .as_ptr()
            .map_or(false, |p| matches!((*p).data, ObjData::Dict(_)));
        if !is_a_dict || !is_b_dict {
            return MbValue::not_implemented();
        }
        // Clone a's map, then merge b's entries
        let mut merged = a
            .as_ptr()
            .and_then(|p| {
                if let ObjData::Dict(ref lock) = (*p).data {
                    Some(lock.read().unwrap().clone())
                } else {
                    None
                }
            })
            .expect("a is dict — checked above");
        let b_entries = b
            .as_ptr()
            .and_then(|p| {
                if let ObjData::Dict(ref lock) = (*p).data {
                    Some(lock.read().unwrap().clone())
                } else {
                    None
                }
            })
            .expect("b is dict — checked above");
        for (k, v) in b_entries {
            merged.insert(k, v);
        }
        let obj = Box::new(MbObject {
            header: super::rc::MbObjectHeader {
                rc: std::sync::atomic::AtomicU32::new(1),
                kind: super::rc::ObjKind::Dict,
            },
            data: ObjData::Dict(crate::runtime::rc::MbRwLock::new(merged)),
        });
        MbValue::from_ptr(Box::into_raw(obj))
    }
}

/// Collect (key, value) pairs from `other` for an in-place dict merge
/// (`dict.__ior__` / `dict |= other`). Unlike `__or__`, the in-place form is as
/// permissive as `dict.update`: it accepts another mapping OR any iterable of
/// key/value pairs. Returns `Ok(pairs)` on success; `Err(exc)` carries the
/// exception class name to raise — `TypeError` when `other` is not iterable
/// (e.g. `None`), `ValueError` when an element of an iterable is not a 2-element
/// sequence (CPython: "dictionary update sequence element #N has length L; 2 is
/// required"). The string case iterates characters, each of length 1.
unsafe fn collect_ior_pairs(other: MbValue) -> Result<Vec<(DictKey, MbValue)>, &'static str> {
    let Some(ptr) = other.as_ptr() else {
        // None / unboxed non-iterable scalar → not iterable.
        return Err("TypeError");
    };
    // Build a list of candidate "elements" to interpret as 2-tuples.
    let elements: Vec<MbValue> = match &(*ptr).data {
        ObjData::Dict(ref lock) => {
            // Mapping fast-path: copy entries directly (no pair unpacking).
            return Ok(lock
                .read()
                .unwrap()
                .iter()
                .map(|(k, v)| {
                    super::rc::retain_if_ptr(*v);
                    (k.clone(), *v)
                })
                .collect());
        }
        ObjData::List(ref lock) => lock.read().unwrap().to_vec(),
        ObjData::Tuple(ref items) => items.iter().copied().collect(),
        ObjData::Set(ref lock) => lock.read().unwrap().iter().copied().collect(),
        ObjData::Str(ref s) => {
            // Each character is a length-1 string → if any exist, the first one
            // fails the length-2 requirement with a ValueError.
            if s.is_empty() {
                return Ok(Vec::new());
            }
            return Err("ValueError");
        }
        ObjData::Instance { .. } => {
            // dict-like collections instances iterate as pairs via their
            // backing dict (defaultdict / Counter / OrderedDict).
            if let Some(backing) = super::class::unwrap_dictlike_data(other) {
                if let Some(bp) = backing.as_ptr() {
                    if let ObjData::Dict(ref src) = (*bp).data {
                        return Ok(src
                            .read()
                            .unwrap()
                            .iter()
                            .map(|(k, v)| {
                                super::rc::retain_if_ptr(*v);
                                (k.clone(), *v)
                            })
                            .collect());
                    }
                }
            }
            return Err("TypeError");
        }
        _ => return Err("TypeError"),
    };
    // Interpret each element as a (key, value) 2-tuple/2-list.
    let mut out = Vec::with_capacity(elements.len());
    for item in elements {
        let pair = item.as_ptr().and_then(|pp| match &(*pp).data {
            ObjData::Tuple(ref t) if t.len() == 2 => Some((t[0], t[1])),
            ObjData::List(ref l) => {
                let l = l.read().unwrap();
                if l.len() == 2 {
                    Some((l[0], l[1]))
                } else {
                    None
                }
            }
            _ => None,
        });
        match pair {
            Some((k, v)) => {
                super::rc::retain_if_ptr(v);
                out.push((to_dict_key(k), v));
            }
            // Element is not a 2-sequence → CPython raises ValueError.
            None => return Err("ValueError"),
        }
    }
    Ok(out)
}

/// dict |= other -> merged in-place (Python 3.9+ PEP 584, `__ior__`).
/// Merges `other` into `a` in-place and returns `a`. Like `dict.update`, the
/// in-place merge accepts another mapping OR any iterable of key/value pairs
/// (this is more permissive than `__or__`, which only accepts a mapping).
/// Raises TypeError when `other` is not iterable and ValueError when an element
/// is not a 2-sequence, matching CPython.
// REQ: R7
pub fn mb_dict_ior(a: MbValue, b: MbValue) -> MbValue {
    unsafe {
        let is_a_dict = a
            .as_ptr()
            .map_or(false, |p| matches!((*p).data, ObjData::Dict(_)));
        if !is_a_dict {
            // The receiver is not a dict — defer to the generic operator path.
            return MbValue::not_implemented();
        }
        // Collect b's pairs first (avoids holding two locks / nested borrows).
        let pairs = match collect_ior_pairs(b) {
            Ok(pairs) => pairs,
            Err("ValueError") => {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(
                        "dictionary update sequence element #0 has length 1; 2 is required"
                            .to_string(),
                    )),
                );
                return MbValue::none();
            }
            Err(_) => {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "'{}' object is not iterable",
                        super::builtins::value_type_name(b)
                    ))),
                );
                return MbValue::none();
            }
        };
        // Merge into a in-place.
        if let Some(pa) = a.as_ptr() {
            if let ObjData::Dict(ref lock) = (*pa).data {
                let mut map = lock.write().unwrap();
                for (k, v) in pairs {
                    map.insert(k, v);
                }
            }
        }
        super::rc::retain_if_ptr(a);
        a
    }
}

// ── Method Dispatch ──

/// Dispatch a method call on a dict object.
pub fn dispatch_dict_method(name: &str, receiver: MbValue, args: MbValue) -> MbValue {
    let arg = |i: usize| -> MbValue {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock
                        .read()
                        .unwrap()
                        .get(i)
                        .copied()
                        .unwrap_or(MbValue::none());
                }
            }
            MbValue::none()
        }
    };
    let argc = || -> usize {
        unsafe {
            if let Some(ptr) = args.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
            0
        }
    };
    let stub_class = dict_stub_class(receiver);
    // __class__-tagged xml stubs (Element / ElementTree / XMLParser /
    // TreeBuilder) route
    // their method surface to xml_mod; None falls through to plain-dict
    // semantics (only for dunders the dict intrinsics already guard).
    if let Some(ref cls) = stub_class {
        if matches!(cls.as_str(), "Element" | "ElementTree" | "XMLParser" | "TreeBuilder") {
            if let Some(result) =
                super::stdlib::xml_mod::dispatch_xml_stub_method(cls, name, receiver, args)
            {
                return result;
            }
        }
    }
    // __class__-tagged tarfile stubs (TarFile / TarInfo) route their method
    // surface to tarfile_mod; None falls through to plain-dict semantics.
    if let Some(ref cls) = stub_class {
        if matches!(cls.as_str(), "TarFile" | "TarInfo") {
            if let Some(result) =
                super::stdlib::tarfile_mod::dispatch_tar_stub_method(cls, name, receiver, args)
            {
                return result;
            }
        }
    }
    if stub_class.as_deref() == Some("ConfigParser") {
        match name {
            "read_string" => {
                return super::stdlib::configparser_mod::mb_configparser_read_string(
                    receiver,
                    arg(0),
                );
            }
            "get" => {
                return super::stdlib::configparser_mod::mb_configparser_get(
                    receiver,
                    arg(0),
                    arg(1),
                );
            }
            "set" => {
                return super::stdlib::configparser_mod::mb_configparser_set(
                    receiver,
                    arg(0),
                    arg(1),
                    arg(2),
                );
            }
            "sections" => {
                return super::stdlib::configparser_mod::mb_configparser_sections(receiver);
            }
            "options" => {
                return super::stdlib::configparser_mod::mb_configparser_options(receiver, arg(0));
            }
            _ => {}
        }
    }
    match name {
        "get" => {
            let default = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_dict_get(receiver, arg(0), default)
        }
        "setdefault" => {
            let default = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_dict_setdefault(receiver, arg(0), default)
        }
        "keys" => mb_dict_keys_view(receiver),
        "values" => mb_dict_values_view(receiver),
        "items" => mb_dict_items_view(receiver),
        "pop" => {
            if argc() > 1 {
                mb_dict_pop(receiver, arg(0), arg(1))
            } else {
                // No default — raise KeyError if key not found (CPython semantics)
                mb_dict_pop_no_default(receiver, arg(0))
            }
        }
        "update" => {
            // Method calls with kwargs pass the kwargs dict as the last
            // positional arg. `d.update()` has 1 trailing empty dict,
            // `d.update({...})` has 2 args (positional dict + kwargs),
            // `d.update(b=2)` has 1 arg (kwargs dict only).
            let n = argc();
            if n == 0 {
                return MbValue::none();
            }
            // Merge every positional arg that is itself a dict/list-of-pairs.
            // The trailing arg is always the kwargs dict produced by HIR
            // lowering when the call has keyword args — merge it too.
            for i in 0..n {
                mb_dict_update(receiver, arg(i));
            }
            MbValue::none()
        }
        "clear" => {
            mb_dict_clear(receiver);
            MbValue::none()
        }
        "copy" => mb_dict_copy(receiver),
        "popitem" => mb_dict_popitem(receiver),
        // `d.fromkeys(iterable[, value])` — a classmethod, but Python also
        // exposes it on instances. Builds a NEW dict (ignores the receiver's
        // contents) mapping each key to `value` (default None).
        "fromkeys" => {
            let value = if argc() > 1 { arg(1) } else { MbValue::none() };
            mb_dict_fromkeys(arg(0), value)
        }
        // ── Explicit dunder access (for `d.__getitem__(k)` etc.) ──
        // CPython exposes these on every dict; previously each raised
        // AttributeError because the dispatch table was method-only.
        "__getitem__" => mb_dict_getitem(receiver, arg(0)),
        "__setitem__" => {
            mb_dict_setitem(receiver, arg(0), arg(1));
            MbValue::none()
        }
        "__delitem__" => {
            mb_dict_delitem(receiver, arg(0));
            MbValue::none()
        }
        "__contains__" => mb_dict_contains(receiver, arg(0)),
        "__len__" => mb_dict_len(receiver),
        "__iter__" => super::iter::mb_iter(receiver),
        "__or__" => mb_dict_or(receiver, arg(0)),
        "__ror__" => mb_dict_or(arg(0), receiver),
        // `d.__ior__(other)` — PEP 584 in-place merge. Accepts a mapping or any
        // iterable of key/value pairs (like `dict.update`), mutates the receiver
        // in place, and returns it. The `|=` operator reaches the same
        // `mb_dict_ior` via `mb_ior` → `mb_inplace` → `mb_bitor`.
        "__ior__" => mb_dict_ior(receiver, arg(0)),
        "__eq__" => mb_dict_eq(receiver, arg(0)),
        "__ne__" => {
            let eq = mb_dict_eq(receiver, arg(0));
            MbValue::from_bool(eq.as_bool() != Some(true))
        }
        "__repr__" | "__str__" => super::builtins::mb_repr(receiver),
        _ => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'dict' object has no attribute '{name}'"
                ))),
            );
            MbValue::none()
        }
    }
}

/// Read the `__class__` marker of a string dict-stub (e.g. ConfigParser),
/// if present.
pub(crate) fn dict_stub_class(receiver: MbValue) -> Option<String> {
    unsafe {
        let ptr = receiver.as_ptr()?;
        if let ObjData::Dict(ref lock) = (*ptr).data {
            let map = lock.read().unwrap();
            if let Some(v) = map.get(&DictKey::Str("__class__".to_string())) {
                return v.as_ptr().and_then(|p| {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                });
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_val(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    // ── helpers ──

    /// Read the list length from an MbValue that wraps a List.
    fn list_len(v: MbValue) -> usize {
        unsafe {
            if let Some(ptr) = v.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock.read().unwrap().len();
                }
            }
        }
        0
    }

    /// Read list element at index i.
    fn list_get(v: MbValue, i: usize) -> MbValue {
        unsafe {
            if let Some(ptr) = v.as_ptr() {
                if let ObjData::List(ref lock) = (*ptr).data {
                    return lock
                        .read()
                        .unwrap()
                        .get(i)
                        .copied()
                        .unwrap_or(MbValue::none());
                }
            }
        }
        MbValue::none()
    }

    /// Extract the string from a Str MbValue.
    fn extract_str(v: MbValue) -> Option<String> {
        unsafe {
            if let Some(ptr) = v.as_ptr() {
                if let ObjData::Str(ref s) = (*ptr).data {
                    return Some(s.clone());
                }
            }
        }
        None
    }

    /// Build args list for dispatch_dict_method.
    fn make_args(vals: Vec<MbValue>) -> MbValue {
        MbValue::from_ptr(MbObject::new_list(vals))
    }

    // ── new ──

    #[test]
    fn test_new_empty() {
        let d = mb_dict_new();
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    // ── setitem / getitem ──

    #[test]
    fn test_set_get_str_key() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("key"), MbValue::from_int(42));
        assert_eq!(mb_dict_getitem(d, str_val("key")).as_int(), Some(42));
    }

    /// Two literal tuples with structurally-equal elements must hash to the
    /// same DictKey bucket so `m[(0, 1)]` round-trips after `m[(0, 1)] = …`
    /// (#1608). Pre-fix this fell through to `DictKey::Other(bits)` which
    /// keyed by tuple-object identity, yielding a `KeyError` on the second
    /// literal because each literal allocated a fresh heap tuple.
    #[test]
    fn test_set_get_tuple_key_structural() {
        let d = mb_dict_new();
        let k1 = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
        ]));
        let k2 = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
        ]));
        assert_ne!(
            k1.to_bits(),
            k2.to_bits(),
            "literals must be distinct objects"
        );
        mb_dict_setitem(d, k1, MbValue::from_int(42));
        assert_eq!(mb_dict_getitem(d, k2).as_int(), Some(42));
    }

    #[test]
    fn test_get_missing_key() {
        let d = mb_dict_new();
        assert!(mb_dict_getitem(d, str_val("missing")).is_none());
    }

    #[test]
    fn test_set_overwrite() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(1));
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(2));
        assert_eq!(mb_dict_getitem(d, str_val("k")).as_int(), Some(2));
        assert_eq!(mb_dict_len(d).as_int(), Some(1));
    }

    #[test]
    fn test_setitem_int_key() {
        let d = mb_dict_new();
        mb_dict_setitem(d, MbValue::from_int(10), MbValue::from_int(100));
        assert_eq!(
            mb_dict_getitem(d, MbValue::from_int(10)).as_int(),
            Some(100)
        );
    }

    #[test]
    fn test_getitem_non_dict() {
        assert!(mb_dict_getitem(MbValue::from_int(1), str_val("k")).is_none());
    }

    // ── get (with default) ──

    #[test]
    fn test_get_found() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        assert_eq!(
            mb_dict_get(d, str_val("a"), MbValue::from_int(-1)).as_int(),
            Some(1)
        );
    }

    #[test]
    fn test_get_missing_returns_default() {
        let d = mb_dict_new();
        assert_eq!(
            mb_dict_get(d, str_val("x"), MbValue::from_int(99)).as_int(),
            Some(99)
        );
    }

    #[test]
    fn test_get_non_dict() {
        let r = mb_dict_get(MbValue::from_int(0), str_val("k"), MbValue::from_int(5));
        assert_eq!(r.as_int(), Some(5));
    }

    // ── delitem ──

    #[test]
    fn test_delitem() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        mb_dict_delitem(d, str_val("a"));
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    #[test]
    fn test_delitem_missing_noop() {
        let d = mb_dict_new();
        mb_dict_delitem(d, str_val("z"));
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    // ── contains ──

    #[test]
    fn test_contains_true() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        assert_eq!(mb_dict_contains(d, str_val("a")).as_bool(), Some(true));
    }

    #[test]
    fn test_contains_false() {
        let d = mb_dict_new();
        assert_eq!(mb_dict_contains(d, str_val("z")).as_bool(), Some(false));
    }

    #[test]
    fn test_contains_non_dict() {
        assert_eq!(
            mb_dict_contains(MbValue::from_int(0), str_val("k")).as_bool(),
            Some(false)
        );
    }

    // ── len ──

    #[test]
    fn test_len() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        mb_dict_setitem(d, str_val("b"), MbValue::from_int(2));
        assert_eq!(mb_dict_len(d).as_int(), Some(2));
    }

    #[test]
    fn test_len_non_dict() {
        assert_eq!(mb_dict_len(MbValue::from_int(0)).as_int(), Some(0));
    }

    // ── keys / values / items ──

    #[test]
    fn test_keys() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("x"), MbValue::from_int(1));
        let keys = mb_dict_keys(d);
        assert_eq!(list_len(keys), 1);
    }

    #[test]
    fn test_values() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("x"), MbValue::from_int(7));
        let vals = mb_dict_values(d);
        assert_eq!(list_len(vals), 1);
    }

    #[test]
    fn test_items() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("x"), MbValue::from_int(7));
        let items = mb_dict_items(d);
        assert_eq!(list_len(items), 1);
    }

    #[test]
    fn test_keys_empty() {
        let d = mb_dict_new();
        let keys = mb_dict_keys(d);
        assert_eq!(list_len(keys), 0);
    }

    // ── keys/values/items content verification ──

    #[test]
    fn test_keys_returns_correct_strings() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("x"), MbValue::from_int(10));
        mb_dict_setitem(d, str_val("y"), MbValue::from_int(20));
        let keys = mb_dict_keys(d);
        assert_eq!(list_len(keys), 2);
        let mut names: Vec<String> = (0..2)
            .filter_map(|i| extract_str(list_get(keys, i)))
            .collect();
        names.sort();
        assert_eq!(names, vec!["x", "y"]);
    }

    #[test]
    fn test_values_returns_correct_ints() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(10));
        mb_dict_setitem(d, str_val("b"), MbValue::from_int(20));
        let vals = mb_dict_values(d);
        assert_eq!(list_len(vals), 2);
        let mut ints: Vec<i64> = (0..2).filter_map(|i| list_get(vals, i).as_int()).collect();
        ints.sort();
        assert_eq!(ints, vec![10, 20]);
    }

    #[test]
    fn test_items_returns_key_value_tuples() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(7));
        let items = mb_dict_items(d);
        assert_eq!(list_len(items), 1);
        let tuple = list_get(items, 0);
        unsafe {
            if let Some(ptr) = tuple.as_ptr() {
                if let ObjData::Tuple(ref elems) = (*ptr).data {
                    assert_eq!(elems.len(), 2);
                    assert_eq!(extract_str(elems[0]), Some("k".to_string()));
                    assert_eq!(elems[1].as_int(), Some(7));
                } else {
                    panic!("expected Tuple");
                }
            } else {
                panic!("expected ptr");
            }
        }
    }

    // ── pop ──

    #[test]
    fn test_pop_found() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(10));
        assert_eq!(
            mb_dict_pop(d, str_val("k"), MbValue::from_int(-1)).as_int(),
            Some(10)
        );
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    #[test]
    fn test_pop_missing() {
        let d = mb_dict_new();
        assert_eq!(
            mb_dict_pop(d, str_val("k"), MbValue::from_int(-1)).as_int(),
            Some(-1)
        );
    }

    // ── setdefault ──

    #[test]
    fn test_setdefault_missing() {
        let d = mb_dict_new();
        let r = mb_dict_setdefault(d, str_val("k"), MbValue::from_int(42));
        assert_eq!(r.as_int(), Some(42));
        assert_eq!(mb_dict_getitem(d, str_val("k")).as_int(), Some(42));
    }

    #[test]
    fn test_setdefault_existing() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(1));
        let r = mb_dict_setdefault(d, str_val("k"), MbValue::from_int(99));
        assert_eq!(r.as_int(), Some(1));
    }

    // ── update ──

    #[test]
    fn test_update() {
        let d1 = mb_dict_new();
        mb_dict_setitem(d1, str_val("a"), MbValue::from_int(1));
        let d2 = mb_dict_new();
        mb_dict_setitem(d2, str_val("b"), MbValue::from_int(2));
        mb_dict_update(d1, d2);
        assert_eq!(mb_dict_len(d1).as_int(), Some(2));
        assert_eq!(mb_dict_getitem(d1, str_val("b")).as_int(), Some(2));
    }

    #[test]
    fn test_update_overwrites() {
        let d1 = mb_dict_new();
        mb_dict_setitem(d1, str_val("a"), MbValue::from_int(1));
        let d2 = mb_dict_new();
        mb_dict_setitem(d2, str_val("a"), MbValue::from_int(99));
        mb_dict_update(d1, d2);
        assert_eq!(mb_dict_getitem(d1, str_val("a")).as_int(), Some(99));
    }

    // ── clear ──

    #[test]
    fn test_clear() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        mb_dict_clear(d);
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    // ── copy ──

    #[test]
    fn test_copy() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let cp = mb_dict_copy(d);
        assert_eq!(mb_dict_len(cp).as_int(), Some(1));
        assert_eq!(mb_dict_getitem(cp, str_val("a")).as_int(), Some(1));
    }

    #[test]
    fn test_copy_independence() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let cp = mb_dict_copy(d);
        mb_dict_setitem(d, str_val("b"), MbValue::from_int(2));
        assert_eq!(mb_dict_len(cp).as_int(), Some(1));
    }

    // ── eq ──

    #[test]
    fn test_eq_same() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("k"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("k"), MbValue::from_int(1));
        assert_eq!(mb_dict_eq(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_different_value() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("k"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("k"), MbValue::from_int(2));
        assert_eq!(mb_dict_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_different_keys() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("a"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("b"), MbValue::from_int(1));
        assert_eq!(mb_dict_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_different_len() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("a"), MbValue::from_int(1));
        let b = mb_dict_new();
        assert_eq!(mb_dict_eq(a, b).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_non_dict() {
        // Delegates to mb_eq, so non-dict inputs still get sane equality.
        assert_eq!(
            mb_dict_eq(MbValue::from_int(1), MbValue::from_int(2)).as_bool(),
            Some(false)
        );
    }

    // ── merge ──

    #[test]
    fn test_merge() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("a"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("b"), MbValue::from_int(2));
        let m = mb_dict_merge(a, b);
        assert_eq!(mb_dict_len(m).as_int(), Some(2));
    }

    #[test]
    fn test_merge_overwrite() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("k"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("k"), MbValue::from_int(2));
        let m = mb_dict_merge(a, b);
        assert_eq!(mb_dict_getitem(m, str_val("k")).as_int(), Some(2));
    }

    #[test]
    fn test_merge_non_dict() {
        assert!(mb_dict_merge(MbValue::from_int(1), MbValue::from_int(2)).is_none());
    }

    // ── PEP 584 dict | dict ──

    // REQ: R7
    #[test]
    fn test_dict_or_basic() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("a"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("b"), MbValue::from_int(2));
        let result = mb_dict_or(a, b);
        assert_eq!(mb_dict_len(result).as_int(), Some(2));
        assert_eq!(mb_dict_getitem(result, str_val("a")).as_int(), Some(1));
        assert_eq!(mb_dict_getitem(result, str_val("b")).as_int(), Some(2));
    }

    // REQ: R7
    #[test]
    fn test_dict_or_overwrite() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("k"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("k"), MbValue::from_int(2));
        let result = mb_dict_or(a, b);
        // b wins on key conflict
        assert_eq!(mb_dict_getitem(result, str_val("k")).as_int(), Some(2));
        assert_eq!(mb_dict_len(result).as_int(), Some(1));
    }

    // REQ: R7
    #[test]
    fn test_dict_or_does_not_mutate_operands() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("a"), MbValue::from_int(1));
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("b"), MbValue::from_int(2));
        let _result = mb_dict_or(a, b);
        // a must be unchanged
        assert_eq!(mb_dict_len(a).as_int(), Some(1));
        assert!(mb_dict_getitem(a, str_val("b")).is_none());
        // b must be unchanged
        assert_eq!(mb_dict_len(b).as_int(), Some(1));
        assert!(mb_dict_getitem(b, str_val("a")).is_none());
    }

    // REQ: R7
    #[test]
    fn test_dict_or_non_dict_returns_not_implemented() {
        // CPython's `dict.__or__` only accepts another mapping; for any other
        // right operand it returns `NotImplemented` (so the operator can fall
        // back / raise TypeError at the operator level) rather than eagerly
        // raising itself.
        super::super::exception::mb_clear_exception();
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let result = mb_dict_or(d, MbValue::from_int(42));
        assert!(
            result.is_not_implemented(),
            "dict.__or__(non-dict) must return NotImplemented"
        );
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(false),
            "dict.__or__(non-dict) must NOT raise",
        );
        super::super::exception::mb_clear_exception();
    }

    // ── PEP 584 dict |= dict ──

    // REQ: R7
    #[test]
    fn test_dict_ior_basic() {
        let d1 = mb_dict_new();
        mb_dict_setitem(d1, str_val("a"), MbValue::from_int(1));
        let d2 = mb_dict_new();
        mb_dict_setitem(d2, str_val("b"), MbValue::from_int(2));
        let result = mb_dict_ior(d1, d2);
        // result is d1 (same pointer)
        assert_eq!(mb_dict_len(result).as_int(), Some(2));
        assert_eq!(mb_dict_getitem(result, str_val("a")).as_int(), Some(1));
        assert_eq!(mb_dict_getitem(result, str_val("b")).as_int(), Some(2));
        // d1 itself is mutated
        assert_eq!(mb_dict_len(d1).as_int(), Some(2));
    }

    // REQ: R7
    #[test]
    fn test_dict_ior_non_dict_raises_type_error() {
        super::super::exception::mb_clear_exception();
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let result = mb_dict_ior(d, MbValue::from_int(42));
        assert!(result.is_none(), "dict |= int must return none sentinel");
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "TypeError must be pending after dict |= int",
        );
        let exc = super::super::exception::mb_get_exception();
        let exc_type = super::super::exception::get_exception_type_pub(exc);
        assert_eq!(exc_type.as_deref(), Some("TypeError"));
        super::super::exception::mb_clear_exception();
    }

    // REQ: R7 — edge cases
    #[test]
    fn test_dict_or_empty_lhs() {
        let empty = mb_dict_new();
        let b = mb_dict_new();
        mb_dict_setitem(b, str_val("x"), MbValue::from_int(1));
        let result = mb_dict_or(empty, b);
        assert_eq!(mb_dict_len(result).as_int(), Some(1));
        assert_eq!(mb_dict_getitem(result, str_val("x")).as_int(), Some(1));
    }

    #[test]
    fn test_dict_or_empty_rhs() {
        let a = mb_dict_new();
        mb_dict_setitem(a, str_val("x"), MbValue::from_int(1));
        let empty = mb_dict_new();
        let result = mb_dict_or(a, empty);
        assert_eq!(mb_dict_len(result).as_int(), Some(1));
    }

    #[test]
    fn test_dict_or_both_empty() {
        let result = mb_dict_or(mb_dict_new(), mb_dict_new());
        assert_eq!(mb_dict_len(result).as_int(), Some(0));
    }

    // ── dispatch_dict_method ──

    #[test]
    fn test_dispatch_get() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(5));
        let args = make_args(vec![str_val("k")]);
        let r = dispatch_dict_method("get", d, args);
        assert_eq!(r.as_int(), Some(5));
    }

    #[test]
    fn test_dispatch_get_default() {
        let d = mb_dict_new();
        let args = make_args(vec![str_val("k"), MbValue::from_int(99)]);
        let r = dispatch_dict_method("get", d, args);
        assert_eq!(r.as_int(), Some(99));
    }

    #[test]
    fn test_dispatch_keys() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let args = make_args(vec![]);
        let r = dispatch_dict_method("keys", d, args);
        assert!(r.is_ptr());
    }

    #[test]
    fn test_dispatch_values() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let args = make_args(vec![]);
        let r = dispatch_dict_method("values", d, args);
        assert!(r.is_ptr());
    }

    #[test]
    fn test_dispatch_items() {
        let d = mb_dict_new();
        let args = make_args(vec![]);
        let r = dispatch_dict_method("items", d, args);
        assert!(r.is_ptr());
    }

    #[test]
    fn test_dispatch_pop() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(10));
        let args = make_args(vec![str_val("k"), MbValue::from_int(-1)]);
        let r = dispatch_dict_method("pop", d, args);
        assert_eq!(r.as_int(), Some(10));
    }

    #[test]
    fn test_dispatch_update() {
        let d = mb_dict_new();
        let other = mb_dict_new();
        mb_dict_setitem(other, str_val("x"), MbValue::from_int(1));
        let args = make_args(vec![other]);
        let r = dispatch_dict_method("update", d, args);
        assert!(r.is_none());
        assert_eq!(mb_dict_len(d).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_clear() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let args = make_args(vec![]);
        dispatch_dict_method("clear", d, args);
        assert_eq!(mb_dict_len(d).as_int(), Some(0));
    }

    #[test]
    fn test_dispatch_copy() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let args = make_args(vec![]);
        let cp = dispatch_dict_method("copy", d, args);
        assert_eq!(mb_dict_len(cp).as_int(), Some(1));
    }

    #[test]
    fn test_dispatch_setdefault() {
        let d = mb_dict_new();
        let args = make_args(vec![str_val("k"), MbValue::from_int(7)]);
        let r = dispatch_dict_method("setdefault", d, args);
        assert_eq!(r.as_int(), Some(7));
    }

    #[test]
    fn test_dispatch_unknown() {
        let d = mb_dict_new();
        let args = make_args(vec![]);
        let r = dispatch_dict_method("nonexistent", d, args);
        assert!(r.is_none());
    }

    // -- Py3.12 conformance --

    #[test]
    fn test_py312_dict_insertion_order() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("c"), MbValue::from_int(3));
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        mb_dict_setitem(d, str_val("b"), MbValue::from_int(2));
        let keys = mb_dict_keys(d);
        assert!(keys.is_ptr());
        assert_eq!(mb_dict_len(d).as_int(), Some(3));
    }

    #[test]
    fn test_py312_dict_popitem_reduces_len() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        mb_dict_setitem(d, str_val("b"), MbValue::from_int(2));
        let args = make_args(vec![]);
        let result = dispatch_dict_method("popitem", d, args);
        assert!(result.is_ptr());
        assert_eq!(mb_dict_len(d).as_int(), Some(1));
    }

    #[test]
    fn test_dict_popitem_empty_raises_keyerror() {
        super::super::exception::mb_clear_exception();
        let d = mb_dict_new();
        let result = mb_dict_popitem(d);
        assert!(
            result.is_none(),
            "popitem on empty dict must return the none sentinel"
        );
        assert_eq!(
            super::super::exception::mb_has_exception().as_bool(),
            Some(true),
            "KeyError must be pending after popitem on empty dict",
        );
        let exc = super::super::exception::mb_get_exception();
        let exc_type = super::super::exception::get_exception_type_pub(exc);
        assert_eq!(exc_type.as_deref(), Some("KeyError"));
        super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_py312_dict_contains_after_delete() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("x"), MbValue::from_int(1));
        assert_eq!(mb_dict_contains(d, str_val("x")).as_bool(), Some(true));
        mb_dict_pop(d, str_val("x"), MbValue::none());
        assert_eq!(mb_dict_contains(d, str_val("x")).as_bool(), Some(false));
    }

    #[test]
    fn test_py312_dict_setdefault_no_overwrite() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("k"), MbValue::from_int(99));
        let args = make_args(vec![str_val("k"), MbValue::from_int(0)]);
        let result = dispatch_dict_method("setdefault", d, args);
        assert_eq!(result.as_int(), Some(99));
    }

    #[test]
    fn test_py312_dict_update_overwrites() {
        let d = mb_dict_new();
        mb_dict_setitem(d, str_val("a"), MbValue::from_int(1));
        let other = mb_dict_new();
        mb_dict_setitem(other, str_val("a"), MbValue::from_int(99));
        let args = make_args(vec![other]);
        dispatch_dict_method("update", d, args);
        assert_eq!(mb_dict_getitem(d, str_val("a")).as_int(), Some(99));
    }

    #[test]
    fn test_py312_dict_pop_with_default() {
        let d = mb_dict_new();
        let args = make_args(vec![str_val("missing"), MbValue::from_int(-1)]);
        let r = dispatch_dict_method("pop", d, args);
        assert_eq!(r.as_int(), Some(-1));
    }
}
