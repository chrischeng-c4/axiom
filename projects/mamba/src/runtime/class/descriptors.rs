use super::*;

/// Check if a value is a descriptor (has __get__).
pub(crate) fn classmethod_descriptor_get(
    desc: MbValue,
    instance: MbValue,
    owner: MbValue,
) -> MbValue {
    let bind_owner = if !owner.is_none() {
        owner
    } else if !instance.is_none() {
        crate::runtime::builtins::mb_type(instance)
    } else {
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "classmethod.__get__(None, None) is invalid".to_string(),
            )),
        );
        return MbValue::none();
    };
    make_bound_method(mb_descriptor_unwrap(desc), bind_owner)
}

pub(crate) fn staticmethod_descriptor_get(desc: MbValue) -> MbValue {
    mb_descriptor_unwrap(desc)
}

/// Descriptor kind for classmethod/staticmethod/regular method dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DescriptorKind {
    Regular,
    ClassMethod,
    StaticMethod,
}

/// Unwrap a `__classmethod__` or `__staticmethod__` wrapper to get the underlying
/// function pointer (TAG_FUNC). Returns (func_mbvalue, descriptor_kind).
pub(crate) fn unwrap_descriptor_method(method: MbValue) -> (MbValue, DescriptorKind) {
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
    let addr = extract_registered_func_addr(unwrapped);
    if addr != 0 && CALLABLE_REGISTRY.with(|reg| reg.borrow().contains(&addr)) {
        addr
    } else {
        0
    }
}

/// Synthesize a `member_descriptor` instance for a `__slots__` entry —
/// CPython's `Slotted.x` class read yields the slot descriptor, not a value.
pub(crate) fn make_member_descriptor(class_name: &str, attr: &str) -> MbValue {
    let inst = crate::runtime::rc::MbObject::new_instance("member_descriptor".to_string());
    unsafe {
        if let ObjData::Instance { ref fields, .. } = (*inst).data {
            let mut g = fields.write().unwrap();
            g.insert(
                "__objclass__".to_string(),
                MbValue::from_ptr(crate::runtime::rc::MbObject::new_str(
                    class_name.to_string(),
                )),
            );
            g.insert(
                "__name__".to_string(),
                MbValue::from_ptr(crate::runtime::rc::MbObject::new_str(attr.to_string())),
            );
        }
    }
    MbValue::from_ptr(inst)
}

fn member_descriptor_slot_name(desc: MbValue) -> Option<String> {
    desc.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
            ..
        } = (*ptr).data
        {
            if class_name == "member_descriptor" {
                return fields
                    .read()
                    .unwrap()
                    .get("__name__")
                    .and_then(|name| extract_str(*name));
            }
        }
        None
    })
}

pub(crate) fn is_member_descriptor(val: MbValue) -> bool {
    val.as_ptr().is_some_and(|ptr| unsafe {
        matches!(&(*ptr).data, ObjData::Instance { class_name, .. } if class_name == "member_descriptor")
    })
}

pub(crate) fn member_descriptor_get(desc: MbValue, instance: MbValue) -> MbValue {
    if instance.is_none() {
        unsafe {
            crate::runtime::rc::retain_if_ptr(desc);
        }
        return desc;
    }
    let Some(slot_name) = member_descriptor_slot_name(desc) else {
        return MbValue::none();
    };
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
                if let Some(value) = fields.read().unwrap().get(&slot_name).copied() {
                    crate::runtime::rc::retain_if_ptr(value);
                    return value;
                }
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "'{}' object has no attribute '{slot_name}'",
                        class_display_name(class_name)
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "descriptor requires an instance".to_string(),
        )),
    );
    MbValue::none()
}

pub(crate) fn member_descriptor_set(desc: MbValue, instance: MbValue, value: MbValue) {
    let Some(slot_name) = member_descriptor_slot_name(desc) else {
        return;
    };
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                crate::runtime::rc::retain_if_ptr(value);
                let old = fields.write().unwrap().insert(slot_name, value);
                if let Some(prev) = old {
                    crate::runtime::rc::release_if_ptr(prev);
                }
                return;
            }
        }
    }
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "descriptor requires an instance".to_string(),
        )),
    );
}

pub(crate) fn member_descriptor_delete(desc: MbValue, instance: MbValue) {
    let Some(slot_name) = member_descriptor_slot_name(desc) else {
        return;
    };
    if let Some(ptr) = instance.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
                ..
            } = (*ptr).data
            {
                let old = fields.write().unwrap().remove(&slot_name);
                if let Some(prev) = old {
                    crate::runtime::rc::release_if_ptr(prev);
                    return;
                }
                crate::runtime::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "'{}' object has no attribute '{slot_name}'",
                        class_display_name(class_name)
                    ))),
                );
                return;
            }
        }
    }
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "descriptor requires an instance".to_string(),
        )),
    );
}

/// Create a @property descriptor.
/// Stores getter, setter, deleter as fields on a __property__ instance.
pub fn mb_property_new(getter: MbValue) -> MbValue {
    let prop = MbObject::new_instance("__property__".to_string());
    let ptr = MbValue::from_ptr(prop);
    let key = MbValue::from_ptr(MbObject::new_str("fget".to_string()));
    mb_setattr(ptr, key, getter);
    ptr
}

/// Read one accessor field (`fget`/`fset`/`fdel`) off a property instance,
/// returning `None` when the field is absent or stored as `None`.
fn property_accessor(prop: MbValue, field: &str) -> Option<MbValue> {
    let key = MbValue::from_ptr(MbObject::new_str(field.to_string()));
    let v = mb_getattr(prop, key);
    if v.is_none() {
        None
    } else {
        Some(v)
    }
}

/// Build a NEW `__property__` instance that copies `fget`/`fset`/`fdel` from
/// `src`, applying any overrides supplied. CPython's `property.setter` /
/// `.deleter` return a fresh property sharing the other accessors rather than
/// mutating the original in place. (#82)
fn property_clone_with(
    src: MbValue,
    fget: Option<MbValue>,
    fset: Option<MbValue>,
    fdel: Option<MbValue>,
) -> MbValue {
    let prop = mb_property_new(
        fget.or_else(|| property_accessor(src, "fget"))
            .unwrap_or_else(MbValue::none),
    );
    if let Some(fs) = fset.or_else(|| property_accessor(src, "fset")) {
        let key = MbValue::from_ptr(MbObject::new_str("fset".to_string()));
        mb_setattr(prop, key, fs);
    }
    if let Some(fd) = fdel.or_else(|| property_accessor(src, "fdel")) {
        let key = MbValue::from_ptr(MbObject::new_str("fdel".to_string()));
        mb_setattr(prop, key, fd);
    }
    prop
}

/// property.setter(fn) → returns a NEW property sharing fget/fdel, with fset set.
pub fn mb_property_setter(prop: MbValue, setter: MbValue) -> MbValue {
    property_clone_with(prop, None, Some(setter), None)
}

/// property.deleter(fn) → returns a NEW property sharing fget/fset, with fdel set.
pub fn mb_property_deleter(prop: MbValue, deleter: MbValue) -> MbValue {
    property_clone_with(prop, None, None, Some(deleter))
}

/// property.getter(fn) → returns a NEW property sharing fset/fdel, with fget set.
pub fn mb_property_getter(prop: MbValue, getter: MbValue) -> MbValue {
    property_clone_with(prop, Some(getter), None, None)
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
                        if let crate::runtime::dict_ops::DictKey::Str(s) = k {
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
    let mut prop = mb_property_new(fget.unwrap_or_else(MbValue::none));
    if let Some(fs) = fset {
        if !fs.is_none() {
            prop = mb_property_setter(prop, fs);
        }
    }
    if let Some(fd) = fdel {
        if !fd.is_none() {
            prop = mb_property_deleter(prop, fd);
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
        crate::runtime::exception::mb_raise(
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
        && !crate::runtime::closure::mb_closure_get_func(getter).is_none()
    {
        let result = mb_call1_val(getter, instance);
        unsafe {
            crate::runtime::rc::retain_if_ptr(result);
        }
        return result;
    }
    // Call the stored getter with instance. Try mb_call_method1 first
    // (CALLABLE_REGISTRY path for heap-pointer methods), then fall back to
    // direct TAG_FUNC invocation for JIT-compiled class methods that are
    // registered as func pointers.
    let val = mb_call_method1(getter, instance);
    if !val.is_none() {
        unsafe {
            crate::runtime::rc::retain_if_ptr(val);
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
                crate::runtime::rc::retain_if_ptr(result);
            }
            return result;
        }
    }
    let addr = extract_func_addr(getter);
    if addr > 4096 {
        let f: extern "C" fn(MbValue) -> MbValue = unsafe { std::mem::transmute(addr as usize) };
        let result = f(instance);
        unsafe {
            crate::runtime::rc::retain_if_ptr(result);
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
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
            MbValue::from_ptr(MbObject::new_str("can't set attribute".to_string())),
        );
        return;
    }
    if !crate::runtime::closure::mb_closure_get_func(setter).is_none() {
        let args = MbValue::from_ptr(MbObject::new_list_borrowed(vec![instance, value]));
        let _ = crate::runtime::builtins::mb_call_spread(setter, args);
        unsafe {
            crate::runtime::rc::release_if_ptr(args);
        }
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
            crate::runtime::rc::retain_if_ptr(val);
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
