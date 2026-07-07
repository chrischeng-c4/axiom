use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// Order-independent hash of a frozenset's elements, mirroring CPython's
/// `frozenset_hash` (Objects/setobject.c). Each element hash is scrambled
/// individually and folded in with XOR — which is commutative, so two
/// frozensets with equal elements hash equal regardless of insertion order.
/// The previous running-multiply accumulator was order-dependent
/// (`hash(frozenset([1,2,3])) != hash(frozenset([3,2,1]))`), which broke
/// frozenset-keyed dict lookups and hash-quality invariants. Computed in u64
/// then masked to Mamba's 48-bit signed int payload; XOR survives masking so
/// order-independence is preserved.
fn frozenset_hash(items: &[MbValue]) -> i64 {
    // CPython's per-element bit-shuffle: spreads low-entropy element hashes.
    fn shuffle_bits(h: u64) -> u64 {
        ((h ^ 89_869_747_u64) ^ (h << 16)).wrapping_mul(3_644_798_167_u64)
    }
    let mut hash: u64 = 0;
    for item in items {
        let eh = mb_hash(*item).as_int().unwrap_or(0) as u64;
        hash ^= shuffle_bits(eh);
    }
    // Fold in the cardinality and finalize (CPython's avalanche tail).
    hash ^= ((items.len() as u64).wrapping_add(1)).wrapping_mul(1_927_868_237_u64);
    hash ^= (hash >> 11) ^ (hash >> 25);
    hash = hash.wrapping_mul(69_069_u64).wrapping_add(907_133_923_u64);
    // Mask to the 48-bit signed payload Mamba ints carry.
    (hash & 0x0000_7FFF_FFFF_FFFF) as i64
}

/// hash(value) — return hash of a value.
/// Hash a float into mamba's 48-bit signed int hash domain. Integral floats
/// hash like the equivalent int (so `hash(7.0) == hash(7)`); ±inf / nan use
/// CPython's fixed sentinels; fractional values fold their bits. Shared by the
/// float and complex arms of mb_hash so `hash(complex(x, 0)) == hash(x)`.
fn float_hash_i64(f: f64) -> i64 {
    if f.is_finite() && f == f.floor() && f.abs() < (1i64 << 53) as f64 {
        let i = f as i64;
        if i == -1 {
            -2
        } else {
            i
        }
    } else if f.is_nan() {
        0
    } else if f.is_infinite() {
        if f > 0.0 {
            314159
        } else {
            -314159
        }
    } else {
        let folded = (f.to_bits() ^ (f.to_bits() >> 32)) & 0x0000_FFFF_FFFF_FFFF;
        let hash = ((folded as i64) << 16) >> 16;
        if hash == -1 {
            -2
        } else {
            hash
        }
    }
}

pub fn mb_hash(val: MbValue) -> MbValue {
    let val = super::int_enum_like_value(val).unwrap_or(val);
    // Python 3.12: slice is hashable, with `hash(slice(a,b,c)) ==
    // hash((a,b,c))`. Delegating to the tuple hash also reproduces CPython's
    // error for an unhashable component — `hash(slice(1,2,[]))` raises
    // `TypeError: unhashable type: 'list'` (not 'slice').
    if let Some(ptr) = val.as_ptr() {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = unsafe { &(*ptr).data }
        {
            if class_name == "slice" {
                let (start, stop, step) = {
                    let f = fields.read().unwrap();
                    (
                        f.get("start").copied().unwrap_or(MbValue::none()),
                        f.get("stop").copied().unwrap_or(MbValue::none()),
                        f.get("step").copied().unwrap_or(MbValue::none()),
                    )
                };
                let tup = MbValue::from_ptr(MbObject::new_tuple(vec![start, stop, step]));
                return super::super::tuple_ops::mb_tuple_hash(tup);
            }
        }
    }
    if super::is_decimal_handle_value(val) || super::is_fraction_handle_value(val) {
        // Hash must agree with `==` across numeric types: integral values
        // hash like the int, float-exact values hash like the float.
        if let Some(i) = super::super::stdlib::decimal_mod::mb_numeric_handle_integral_i64(val)
            .filter(|i| (-(1i64 << 47)..(1i64 << 47)).contains(i))
        {
            return MbValue::from_int(if i == -1 { -2 } else { i });
        }
        if let Some(f) = super::super::stdlib::decimal_mod::mb_numeric_handle_exact_f64(val) {
            return mb_hash(MbValue::from_float(f));
        }
        return super::super::string_ops::mb_str_hash(super::mb_str(val));
    }
    if let Some(hash) = super::super::iter::range_hash(val) {
        return hash;
    }
    if let Some(i) = val.as_int() {
        // CPython remaps hash(-1) to -2 because -1 is used internally
        // as an error sentinel in the C API.
        MbValue::from_int(if i == -1 { -2 } else { i })
    } else if let Some(f) = val.as_float() {
        // CPython: hash(float) == hash(int) when float is integral. Folds
        // fractional values; see float_hash_i64.
        MbValue::from_int(float_hash_i64(f))
    } else if let Some(b) = val.as_bool() {
        MbValue::from_int(b as i64)
    } else if val.is_none() {
        MbValue::from_int(0)
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => super::super::string_ops::mb_str_hash(val),
                ObjData::Tuple(_) => super::super::tuple_ops::mb_tuple_hash(val),
                ObjData::FrozenSet(items) => MbValue::from_int(frozenset_hash(items)),
                // CPython: hash(z) = float_hash(re) + HASH_IMAG * float_hash(im)
                // (HASH_IMAG = 1000003). A real-valued complex (im == 0) hashes
                // exactly like float_hash(re), so hash(complex(x, 0)) == hash(x).
                ObjData::Complex(re, im) => {
                    let h = float_hash_i64(*re)
                        .wrapping_add(1000003i64.wrapping_mul(float_hash_i64(*im)));
                    MbValue::from_int(if h == -1 { -2 } else { h })
                }
                ObjData::Instance { class_name, fields } => {
                    if matches!(class_name.as_str(), "ProxyType" | "CallableProxyType") {
                        super::raise_type_error(format!("unhashable type: 'weakref.{class_name}'"));
                        return MbValue::none();
                    }
                    // Only read-only memoryviews are hashable (matches
                    // CPython's Py_buffer readonly check); a writable view
                    // raises ValueError.
                    if class_name == "memoryview" {
                        let buf = fields.read().unwrap().get("_buffer").copied();
                        let readonly = fields
                            .read()
                            .unwrap()
                            .get("_readonly")
                            .and_then(|v| v.as_bool())
                            .unwrap_or_else(|| {
                                !buf.is_some_and(|b| {
                                    b.as_ptr().is_some_and(|bp| unsafe {
                                        matches!((*bp).data, ObjData::ByteArray(_))
                                    })
                                })
                            });
                        if !readonly {
                            super::super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(
                                    "cannot hash writable memoryview object".to_string(),
                                )),
                            );
                            return MbValue::none();
                        }
                    }
                    // namedtuple instances hash like the equivalent plain
                    // tuple: hash(Point(11, 22)) == hash((11, 22)).
                    if let Some(vals) =
                        super::super::stdlib::collections_mod::namedtuple_values(val)
                    {
                        let tup = MbValue::from_ptr(MbObject::new_tuple(vals));
                        return super::super::tuple_ops::mb_tuple_hash(tup);
                    }
                    // Mutable-mapping collections (Counter / defaultdict /
                    // OrderedDict — dict subclasses) are unhashable.
                    if class_name == "collections.Counter"
                        || class_name == "collections.defaultdict"
                        || class_name == "collections.OrderedDict"
                    {
                        let short = class_name.rsplit('.').next().unwrap_or(class_name);
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "unhashable type: '{short}'"
                            ))),
                        );
                        return MbValue::none();
                    }
                    // functools.cmp_to_key key objects set __hash__ = None and are
                    // therefore unhashable (CPython raises TypeError).
                    if class_name == "functools.cmp_to_key_obj" {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "unhashable type: 'functools.KeyWrapper'".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    // PEP 604 `X | Y` union: hash by member set, matching
                    // typing.Union[...] so the two representations hash alike
                    // (int | str and typing.Union[int, str] are equal).
                    if class_name == "UnionType" {
                        return super::super::stdlib::typing_mod::alias_hash_value(val);
                    }
                    // __hash__ dunder dispatch. Explicit `__hash__ = None`
                    // is not a miss: it makes instances unhashable.
                    if let Some(hash_method) =
                        super::super::class::lookup_method_including_none(class_name, "__hash__")
                    {
                        if hash_method.is_none() {
                            super::raise_type_error(format!(
                                "unhashable type: '{}'",
                                super::value_type_name(val)
                            ));
                            return MbValue::none();
                        }
                        let result = super::super::class::mb_call_method1(hash_method, val);
                        if super::super::exception::current_exception_type().is_some() {
                            return MbValue::none();
                        }
                        if let Some(i) = result.as_int() {
                            return MbValue::from_int(if i == -1 { -2 } else { i });
                        }
                        if let Some(b) = result.as_bool() {
                            // bool is an int subclass: True hashes to 1.
                            return MbValue::from_int(b as i64);
                        }
                        // CPython: a __hash__ that returns a non-int raises.
                        super::raise_type_error(
                            "__hash__ method should return an integer".to_string(),
                        );
                        return MbValue::none();
                    }
                    // PEP 557: dataclass synthesized __hash__ (frozen, or
                    // unsafe_hash=True) — hash of the compare=True field
                    // tuple, exactly matching `hash((f1, f2, ...))`.
                    if let Some(names) =
                        super::super::stdlib::dataclasses_mod::dc_hash_field_names(class_name)
                    {
                        if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                            let values: Vec<MbValue> = {
                                let guard = fields.read().unwrap();
                                names
                                    .iter()
                                    .map(|n| guard.get(n).copied().unwrap_or_else(MbValue::none))
                                    .collect()
                            };
                            // new_tuple_borrowed retains the elements; the
                            // release below frees the temp tuple and returns
                            // those refs.
                            let tup = MbValue::from_ptr(MbObject::new_tuple_borrowed(values));
                            let h = super::super::tuple_ops::mb_tuple_hash(tup);
                            super::super::rc::release_if_ptr(tup);
                            return h;
                        }
                    }
                    // PEP 557: a plain dataclass (eq=True, not frozen, not
                    // unsafe_hash) has __hash__ set to None — its instances are
                    // unhashable, so hash() raises rather than pointer-hashing.
                    if super::super::stdlib::dataclasses_mod::is_unhashable_dataclass(class_name) {
                        super::raise_type_error(format!(
                            "unhashable type: '{}'",
                            super::value_type_name(val)
                        ));
                        return MbValue::none();
                    }
                    MbValue::from_int((ptr as u64 >> 17) as i64)
                }
                // Mutable containers are unhashable in CPython — raise the
                // exact TypeError instead of silently pointer-hashing.
                ObjData::List(_) | ObjData::Dict(_) | ObjData::Set(_) | ObjData::ByteArray(_) => {
                    super::raise_type_error(format!(
                        "unhashable type: '{}'",
                        super::value_type_name(val)
                    ));
                    MbValue::none()
                }
                _ => MbValue::from_int((ptr as u64 >> 17) as i64),
            }
        }
    } else {
        MbValue::from_int(0)
    }
}
