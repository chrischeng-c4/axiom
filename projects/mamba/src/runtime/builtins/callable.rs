use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;

/// callable(obj) — return True if the object appears callable.
pub fn mb_callable(obj: MbValue) -> MbValue {
    // TAG_FUNC values + closure handles (TAG_INT carrying a closure id):
    // resolve_callable recognises both kinds of compiled-function references,
    // so user-defined `def`s and `lambda`s round-trip correctly.
    if super::resolve_callable(obj).is_some() {
        return MbValue::from_bool(true);
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance {
                    class_name,
                    ref fields,
                } => {
                    if super::super::stdlib::enum_mod::is_functional_enum_class(obj) {
                        return MbValue::from_bool(true);
                    }
                    if matches!(class_name.as_str(), "typing.Alias" | "Alias") {
                        let (kind, origin) = {
                            let guard = fields.read().unwrap();
                            (
                                guard.get("_kind").copied().and_then(|value| {
                                    value.as_ptr().and_then(|ptr| unsafe {
                                        match &(*ptr).data {
                                            ObjData::Str(s) => Some(s.clone()),
                                            _ => None,
                                        }
                                    })
                                }),
                                guard.get("__origin__").copied(),
                            )
                        };
                        let is_callable = kind.as_deref() == Some("generic")
                            && origin.is_some_and(|origin| {
                                origin.to_bits() != obj.to_bits()
                                    && super::mb_callable(origin).as_bool() == Some(true)
                            });
                        return MbValue::from_bool(is_callable);
                    }
                    if class_name == "__exec_function__" {
                        return MbValue::from_bool(true);
                    }
                    if class_name == "__unbound_method__" || class_name == "__bound_native_method__"
                    {
                        return MbValue::from_bool(true);
                    }
                    // A `functools.partial` (and partial-shaped bound methods,
                    // e.g. the bound `Struct.pack` / `Struct.unpack` methods)
                    // is callable: `mb_call_spread` knows how to prepend the
                    // bound args and dispatch the wrapped func.
                    if class_name == "functools.partial" {
                        return MbValue::from_bool(true);
                    }
                    if class_name == "functools._singledispatchmethod_bound" {
                        return MbValue::from_bool(true);
                    }
                    if class_name == "collections.abc._register_bound"
                        || class_name == "abc._user_register_bound"
                    {
                        return MbValue::from_bool(true);
                    }
                    // A type object (the value bound to a class name like `C`
                    // or returned by `type(name, bases, dict)`) is itself
                    // callable — calling it constructs an instance.
                    if class_name == "type" {
                        return MbValue::from_bool(true);
                    }
                    // For ordinary user instances, callability is determined
                    // by the presence of a `__call__` dunder.
                    let method = super::super::class::mb_lookup_dunder(
                        obj,
                        MbValue::from_ptr(MbObject::new_str("__call__".to_string())),
                    );
                    return MbValue::from_bool(!method.is_none());
                }
                ObjData::Str(s) => {
                    // Builtin type-name strings (`int`, `str`, `list`, ...) are
                    // resolved as string identifiers at compile time but behave
                    // as callable type constructors at runtime.
                    if matches!(
                        s.as_str(),
                        "int"
                            | "str"
                            | "float"
                            | "bool"
                            | "list"
                            | "dict"
                            | "set"
                            | "frozenset"
                            | "tuple"
                            | "bytes"
                            | "bytearray"
                            | "complex"
                            | "type"
                            | "object"
                            | "range"
                            | "enumerate"
                            | "zip"
                            | "map"
                            | "filter"
                            | "iter"
                            | "reversed"
                            | "abs"
                            | "len"
                            | "repr"
                            | "chr"
                            | "ord"
                            | "print"
                            | "sorted"
                            | "sum"
                            | "min"
                            | "max"
                            | "any"
                            | "all"
                    ) {
                        return MbValue::from_bool(true);
                    }
                    // User-defined class names also flow through as bare
                    // strings — calling `C(...)` invokes the registered ctor.
                    if super::super::class::class_is_registered(s) {
                        return MbValue::from_bool(true);
                    }
                }
                _ => {}
            }
        }
    }
    // Primitives (int, float, bool, None, etc.) are not callable
    MbValue::from_bool(false)
}
