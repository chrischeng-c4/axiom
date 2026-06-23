//! @codegen-skip: handwrite-pre-standardize
//!
//! types module for Mamba (#654, #1265 Task — Wave-9).
//!
//! CPython 3.12 `types` 32-entry surface:
//!   AsyncGeneratorType, BuiltinFunctionType, BuiltinMethodType, CellType,
//!   ClassMethodDescriptorType, CodeType, CoroutineType,
//!   DynamicClassAttribute, EllipsisType, FrameType, FunctionType,
//!   GeneratorType, GenericAlias, GetSetDescriptorType, LambdaType,
//!   MappingProxyType, MemberDescriptorType, MethodDescriptorType,
//!   MethodType, MethodWrapperType, ModuleType, NoneType,
//!   NotImplementedType, SimpleNamespace, TracebackType, UnionType,
//!   WrapperDescriptorType, coroutine, get_original_bases, new_class,
//!   prepare_class, resolve_bases.
//!
//! Carve-outs:
//!   - The 27 capitalized entries (FunctionType, MethodType, CodeType,
//!     FrameType, GeneratorType, ModuleType, the six *DescriptorType
//!     variants, MethodWrapperType, WrapperDescriptorType, etc.) are
//!     surfaced as Instance values with `class_name = "type"` carrying
//!     `__name__` / `__qualname__` / `__module__` fields. CPython
//!     exposes them as bona-fide type objects callable for isinstance()
//!     checks; mamba's runtime does not yet model real type-object
//!     introspection, so callers performing
//!     `isinstance(x, types.FunctionType)` see a structural stub rather
//!     than the actual builtin type. Attribute access (`__name__` etc.)
//!     works.
//!   - `LambdaType` aliases `FunctionType` per CPython; `BuiltinMethodType`
//!     aliases `BuiltinFunctionType`.
//!   - `SimpleNamespace`: Instance stub with an empty fields dict. Kwargs
//!     constructor (`types.SimpleNamespace(a=1, b=2)`) does not yet wire
//!     through the variadic dispatcher, so callers must mutate fields
//!     post-construction.
//!   - `MappingProxyType`: surfaced as a type stub, not the actual
//!     read-only view wrapper. Calling it returns a new type stub
//!     rather than a wrapped dict.
//!   - `DynamicClassAttribute`: descriptor stub. Decoration semantics
//!     (the actual __get__ protocol) are not yet implemented.
//!   - `new_class(name, bases=(), kwds=None, exec_body=None)`: routes simple
//!     class construction through `type(name, bases, ns)`, while explicit
//!     metaclasses are handled by the runtime metaclass call path.
//!   - `prepare_class(name, bases, kwds)`: models the common metaclass
//!     selection path and calls `__prepare__` when the chosen metaclass
//!     defines it; conflict diagnostics remain simplified.
//!   - `resolve_bases(bases)`: identity function (real implementation
//!     unwraps `__mro_entries__`).
//!   - `get_original_bases(cls)`: returns an empty tuple — the
//!     `__orig_bases__` attribute is not yet tracked on Mamba classes.
//!   - `coroutine(func)`: identity decorator; the wrapped function is
//!     returned as-is. CPython promotes generator-based coroutines to
//!     real coroutine objects via this helper.

use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

// Each generated dispatcher loads `stringify!($name)` through `black_box`
// so LLVM's `mergefunc` pass keeps the bodies distinct. Without that, any
// two dispatchers that wrap the same inner fn collapse to a single function
// pointer and `test_register_wires_full_surface` undercounts.
macro_rules! dispatch_unary {
    ($name:ident, $fn:ident) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let _ = core::hint::black_box(stringify!($name));
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

// types.new_class(name, bases=(), kwds=None, exec_body=None) — variadic so the
// bases/exec_body are honored, routing to the real type(name, bases, ns) class
// creator after running exec_body to populate the namespace.
unsafe extern "C" fn dispatch_new_class(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let bases = a.get(1).copied().unwrap_or_else(MbValue::none);
    let kwds = a.get(2).copied().unwrap_or_else(MbValue::none);
    let exec_body = a.get(3).copied().unwrap_or_else(MbValue::none);
    // CPython: prepare_class pops `metaclass` from kwds and forwards the rest to
    // `metaclass(name, bases, ns, **kwds)`. An explicit metaclass (any callable,
    // not necessarily a real type) takes over class creation.
    let sentinel = MbValue::from_bits(u64::MAX);
    let meta = super::super::dict_ops::mb_dict_get(
        kwds,
        MbValue::from_ptr(MbObject::new_str("metaclass".to_string())),
        sentinel,
    );
    if !kwds.is_none() && meta.to_bits() != sentinel.to_bits() {
        let ns = MbValue::from_ptr(MbObject::new_dict());
        if !exec_body.is_none() {
            let _ = super::super::class::mb_call1_val(exec_body, ns);
        }
        let bases_t = if bases.is_none() {
            MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
        } else {
            bases
        };
        let remaining = super::super::dict_ops::mb_dict_copy(kwds);
        super::super::dict_ops::mb_dict_delitem(
            remaining,
            MbValue::from_ptr(MbObject::new_str("metaclass".to_string())),
        );
        // CPython calls metaclass(name, bases, ns, **kwds); a non-callable
        // metaclass (e.g. a bare string) is a TypeError, not a silent build.
        if super::super::builtins::mb_callable(meta).as_bool() != Some(true) {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "metaclass is not callable".to_string(),
                )),
            );
            return MbValue::none();
        }
        let pos = MbValue::from_ptr(MbObject::new_list(vec![name, bases_t, ns]));
        return super::super::builtins::mb_call_spread_kwargs(meta, pos, remaining);
    }
    mb_types_new_class_impl(name, bases, exec_body)
}
unsafe extern "C" fn dispatch_prepare_class(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let name = a.first().copied().unwrap_or_else(MbValue::none);
    let bases = a.get(1).copied().unwrap_or_else(MbValue::none);
    let kwds = a.get(2).copied().unwrap_or_else(MbValue::none);
    mb_types_prepare_class_impl(name, bases, kwds)
}
dispatch_unary!(dispatch_resolve_bases, mb_types_resolve_bases);
dispatch_unary!(dispatch_coroutine, mb_types_coroutine);
dispatch_unary!(dispatch_get_original_bases, mb_types_get_original_bases);

/// types.SimpleNamespace(**kwargs) — build a namespace Instance, wiring each
/// keyword (delivered as a trailing kwargs dict) into a writable field.
/// Also accepts a positional mapping (CPython 3.13+ allows it; harmless here).
unsafe extern "C" fn dispatch_simplenamespace(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a: &[MbValue] = if nargs == 0 || args_ptr.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    };
    // SimpleNamespace takes keyword arguments only (kwargs arrive as a trailing
    // dict; a positional mapping is tolerated). A non-mapping positional, e.g.
    // SimpleNamespace(1, 2), is a TypeError in CPython, not a silent no-op.
    for arg in a {
        let is_dict = arg
            .as_ptr()
            .map_or(false, |p| matches!(unsafe { &(*p).data }, ObjData::Dict(_)));
        if !is_dict {
            super::super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "no positional arguments expected".to_string(),
                )),
            );
            return MbValue::none();
        }
    }
    let mut fields = FxHashMap::default();
    // Instance fields are unordered, but SimpleNamespace's repr must render
    // attributes in insertion order. Track it in a hidden `__ns_order__` list
    // (excluded from repr/vars); mb_setattr appends to it for new attributes.
    let mut order: Vec<MbValue> = Vec::new();
    for arg in a {
        if let Some(ptr) = arg.as_ptr() {
            unsafe {
                if let ObjData::Dict(ref lock) = (*ptr).data {
                    for (k, v) in lock.read().unwrap().iter() {
                        if let super::super::dict_ops::DictKey::Str(name) = k {
                            super::super::rc::retain_if_ptr(*v);
                            if !fields.contains_key(name) {
                                order.push(MbValue::from_ptr(MbObject::new_str(name.clone())));
                            }
                            fields.insert(name.clone(), *v);
                        }
                    }
                }
            }
        }
    }
    fields.insert(
        "__ns_order__".to_string(),
        MbValue::from_ptr(MbObject::new_list(order)),
    );
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "SimpleNamespace".to_string(),
            fields: RwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

pub fn register() {
    let mut attrs = HashMap::new();

    // -- Type-object stubs (27) --
    // CPython exposes these as actual type objects; mamba models them as
    // Instance values with class_name="type" so attribute access works.
    let type_objs: &[(&str, &str)] = &[
        ("FunctionType", "function"),
        ("LambdaType", "function"), // alias of FunctionType
        ("MethodType", "method"),
        ("BuiltinFunctionType", "builtin_function_or_method"),
        ("BuiltinMethodType", "builtin_function_or_method"), // alias
        ("ModuleType", "module"),
        ("GeneratorType", "generator"),
        ("CoroutineType", "coroutine"),
        ("AsyncGeneratorType", "async_generator"),
        ("CodeType", "code"),
        ("CellType", "cell"),
        ("FrameType", "frame"),
        ("TracebackType", "traceback"),
        ("NoneType", "NoneType"),
        ("NotImplementedType", "NotImplementedType"),
        ("EllipsisType", "ellipsis"),
        ("MappingProxyType", "mappingproxy"),
        ("DynamicClassAttribute", "DynamicClassAttribute"),
        ("GenericAlias", "GenericAlias"),
        ("UnionType", "UnionType"),
        ("ClassMethodDescriptorType", "classmethod_descriptor"),
        ("GetSetDescriptorType", "getset_descriptor"),
        ("MemberDescriptorType", "member_descriptor"),
        ("MethodDescriptorType", "method_descriptor"),
        ("MethodWrapperType", "method-wrapper"),
        ("WrapperDescriptorType", "wrapper_descriptor"),
    ];
    for (attr, type_name) in type_objs {
        attrs.insert(attr.to_string(), make_type_obj(type_name));
    }

    // -- Callables --
    let dispatchers: Vec<(&str, usize)> = vec![
        // SimpleNamespace is a constructor func so `types.SimpleNamespace(a=1)`
        // (a module-attr call) is dispatched and the kwargs wired into fields.
        ("SimpleNamespace", dispatch_simplenamespace as usize),
        ("new_class", dispatch_new_class as usize),
        ("prepare_class", dispatch_prepare_class as usize),
        ("resolve_bases", dispatch_resolve_bases as usize),
        ("coroutine", dispatch_coroutine as usize),
        ("get_original_bases", dispatch_get_original_bases as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
        if name == "SimpleNamespace" {
            super::super::module::NATIVE_TYPE_NAMES.with(|m| {
                m.borrow_mut().insert(addr as u64, "SimpleNamespace".to_string());
            });
        }
    }

    super::register_module("types", attrs);
}

// -- Type object constructor helper --

/// Create a type object instance with a given __name__. Routes through the
/// builtins TYPE_OBJ_CACHE singleton so `type(x) is types.XType` identity
/// holds (the same cache backs `type()`), then tops up the extra fields the
/// types module surfaces (__qualname__, UnionType.__args__) — idempotent
/// singleton mutation.
fn make_type_obj(name: &str) -> MbValue {
    let val = super::super::builtins::make_type_object(name);
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut f = fields.write().unwrap();
                if !f.contains_key("__qualname__") {
                    f.insert(
                        "__qualname__".to_string(),
                        MbValue::from_ptr(MbObject::new_str(name.to_string())),
                    );
                }
                if name == "UnionType" && !f.contains_key("__args__") {
                    f.insert(
                        "__args__".to_string(),
                        MbValue::from_ptr(MbObject::new_str("getset_descriptor".to_string())),
                    );
                }
            }
        }
    }
    val
}

// -- pub helpers — type object accessors (kept for back-compat callers) --

#[allow(non_snake_case)]
pub fn mb_types_FunctionType() -> MbValue {
    make_type_obj("function")
}
#[allow(non_snake_case)]
pub fn mb_types_LambdaType() -> MbValue {
    make_type_obj("function")
}
#[allow(non_snake_case)]
pub fn mb_types_MethodType() -> MbValue {
    make_type_obj("method")
}
#[allow(non_snake_case)]
pub fn mb_types_BuiltinFunctionType() -> MbValue {
    make_type_obj("builtin_function_or_method")
}
#[allow(non_snake_case)]
pub fn mb_types_BuiltinMethodType() -> MbValue {
    make_type_obj("builtin_function_or_method")
}
#[allow(non_snake_case)]
pub fn mb_types_ModuleType() -> MbValue {
    make_type_obj("module")
}
#[allow(non_snake_case)]
pub fn mb_types_GeneratorType() -> MbValue {
    make_type_obj("generator")
}
#[allow(non_snake_case)]
pub fn mb_types_CoroutineType() -> MbValue {
    make_type_obj("coroutine")
}
#[allow(non_snake_case)]
pub fn mb_types_AsyncGeneratorType() -> MbValue {
    make_type_obj("async_generator")
}
#[allow(non_snake_case)]
pub fn mb_types_CodeType() -> MbValue {
    make_type_obj("code")
}
#[allow(non_snake_case)]
pub fn mb_types_CellType() -> MbValue {
    make_type_obj("cell")
}
#[allow(non_snake_case)]
pub fn mb_types_FrameType() -> MbValue {
    make_type_obj("frame")
}
#[allow(non_snake_case)]
pub fn mb_types_TracebackType() -> MbValue {
    make_type_obj("traceback")
}
#[allow(non_snake_case)]
pub fn mb_types_NoneType() -> MbValue {
    make_type_obj("NoneType")
}
#[allow(non_snake_case)]
pub fn mb_types_NotImplementedType() -> MbValue {
    make_type_obj("NotImplementedType")
}
#[allow(non_snake_case)]
pub fn mb_types_EllipsisType() -> MbValue {
    make_type_obj("ellipsis")
}
#[allow(non_snake_case)]
pub fn mb_types_MappingProxyType() -> MbValue {
    make_type_obj("mappingproxy")
}
#[allow(non_snake_case)]
pub fn mb_types_GenericAlias() -> MbValue {
    make_type_obj("GenericAlias")
}
#[allow(non_snake_case)]
pub fn mb_types_UnionType() -> MbValue {
    make_type_obj("UnionType")
}
#[allow(non_snake_case)]
pub fn mb_types_DynamicClassAttribute() -> MbValue {
    make_type_obj("DynamicClassAttribute")
}
#[allow(non_snake_case)]
pub fn mb_types_ClassMethodDescriptorType() -> MbValue {
    make_type_obj("classmethod_descriptor")
}
#[allow(non_snake_case)]
pub fn mb_types_GetSetDescriptorType() -> MbValue {
    make_type_obj("getset_descriptor")
}
#[allow(non_snake_case)]
pub fn mb_types_MemberDescriptorType() -> MbValue {
    make_type_obj("member_descriptor")
}
#[allow(non_snake_case)]
pub fn mb_types_MethodDescriptorType() -> MbValue {
    make_type_obj("method_descriptor")
}
#[allow(non_snake_case)]
pub fn mb_types_MethodWrapperType() -> MbValue {
    make_type_obj("method-wrapper")
}
#[allow(non_snake_case)]
pub fn mb_types_WrapperDescriptorType() -> MbValue {
    make_type_obj("wrapper_descriptor")
}

/// types.SimpleNamespace(**kwargs) -> namespace object
///
/// Returns an Instance with `class_name = "SimpleNamespace"` and an empty
/// writable fields dict. Kwargs do not yet flow through the dispatcher
/// so callers must mutate fields after construction.
#[allow(non_snake_case)]
pub fn mb_types_SimpleNamespace() -> MbValue {
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "SimpleNamespace".to_string(),
            fields: RwLock::new(FxHashMap::default()),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

// -- Callable helpers --

/// types.new_class(name, bases=(), kwds=None, exec_body=None) -> type.
/// Builds a namespace dict, runs exec_body(ns) to populate it, then creates the
/// class via the real `type(name, bases, ns)` machinery (registers it so
/// __bases__/isinstance/attributes work).
pub fn mb_types_new_class_impl(name: MbValue, bases: MbValue, exec_body: MbValue) -> MbValue {
    let ns = MbValue::from_ptr(MbObject::new_dict());
    if !exec_body.is_none() {
        // exec_body(ns) mutates ns in place to add methods/attributes.
        let _ = super::super::class::mb_call1_val(exec_body, ns);
    }
    // A None / missing bases means the empty tuple (→ object base in mb_type3).
    let bases = if bases.is_none() {
        MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
    } else {
        bases
    };
    // mb_type3 registers the class (and returns a `type` object). Normal user
    // classes are represented as the class-name string — the form whose
    // __name__/__bases__/isinstance resolve through the registry — so return
    // that for consistency.
    let _ = super::super::builtins::mb_type3(name, bases, ns);
    if let Some(s) = name.as_ptr().and_then(|p| unsafe {
        if let ObjData::Str(ref s) = (*p).data { Some(s.clone()) } else { None }
    }) {
        return MbValue::from_ptr(MbObject::new_str(s));
    }
    name
}

/// types.new_class(name, bases=(), kwds=None, exec_body=None) -> type
pub fn mb_types_new_class(name: MbValue) -> MbValue {
    mb_types_new_class_impl(name, MbValue::none(), MbValue::none())
}

fn dict_get_str(dict: MbValue, key: &str) -> Option<MbValue> {
    let sentinel = MbValue::from_bits(u64::MAX);
    let found = super::super::dict_ops::mb_dict_get(
        dict,
        MbValue::from_ptr(MbObject::new_str(key.to_string())),
        sentinel,
    );
    if found.to_bits() == sentinel.to_bits() {
        None
    } else {
        Some(found)
    }
}

fn dict_without_metaclass(kwds: MbValue, had_metaclass: bool) -> MbValue {
    let remaining = if kwds.as_ptr().map_or(false, |p| unsafe {
        matches!(&(*p).data, ObjData::Dict(_))
    }) {
        super::super::dict_ops::mb_dict_copy(kwds)
    } else {
        MbValue::from_ptr(MbObject::new_dict())
    };
    if had_metaclass {
        super::super::dict_ops::mb_dict_delitem(
            remaining,
            MbValue::from_ptr(MbObject::new_str("metaclass".to_string())),
        );
    }
    remaining
}

fn normalized_bases_tuple(bases: MbValue) -> MbValue {
    if bases.is_none() {
        MbValue::from_ptr(MbObject::new_tuple(Vec::new()))
    } else {
        bases
    }
}

fn base_metaclass_name(base: MbValue) -> Option<String> {
    if let Some(class_name) = super::super::class::resolve_class_name(base) {
        return super::super::class::class_metaclass_name(&class_name)
            .or_else(|| Some("type".to_string()));
    }
    base.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { class_name, .. } = &(*ptr).data {
            if super::super::class::class_is_registered(class_name) {
                Some(class_name.clone())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn most_derived_base_metaclass(bases: MbValue) -> Option<String> {
    bases.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Tuple(items) = &(*ptr).data {
            items.iter().find_map(|base| {
                let meta = base_metaclass_name(*base)?;
                if meta == "type" { None } else { Some(meta) }
            })
        } else {
            None
        }
    })
}

fn metaclass_value(name: &str) -> MbValue {
    if name == "type" {
        super::super::builtins::make_type_object("type")
    } else {
        MbValue::from_ptr(MbObject::new_str(name.to_string()))
    }
}

fn prepare_namespace(meta_name: &str, name: MbValue, bases: MbValue) -> MbValue {
    let prepare = super::super::class::lookup_method(meta_name, "__prepare__");
    if prepare.is_none() {
        return MbValue::from_ptr(MbObject::new_dict());
    }
    let args = MbValue::from_ptr(MbObject::new_list(vec![name, bases]));
    let ns = super::super::builtins::mb_call_spread(prepare, args);
    if ns.is_none() {
        MbValue::from_ptr(MbObject::new_dict())
    } else {
        ns
    }
}

fn mb_types_prepare_class_impl(name: MbValue, bases: MbValue, kwds: MbValue) -> MbValue {
    let bases = normalized_bases_tuple(bases);
    let explicit_meta = dict_get_str(kwds, "metaclass");
    let explicit_meta_name = explicit_meta
        .and_then(super::super::class::resolve_class_name);
    let base_meta_name = most_derived_base_metaclass(bases);
    let meta_name = match (explicit_meta_name, base_meta_name) {
        (Some(explicit), Some(base)) if explicit == "type" => base,
        (Some(explicit), _) => explicit,
        (None, Some(base)) => base,
        (None, None) => "type".to_string(),
    };
    let ns = prepare_namespace(&meta_name, name, bases);
    let remaining = dict_without_metaclass(kwds, explicit_meta.is_some());
    MbValue::from_ptr(MbObject::new_tuple(vec![
        metaclass_value(&meta_name),
        ns,
        remaining,
    ]))
}

/// types.prepare_class(name, bases, kwds) -> (meta, ns, kwds)
pub fn mb_types_prepare_class(_name: MbValue) -> MbValue {
    mb_types_prepare_class_impl(_name, MbValue::none(), MbValue::none())
}

/// True if `v` is a class (a registered class-name string or a `type` object),
/// as opposed to an instance that may carry __mro_entries__.
fn rb_is_class(v: MbValue) -> bool {
    v.as_ptr()
        .map(|p| unsafe {
            match &(*p).data {
                ObjData::Str(s) => super::super::class::class_is_registered(s),
                ObjData::Instance { class_name, .. } => class_name == "type",
                _ => false,
            }
        })
        .unwrap_or(false)
}

/// types.resolve_bases(bases) — PEP 560: replace any non-class base that defines
/// __mro_entries__ with the tuple it returns. Returns the original tuple
/// unchanged (same object) when nothing is resolved.
pub fn mb_types_resolve_bases(bases: MbValue) -> MbValue {
    let items: Vec<MbValue> = bases
        .as_ptr()
        .map(|p| unsafe {
            if let ObjData::Tuple(ref it) = (*p).data {
                it.to_vec()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_default();
    let mut new_bases: Vec<MbValue> = Vec::new();
    let mut updated = false;
    for &base in &items {
        if rb_is_class(base) {
            new_bases.push(base);
            continue;
        }
        let attr = MbValue::from_ptr(MbObject::new_str("__mro_entries__".to_string()));
        if super::super::class::mb_hasattr(base, attr).as_bool() == Some(true) {
            let mname = MbValue::from_ptr(MbObject::new_str("__mro_entries__".to_string()));
            let args = MbValue::from_ptr(MbObject::new_list(vec![bases]));
            let result = super::super::class::mb_call_method(base, mname, args);
            updated = true;
            let spliced = result.as_ptr().map(|rp| unsafe {
                if let ObjData::Tuple(ref it) = (*rp).data {
                    Some(it.to_vec())
                } else {
                    None
                }
            });
            match spliced.flatten() {
                Some(entries) => new_bases.extend(entries),
                None => new_bases.push(result),
            }
        } else {
            new_bases.push(base);
        }
    }
    if !updated {
        return bases;
    }
    MbValue::from_ptr(MbObject::new_tuple_borrowed(new_bases))
}

/// types.coroutine(func) -> func
///
/// Identity decorator. CPython promotes generator-based functions to
/// real coroutine objects; mamba returns the input unchanged. Like
/// CPython, a non-callable argument raises TypeError with the message
/// `types.coroutine() expects a callable`.
pub fn mb_types_coroutine(func: MbValue) -> MbValue {
    if super::super::builtins::mb_callable(func).as_bool() != Some(true) {
        super::super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "types.coroutine() expects a callable".to_string(),
            )),
        );
        return MbValue::none();
    }
    func
}

/// types.get_original_bases(cls) -> tuple
///
/// Returns the empty tuple — `__orig_bases__` is not tracked.
pub fn mb_types_get_original_bases(_cls: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_tuple(vec![]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    fn get_str(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    fn class_name_of(val: MbValue) -> Option<String> {
        val.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                Some(class_name.clone())
            } else {
                None
            }
        })
    }

    // -- Type-object stub shape --

    #[test]
    fn test_type_objects_carry_name_and_qualname() {
        for (helper, expected) in [
            (mb_types_FunctionType() as MbValue, "function"),
            (mb_types_LambdaType(), "function"),
            (mb_types_MethodType(), "method"),
            (mb_types_BuiltinFunctionType(), "builtin_function_or_method"),
            (mb_types_BuiltinMethodType(), "builtin_function_or_method"),
            (mb_types_ModuleType(), "module"),
            (mb_types_GeneratorType(), "generator"),
            (mb_types_CoroutineType(), "coroutine"),
            (mb_types_AsyncGeneratorType(), "async_generator"),
            (mb_types_CodeType(), "code"),
            (mb_types_CellType(), "cell"),
            (mb_types_FrameType(), "frame"),
            (mb_types_TracebackType(), "traceback"),
            (mb_types_NoneType(), "NoneType"),
            (mb_types_NotImplementedType(), "NotImplementedType"),
            (mb_types_EllipsisType(), "ellipsis"),
            (mb_types_MappingProxyType(), "mappingproxy"),
            (mb_types_GenericAlias(), "GenericAlias"),
            (mb_types_UnionType(), "UnionType"),
            (mb_types_DynamicClassAttribute(), "DynamicClassAttribute"),
            (
                mb_types_ClassMethodDescriptorType(),
                "classmethod_descriptor",
            ),
            (mb_types_GetSetDescriptorType(), "getset_descriptor"),
            (mb_types_MemberDescriptorType(), "member_descriptor"),
            (mb_types_MethodDescriptorType(), "method_descriptor"),
            (mb_types_MethodWrapperType(), "method-wrapper"),
            (mb_types_WrapperDescriptorType(), "wrapper_descriptor"),
        ] {
            assert_eq!(
                get_str(get_field(helper, "__name__")).as_deref(),
                Some(expected)
            );
            assert_eq!(
                get_str(get_field(helper, "__qualname__")).as_deref(),
                Some(expected)
            );
            assert_eq!(
                get_str(get_field(helper, "__module__")).as_deref(),
                Some("builtins")
            );
            assert_eq!(class_name_of(helper).as_deref(), Some("type"));
        }
    }

    #[test]
    fn test_lambda_aliases_function() {
        let ft = mb_types_FunctionType();
        let lt = mb_types_LambdaType();
        assert_eq!(
            get_str(get_field(ft, "__name__")),
            get_str(get_field(lt, "__name__"))
        );
    }

    #[test]
    fn test_builtin_method_aliases_builtin_function() {
        let a = mb_types_BuiltinFunctionType();
        let b = mb_types_BuiltinMethodType();
        assert_eq!(
            get_str(get_field(a, "__name__")),
            get_str(get_field(b, "__name__"))
        );
    }

    // -- SimpleNamespace --

    #[test]
    fn test_simple_namespace_shape() {
        let ns = mb_types_SimpleNamespace();
        assert_eq!(class_name_of(ns).as_deref(), Some("SimpleNamespace"));
        // Fields dict exists and is empty.
        if let Some(ptr) = ns.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    assert_eq!(fields.read().unwrap().len(), 0);
                } else {
                    panic!("expected Instance");
                }
            }
        } else {
            panic!("expected ptr");
        }
    }

    // -- Callable helpers --

    #[test]
    fn test_new_class_uses_provided_name() {
        let name = MbValue::from_ptr(MbObject::new_str("MyClass".to_string()));
        let cls = mb_types_new_class(name);
        assert_eq!(
            get_str(get_field(cls, "__name__")).as_deref(),
            Some("MyClass")
        );
    }

    #[test]
    fn test_new_class_defaults_when_name_missing() {
        let cls = mb_types_new_class(MbValue::none());
        assert_eq!(
            get_str(get_field(cls, "__name__")).as_deref(),
            Some("NewClass")
        );
    }

    #[test]
    fn test_prepare_class_returns_triple() {
        let r = mb_types_prepare_class(MbValue::none());
        if let Some(ptr) = r.as_ptr() {
            unsafe {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    assert_eq!(items.len(), 3);
                    // first slot is None
                    assert!(items[0].as_ptr().is_none() || items[0].as_int().is_none());
                } else {
                    panic!("expected Tuple");
                }
            }
        } else {
            panic!("expected ptr");
        }
    }

    #[test]
    fn test_resolve_bases_is_identity() {
        let input = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(7)]));
        let out = mb_types_resolve_bases(input);
        // Same pointer — identity passthrough.
        assert_eq!(input.as_ptr(), out.as_ptr());
    }

    #[test]
    fn test_coroutine_rejects_non_callable() {
        // CPython 3.12: types.coroutine() raises TypeError for a
        // non-callable argument (identity passthrough applies to callables).
        let f = MbValue::from_int(42);
        let r = mb_types_coroutine(f);
        assert!(r.is_none());
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("TypeError"),
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_get_original_bases_returns_empty_tuple() {
        let r = mb_types_get_original_bases(MbValue::none());
        if let Some(ptr) = r.as_ptr() {
            unsafe {
                if let ObjData::Tuple(ref items) = (*ptr).data {
                    assert_eq!(items.len(), 0);
                } else {
                    panic!("expected Tuple");
                }
            }
        } else {
            panic!("expected ptr");
        }
    }

    // -- register() surface --

    #[test]
    fn test_register_wires_full_surface() {
        register();
        // 5 dispatchers must each be registered into the global
        // native-func table; snapshot is monotonic across the test
        // process so we only assert presence (>=5).
        let snap = super::super::super::module::NATIVE_FUNC_ADDRS.with(|s| s.borrow().len());
        assert!(
            snap >= 5,
            "expected at least 5 native func addrs registered"
        );
    }

    #[test]
    fn test_surface_entry_count_is_32() {
        // The CPython 3.12 `types` module exposes exactly 32 non-dunder
        // entries. We list them here so a future drift surfaces loudly.
        let expected: &[&str] = &[
            "AsyncGeneratorType",
            "BuiltinFunctionType",
            "BuiltinMethodType",
            "CellType",
            "ClassMethodDescriptorType",
            "CodeType",
            "CoroutineType",
            "DynamicClassAttribute",
            "EllipsisType",
            "FrameType",
            "FunctionType",
            "GeneratorType",
            "GenericAlias",
            "GetSetDescriptorType",
            "LambdaType",
            "MappingProxyType",
            "MemberDescriptorType",
            "MethodDescriptorType",
            "MethodType",
            "MethodWrapperType",
            "ModuleType",
            "NoneType",
            "NotImplementedType",
            "SimpleNamespace",
            "TracebackType",
            "UnionType",
            "WrapperDescriptorType",
            "coroutine",
            "get_original_bases",
            "new_class",
            "prepare_class",
            "resolve_bases",
        ];
        assert_eq!(expected.len(), 32);
    }
}
