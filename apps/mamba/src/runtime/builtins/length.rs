use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// len(value) — return the length of a collection.
/// Validate the result of a user `__len__` call per CPython: it must be a
/// non-negative integer. A non-integer return raises `TypeError`; a negative
/// one raises `ValueError`. If the `__len__` call itself already raised, the
/// pending exception is propagated untouched. `True`/`False` count as 1/0 and
/// arbitrary-precision ints pass through. Returns the validated value, or
/// `none()` after raising (the caller's epilogue check observes the pending
/// exception). This keeps `len(obj)` and `bool(obj)` reporting the *same*
/// error for an illegal `__len__`.
pub(crate) fn validate_len_result(result: MbValue) -> MbValue {
    // The __len__ call itself raised — propagate without re-judging the value.
    if super::super::exception::current_exception_type().is_some() {
        return result;
    }
    // bool is an int subclass; True == 1, False == 0 (both >= 0).
    if result.is_bool() {
        return result;
    }
    if let Some(n) = result.as_int() {
        if n < 0 {
            super::raise_value_error("__len__() should return >= 0".to_string());
            return MbValue::none();
        }
        return result;
    }
    // Arbitrary-precision ints are valid (non-negative) lengths.
    if let Some(ptr) = result.as_ptr() {
        unsafe {
            if let ObjData::BigInt(_) = (*ptr).data {
                return result;
            }
        }
    }
    // Any non-integer return is a TypeError, matching CPython's len().
    let tn = super::value_type_name(result);
    super::raise_type_error(format!("'{tn}' object cannot be interpreted as an integer"));
    MbValue::none()
}

fn len_type_error(val: MbValue) -> MbValue {
    super::raise_type_error(format!(
        "object of type '{}' has no len()",
        super::value_type_name(val),
    ));
    MbValue::none()
}

pub fn mb_len(val: MbValue) -> MbValue {
    if let Some(target) = super::super::stdlib::weakref_mod::proxy_target_or_raise(val) {
        if target.is_none() {
            return MbValue::none();
        }
        return mb_len(target);
    }
    // Iterator handles encode as tagged ints. For range iterators we can
    // compute the remaining element count in O(1) from (current, stop, step);
    // for other iterators we fall through and return 0 to match the prior
    // behavior (non-sequence iterators don't have len() in CPython either).
    if val.is_int() {
        if let Some(n) = super::super::iter::mb_iter_range_len(val) {
            // A range length can exceed 2^47 (e.g. `len(range(sys.maxsize))`),
            // which overflows a NaN-boxed int — promote to BigInt.
            return super::super::iter::mb_iter_range_len_value(val)
                .unwrap_or_else(|| super::super::bigint_ops::int_from_i64(n));
        }
        let id = val.as_int().unwrap_or(0) as u64;
        if super::super::stdlib::array_mod::is_array_handle(id) {
            return super::super::stdlib::array_mod::mb_array_len(val);
        }
        return len_type_error(val);
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                // Python 3 `len(str)` is the number of Unicode code points, not bytes.
                ObjData::Str(s) => {
                    if let Some(n) = super::super::string_ops::surrogate_len(val) {
                        return MbValue::from_int(n as i64);
                    }
                    // Class-body enum classes: len(Color) is the canonical
                    // member count, not the class-name string length.
                    if let Some(n) = super::super::stdlib::enum_class::class_member_count(s) {
                        return MbValue::from_int(n);
                    }
                    MbValue::from_int(s.chars().count() as i64)
                }
                ObjData::List(ref lock) => MbValue::from_int(lock.read().unwrap().len() as i64),
                ObjData::Dict(ref lock) => {
                    // ET.Element stub dicts: len(e) is the child count.
                    if let Some(children) =
                        super::super::stdlib::xml_mod::element_stub_children(val)
                    {
                        if let Some(cp) = children.as_ptr() {
                            if let ObjData::List(ref clock) = (*cp).data {
                                return MbValue::from_int(clock.read().unwrap().len() as i64);
                            }
                        }
                    }
                    MbValue::from_int(lock.read().unwrap().len() as i64)
                }
                ObjData::Tuple(items) => MbValue::from_int(items.len() as i64),
                ObjData::Set(ref lock) => MbValue::from_int(lock.read().unwrap().len() as i64),
                ObjData::FrozenSet(items) => MbValue::from_int(items.len() as i64),
                ObjData::Bytes(data) => MbValue::from_int(data.len() as i64),
                ObjData::ByteArray(ref lock) => {
                    MbValue::from_int(lock.read().unwrap().len() as i64)
                }
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    // Instance `__dict__` proxy (#1036): route to the same
                    // dict-aware `mb_dict_len` the write paths (#969) use —
                    // it never reaches `dict_view_len` below (that's for
                    // `dict_keys`/`dict_values`/`dict_items` *view* objects,
                    // not the raw proxy), so `len(obj.__dict__)` fell through
                    // to the generic `len_type_error` at the bottom of this
                    // arm ("object of type 'dict' has no len()").
                    if class_name == "__instance_dict_proxy__" {
                        return super::super::dict_ops::mb_dict_len(val);
                    }
                    if let Some(n) = super::super::dict_ops::dict_view_len(val) {
                        return MbValue::from_int(n);
                    }
                    // Class-body enum classes may surface as type objects;
                    // resolve them back to the class-name registry entry.
                    if let Some(n) =
                        super::super::stdlib::enum_class::class_member_count_for_value(val)
                    {
                        return MbValue::from_int(n);
                    }
                    // namedtuple instances: len reflects declared field count.
                    if let Some(vals) =
                        super::super::stdlib::collections_mod::namedtuple_values(val)
                    {
                        return MbValue::from_int(vals.len() as i64);
                    }
                    // Functional-API enum class objects: len() is the member count.
                    if let Some(items) =
                        super::super::stdlib::enum_mod::functional_enum_members(val)
                    {
                        return MbValue::from_int(items.len() as i64);
                    }
                    // memoryview: len() is the first shape dimension, not
                    // the raw byte length for multi-dimensional casts.
                    if class_name == "memoryview" {
                        let shape = fields.read().unwrap().get("_shape").copied();
                        if let Some(n) = shape.and_then(super::mb_first_index_value) {
                            return MbValue::from_int(n);
                        }
                        let buf = fields.read().unwrap().get("_buffer").copied();
                        if let Some(b) = buf {
                            if let Some(bp) = b.as_ptr() {
                                match (*bp).data {
                                    ObjData::Bytes(ref data) => {
                                        return MbValue::from_int(data.len() as i64)
                                    }
                                    ObjData::ByteArray(ref lock) => {
                                        return MbValue::from_int(lock.read().unwrap().len() as i64)
                                    }
                                    _ => {}
                                }
                            }
                        }
                        return MbValue::from_int(0);
                    }
                    // UserDict / UserList / UserString: len of the payload.
                    if let Some((_, data)) =
                        super::super::stdlib::collections_mod::user_wrapper_data(val)
                    {
                        return mb_len(data);
                    }
                    // collections.deque: len() is its backing `_items` list.
                    if class_name == "collections.deque" {
                        let items = fields.read().unwrap().get("_items").copied();
                        if let Some(d) = items {
                            if let Some(dp) = d.as_ptr() {
                                if let ObjData::List(ref lock) = (*dp).data {
                                    return MbValue::from_int(lock.read().unwrap().len() as i64);
                                }
                            }
                        }
                        return MbValue::from_int(0);
                    }
                    // dict-like collections (defaultdict, Counter, OrderedDict)
                    // and contextvars.Context: forward len() to the backing
                    // `_data` dict. Context's `_data` is the captured
                    // ContextVar -> value snapshot. Issue #282.
                    if class_name == "collections.defaultdict"
                        || class_name == "collections.Counter"
                        || class_name == "collections.OrderedDict"
                        || class_name == "Context"
                    {
                        let data = fields.read().unwrap().get("_data").copied();
                        if let Some(d) = data {
                            if let Some(dp) = d.as_ptr() {
                                if let ObjData::Dict(ref lock) = (*dp).data {
                                    return MbValue::from_int(lock.read().unwrap().len() as i64);
                                }
                            }
                        }
                        return MbValue::from_int(0);
                    }
                    // __len__ dunder dispatch — class-level first, then
                    // instance-level fallback for stdlib stub Instances
                    // (e.g. contextvars.Context) that wire dispatchers as
                    // instance fields without a registered class.
                    let len_method = super::super::class::lookup_method(class_name, "__len__");
                    if !len_method.is_none() {
                        let method_name =
                            MbValue::from_ptr(MbObject::new_str("__len__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                        let result = super::super::class::mb_call_method(val, method_name, args);
                        return validate_len_result(result);
                    }
                    if let Some(f) = fields.read().unwrap().get("__len__").copied() {
                        if let Some(addr) = f.as_func() {
                            if super::super::module::is_bound_dispatcher(addr as u64) {
                                let items = [val];
                                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                    std::mem::transmute(addr);
                                return f(items.as_ptr(), items.len());
                            }
                        }
                    }
                    if let Some((_base, payload)) =
                        super::super::class::builtin_data_payload_if_unoverridden(val, "__len__")
                    {
                        return mb_len(payload);
                    }
                    // Plain Mock / AsyncMock have no __len__ (only MagicMock
                    // registers the magic table): len() raises TypeError.
                    if matches!(
                        class_name.as_str(),
                        "Mock" | "AsyncMock" | "NonCallableMock"
                    ) {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "object of type '{class_name}' has no len()"
                            ))),
                        );
                        return MbValue::none();
                    }
                    // types.SimpleNamespace has no __len__ and is not a sized
                    // container: len() raises TypeError, matching CPython. (#654)
                    if class_name == "SimpleNamespace" {
                        super::super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "object of type 'types.SimpleNamespace' has no len()".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    len_type_error(val)
                }
                _ => len_type_error(val),
            }
        }
    } else {
        len_type_error(val)
    }
}
