/// builtins module for Mamba (#997).
///
/// Exposes all built-in functions and constants as an importable module.
/// In CPython, `import builtins` gives access to the built-in namespace.
/// Each function is wrapped in an `unsafe extern "C" fn dispatch_*` wrapper
/// using the `(args_ptr, nargs)` ABI and registered via `NATIVE_FUNC_ADDRS`.

use std::collections::HashMap;
use super::super::value::MbValue;
use super::super::rc::{MbObject, ObjData};

// ── Dispatch wrappers ──
//
// Each wrapper follows the native (args_ptr, nargs) ABI so that
// `mb_call_spread` can invoke them directly.

/// Safe arg extraction: handles null pointer when nargs == 0.
#[inline(always)]
unsafe fn safe_args<'a>(args_ptr: *const MbValue, nargs: usize) -> &'a [MbValue] {
    if nargs == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(args_ptr, nargs) }
    }
}

fn raise_type_error(msg: &str) -> MbValue {
    super::super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

fn is_builtin_type_surface(name: &str) -> bool {
    matches!(
        name,
        "bool"
            | "int"
            | "float"
            | "str"
            | "list"
            | "dict"
            | "set"
            | "tuple"
            | "frozenset"
            | "complex"
            | "bytes"
            | "bytearray"
            | "object"
            | "type"
            | "range"
            | "enumerate"
            | "zip"
            | "map"
            | "filter"
            | "reversed"
            | "memoryview"
            | "slice"
            | "property"
            | "classmethod"
            | "staticmethod"
    )
}

fn is_str_value(val: MbValue) -> bool {
    match val.as_ptr() {
        Some(ptr) => unsafe { matches!(&(*ptr).data, ObjData::Str(_)) },
        None => false,
    }
}

// @spec .aw/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md#R1

unsafe extern "C" fn dispatch_print(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let list = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
    super::super::builtins::mb_print_args(list)
}

unsafe extern "C" fn dispatch_len(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_len(args.first().copied().unwrap_or_else(MbValue::none))
}

// @spec .aw/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md#R1
unsafe extern "C" fn dispatch_type(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs == 3 {
        super::super::builtins::mb_type3(args[0], args[1], args[2])
    } else if nargs == 1 {
        super::super::builtins::mb_type(args[0])
    } else {
        raise_type_error("type() takes 1 or 3 arguments")
    }
}

unsafe extern "C" fn dispatch_range(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    match nargs {
        0 => raise_type_error("range expected at least 1 argument, got 0"),
        1 => super::super::builtins::mb_range(args[0]),
        2 => super::super::builtins::mb_range_2(args[0], args[1]),
        _ => super::super::builtins::mb_range_3(args[0], args[1], args[2]),
    }
}

unsafe extern "C" fn dispatch_int(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::from_int(0);
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs >= 2 {
        super::super::builtins::mb_int_base(args[0], args[1])
    } else {
        super::super::builtins::mb_int(args[0])
    }
}

unsafe extern "C" fn dispatch_float(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::from_float(0.0);
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_float(args[0])
}

unsafe extern "C" fn dispatch_str(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::from_ptr(MbObject::new_str(String::new()));
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_str(args[0])
}

unsafe extern "C" fn dispatch_bool(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return MbValue::from_bool(false);
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_bool(args[0])
}

unsafe extern "C" fn dispatch_abs(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_abs(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_min(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs == 1 {
        super::super::builtins::mb_min(args[0])
    } else {
        let list = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
        super::super::builtins::mb_min(list)
    }
}

unsafe extern "C" fn dispatch_max(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs == 1 {
        super::super::builtins::mb_max(args[0])
    } else {
        let list = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
        super::super::builtins::mb_max(list)
    }
}

unsafe extern "C" fn dispatch_sum(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs >= 2 {
        super::super::builtins::mb_sum_with_start(args[0], args[1])
    } else {
        super::super::builtins::mb_sum(args.first().copied().unwrap_or_else(MbValue::none))
    }
}

unsafe extern "C" fn dispatch_sorted(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let iterable = args.first().copied().unwrap_or_else(MbValue::none);
    if nargs >= 3 {
        super::super::builtins::mb_sorted_kwargs(iterable, args[1], args[2])
    } else if nargs >= 2 {
        super::super::builtins::mb_sorted(iterable, args[1])
    } else {
        super::super::builtins::mb_sorted(iterable, MbValue::from_bool(false))
    }
}

unsafe extern "C" fn dispatch_reversed(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::iter::mb_reversed(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_enumerate(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let iterable = args.first().copied().unwrap_or_else(MbValue::none);
    let start = if nargs >= 2 { args[1] } else { MbValue::from_int(0) };
    super::super::iter::mb_enumerate(iterable, start)
}

unsafe extern "C" fn dispatch_zip(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs == 2 {
        super::super::iter::mb_zip(args[0], args[1])
    } else {
        let list = MbValue::from_ptr(MbObject::new_list(args.to_vec()));
        super::super::iter::mb_zip_n(list)
    }
}

unsafe extern "C" fn dispatch_map(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    let iterable = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::builtins::mb_map(func, iterable)
}

unsafe extern "C" fn dispatch_filter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let func = args.first().copied().unwrap_or_else(MbValue::none);
    let iterable = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::builtins::mb_filter(func, iterable)
}

unsafe extern "C" fn dispatch_all(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_all(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_any(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_any(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_input(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_input(
        args.first().copied().unwrap_or_else(|| MbValue::from_ptr(MbObject::new_str(String::new()))),
    )
}

unsafe extern "C" fn dispatch_open(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let path = args.first().copied().unwrap_or_else(MbValue::none);
    let mode = args.get(1).copied().unwrap_or_else(|| {
        MbValue::from_ptr(MbObject::new_str("r".to_string()))
    });
    super::super::file_io::mb_open(path, mode)
}

unsafe extern "C" fn dispatch_chr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_chr(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_ord(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_ord(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_hex(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_hex(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_oct(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_oct(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_bin(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_bin(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_round(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let val = args.first().copied().unwrap_or_else(MbValue::none);
    let ndigits = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::builtins::mb_round(val, ndigits)
}

unsafe extern "C" fn dispatch_pow(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let base = args.first().copied().unwrap_or_else(MbValue::none);
    let exp = args.get(1).copied().unwrap_or_else(MbValue::none);
    if nargs >= 3 {
        super::super::builtins::mb_pow_mod(base, exp, args[2])
    } else {
        super::super::builtins::mb_pow(base, exp)
    }
}

unsafe extern "C" fn dispatch_divmod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let a = args.first().copied().unwrap_or_else(MbValue::none);
    let b = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::builtins::mb_divmod(a, b)
}

unsafe extern "C" fn dispatch_repr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_repr(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_hash(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_hash(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_id(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_id(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_isinstance(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let cls = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_isinstance(obj, cls)
}

unsafe extern "C" fn dispatch_issubclass(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let child = args.first().copied().unwrap_or_else(MbValue::none);
    let parent = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_issubclass(child, parent)
}

unsafe extern "C" fn dispatch_hasattr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let attr = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_hasattr(obj, attr)
}

unsafe extern "C" fn dispatch_getattr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let attr = args.get(1).copied().unwrap_or_else(MbValue::none);
    if nargs >= 3 {
        super::super::class::mb_getattr_default(obj, attr, args[2])
    } else {
        super::super::class::mb_getattr(obj, attr)
    }
}

unsafe extern "C" fn dispatch_setattr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let attr = args.get(1).copied().unwrap_or_else(MbValue::none);
    let value = args.get(2).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_setattr(obj, attr, value);
    MbValue::none()
}

unsafe extern "C" fn dispatch_delattr(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let attr = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_delattr(obj, attr);
    MbValue::none()
}

unsafe extern "C" fn dispatch_callable(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_callable(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_format(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let val = args.first().copied().unwrap_or_else(MbValue::none);
    let spec = match args.get(1).copied() {
        Some(spec) if is_str_value(spec) => spec,
        Some(_) => return raise_type_error("format() argument 2 must be str"),
        None => MbValue::from_ptr(MbObject::new_str(String::new())),
    };
    super::super::builtins::mb_format(val, spec)
}

unsafe extern "C" fn dispatch_ascii(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_ascii(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_eval(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_eval(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_exec(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_exec(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_compile(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let source = args.first().copied().unwrap_or_else(MbValue::none);
    let filename = args.get(1).copied().unwrap_or_else(MbValue::none);
    let mode = args.get(2).copied().unwrap_or_else(MbValue::none);
    super::super::builtins::mb_compile(source, filename, mode)
}

unsafe extern "C" fn dispatch_dunder_import(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_dunder_import(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_globals(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    super::super::builtins::mb_globals()
}

unsafe extern "C" fn dispatch_locals(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    super::super::builtins::mb_locals()
}

unsafe extern "C" fn dispatch_dir(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs == 0 {
        super::super::class::mb_dir_no_args()
    } else {
        super::super::class::mb_dir(args[0])
    }
}

unsafe extern "C" fn dispatch_vars(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::class::mb_vars(args.first().copied().unwrap_or_else(MbValue::none))
}

unsafe extern "C" fn dispatch_iter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    if nargs >= 2 {
        super::super::iter::mb_iter_sentinel(args[0], args[1])
    } else {
        super::super::iter::mb_iter(args.first().copied().unwrap_or_else(MbValue::none))
    }
}

unsafe extern "C" fn dispatch_next(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let iter_handle = args.first().copied().unwrap_or_else(MbValue::none);
    if nargs >= 2 {
        super::super::iter::mb_next_default(iter_handle, args[1])
    } else {
        super::super::iter::mb_next(iter_handle)
    }
}

unsafe extern "C" fn dispatch_list(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::list_ops::mb_list_new();
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::list_ops::mb_list_from_iterable(args[0])
}

unsafe extern "C" fn dispatch_dict(
    args_ptr: *const MbValue,
    nargs: usize,
) -> MbValue {
    if nargs == 0 {
        return super::super::dict_ops::mb_dict_new();
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::dict_ops::mb_dict_from_pairs(args[0])
}

unsafe extern "C" fn dispatch_set(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::set_ops::mb_set_new();
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_set_from_iterable(args[0])
}

unsafe extern "C" fn dispatch_tuple(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::tuple_ops::mb_tuple_new();
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::tuple_ops::mb_tuple_from_iterable(args[0])
}

unsafe extern "C" fn dispatch_frozenset(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::builtins::mb_frozenset_new(MbValue::none());
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_frozenset_new(args[0])
}

unsafe extern "C" fn dispatch_complex(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let real = args.first().copied().unwrap_or_else(|| MbValue::from_int(0));
    let imag = args.get(1).copied().unwrap_or_else(|| MbValue::from_int(0));
    super::super::builtins::mb_complex(real, imag)
}

unsafe extern "C" fn dispatch_bytes(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::bytes_ops::mb_bytes_new_checked(MbValue::none());
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    if let Some(encoding) = args.get(1).copied() {
        return super::super::bytes_ops::mb_bytes_new_encoded(args[0], encoding);
    }
    super::super::bytes_ops::mb_bytes_new_checked(args[0])
}

unsafe extern "C" fn dispatch_bytearray(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::bytes_ops::mb_bytearray_new_checked(MbValue::none());
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    if let Some(encoding) = args.get(1).copied() {
        return super::super::bytes_ops::mb_bytearray_new_encoded(args[0], encoding);
    }
    super::super::bytes_ops::mb_bytearray_new_checked(args[0])
}

unsafe extern "C" fn dispatch_memoryview(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::builtins::mb_memoryview(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_slice(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    if nargs == 0 {
        return super::super::builtins::mb_slice_no_args();
    }
    let args = unsafe { safe_args(args_ptr, nargs) };
    match nargs {
        1 => super::super::builtins::mb_slice(MbValue::none(), args[0], MbValue::none()),
        2 => super::super::builtins::mb_slice(args[0], args[1], MbValue::none()),
        _ => super::super::builtins::mb_slice(args[0], args[1], args[2]),
    }
}

unsafe extern "C" fn dispatch_object(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    // object() returns a featureless object -- stub as None
    MbValue::none()
}

unsafe extern "C" fn dispatch_super(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    let class_name = args.first().copied().unwrap_or_else(MbValue::none);
    let instance = args.get(1).copied().unwrap_or_else(MbValue::none);
    super::super::class::mb_super(class_name, instance)
}

unsafe extern "C" fn dispatch_property(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::class::mb_property_new(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_classmethod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::class::mb_classmethod_new(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_staticmethod(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    super::super::class::mb_staticmethod_new(
        args.first().copied().unwrap_or_else(MbValue::none),
    )
}

unsafe extern "C" fn dispatch_breakpoint(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    // PEP 553 default sys.breakpointhook — see `mb_breakpoint`. Args
    // and kwargs are dropped; the hook only prints a "breakpoint hit"
    // notice (or stays silent under PYTHONBREAKPOINT=0). (#1256)
    super::super::builtins::mb_breakpoint()
}

// ── Present-but-stub callables ──
//
// CPython exposes these names in `builtins` and they are callable. mamba does
// not yet implement their behaviour; registering a callable stub satisfies the
// module surface (`hasattr(builtins, NAME)` / `callable(builtins.NAME)`)
// without claiming real semantics. Each returns None.

unsafe extern "C" fn dispatch_aiter(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    // aiter(async_iterable) — async iteration is not yet supported; return the
    // argument unchanged so the name is present and callable.
    let args = unsafe { safe_args(args_ptr, nargs) };
    args.first().copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn dispatch_anext(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { safe_args(args_ptr, nargs) };
    args.get(1).copied().unwrap_or_else(MbValue::none)
}

unsafe extern "C" fn dispatch_builtins_stub(
    _args_ptr: *const MbValue,
    _nargs: usize,
) -> MbValue {
    // exit / quit / help / copyright / credits / license: interactive helpers
    // present in the builtins namespace. Stubbed to a no-op returning None.
    MbValue::none()
}

/// Register the builtins module.
// @spec .aw/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md#R1
pub fn register() {
    let mut attrs = HashMap::new();

    // -- Constants --
    attrs.insert("True".to_string(), MbValue::from_bool(true));
    attrs.insert("False".to_string(), MbValue::from_bool(false));
    attrs.insert("None".to_string(), MbValue::none());

    // Sentinel / special constants. NotImplemented is a real singleton
    // (TAG_NOTIMPLEMENTED in the NaN-box tag space); print() / str() /
    // repr() already render `NotImplemented`. Ellipsis stays as None
    // pending a real ellipsis runtime entity — `...` literal in source
    // also lowers to None so user code is internally consistent.
    attrs.insert("Ellipsis".to_string(), MbValue::none());
    attrs.insert("NotImplemented".to_string(), MbValue::not_implemented());
    attrs.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str("builtins".to_string())),
    );
    attrs.insert(
        "__doc__".to_string(),
        MbValue::from_ptr(MbObject::new_str(
            "Built-in functions, types, exceptions, and other objects.".to_string(),
        )),
    );

    // -- Functions via dispatch wrappers --
    // Each entry: (python_name, dispatch_fn_address)
    let dispatchers: &[(&str, usize)] = &[
        ("print", dispatch_print as *const () as usize),
        ("len", dispatch_len as *const () as usize),
        ("type", dispatch_type as *const () as usize),
        ("range", dispatch_range as *const () as usize),
        ("int", dispatch_int as *const () as usize),
        ("float", dispatch_float as *const () as usize),
        ("str", dispatch_str as *const () as usize),
        ("bool", dispatch_bool as *const () as usize),
        ("abs", dispatch_abs as *const () as usize),
        ("min", dispatch_min as *const () as usize),
        ("max", dispatch_max as *const () as usize),
        ("sum", dispatch_sum as *const () as usize),
        ("sorted", dispatch_sorted as *const () as usize),
        ("reversed", dispatch_reversed as *const () as usize),
        ("enumerate", dispatch_enumerate as *const () as usize),
        ("zip", dispatch_zip as *const () as usize),
        ("map", dispatch_map as *const () as usize),
        ("filter", dispatch_filter as *const () as usize),
        ("all", dispatch_all as *const () as usize),
        ("any", dispatch_any as *const () as usize),
        ("input", dispatch_input as *const () as usize),
        ("open", dispatch_open as *const () as usize),
        ("chr", dispatch_chr as *const () as usize),
        ("ord", dispatch_ord as *const () as usize),
        ("hex", dispatch_hex as *const () as usize),
        ("oct", dispatch_oct as *const () as usize),
        ("bin", dispatch_bin as *const () as usize),
        ("round", dispatch_round as *const () as usize),
        ("pow", dispatch_pow as *const () as usize),
        ("divmod", dispatch_divmod as *const () as usize),
        ("repr", dispatch_repr as *const () as usize),
        ("hash", dispatch_hash as *const () as usize),
        ("id", dispatch_id as *const () as usize),
        ("isinstance", dispatch_isinstance as *const () as usize),
        ("issubclass", dispatch_issubclass as *const () as usize),
        ("hasattr", dispatch_hasattr as *const () as usize),
        ("getattr", dispatch_getattr as *const () as usize),
        ("setattr", dispatch_setattr as *const () as usize),
        ("delattr", dispatch_delattr as *const () as usize),
        ("callable", dispatch_callable as *const () as usize),
        ("format", dispatch_format as *const () as usize),
        ("ascii", dispatch_ascii as *const () as usize),
        ("eval", dispatch_eval as *const () as usize),
        ("exec", dispatch_exec as *const () as usize),
        ("compile", dispatch_compile as *const () as usize),
        ("__import__", dispatch_dunder_import as *const () as usize),
        ("globals", dispatch_globals as *const () as usize),
        ("locals", dispatch_locals as *const () as usize),
        ("dir", dispatch_dir as *const () as usize),
        ("vars", dispatch_vars as *const () as usize),
        ("iter", dispatch_iter as *const () as usize),
        ("next", dispatch_next as *const () as usize),
        ("list", dispatch_list as *const () as usize),
        ("dict", dispatch_dict as *const () as usize),
        ("set", dispatch_set as *const () as usize),
        ("tuple", dispatch_tuple as *const () as usize),
        ("frozenset", dispatch_frozenset as *const () as usize),
        ("complex", dispatch_complex as *const () as usize),
        ("bytes", dispatch_bytes as *const () as usize),
        ("bytearray", dispatch_bytearray as *const () as usize),
        ("memoryview", dispatch_memoryview as *const () as usize),
        ("slice", dispatch_slice as *const () as usize),
        ("object", dispatch_object as *const () as usize),
        ("super", dispatch_super as *const () as usize),
        ("property", dispatch_property as *const () as usize),
        ("classmethod", dispatch_classmethod as *const () as usize),
        ("staticmethod", dispatch_staticmethod as *const () as usize),
        ("breakpoint", dispatch_breakpoint as *const () as usize),
        ("aiter", dispatch_aiter as *const () as usize),
        ("anext", dispatch_anext as *const () as usize),
        ("exit", dispatch_builtins_stub as *const () as usize),
        ("quit", dispatch_builtins_stub as *const () as usize),
        ("help", dispatch_builtins_stub as *const () as usize),
        ("copyright", dispatch_builtins_stub as *const () as usize),
        ("credits", dispatch_builtins_stub as *const () as usize),
        ("license", dispatch_builtins_stub as *const () as usize),
    ];

    for (name, addr) in dispatchers {
        let func = MbValue::from_func(*addr);
        if is_builtin_type_surface(name) {
            let type_name = MbValue::from_ptr(MbObject::new_str((*name).to_string()));
            attrs.insert(
                name.to_string(),
                super::super::builtins::mb_builtin_type_obj(type_name),
            );
        } else {
            attrs.insert(name.to_string(), func);
            super::super::closure::mb_func_set_name(
                func,
                MbValue::from_ptr(MbObject::new_str((*name).to_string())),
            );
            super::super::closure::mb_func_set_module(
                func,
                MbValue::from_ptr(MbObject::new_str("builtins".to_string())),
            );
            super::super::closure::mb_func_set_doc(
                func,
                MbValue::from_ptr(MbObject::new_str(format!(
                    "Built-in function {name}."
                ))),
            );
        }
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(*addr as u64);
        });
    }

    // -- Built-in exception & warning class objects --
    //
    // CPython exposes the whole exception hierarchy plus the warning categories
    // in `builtins`. Each is a `type` object, so `type(builtins.ValueError)` is
    // `type` and `callable(builtins.ValueError)` is True. We register each as a
    // cached builtin type object (class_name == "type"), matching how the bare
    // builtin types (`int`, `list`, ...) are surfaced above.
    let exception_class_names: &[&str] = &[
        "BaseException",
        "BaseExceptionGroup",
        "Exception",
        "ExceptionGroup",
        "ArithmeticError",
        "AssertionError",
        "AttributeError",
        "BlockingIOError",
        "BrokenPipeError",
        "BufferError",
        "ChildProcessError",
        "ConnectionAbortedError",
        "ConnectionError",
        "ConnectionRefusedError",
        "ConnectionResetError",
        "EOFError",
        "EnvironmentError",
        "FileExistsError",
        "FileNotFoundError",
        "FloatingPointError",
        "GeneratorExit",
        "IOError",
        "ImportError",
        "IndentationError",
        "IndexError",
        "InterruptedError",
        "IsADirectoryError",
        "KeyError",
        "KeyboardInterrupt",
        "LookupError",
        "MemoryError",
        "ModuleNotFoundError",
        "NameError",
        "NotADirectoryError",
        "NotImplementedError",
        "OSError",
        "OverflowError",
        "PermissionError",
        "ProcessLookupError",
        "RecursionError",
        "ReferenceError",
        "RuntimeError",
        "StopAsyncIteration",
        "StopIteration",
        "SyntaxError",
        "SystemError",
        "SystemExit",
        "TabError",
        "TimeoutError",
        "TypeError",
        "UnboundLocalError",
        "UnicodeDecodeError",
        "UnicodeEncodeError",
        "UnicodeError",
        "UnicodeTranslateError",
        "ValueError",
        "ZeroDivisionError",
        // Warning categories
        "Warning",
        "BytesWarning",
        "DeprecationWarning",
        "EncodingWarning",
        "FutureWarning",
        "ImportWarning",
        "PendingDeprecationWarning",
        "ResourceWarning",
        "RuntimeWarning",
        "SyntaxWarning",
        "UnicodeWarning",
        "UserWarning",
    ];
    for name in exception_class_names {
        let type_name = MbValue::from_ptr(MbObject::new_str((*name).to_string()));
        attrs.insert(
            (*name).to_string(),
            super::super::builtins::mb_builtin_type_obj(type_name),
        );
    }

    super::register_module("builtins", attrs);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: produce a valid aligned pointer for zero-arg dispatch calls.
    fn empty_args_ptr() -> *const MbValue {
        static EMPTY: [u8; 8] = [0; 8];
        EMPTY.as_ptr() as *const MbValue
    }

    #[test]
    fn test_register_module() {
        // Calling register() should not panic
        register();
    }

    #[test]
    fn test_true_constant() {
        let v = MbValue::from_bool(true);
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_false_constant() {
        let v = MbValue::from_bool(false);
        assert_eq!(v.as_bool(), Some(false));
    }

    #[test]
    fn test_none_constant() {
        let v = MbValue::none();
        assert!(v.is_none());
    }

    #[test]
    fn test_dispatch_len() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let args = [list];
        let result = unsafe { dispatch_len(args.as_ptr(), 1) };
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_dispatch_abs() {
        let args = [MbValue::from_int(-42)];
        let result = unsafe { dispatch_abs(args.as_ptr(), 1) };
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_dispatch_int_no_args() {
        let result = unsafe { dispatch_int(empty_args_ptr(), 0) };
        assert_eq!(result.as_int(), Some(0));
    }

    #[test]
    fn test_dispatch_float_no_args() {
        let result = unsafe { dispatch_float(empty_args_ptr(), 0) };
        assert_eq!(result.as_float(), Some(0.0));
    }

    #[test]
    fn test_dispatch_bool_no_args() {
        let result = unsafe { dispatch_bool(empty_args_ptr(), 0) };
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_dispatch_str_no_args() {
        let result = unsafe { dispatch_str(empty_args_ptr(), 0) };
        let ptr = result.as_ptr().expect("expected string object");
        unsafe {
            if let super::super::super::rc::ObjData::Str(ref s) = (*ptr).data {
                assert_eq!(s.as_str(), "");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_dispatch_chr() {
        let args = [MbValue::from_int(65)];
        let result = unsafe { dispatch_chr(args.as_ptr(), 1) };
        let ptr = result.as_ptr().expect("expected string object");
        unsafe {
            if let super::super::super::rc::ObjData::Str(ref s) = (*ptr).data {
                assert_eq!(s.as_str(), "A");
            } else {
                panic!("expected Str");
            }
        }
    }

    #[test]
    fn test_dispatch_ord() {
        let s = MbValue::from_ptr(MbObject::new_str("A".to_string()));
        let args = [s];
        let result = unsafe { dispatch_ord(args.as_ptr(), 1) };
        assert_eq!(result.as_int(), Some(65));
    }

    #[test]
    fn test_dispatch_bool_from_int() {
        let args = [MbValue::from_int(1)];
        let result = unsafe { dispatch_bool(args.as_ptr(), 1) };
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_dispatch_int_from_float() {
        let args = [MbValue::from_float(3.7)];
        let result = unsafe { dispatch_int(args.as_ptr(), 1) };
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_dispatch_list_no_args() {
        let result = unsafe { dispatch_list(empty_args_ptr(), 0) };
        let ptr = result.as_ptr().expect("expected list object");
        unsafe {
            if let super::super::super::rc::ObjData::List(ref lock) = (*ptr).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected List");
            }
        }
    }

    #[test]
    fn test_dispatch_dict_no_args() {
        let result = unsafe { dispatch_dict(empty_args_ptr(), 0) };
        let ptr = result.as_ptr().expect("expected dict object");
        unsafe {
            if let super::super::super::rc::ObjData::Dict(ref lock) = (*ptr).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected Dict");
            }
        }
    }

    #[test]
    fn test_dispatch_tuple_no_args() {
        let result = unsafe { dispatch_tuple(empty_args_ptr(), 0) };
        let ptr = result.as_ptr().expect("expected tuple object");
        unsafe {
            if let super::super::super::rc::ObjData::Tuple(ref elems) = (*ptr).data {
                assert_eq!(elems.len(), 0);
            } else {
                panic!("expected Tuple");
            }
        }
    }

    #[test]
    fn test_dispatch_set_no_args() {
        let result = unsafe { dispatch_set(empty_args_ptr(), 0) };
        let ptr = result.as_ptr().expect("expected set object");
        unsafe {
            if let super::super::super::rc::ObjData::Set(ref lock) = (*ptr).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected Set");
            }
        }
    }
}
