use super::output::{write_captured, writeln_captured};
use super::rc::{MbObject, ObjData};
/// Runtime built-in function implementations (#281).
///
/// These are the actual implementations of built-in functions that the
/// compiled code calls at runtime via function pointers or extern calls.
use super::value::MbValue;
use rustc_hash::FxHashMap;

mod any_all;
mod ascii;
mod assertions;
mod async_iteration;
mod boxing;
mod breakpoint;
mod char_radix;
mod eval_exec;
mod hash;
mod import_hook;
mod input;
mod length;
mod map_filter;
mod memoryview;
mod numeric_format;
mod object_identity;
mod range_slice;
mod set_constructors;
mod truthiness;
mod type_objects;

pub use any_all::{mb_all, mb_any};
pub use ascii::mb_ascii;
pub use assertions::{mb_assertion_error, mb_assertion_error_no_msg};
pub use async_iteration::{mb_aiter, mb_anext, mb_anext_default};
use boxing::format_bytes_inner;
pub(crate) use boxing::{box_raw_i64_or_bigint, passthrough_boxed_int_candidate};
pub use boxing::{
    mb_box_bool, mb_box_float, mb_box_int, mb_box_int_for_compare, mb_pow_float, mb_pow_int,
    mb_unbox_bool, mb_unbox_bool_if_boxed, mb_unbox_float, mb_unbox_float_if_boxed,
    mb_unbox_inline_int_if_boxed, mb_unbox_int, mb_unbox_int_if_boxed,
};
pub use breakpoint::{mb_breakpoint, mb_breakpoint_call};
pub(crate) use char_radix::resolve_index_value;
pub use char_radix::{mb_bin, mb_chr, mb_hex, mb_oct, mb_ord};
use eval_exec::{eval_pending, eval_str_value};
pub use eval_exec::{
    mb_compile, mb_compile_5, mb_compile_kwargs, mb_eval, mb_eval_with_globals,
    mb_eval_with_namespaces, mb_exec, mb_exec_function_call, mb_exec_function_is_async,
    mb_exec_with_globals, mb_exec_with_globals_locals, mb_globals, mb_locals,
};
pub use hash::mb_hash;
pub use import_hook::mb_dunder_import;
pub use input::mb_input;
pub use length::mb_len;
pub(crate) use length::validate_len_result;
use map_filter::call_named_callable;
pub use map_filter::{
    call_any_callable, call_named_callable_pub, callable_as_type_name, mb_filter, mb_map,
};
pub use memoryview::mb_memoryview;
pub use numeric_format::{mb_divmod, mb_format, mb_round};
pub use object_identity::mb_id;
pub use range_slice::{
    mb_range, mb_range_2, mb_range_3, mb_range_no_args, mb_range_too_many_args, mb_slice,
    mb_slice_no_args,
};
pub use set_constructors::{mb_frozenset_empty, mb_frozenset_new, mb_set_from_iterable};
pub use truthiness::{
    mb_is_identity, mb_is_none, mb_is_not_identity, mb_is_not_none, mb_is_truthy, mb_not,
};
pub(crate) use type_objects::{make_type_object, reject_non_constructible_type_object};
pub use type_objects::{
    mb_builtin_type_obj, mb_type, mb_type2, mb_type3, mb_type3_kwargs, mb_type_no_args,
};

/// Write to capture buffer if active, else to stdout.
macro_rules! mb_out {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        if !write_captured(&s) {
            print!("{}", s);
        }
    }};
}

/// Writeln to capture buffer if active, else to stdout.
macro_rules! mb_outln {
    ($($arg:tt)*) => {{
        let s = format!($($arg)*);
        if !writeln_captured(&s) {
            println!("{}", s);
        }
    }};
}

/// print(value) — print a value to stdout (or capture buffer).
pub fn mb_print(val: MbValue) -> MbValue {
    // TAG_FUNC user-defined functions render as `<function NAME at 0xADDR>`
    // to match CPython. Closure handles share TAG_INT with regular ints and
    // can't be reliably detected here without colliding with low-value ints,
    // so they fall through to the int branch (rendering as their handle ID).
    if let Some(name) = pep695_display_name(val) {
        mb_outln!("{name}");
        return MbValue::none();
    }
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        mb_outln!("<function {name} at 0x{addr:x}>");
        return MbValue::none();
    }
    if let Some(i) = val.as_int() {
        mb_outln!("{i}");
    } else if let Some(f) = val.as_float() {
        mb_outln!("{}", super::string_ops::python_float_repr(f));
    } else if let Some(b) = val.as_bool() {
        mb_outln!("{}", if b { "True" } else { "False" });
    } else if val.is_none() {
        mb_outln!("None");
    } else if val.is_not_implemented() {
        mb_outln!("NotImplemented");
    } else if val.is_ellipsis() {
        mb_outln!("Ellipsis");
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => mb_outln!("{s}"),
                ObjData::List(ref lock) => {
                    // Cycle guard (#1061): a self-referential list
                    // (`l.append(l)`) would recurse into `print_repr`
                    // forever without this — CPython's Py_ReprEnter detects
                    // the re-entry and prints `[...]` instead. This is the
                    // top-level print(x) entry, so the guard almost always
                    // succeeds here; it only matters when print() itself is
                    // reached from inside another repr in progress.
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_outln!("[...]");
                    } else {
                        let items = lock.read().unwrap();
                        mb_out!("[");
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_repr(*item);
                        }
                        mb_outln!("]");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Dict(ref lock) => {
                    // Cycle guard — see the List arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_outln!("{{...}}");
                    } else {
                        let map = lock.read().unwrap();
                        mb_out!("{{");
                        for (i, (k, v)) in map.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_dict_key(k);
                            mb_out!(": ");
                            print_repr(*v);
                        }
                        mb_outln!("}}");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Tuple(items) => {
                    // Cycle guard — see the List arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_outln!("(...)");
                    } else {
                        mb_out!("(");
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_repr(*item);
                        }
                        if items.len() == 1 {
                            mb_out!(",");
                        }
                        mb_outln!(")");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                    ..
                } => {
                    if matches!(class_name.as_str(), "GenericAlias" | "types.GenericAlias") {
                        if let Some(s) = generic_alias_repr_string(class_name, fields) {
                            mb_outln!("{s}");
                            return MbValue::none();
                        }
                    }
                    // Namedtuple: Point(x=1, y=2) format
                    let f = fields.read().unwrap();
                    if let Some(nt_fields) = f.get("_namedtuple_fields") {
                        let nt_name = f
                            .get("_namedtuple_name")
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| class_name.clone());
                        mb_out!("{nt_name}(");
                        let field_names: Vec<String> = nt_fields
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::List(ref lk) = (*p).data {
                                    Some(
                                        lk.read()
                                            .unwrap()
                                            .iter()
                                            .filter_map(|v| {
                                                v.as_ptr().and_then(|pp| {
                                                    if let ObjData::Str(ref s) = (*pp).data {
                                                        Some(s.clone())
                                                    } else {
                                                        None
                                                    }
                                                })
                                            })
                                            .collect(),
                                    )
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        for (i, fname) in field_names.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            let val = f.get(fname).copied().unwrap_or(MbValue::none());
                            mb_out!("{fname}=");
                            print_repr(val);
                        }
                        drop(f);
                        mb_outln!(")");
                    } else if let Some(args_v) = f.get("args").copied() {
                        // Exception print: 0 args → blank; 1 arg → str(arg0);
                        // ≥2 args → repr of args tuple. KeyError quirk on the
                        // 1-arg path. (#1652)
                        let items: Option<Vec<MbValue>> = args_v.as_ptr().and_then(|p| {
                            if let ObjData::Tuple(ref it) = (*p).data {
                                Some(it.clone())
                            } else {
                                None
                            }
                        });
                        drop(f);
                        if let Some(items) = items {
                            match items.len() {
                                0 => mb_out!("\n"),
                                1 => {
                                    let a0 = items[0];
                                    if class_name == "KeyError" {
                                        if let Some(p) = a0.as_ptr() {
                                            if let ObjData::Str(ref s) = (*p).data {
                                                mb_outln!("'{}'", s.replace('\'', "\\'"));
                                                return MbValue::none();
                                            }
                                        }
                                    }
                                    print_value_str(a0);
                                    mb_out!("\n");
                                }
                                _ => {
                                    let tuple_val = MbValue::from_ptr(MbObject::new_tuple(items));
                                    print_repr(tuple_val);
                                    mb_out!("\n");
                                }
                            }
                            return MbValue::none();
                        }
                        // Fall through if `args` isn't a tuple (defensive).
                    } else if let Some(msg_val) = f.get("message") {
                        let msg_copy = *msg_val;
                        let is_key_error = class_name == "KeyError";
                        drop(f);
                        if is_key_error {
                            if let Some(ptr) = msg_copy.as_ptr() {
                                if let ObjData::Str(ref s) = (*ptr).data {
                                    mb_outln!("'{}'", s.replace('\'', "\\'"));
                                    return MbValue::none();
                                }
                            }
                        }
                        print_value_str(msg_copy);
                        mb_out!("\n");
                    } else if class_name == "type" {
                        // Type objects: print <class 'name'> matching CPython
                        let type_name = f
                            .get("__name__")
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "unknown".to_string());
                        drop(f);
                        mb_outln!("<class '{type_name}'>");
                    } else if class_name == "slice" {
                        let s = f.get("start").copied().unwrap_or(MbValue::none());
                        let e = f.get("stop").copied().unwrap_or(MbValue::none());
                        let st = f.get("step").copied().unwrap_or(MbValue::none());
                        drop(f);
                        mb_out!("slice(");
                        print_repr(s);
                        mb_out!(", ");
                        print_repr(e);
                        mb_out!(", ");
                        print_repr(st);
                        mb_outln!(")");
                    } else if class_name == "memoryview" {
                        // memoryview repr matches CPython's "<memory at 0x...>"
                        // surface — keep print() / str() / repr() consistent. (#1256)
                        drop(f);
                        mb_outln!("<memory at 0x{:x}>", ptr as usize);
                    } else if class_name == "collections.Counter" {
                        // print(c) emits CPython-style Counter repr. (#1638)
                        drop(f);
                        mb_outln!("{}", super::stdlib::collections_mod::counter_repr(val));
                    } else if class_name == "collections.defaultdict" {
                        // print(dd) emits CPython-style defaultdict repr. (#1640)
                        drop(f);
                        mb_outln!("{}", super::stdlib::collections_mod::defaultdict_repr(val));
                    } else if class_name == "collections.deque" {
                        // print(dq) emits CPython-style deque repr. (#1640)
                        drop(f);
                        mb_outln!("{}", super::stdlib::collections_mod::deque_repr(val));
                    } else if class_name == "collections.OrderedDict" {
                        // print(od) emits CPython-style OrderedDict repr. (#1650)
                        drop(f);
                        mb_outln!("{}", super::stdlib::collections_mod::ordereddict_repr(val));
                    } else if class_name == "re.Match" {
                        // print(m) emits CPython-style re.Match repr. (#1642)
                        drop(f);
                        mb_outln!("{}", super::stdlib::re_mod::match_repr(val));
                    } else if class_name == "re.Pattern" {
                        // print(p) emits CPython-style re.Pattern repr. (#1642)
                        drop(f);
                        mb_outln!("{}", super::stdlib::re_mod::pattern_repr(val));
                    } else if class_name == "datetime.datetime" {
                        // print(dt) emits CPython str form (no microsecond). (#1644)
                        drop(f);
                        mb_outln!("{}", super::stdlib::datetime_mod::datetime_str(val));
                    } else if class_name == "datetime.timedelta" {
                        // print(td) emits CPython str form. (#1644)
                        drop(f);
                        mb_outln!("{}", super::stdlib::datetime_mod::timedelta_str(val));
                    } else if class_name == "datetime.time" {
                        drop(f);
                        mb_outln!("{}", super::stdlib::datetime_mod::time_str(val));
                    } else if class_name == "datetime.timezone" {
                        drop(f);
                        mb_outln!("{}", super::stdlib::datetime_mod::timezone_str(val));
                    } else if let Some(s) = super::stdlib::enum_class::member_str(val) {
                        // Class-body enum member without a user __str__:
                        // print(Color.RED) → "Color.RED".
                        drop(f);
                        mb_outln!("{s}");
                    } else {
                        drop(f);
                        // __str__ dunder dispatch for print(); CPython falls back
                        // to __repr__ when __str__ is not defined.
                        let str_method = super::class::lookup_method(class_name, "__str__");
                        let method = if !str_method.is_none() {
                            str_method
                        } else {
                            super::class::lookup_method(class_name, "__repr__")
                        };
                        if !method.is_none() {
                            let result = super::class::mb_call_method1(method, val);
                            if let Some(p) = result.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    mb_outln!("{s}");
                                } else {
                                    mb_outln!("<{class_name} instance>");
                                }
                            } else {
                                mb_outln!("<{class_name} instance>");
                            }
                        } else if let Some((base, payload)) =
                            super::class::builtin_data_payload(val)
                        {
                            // Builtin-subclass instance without a __str__/
                            // __repr__ override prints as its payload's str(),
                            // matching mb_str/value_to_string's fallback for
                            // str()/f-string (#1032). set/frozenset subclasses
                            // fall back to repr (CPython's default
                            // object.__str__), so print_repr's class-name-
                            // prefixed fallback wins here instead of the raw
                            // payload's un-prefixed print. (#1051)
                            if base == "set" || base == "frozenset" {
                                print_repr(val);
                                mb_out!("\n");
                            } else {
                                print_value_str(payload);
                                mb_outln!("");
                            }
                        } else {
                            mb_outln!("<{class_name} instance>");
                        }
                    }
                }
                ObjData::Set(ref lock) => {
                    // Cycle guard — see the List arm above (#1061). Sets
                    // can't directly self-reference (elements must be
                    // hashable), but an element's __repr__ can call back
                    // into the same set's print indirectly.
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_outln!("set(...)");
                    } else {
                        let items = lock.read().unwrap();
                        if items.is_empty() {
                            mb_outln!("set()");
                        } else {
                            mb_out!("{{");
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 {
                                    mb_out!(", ");
                                }
                                print_repr(*item);
                            }
                            mb_outln!("}}");
                        }
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::FrozenSet(items) => {
                    // Cycle guard — see the Set arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_outln!("frozenset(...)");
                    } else {
                        if items.is_empty() {
                            mb_outln!("frozenset()");
                        } else {
                            mb_out!("frozenset({{");
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 {
                                    mb_out!(", ");
                                }
                                print_repr(*item);
                            }
                            mb_outln!("}})");
                        }
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Bytes(data) => mb_outln!("b{}", format_bytes_inner(data)),
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    mb_outln!("bytearray(b{})", format_bytes_inner(&data));
                }
                ObjData::BigInt(big) => mb_outln!("{big}"),
                ObjData::Complex(re, im) => {
                    // CPython: when real component is +0.0, print just `{im}j`
                    // (no parens). Negative-zero real keeps the parenthesized form.
                    mb_outln!("{}", super::string_ops::complex_repr_string(*re, *im));
                }
                ObjData::CodeObject { filename, mode, .. } => {
                    mb_outln!("<code object <module> at {filename} mode={mode}>")
                }
            }
        }
    }
    MbValue::none()
}

/// print(*args) — print multiple values separated by spaces, like Python.
/// Takes a list MbValue containing the arguments.
pub fn mb_print_args(args_list: MbValue) -> MbValue {
    if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        mb_out!(" ");
                    }
                    print_value_str(*item);
                }
                mb_outln!("");
                return MbValue::none();
            }
        }
    }
    // Fallback: single value
    mb_print(args_list)
}

/// Print a value using str() semantics (not repr).
fn print_value_str(val: MbValue) {
    if let Some(name) = pep695_display_name(val) {
        mb_out!("{name}");
        return;
    }
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        mb_out!("<function {name} at 0x{addr:x}>");
        return;
    }
    if let Some(i) = val.as_int() {
        mb_out!("{i}");
    } else if let Some(f) = val.as_float() {
        mb_out!("{}", super::string_ops::python_float_repr(f));
    } else if let Some(b) = val.as_bool() {
        mb_out!("{}", if b { "True" } else { "False" });
    } else if val.is_none() {
        mb_out!("None");
    } else if val.is_not_implemented() {
        mb_out!("NotImplemented");
    } else if val.is_ellipsis() {
        mb_out!("Ellipsis");
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => mb_out!("{s}"),
                // Exception instances: print the message (Python's str(exc))
                ObjData::Instance {
                    ref class_name,
                    ref fields,
                } => {
                    // namedtuple has a dynamic class_name (the user-given tuple
                    // name), so dispatch by marker field. (#1648)
                    if let Some(s) = super::stdlib::collections_mod::namedtuple_repr(val) {
                        mb_out!("{s}");
                        return;
                    }
                    // Stdlib types with structured str/repr — same set wired in
                    // `mb_print` / `value_to_string`; missing here breaks
                    // multi-arg `print(x, y)` paths. (#1646)
                    if class_name == "UnionType" {
                        mb_out!("{}", union_type_repr(val));
                        return;
                    }
                    if class_name == "collections.Counter" {
                        mb_out!("{}", super::stdlib::collections_mod::counter_repr(val));
                        return;
                    }
                    if class_name == "collections.defaultdict" {
                        mb_out!("{}", super::stdlib::collections_mod::defaultdict_repr(val));
                        return;
                    }
                    if class_name == "collections.deque" {
                        mb_out!("{}", super::stdlib::collections_mod::deque_repr(val));
                        return;
                    }
                    if class_name == "collections.OrderedDict" {
                        mb_out!("{}", super::stdlib::collections_mod::ordereddict_repr(val));
                        return;
                    }
                    if class_name == "re.Match" {
                        mb_out!("{}", super::stdlib::re_mod::match_repr(val));
                        return;
                    }
                    if class_name == "re.Pattern" {
                        mb_out!("{}", super::stdlib::re_mod::pattern_repr(val));
                        return;
                    }
                    if class_name == "datetime.datetime" {
                        mb_out!("{}", super::stdlib::datetime_mod::datetime_str(val));
                        return;
                    }
                    if class_name == "datetime.time" {
                        mb_out!("{}", super::stdlib::datetime_mod::time_str(val));
                    } else if class_name == "datetime.timezone" {
                        mb_out!("{}", super::stdlib::datetime_mod::timezone_str(val));
                    } else if class_name == "datetime.timedelta" {
                        mb_out!("{}", super::stdlib::datetime_mod::timedelta_str(val));
                        return;
                    }
                    let fields = fields.read().unwrap();
                    // Exception str semantics: 0 args → ""; 1 arg → str(arg0);
                    // ≥2 args → repr(args) (the tuple's repr). KeyError keeps
                    // its quirk where __str__ is repr(args[0]). (#1652)
                    if let Some(args_v) = fields.get("args").copied() {
                        let arg_items: Option<Vec<MbValue>> = args_v.as_ptr().and_then(|p| {
                            if let ObjData::Tuple(ref items) = (*p).data {
                                Some(items.clone())
                            } else {
                                None
                            }
                        });
                        if let Some(items) = arg_items {
                            drop(fields);
                            match items.len() {
                                0 => return,
                                1 => {
                                    let a0 = items[0];
                                    if class_name == "KeyError" {
                                        // KeyError.__str__ is repr(key); use the
                                        // shared repr so quote selection matches
                                        // CPython (a key containing `'` is
                                        // double-quoted, not single-escaped).
                                        print_repr(a0);
                                        return;
                                    }
                                    print_value_str(a0);
                                    return;
                                }
                                _ => {
                                    // Render as the tuple's repr.
                                    let tuple_val = MbValue::from_ptr(MbObject::new_tuple(items));
                                    print_repr(tuple_val);
                                    return;
                                }
                            }
                        }
                    }
                    if let Some(msg) = fields.get("message") {
                        if class_name == "KeyError" {
                            // KeyError.__str__ is repr(key) — defer to the shared
                            // repr for CPython-matching quote selection.
                            print_repr(*msg);
                            return;
                        }
                        print_value_str(*msg);
                    } else {
                        // __str__ dunder dispatch (falling back to __repr__) —
                        // mirrors mb_print's single-arg dispatch so multi-arg
                        // print / f-string / str.join don't diverge for a
                        // plain instance (no "args"/"message" field) (#1032).
                        drop(fields);
                        let str_method = super::class::lookup_method(class_name, "__str__");
                        let method = if !str_method.is_none() {
                            Some(str_method)
                        } else {
                            let repr_method = super::class::lookup_method(class_name, "__repr__");
                            (!repr_method.is_none()).then_some(repr_method)
                        };
                        if let Some(method) = method {
                            let result = super::class::mb_call_method1(method, val);
                            if let Some(p) = result.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    mb_out!("{s}");
                                    return;
                                }
                            }
                            mb_out!("<{class_name} instance>");
                            return;
                        }
                        // Builtin-subclass instance without a __str__/
                        // __repr__ override stringifies as its payload's
                        // str(), matching mb_str/value_to_string's fallback
                        // (#1032). set/frozenset subclasses fall back to
                        // repr (CPython's default object.__str__), so
                        // print_repr's class-name-prefixed fallback wins
                        // here instead of the raw payload's un-prefixed
                        // print. (#1051)
                        if let Some((base, payload)) = super::class::builtin_data_payload(val) {
                            if base == "set" || base == "frozenset" {
                                print_repr(val);
                            } else {
                                print_value_str(payload);
                            }
                            return;
                        }
                        print_repr(val);
                    }
                }
                _ => {
                    print_repr(val);
                }
            }
        }
    }
}

/// Extract the element list from a raw set/frozenset payload object —
/// shared by the set/frozenset-subclass repr prefix fallback (#1051) in
/// both `print_repr` and `mb_repr`.
fn set_family_payload_items(payload: MbValue) -> Option<Vec<MbValue>> {
    payload.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Set(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::FrozenSet(items) => Some(items.clone()),
            _ => None,
        }
    })
}

fn print_repr(val: MbValue) {
    if let Some(name) = pep695_display_name(val) {
        mb_out!("{name}");
        return;
    }
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        mb_out!("<function {name} at 0x{addr:x}>");
        return;
    }
    if let Some(i) = val.as_int() {
        mb_out!("{i}");
    } else if let Some(f) = val.as_float() {
        mb_out!("{}", super::string_ops::python_float_repr(f));
    } else if let Some(b) = val.as_bool() {
        mb_out!("{}", if b { "True" } else { "False" });
    } else if val.is_none() {
        mb_out!("None");
    } else if val.is_not_implemented() {
        mb_out!("NotImplemented");
    } else if val.is_ellipsis() {
        mb_out!("Ellipsis");
    } else if let Some(s) = super::stdlib::enum_class::member_repr(val) {
        // Class-body enum member inside a container: "<Color.RED: 1>".
        mb_out!("{s}");
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(codepoints) = super::string_ops::surrogate_codepoints(val) {
            mb_out!(
                "{}",
                super::string_ops::repr_string_from_codepoints(&codepoints)
            );
            return;
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    mb_out!("{}", super::string_ops::repr_string(s));
                }
                ObjData::List(ref lock) => {
                    // Cycle guard (#1061): a self-referential list
                    // (`l.append(l)`) recurses into `print_repr` for its
                    // own element without this — CPython's Py_ReprEnter
                    // detects the re-entry and prints `[...]` instead.
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_out!("[...]");
                    } else {
                        let items = lock.read().unwrap();
                        mb_out!("[");
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_repr(*item);
                        }
                        mb_out!("]");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Tuple(items) => {
                    // Cycle guard — see the List arm above (#1061). Tuples
                    // are immutable but can still cycle through a mutable
                    // element that holds a reference back to the tuple.
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_out!("(...)");
                    } else {
                        mb_out!("(");
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_repr(*item);
                        }
                        if items.len() == 1 {
                            mb_out!(",");
                        }
                        mb_out!(")");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Dict(ref lock) => {
                    // Cycle guard — see the List arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_out!("{{...}}");
                    } else {
                        let map = lock.read().unwrap();
                        mb_out!("{{");
                        for (i, (k, v)) in map.iter().enumerate() {
                            if i > 0 {
                                mb_out!(", ");
                            }
                            print_dict_key(k);
                            mb_out!(": ");
                            print_repr(*v);
                        }
                        mb_out!("}}");
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Set(ref lock) => {
                    // Cycle guard — see the List arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_out!("set(...)");
                    } else {
                        let items = lock.read().unwrap();
                        if items.is_empty() {
                            mb_out!("set()");
                        } else {
                            mb_out!("{{");
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 {
                                    mb_out!(", ");
                                }
                                print_repr(*item);
                            }
                            mb_out!("}}");
                        }
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::FrozenSet(items) => {
                    // Cycle guard — see the Set arm above (#1061).
                    let self_ptr = ptr as usize;
                    if !super::repr_guard::enter(self_ptr) {
                        mb_out!("frozenset(...)");
                    } else {
                        if items.is_empty() {
                            mb_out!("frozenset()");
                        } else {
                            mb_out!("frozenset({{");
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 {
                                    mb_out!(", ");
                                }
                                print_repr(*item);
                            }
                            mb_out!("}})");
                        }
                        super::repr_guard::leave(self_ptr);
                    }
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                } => {
                    if matches!(class_name.as_str(), "GenericAlias" | "types.GenericAlias") {
                        if let Some(s) = generic_alias_repr_string(class_name, fields) {
                            mb_out!("{s}");
                            return;
                        }
                    }
                    // In container context Python calls repr() on each element;
                    // check for a user __repr__ before the built-in fallbacks.
                    let repr_method = super::class::lookup_method(class_name, "__repr__");
                    if !repr_method.is_none() {
                        let result = super::class::mb_call_method1(repr_method, val);
                        if let Some(rp) = result.as_ptr() {
                            if let ObjData::Str(ref s) = (*rp).data {
                                mb_out!("{s}");
                                return;
                            }
                        }
                    }
                    // Type objects in a container repr as `<class 'name'>`,
                    // matching mb_repr / the print path (e.g. `[int, str]`).
                    if class_name == "type" {
                        let name = fields
                            .read()
                            .ok()
                            .and_then(|f| f.get("__name__").copied())
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            });
                        if let Some(name) = name {
                            mb_out!("<class '{name}'>");
                            return;
                        }
                    }
                    // namedtuple instances render as Point(x=1, y=2) in
                    // container context — class_name is the dynamic tuple
                    // name, so dispatch by marker field. (#1648)
                    if let Some(s) = super::stdlib::collections_mod::namedtuple_repr(val) {
                        mb_out!("{s}");
                        return;
                    }
                    // Stdlib types with structured repr — same set wired in
                    // `mb_repr`; missing here breaks `[Counter(...)]` and other
                    // container element renders. (#1646)
                    if class_name == "UnionType" {
                        mb_out!("{}", union_type_repr(val));
                        return;
                    }
                    if class_name == "collections.Counter" {
                        mb_out!("{}", super::stdlib::collections_mod::counter_repr(val));
                        return;
                    }
                    if class_name == "collections.defaultdict" {
                        mb_out!("{}", super::stdlib::collections_mod::defaultdict_repr(val));
                        return;
                    }
                    if class_name == "collections.deque" {
                        mb_out!("{}", super::stdlib::collections_mod::deque_repr(val));
                        return;
                    }
                    if class_name == "collections.OrderedDict" {
                        mb_out!("{}", super::stdlib::collections_mod::ordereddict_repr(val));
                        return;
                    }
                    if class_name == "re.Match" {
                        mb_out!("{}", super::stdlib::re_mod::match_repr(val));
                        return;
                    }
                    if class_name == "re.Pattern" {
                        mb_out!("{}", super::stdlib::re_mod::pattern_repr(val));
                        return;
                    }
                    if class_name == "datetime.datetime" {
                        mb_out!("{}", super::stdlib::datetime_mod::datetime_repr(val));
                        return;
                    }
                    if class_name == "datetime.time" {
                        mb_out!("{}", super::stdlib::datetime_mod::time_repr(val));
                    } else if class_name == "datetime.timezone" {
                        mb_out!("{}", super::stdlib::datetime_mod::timezone_repr(val));
                    } else if class_name == "datetime.timedelta" {
                        mb_out!("{}", super::stdlib::datetime_mod::timedelta_repr(val));
                        return;
                    }
                    let f = fields.read().unwrap();
                    if class_name == "type" {
                        // Type objects: <class 'name'>
                        let type_name = f
                            .get("__name__")
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "unknown".to_string());
                        mb_out!("<class '{type_name}'>");
                    } else if class_name == "slice" {
                        let s = f.get("start").copied().unwrap_or(MbValue::none());
                        let e = f.get("stop").copied().unwrap_or(MbValue::none());
                        let st = f.get("step").copied().unwrap_or(MbValue::none());
                        drop(f);
                        mb_out!("slice(");
                        print_repr(s);
                        mb_out!(", ");
                        print_repr(e);
                        mb_out!(", ");
                        print_repr(st);
                        mb_out!(")");
                    } else if class_name == "memoryview" {
                        drop(f);
                        mb_out!("<memory at 0x{:x}>", ptr as usize);
                    } else if let Some(msg_val) = f.get("message") {
                        // Exception repr: ValueError('message')
                        let msg = msg_val.as_ptr().and_then(|p| {
                            if let ObjData::Str(ref s) = (*p).data {
                                Some(s.clone())
                            } else {
                                None
                            }
                        });
                        if let Some(m) = msg {
                            mb_out!("{class_name}('{m}')");
                        } else {
                            mb_out!("{class_name}()");
                        }
                    } else {
                        drop(f);
                        // Builtin-subclass instance without a __repr__
                        // override reprs as its payload's repr, matching
                        // mb_repr's fallback (#1032). set/frozenset
                        // subclasses additionally prefix the class name
                        // onto the literal body — `ClassName({...})` /
                        // `ClassName()` when empty — matching CPython's
                        // set.__repr__ non-exact-type special case
                        // (dict/list/tuple subclasses stay unprefixed). (#1051)
                        if let Some((base, payload)) =
                            super::class::builtin_data_payload_if_unoverridden(val, "__repr__")
                        {
                            let set_items = if base == "set" || base == "frozenset" {
                                set_family_payload_items(payload)
                            } else {
                                None
                            };
                            if let Some(items) = set_items {
                                mb_out!("{class_name}(");
                                if !items.is_empty() {
                                    mb_out!("{{");
                                    for (i, item) in items.iter().enumerate() {
                                        if i > 0 {
                                            mb_out!(", ");
                                        }
                                        print_repr(*item);
                                    }
                                    mb_out!("}}");
                                }
                                mb_out!(")");
                            } else {
                                print_repr(payload);
                            }
                        } else {
                            mb_out!("{class_name}()");
                        }
                    }
                }
                ObjData::Bytes(data) => {
                    mb_out!("b{}", format_bytes_inner(data));
                }
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    mb_out!("bytearray(b{})", format_bytes_inner(&data));
                }
                ObjData::Complex(re, im) => {
                    mb_out!("{}", super::string_ops::complex_repr_string(*re, *im));
                }
                ObjData::BigInt(big) => mb_out!("{big}"),
                _ => mb_out!("..."),
            }
        }
    }
}

/// Print a dict key: integers without quotes, strings with quotes.
fn print_dict_key(k: &super::dict_ops::DictKey) {
    mb_out!("{}", super::dict_ops::dict_key_display(k));
}

/// Strip PEP 515 digit-separator underscores from `s` after validating
/// that placement is legal: no leading/trailing underscore, no
/// consecutive underscores. When `allow_leading` is true (used after
/// stripping a radix prefix like `0x`), an underscore may immediately
/// follow the prefix (`0x_FF` → caller passes `_FF`). Returns `None`
/// when placement is invalid.
fn strip_pep515_underscores(s: &str, allow_leading: bool) -> Option<String> {
    if s.is_empty() {
        return None;
    }
    if !allow_leading && s.starts_with('_') {
        return None;
    }
    if s.ends_with('_') {
        return None;
    }
    if s.contains("__") {
        return None;
    }
    Some(s.replace('_', ""))
}

/// Strip PEP 515 underscores from a float literal string. Underscores
/// may appear between digits in the integer part, fractional part, or
/// exponent — but never adjacent to `.`/`e`/`E`/sign characters, and
/// never leading/trailing in any run. Returns `None` if any rule is
/// violated.
fn strip_float_underscores(s: &str) -> Option<String> {
    // Forbidden adjacencies: `_` next to `.`, `e`/`E`, or sign.
    let bytes = s.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'_' {
            let prev = if i == 0 { None } else { Some(bytes[i - 1]) };
            let next = bytes.get(i + 1).copied();
            let forbidden = |c: Option<u8>| {
                matches!(
                    c,
                    None | Some(b'.')
                        | Some(b'e')
                        | Some(b'E')
                        | Some(b'+')
                        | Some(b'-')
                        | Some(b'_')
                )
            };
            if forbidden(prev) || forbidden(next) {
                return None;
            }
        }
    }
    Some(s.replace('_', ""))
}

/// Plain Python type name of a value — for CPython-exact error messages.
/// Instances report their short class name (`pkg.Cls` → `Cls`).
pub(crate) fn value_type_name(val: MbValue) -> String {
    if val.is_bool() {
        return "bool".to_string();
    }
    if val.is_none() {
        return "NoneType".to_string();
    }
    if val.is_not_implemented() {
        return "NotImplementedType".to_string();
    }
    if val.is_ellipsis() {
        return "ellipsis".to_string();
    }
    if let Some(iter_type) = super::iter::mb_iter_type_name(val) {
        return iter_type.to_string();
    }
    if val.is_int() {
        return "int".to_string();
    }
    if val.is_float() {
        return "float".to_string();
    }
    if val.as_func().is_some() {
        return "function".to_string();
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Instance { class_name, .. } if class_name == "__instance_dict_proxy__" => {
                    "dict"
                }
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                ObjData::CodeObject { .. } => "code",
                ObjData::Instance { class_name, .. } => {
                    return class_name
                        .rsplit('.')
                        .next()
                        .unwrap_or(class_name)
                        .to_string();
                }
            }
            .to_string();
        }
    }
    "object".to_string()
}

/// Raise a TypeError with the given message through the runtime exception
/// machinery so it is catchable from user code.
pub(crate) fn raise_type_error(msg: String) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

fn type_error_value(msg: impl Into<String>) -> MbValue {
    raise_type_error(msg.into());
    MbValue::none()
}

/// Raise a ValueError with the given message through the runtime exception
/// machinery so it is catchable from user code.
pub(crate) fn raise_value_error(msg: String) {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg)),
    );
}

fn int_enum_like_value(val: MbValue) -> Option<MbValue> {
    super::stdlib::enum_class::int_member_value(val)
        .or_else(|| super::stdlib::signal_mod::signal_enum_int_value(val))
        .or_else(|| super::stdlib::http_mod::http_status_member_value(val))
}

fn int_subclass_payload_for_dunder(val: MbValue, dunder: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*p).data
        {
            if !super::class::check_class_hierarchy(class_name, "int")
                || super::class::class_defines_own_method(class_name, dunder)
            {
                return None;
            }
            fields
                .read()
                .unwrap()
                .get(super::class::INT_SUBCLASS_VALUE_FIELD)
                .copied()
        } else {
            None
        }
    })
}

fn float_subclass_payload_for_dunder(val: MbValue, dunder: &str) -> Option<MbValue> {
    val.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*p).data
        {
            if !super::class::check_class_hierarchy(class_name, "float")
                || super::class::class_defines_own_method(class_name, dunder)
            {
                return None;
            }
            fields
                .read()
                .unwrap()
                .get(super::class::FLOAT_SUBCLASS_VALUE_FIELD)
                .copied()
        } else {
            None
        }
    })
}

/// CPython's `PyLong_Check` — used to validate `__int__`/`__index__`
/// dunder-loop results before returning them: plain int, bool, BigInt, and
/// int-subclass instances all pass; anything else (float, str, ...) is a
/// `TypeError` per CPython's return-type contract for these dunders. (#1063)
fn value_is_long_check(v: MbValue) -> bool {
    if v.is_int() || v.is_bool() {
        return true;
    }
    if unsafe { super::bigint_ops::extract_bigint(v) }.is_some() {
        return true;
    }
    v.as_ptr().is_some_and(|p| unsafe {
        matches!(&(*p).data, ObjData::Instance { class_name, .. }
            if super::class::check_class_hierarchy(class_name, "int"))
    })
}

/// CPython's `PyFloat_Check` — used to validate `__float__` dunder-loop
/// results: plain float or a float-subclass instance pass; anything else
/// (int, bool, str, ...) is a `TypeError`. (#1063)
fn value_is_float_check(v: MbValue) -> bool {
    if v.is_float() {
        return true;
    }
    v.as_ptr().is_some_and(|p| unsafe {
        matches!(&(*p).data, ObjData::Instance { class_name, .. }
            if super::class::check_class_hierarchy(class_name, "float"))
    })
}

fn int_subclass_numeric_operands(
    a: MbValue,
    b: MbValue,
    dunder: &str,
) -> Option<(MbValue, MbValue)> {
    let av = int_subclass_payload_for_dunder(a, dunder);
    let bv = int_subclass_payload_for_dunder(b, dunder);
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

fn str_subclass_ordering_operands(
    a: MbValue,
    b: MbValue,
    dunder: &str,
) -> Option<(MbValue, MbValue)> {
    let av = super::class::builtin_data_payload_if_unoverridden(a, dunder)
        .filter(|(base, _)| *base == "str")
        .map(|(_, payload)| payload);
    let bv = super::class::builtin_data_payload_if_unoverridden(b, dunder)
        .filter(|(base, _)| *base == "str")
        .map(|(_, payload)| payload);
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

fn numeric_subclass_operands(a: MbValue, b: MbValue, dunder: &str) -> Option<(MbValue, MbValue)> {
    let av = int_subclass_payload_for_dunder(a, dunder)
        .or_else(|| float_subclass_payload_for_dunder(a, dunder));
    let bv = int_subclass_payload_for_dunder(b, dunder)
        .or_else(|| float_subclass_payload_for_dunder(b, dunder));
    if av.is_some() || bv.is_some() {
        Some((av.unwrap_or(a), bv.unwrap_or(b)))
    } else {
        None
    }
}

/// Unary counterpart of `numeric_subclass_operands`: unwrap a single
/// int/float-SUBCLASS instance (`class P(int): pass` / `class F(float):
/// pass`) to its raw payload for the unary `+`/`-` fallbacks (`mb_neg`,
/// `mb_dispatch_unaryop`'s pos/neg arms). `None` when `a` isn't an
/// unoverridden numeric-derived-class instance (plain int/float, a
/// non-numeric class, or a subclass that defines its own override for
/// `dunder` — the caller's override check/invocation keeps priority). (#1030)
pub(crate) fn numeric_subclass_unary_operand(a: MbValue, dunder: &str) -> Option<MbValue> {
    int_subclass_payload_for_dunder(a, dunder)
        .or_else(|| float_subclass_payload_for_dunder(a, dunder))
}

/// Int-only unary counterpart for `~` (`__invert__`) — float has no
/// `__invert__`, so only an int-derived-class instance unwraps here. (#1030)
pub(crate) fn int_subclass_unary_operand(a: MbValue, dunder: &str) -> Option<MbValue> {
    int_subclass_payload_for_dunder(a, dunder)
}

/// int(value) — convert to integer.
pub fn mb_int(val: MbValue) -> MbValue {
    let val = int_enum_like_value(val).unwrap_or(val);
    if is_decimal_handle_value(val) {
        return super::stdlib::decimal_mod::mb_decimal_int(val);
    }
    if is_fraction_handle_value(val) {
        return super::stdlib::fractions_mod::mb_fraction_int(val);
    }
    if val.is_int() {
        val
    } else if let Some(f) = val.as_float() {
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
        super::bigint_ops::int_from_f64_trunc(f)
    } else if let Some(b) = val.as_bool() {
        MbValue::from_int(b as i64)
    } else if let Some(ptr) = val.as_ptr() {
        // int("42"), int("  -7  ") — parse string to integer; raise ValueError on bad input.
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                let trimmed = s.trim();
                let try_parse = |t: &str| -> Option<MbValue> {
                    if let Ok(i) = t.parse::<i64>() {
                        return Some(super::bigint_ops::int_from_i64(i));
                    }
                    // PEP 515: digit-separator underscores. Strip optional
                    // sign, validate underscore placement on the remaining
                    // digits, then parse.
                    let (negative, digits) = match t.as_bytes().first() {
                        Some(b'-') => (true, &t[1..]),
                        Some(b'+') => (false, &t[1..]),
                        _ => (false, t),
                    };
                    let stripped = strip_pep515_underscores(digits, false)?;
                    if let Ok(mag) = stripped.parse::<i64>() {
                        return Some(super::bigint_ops::int_from_i64(if negative {
                            -mag
                        } else {
                            mag
                        }));
                    }
                    // Beyond i64 — arbitrary precision.
                    if !stripped.is_empty() && stripped.bytes().all(|b| b.is_ascii_digit()) {
                        let mut big: num_bigint::BigInt = stripped.parse().ok()?;
                        if negative {
                            big = -big;
                        }
                        return Some(super::bigint_ops::normalize_bigint(big));
                    }
                    None
                };
                if let Some(v) = try_parse(trimmed) {
                    return v;
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "invalid literal for int() with base 10: '{s}'"
                    ))),
                );
                return MbValue::none();
            }
            // int(b"42") / int(bytearray(b"42")) — bytes-like parses like str
            // (CPython accepts ASCII digit bytes).
            let bytes_body: Option<Vec<u8>> = match (*ptr).data {
                ObjData::Bytes(ref b) => Some(b.clone()),
                ObjData::ByteArray(ref lock) => Some(lock.read().unwrap().clone()),
                _ => None,
            };
            if let Some(body) = bytes_body {
                let text = String::from_utf8_lossy(&body).to_string();
                if let Ok(i) = text.trim().parse::<i64>() {
                    return super::bigint_ops::int_from_i64(i);
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "invalid literal for int() with base 10: b'{text}'"
                    ))),
                );
                return MbValue::none();
            }
            // int(instance) — dispatch the __int__ dunder when the class
            // registers one (ipaddress addresses, user numeric types);
            // fall back to __index__ (CPython int() accepts SupportsIndex).
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                // int-SUBCLASS argument (`int(N(7))` where `class N(int)`) —
                // unwrap to the raw int payload only when the subclass does
                // NOT override `__int__` itself; otherwise fall through to
                // the dunder-dispatch loop below so the override wins
                // (matches the float/str-subclass arms just below). (#1059)
                if let Some((base, payload)) =
                    super::class::builtin_data_payload_if_unoverridden(val, "__int__")
                {
                    if base == "int" {
                        super::rc::retain_if_ptr(payload);
                        return payload;
                    }
                }
                // float-SUBCLASS argument (`int(F(2.5))` where `class F(float)`)
                // — unwrap to its raw float payload and reuse the float-conversion
                // path above (NaN/inf/trunc handling) via recursion, unless the
                // subclass overrides `__int__` (the dunder loop below then wins).
                // Uses the #1030 unwrap-helper family. (#1042)
                if let Some(payload) = float_subclass_payload_for_dunder(val, "__int__") {
                    return mb_int(payload);
                }
                // str-SUBCLASS argument (`int(S("42"))` where `class S(str)`) —
                // unwrap to its raw str payload and reuse the string-parsing
                // path above via recursion. (#1042)
                if let Some((base, payload)) =
                    super::class::builtin_data_payload_if_unoverridden(val, "__int__")
                {
                    if base == "str" {
                        return mb_int(payload);
                    }
                }
                for dunder in ["__int__", "__index__"] {
                    let method = super::class::lookup_method(class_name, dunder);
                    if !method.is_none() {
                        let name = MbValue::from_ptr(MbObject::new_str(dunder.to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                        let result = super::class::mb_call_method(val, name, args);
                        if super::exception::current_exception_type().is_some() {
                            return MbValue::none();
                        }
                        // CPython's int()/index() dunder-return contract: the
                        // result must itself be an int (plain, bool, or an
                        // int-subclass instance) — anything else is a
                        // TypeError, not a silent pass-through. (#1063)
                        if !value_is_long_check(result) {
                            raise_type_error(format!(
                                "{dunder} returned non-int (type {})",
                                value_type_name(result)
                            ));
                            return MbValue::none();
                        }
                        return result;
                    }
                }
            }
            // BigInt is already an int — identity (retain for the caller so
            // input and output can be released independently).
            if matches!((*ptr).data, ObjData::BigInt(_)) {
                super::rc::retain_if_ptr(val);
                return val;
            }
        }
        raise_type_error(format!(
            "int() argument must be a string, a bytes-like object or a real number, not '{}'",
            value_type_name(val)
        ));
        MbValue::none()
    } else {
        raise_type_error(format!(
            "int() argument must be a string, a bytes-like object or a real number, not '{}'",
            value_type_name(val)
        ));
        MbValue::none()
    }
}

/// float(value) — convert to float.
/// Parse the textual form of a float as CPython's `float(str)` / `float(bytes)`
/// does: surrounding whitespace ignored, case-insensitive inf/infinity/nan,
/// PEP 515 underscores. Returns None when the text is not a valid float literal.
fn parse_pyfloat_text(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    if lower == "inf" || lower == "infinity" {
        return Some(f64::INFINITY);
    }
    if lower == "-inf" || lower == "-infinity" {
        return Some(f64::NEG_INFINITY);
    }
    if lower == "nan" || lower == "-nan" {
        return Some(f64::NAN);
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        return Some(f);
    }
    // PEP 515: `1_000.5`, `2_500e-3`, etc. Validate underscore placement, then
    // strip and re-parse.
    if let Some(without) = strip_float_underscores(trimmed) {
        if let Ok(f) = without.parse::<f64>() {
            return Some(f);
        }
    }
    None
}

pub fn mb_float(val: MbValue) -> MbValue {
    if is_decimal_handle_value(val) {
        return super::stdlib::decimal_mod::mb_decimal_float(val);
    }
    if is_fraction_handle_value(val) {
        return super::stdlib::fractions_mod::mb_fraction_float(val);
    }
    if val.is_float() {
        val
    } else if let Some(i) = val.as_int() {
        MbValue::from_float(i as f64)
    } else if let Some(b) = val.as_bool() {
        MbValue::from_float(if b { 1.0 } else { 0.0 })
    } else if let Some(ptr) = val.as_ptr() {
        // float("3.14") — parse string to float; raise ValueError on bad input.
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                if let Some(f) = parse_pyfloat_text(s) {
                    return MbValue::from_float(f);
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "could not convert string to float: '{s}'"
                    ))),
                );
                return MbValue::none();
            }
            // float(b"2.3") / float(bytearray(...)) / float(memoryview(...))
            // — CPython parses bytes-like ASCII the same as a string.
            let bytes_text = try_bytes_like(val);
            if let Some(raw) = bytes_text {
                let text = String::from_utf8_lossy(&raw);
                if let Some(f) = parse_pyfloat_text(&text) {
                    return MbValue::from_float(f);
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "could not convert string to float: {text:?}"
                    ))),
                );
                return MbValue::none();
            }
            // float(instance) — dispatch the __float__ dunder when present;
            // fall back to __index__ (CPython float() accepts SupportsIndex).
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                // float-SUBCLASS argument (`float(Sub(2.5))` where
                // `class Sub(float)`) — unwrap to its raw float payload, unless
                // the subclass overrides `__float__` (the dunder loop below
                // then wins). mb_float previously had no same-type shortcut at
                // all (unlike mb_int's int-hierarchy one), so this
                // unconditionally raised TypeError. (#1042)
                if let Some(payload) = float_subclass_payload_for_dunder(val, "__float__") {
                    return mb_float(payload);
                }
                // int-SUBCLASS argument (`float(P(7))` where `class P(int)`) —
                // unwrap to its raw int payload and reuse the int→float path
                // above via recursion. (#1042)
                if let Some(payload) = int_subclass_payload_for_dunder(val, "__float__") {
                    return mb_float(payload);
                }
                // str-SUBCLASS argument (`float(S("3.14"))` where
                // `class S(str)`) — unwrap to its raw str payload and reuse the
                // string-parsing path above via recursion. (#1042)
                if let Some((base, payload)) =
                    super::class::builtin_data_payload_if_unoverridden(val, "__float__")
                {
                    if base == "str" {
                        return mb_float(payload);
                    }
                }
                for dunder in ["__float__", "__index__"] {
                    let method = super::class::lookup_method(class_name, dunder);
                    if !method.is_none() {
                        let name = MbValue::from_ptr(MbObject::new_str(dunder.to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                        let result = super::class::mb_call_method(val, name, args);
                        if super::exception::current_exception_type().is_some() {
                            return MbValue::none();
                        }
                        // CPython's float()/index() dunder-return contract:
                        // __index__ must yield an int (then widened to
                        // float), __float__ must yield a float (plain or a
                        // float-subclass instance) — anything else is a
                        // TypeError, not a silent pass-through. (#1063)
                        if dunder == "__index__" {
                            if let Some(i) = result.as_int_pyint() {
                                return MbValue::from_float(i as f64);
                            }
                            if let Some(big) = unsafe { super::bigint_ops::extract_bigint(result) }
                            {
                                use num_traits::ToPrimitive;
                                return MbValue::from_float(big.to_f64().unwrap_or(f64::INFINITY));
                            }
                            if !value_is_long_check(result) {
                                raise_type_error(format!(
                                    "__index__ returned non-int (type {})",
                                    value_type_name(result)
                                ));
                                return MbValue::none();
                            }
                        } else if !value_is_float_check(result) {
                            raise_type_error(format!(
                                "{class_name}.__float__ returned non-float (type {})",
                                value_type_name(result)
                            ));
                            return MbValue::none();
                        }
                        return result;
                    }
                }
            }
            // BigInt → float (may overflow to inf, matching f64 semantics).
            if let Some(big) = super::bigint_ops::extract_bigint(val) {
                use num_traits::ToPrimitive;
                return MbValue::from_float(big.to_f64().unwrap_or(f64::INFINITY));
            }
        }
        raise_type_error(format!(
            "float() argument must be a string or a real number, not '{}'",
            value_type_name(val)
        ));
        MbValue::none()
    } else {
        raise_type_error(format!(
            "float() argument must be a string or a real number, not '{}'",
            value_type_name(val)
        ));
        MbValue::none()
    }
}

/// bool(value) — truthiness check.
pub fn mb_bool(val: MbValue) -> MbValue {
    if let Some(target) = super::stdlib::weakref_mod::proxy_target_or_raise(val) {
        if target.is_none() {
            return MbValue::none();
        }
        return mb_bool(target);
    }
    if is_decimal_handle_value(val) {
        return super::stdlib::decimal_mod::mb_decimal_bool(val);
    }
    if is_fraction_handle_value(val) {
        return super::stdlib::fractions_mod::mb_fraction_bool(val);
    }
    let truthy = if val.is_none() {
        false
    } else if let Some(i) = val.as_int() {
        // Iterator handles share TAG_INT. For a `range` iter handle the
        // truth value reflects remaining length (matches CPython's
        // `bool(range(5, 5)) == False`); other iterator kinds are objects,
        // truthy by identity.
        if super::iter::is_iter_handle(val) {
            match super::iter::mb_iter_range_is_nonempty(val) {
                Some(nonempty) => nonempty,
                None => true,
            }
        } else {
            i != 0
        }
    } else if let Some(f) = val.as_float() {
        f != 0.0
    } else if let Some(b) = val.as_bool() {
        b
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(n) = super::string_ops::surrogate_len(val) {
            return MbValue::from_bool(n != 0);
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => !s.is_empty(),
                ObjData::List(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::Dict(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::Tuple(items) => !items.is_empty(),
                ObjData::Set(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::FrozenSet(items) => !items.is_empty(),
                ObjData::Bytes(b) => !b.is_empty(),
                ObjData::ByteArray(ref lock) => !lock.read().unwrap().is_empty(),
                ObjData::BigInt(b) => {
                    use num_traits::Zero;
                    !b.is_zero()
                }
                ObjData::Complex(re, im) => *re != 0.0 || *im != 0.0,
                ObjData::Instance { class_name, .. } => {
                    // __bool__ dunder dispatch
                    let bool_method = super::class::lookup_method(class_name, "__bool__");
                    if !bool_method.is_none() {
                        let result = super::class::mb_call_method1(bool_method, val);
                        if super::exception::mb_has_exception().as_bool() == Some(true) {
                            return MbValue::from_bool(false);
                        }
                        if let Some(bv) = result.as_bool() {
                            return MbValue::from_bool(bv);
                        }
                        if let Some(iv) = result.as_int() {
                            return MbValue::from_bool(iv != 0);
                        }
                        raise_type_error(format!(
                            "__bool__ should return bool, returned {}",
                            value_type_name(result)
                        ));
                        return MbValue::from_bool(false);
                    } else if super::class::class_bool_is_blocked(class_name) {
                        // `__bool__ = None` disables truth-testing; calling the
                        // None slot raises, even when __len__ exists.
                        raise_type_error("'NoneType' object is not callable".to_string());
                        return MbValue::from_bool(false);
                    }
                    // __len__ fallback (validated like len(), so bool() and
                    // len() surface the same error for an illegal __len__).
                    let len_method = super::class::lookup_method(class_name, "__len__");
                    if !len_method.is_none() {
                        let result = super::class::mb_call_method1(len_method, val);
                        let checked = validate_len_result(result);
                        if let Some(iv) = checked.as_int() {
                            return MbValue::from_bool(iv != 0);
                        }
                        if checked.is_bool() {
                            return MbValue::from_bool(checked.as_bool() == Some(true));
                        }
                        if let Some(p) = checked.as_ptr() {
                            if let ObjData::BigInt(ref b) = (*p).data {
                                use num_traits::Zero;
                                return MbValue::from_bool(!b.is_zero());
                            }
                        }
                        // validate_len_result raised: fall through with a
                        // pending exception (the value below is discarded).
                    } else if let Some((_base, payload)) =
                        super::class::builtin_data_payload_if_unoverridden(val, "__len__")
                    {
                        return mb_bool(payload);
                    } else if let Some((_kind, payload)) =
                        super::stdlib::collections_mod::user_wrapper_data(val)
                    {
                        return mb_bool(payload);
                    } else if class_name == "collections.deque" {
                        // deque has no lookup_method-registered __len__ (its
                        // len() is hardcoded in mb_len below), so without this
                        // arm an empty deque fell through to the "true"
                        // default instead of CPython's length-based
                        // truthiness (#868 shlex investigation).
                        if let Some(n) = mb_len(val).as_int() {
                            return MbValue::from_bool(n != 0);
                        }
                    }
                    true
                }
                _ => true,
            }
        }
    } else if val.is_ellipsis() || val.is_not_implemented() {
        // Both singletons are truthy in CPython.
        true
    } else {
        false
    };
    MbValue::from_bool(truthy)
}

/// str(value) — convert to string object.
pub fn mb_str(val: MbValue) -> MbValue {
    // TAG_FUNC user-defined functions render as `<function NAME at 0xADDR>`
    // to match CPython. Closure handles share TAG_INT with low-value ints
    // (closure IDs start at 1), so we restrict detection to TAG_FUNC only
    // to avoid corrupting integer rendering.
    if let Some(addr) = val.as_func().filter(|a| *a > 4096) {
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        return MbValue::from_ptr(MbObject::new_str(format!(
            "<function {name} at 0x{addr:x}>"
        )));
    }
    if let Some(name) = pep695_display_name(val) {
        return MbValue::from_ptr(MbObject::new_str(name));
    }
    // UserDict / UserList / UserString stringify through their payload
    // (str(UserString("hi")) == "hi", str(UserList([1])) == "[1]").
    if let Some((_, data)) = super::stdlib::collections_mod::user_wrapper_data(val) {
        return mb_str(data);
    }
    if let Some(text) = super::stdlib::xml_mod::qname_text_value(val) {
        return MbValue::from_ptr(MbObject::new_str(text));
    }
    let s = if let Some(i) = val.as_int() {
        // UUID handles are int-tagged but render as the canonical
        // 8-4-4-4-12 form (#1475 — keep `print(uuid.uuid4())` honest
        // instead of leaking the i64 handle ID).
        if super::stdlib::uuid_mod::is_uuid_handle(i as u64) {
            return super::stdlib::uuid_mod::mb_uuid_str(val);
        }
        // Decimal / Fraction handles render their numeric value (#2129).
        if super::stdlib::decimal_mod::is_decimal_handle(i as u64) {
            return super::stdlib::decimal_mod::mb_decimal_str(val);
        }
        if super::stdlib::fractions_mod::is_fraction_handle(i as u64) {
            return super::stdlib::fractions_mod::mb_fraction_str(val);
        }
        format!("{i}")
    } else if let Some(f) = val.as_float() {
        super::string_ops::python_float_repr(f)
    } else if let Some(b) = val.as_bool() {
        (if b { "True" } else { "False" }).to_string()
    } else if val.is_none() {
        "None".to_string()
    } else if val.is_not_implemented() {
        "NotImplemented".to_string()
    } else if val.is_ellipsis() {
        "Ellipsis".to_string()
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(codepoints) = super::string_ops::surrogate_codepoints(val) {
            return super::string_ops::new_surrogate_codepoints_str(codepoints);
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    // str(x) must return a new object (owned reference) so
                    // the JIT can safely release input and output independently.
                    return MbValue::from_ptr(MbObject::new_str(s.clone()));
                }
                _ => super::string_ops::value_to_string(val),
            }
        }
    } else {
        String::new()
    };
    let obj = MbObject::new_str(s);
    MbValue::from_ptr(obj)
}

fn pep695_display_name(val: MbValue) -> Option<String> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance {
                class_name,
                ref fields,
            } if matches!(
                class_name.as_str(),
                "TypeVar" | "TypeVarTuple" | "ParamSpec" | "TypeAliasType"
            ) =>
            {
                fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__name__").copied())
                    .and_then(|v| v.as_ptr())
                    .and_then(|name_ptr| {
                        if let ObjData::Str(ref s) = (*name_ptr).data {
                            Some(s.clone())
                        } else {
                            None
                        }
                    })
            }
            _ => None,
        }
    }
}

fn is_str_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
}

fn is_bytes_or_bytearray_value(v: MbValue) -> bool {
    v.as_ptr().is_some_and(|ptr| unsafe {
        matches!((*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_))
    })
}

/// str(object, encoding, errors) constructor form for bytes-like decoding.
pub fn mb_str_construct(object: MbValue, encoding: MbValue, errors: MbValue) -> MbValue {
    let has_encoding = !encoding.is_none();
    let has_errors = !errors.is_none();
    if !has_encoding && !has_errors {
        return mb_str(object);
    }
    if !is_bytes_or_bytearray_value(object) {
        return type_error_value(format!(
            "decoding to str: need a bytes-like object, {} found",
            value_type_name(object)
        ));
    }
    if !has_encoding {
        return type_error_value("str() missing required argument 'encoding' (pos 2)");
    }
    if !is_str_value(encoding) {
        return type_error_value(format!(
            "str() argument 'encoding' must be str, not {}",
            value_type_name(encoding)
        ));
    }
    if has_errors && !is_str_value(errors) {
        return type_error_value(format!(
            "str() argument 'errors' must be str, not {}",
            value_type_name(errors)
        ));
    }
    super::bytes_ops::mb_bytes_decode_with(object, encoding, errors)
}

/// abs(value) — absolute value.
pub fn mb_abs(val: MbValue) -> MbValue {
    if is_decimal_handle_value(val) {
        return super::stdlib::decimal_mod::mb_decimal_abs(val);
    }
    if is_fraction_handle_value(val) {
        return super::stdlib::fractions_mod::mb_fraction_abs(val);
    }
    if let Some(i) = val.as_int() {
        MbValue::from_int(i.abs())
    } else if let Some(f) = val.as_float() {
        MbValue::from_float(f.abs())
    } else if let Some(b) = val.as_bool() {
        MbValue::from_int(if b { 1 } else { 0 })
    } else if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Complex(re, im) => {
                    // abs(complex) = hypot(re, im); a finite input overflowing
                    // to inf raises OverflowError (CPython c_abs).
                    let m = re.hypot(*im);
                    if m.is_infinite() && re.is_finite() && im.is_finite() {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("OverflowError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(
                                "absolute value too large".to_string(),
                            )),
                        );
                        return MbValue::none();
                    }
                    return MbValue::from_float(m);
                }
                ObjData::BigInt(big) => {
                    use num_traits::Signed;
                    return super::bigint_ops::bigint_from_big(big.abs());
                }
                ObjData::Instance { class_name, .. } => {
                    // abs(timedelta) — exact microsecond magnitude.
                    if class_name == "datetime.timedelta" {
                        if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(val) {
                            return super::stdlib::datetime_mod::timedelta_from_us(us.abs());
                        }
                    }
                    let abs_method = super::class::lookup_method(class_name, "__abs__");
                    if !abs_method.is_none() {
                        let method_name =
                            MbValue::from_ptr(MbObject::new_str("__abs__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![]));
                        return super::class::mb_call_method(val, method_name, args);
                    }
                }
                _ => {}
            }
        }
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "bad operand type for abs(): '{}'",
                add_operand_type_name(val),
            ))),
        );
        MbValue::none()
    } else {
        // Preserve the legacy None fallback used by existing runtime tests.
        MbValue::from_int(0)
    }
}

/// Try to extract `(real, imag)` from any numeric `MbValue` — int, float,
/// bool, or `ObjData::Complex`. Returns `None` when `val` is non-numeric.
/// Used by complex-aware arithmetic helpers to coerce mixed operands.
/// (#1256 — complex arithmetic gap)
fn as_complex_pair(val: MbValue) -> Option<(f64, f64)> {
    // Decimal / Fraction are tagged-int handles: as_int() below would read the
    // handle id as a bogus real component (`Decimal("3.0") == 3+0j` compared the
    // handle id against 3.0 → False). They are not directly complex-coercible
    // here; (Decimal/Fraction vs complex) equality flows through the
    // numeric-handle path in mb_values_eq instead.
    if is_decimal_handle_value(val) || is_fraction_handle_value(val) {
        return None;
    }
    if let Some(i) = val.as_int() {
        return Some((i as f64, 0.0));
    }
    if let Some(f) = val.as_float() {
        return Some((f, 0.0));
    }
    if let Some(b) = val.as_bool() {
        return Some((b as i64 as f64, 0.0));
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Complex(re, im) = (*ptr).data {
                return Some((re, im));
            }
        }
    }
    if let Some(("complex", payload)) = super::class::builtin_data_payload(val) {
        return as_complex_pair(payload);
    }
    None
}

/// True when `val` is an `ObjData::Complex` object — distinct from a real
/// number coercible to complex. Used to gate the complex-arithmetic
/// promotion in mb_add/sub/mul/div. (#1256)
fn is_complex_obj(val: MbValue) -> bool {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if matches!((*ptr).data, ObjData::Complex(_, _)) {
                return true;
            }
        }
    }
    matches!(
        super::class::builtin_data_payload(val),
        Some(("complex", _))
    )
}

/// True iff `val` is a number complex comparison can be defined against
/// (int / float / bool / complex / arbitrary-precision int). Anything else
/// makes `complex.__eq__`/`__ne__` return NotImplemented, per CPython.
fn is_complex_cmp_operand(val: MbValue) -> bool {
    if val.is_int() || val.is_float() || val.as_bool().is_some() {
        return true;
    }
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if matches!((*ptr).data, ObjData::Complex(_, _) | ObjData::BigInt(_)) {
                return true;
            }
        }
    }
    matches!(
        super::class::builtin_data_payload(val),
        Some(("complex", _))
    )
}

/// Compute an unbound complex comparison dunder `complex.<method>(a, b)`.
/// Returns `Some(result)` for the six rich-comparison dunders, `None` for any
/// other method name. __eq__/__ne__ yield a bool when `b` is numeric and
/// NotImplemented otherwise; the ordering dunders are always NotImplemented
/// (complex has no ordering). Shared by the unbound-method-wrapper call path
/// and the direct `complex.__eq__(a, b)` method-call path (mb_call_method).
pub(crate) fn complex_cmp_dunder(method: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    match method {
        "__eq__" | "__ne__" => Some(if !is_complex_cmp_operand(b) {
            MbValue::not_implemented()
        } else if method == "__eq__" {
            mb_eq(a, b)
        } else {
            mb_ne(a, b)
        }),
        "__lt__" | "__le__" | "__gt__" | "__ge__" => Some(MbValue::not_implemented()),
        _ => None,
    }
}

/// Parse a CPython-style complex literal from a string body.
/// Accepts forms like `1+2j`, `3-4j`, `5j`, `1.5e-3+2.5e+2j`, `+j`, `j`, `1`.
/// Surrounding whitespace and a single layer of outer `(...)` are tolerated.
/// Returns `(real, imag)` on success, `None` if the string is not a valid
/// complex literal.
/// Parse one real/imag coefficient: validate+strip PEP 515 underscores
/// (`1_000.5`), then preserve the sign of a parsed NaN (`complex("-nan")`
/// → negative-signed NaN; `f64::from_str` does not reliably set the NaN
/// sign bit).
fn parse_complex_part(part: &str) -> Option<f64> {
    let t = part.trim();
    let v = strip_float_underscores(t)?.parse::<f64>().ok()?;
    if v.is_nan() && t.starts_with('-') {
        Some(v.copysign(-1.0))
    } else {
        Some(v)
    }
}

fn parse_complex_str(input: &str) -> Option<(f64, f64)> {
    let mut s = input.trim();
    if s.starts_with('(') && s.ends_with(')') {
        s = s[1..s.len() - 1].trim();
    }
    if s.is_empty() {
        return None;
    }
    // No 'j'/'J' → real-only number.
    let has_imag = s.ends_with('j') || s.ends_with('J');
    if !has_imag {
        return parse_complex_part(s).map(|r| (r, 0.0));
    }
    // Strip the trailing 'j'/'J'.
    let body = &s[..s.len() - 1];
    if body.is_empty() {
        // Bare "j"  → 1j.
        return Some((0.0, 1.0));
    }
    // Find the rightmost '+' or '-' that does NOT follow an 'e'/'E'
    // (exponent sign). Skip a leading sign at position 0.
    let bytes = body.as_bytes();
    let mut split: Option<usize> = None;
    for i in (1..bytes.len()).rev() {
        let c = bytes[i];
        if c == b'+' || c == b'-' {
            let prev = bytes[i - 1];
            if prev != b'e' && prev != b'E' {
                split = Some(i);
                break;
            }
        }
    }
    match split {
        Some(idx) => {
            let real_part = body[..idx].trim();
            let imag_part = body[idx..].trim();
            let re = parse_complex_part(real_part)?;
            // Bare "+"/"-" prefix on imag means ±1j.
            let im = if imag_part == "+" {
                1.0
            } else if imag_part == "-" {
                -1.0
            } else {
                parse_complex_part(imag_part)?
            };
            Some((re, im))
        }
        None => {
            // Whole body is the imag coefficient.
            let im = if body == "+" {
                1.0
            } else if body == "-" {
                -1.0
            } else {
                parse_complex_part(body)?
            };
            Some((0.0, im))
        }
    }
}

/// complex(real, imag) — create a complex number (R3 CPython 3.12 conformance).
/// Accepts numeric `real`/`imag`, an existing `Complex` for `real`, or a
/// string literal for `real` (CPython single-argument form).
pub fn mb_complex(real: MbValue, imag: MbValue) -> MbValue {
    // complex(P(7)) / complex(F(2.5)) — unwrap int/float-SUBCLASS instances
    // to their raw numeric payload up front so the real/imag extraction below
    // (which only recognizes plain int/float/bool) sees the underlying value
    // instead of silently defaulting to 0.0. Uses the #1030 unwrap-helper
    // family; user `__complex__` overrides aren't dispatched at all yet (a
    // separate, out-of-scope dunder-protocol gap), so this only affects
    // unoverridden numeric subclasses. (#1042)
    let real = numeric_subclass_unary_operand(real, "__complex__").unwrap_or(real);
    let imag = numeric_subclass_unary_operand(imag, "__complex__").unwrap_or(imag);
    // String form: `complex("1+2j")`. CPython rejects passing a second
    // argument with a string; we silently ignore `imag` when `real` is
    // a string for now (close enough for #1256 long-tail coverage).
    // Also: complex passthrough `complex(complex(1,2))` should equal arg.
    if let Some(ptr) = real.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                if let Some((re, im)) = parse_complex_str(s) {
                    return MbValue::from_ptr(MbObject::new_complex(re, im));
                }
                // CPython: an unparseable string raises ValueError, not a silent None.
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "could not convert string to complex: '{s}'"
                    ))),
                );
                return MbValue::none();
            }
            if let ObjData::Complex(re, im) = (*ptr).data {
                if imag.is_none() {
                    return MbValue::from_ptr(MbObject::new_complex(re, im));
                }
            }
        }
    }
    let real_parts = real.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Complex(re, im) = (*ptr).data {
            Some((re, im))
        } else {
            None
        }
    });
    let imag_parts = imag.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Complex(re, im) = (*ptr).data {
            Some((re, im))
        } else {
            None
        }
    });
    let (re0, im0) = if let Some((re, im)) = real_parts {
        (re, im)
    } else if let Some(f) = real.as_float() {
        (f, 0.0)
    } else if let Some(i) = real.as_int() {
        (i as f64, 0.0)
    } else if let Some(b) = real.as_bool() {
        (b as i64 as f64, 0.0)
    } else {
        (0.0, 0.0)
    };
    let (re1, im1) = if imag.is_none() {
        (0.0, 0.0)
    } else if let Some((re, im)) = imag_parts {
        (re, im)
    } else if let Some(f) = imag.as_float() {
        (f, 0.0)
    } else if let Some(i) = imag.as_int() {
        (i as f64, 0.0)
    } else if let Some(b) = imag.as_bool() {
        (b as i64 as f64, 0.0)
    } else {
        (0.0, 0.0)
    };
    MbValue::from_ptr(MbObject::new_complex(re0 - im1, im0 + re1))
}

fn mb_str_value(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn mb_first_index_value(val: MbValue) -> Option<i64> {
    let ptr = val.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Tuple(items) => items.first().and_then(|v| resolve_index_value(*v)),
            ObjData::List(lock) => lock
                .read()
                .unwrap()
                .first()
                .and_then(|v| resolve_index_value(*v)),
            _ => None,
        }
    }
}

/// Arithmetic helpers used by compiled code.
/// Best-effort Python type name of an operand, for `+` TypeError messages.
fn add_operand_type_name(v: MbValue) -> &'static str {
    if v.is_int() {
        return "int";
    }
    if v.is_float() {
        return "float";
    }
    if v.is_bool() {
        return "bool";
    }
    if v.is_none() {
        return "NoneType";
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            return match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Dict(_) => "dict",
                ObjData::Tuple(_) => "tuple",
                ObjData::Set(_) => "set",
                ObjData::FrozenSet(_) => "frozenset",
                ObjData::Bytes(_) => "bytes",
                ObjData::ByteArray(_) => "bytearray",
                ObjData::BigInt(_) => "int",
                ObjData::Complex(_, _) => "complex",
                _ => "object",
            };
        }
    }
    "object"
}

fn is_array_handle_value(v: MbValue) -> bool {
    v.as_int()
        .is_some_and(|id| super::stdlib::array_mod::is_array_handle(id as u64))
}

/// #2129 carve-out: `decimal.Decimal` and `fractions.Fraction` values are
/// integer HANDLES (NaN-boxed ints ≥ 2^40), so the dynamic binary-op
/// entry points must intercept them before their int fast paths —
/// otherwise `Decimal('0.1') + Decimal('0.2')` adds raw handle ids
/// (aborting on the 48-bit `from_int` range) and `==` compares ids.
/// The range guard inside each module's `is_*_handle` keeps primitive
/// int hot paths to a single compare before any table probe.
pub(crate) fn is_decimal_handle_value(v: MbValue) -> bool {
    v.as_int()
        .is_some_and(|id| super::stdlib::decimal_mod::is_decimal_handle(id as u64))
}

pub(crate) fn is_fraction_handle_value(v: MbValue) -> bool {
    v.as_int()
        .is_some_and(|id| super::stdlib::fractions_mod::is_fraction_handle(id as u64))
}

/// Approximate f64 readback of a Fraction handle (CPython coerces the
/// Fraction to float when the other operand is a float).
fn fraction_as_f64(v: MbValue) -> Option<f64> {
    let id = v.as_int()? as u64;
    let (n, d) = super::stdlib::fractions_mod::handle_num_den(id)?;
    Some(n as f64 / d as f64)
}

/// Route a binary arithmetic op through the Decimal/Fraction handle
/// protocol when either operand is such a handle. Returns `None` when
/// neither side is a numeric handle (caller falls through to its
/// regular paths).
/// True when `v` is a heap-allocated arbitrary-precision integer (BigInt) —
/// an int value too large for the 48-bit inline NaN-box range (e.g.
/// `sys.maxsize`, `2**70`).
#[inline]
pub(crate) fn is_bigint_value(v: MbValue) -> bool {
    v.as_ptr().map_or(false, |p| {
        matches!(unsafe { &(*p).data }, ObjData::BigInt(_))
    })
}

/// Raise `ZeroDivisionError(msg)` and return `None` (the value the arithmetic
/// builtins yield after raising).
pub(crate) fn raise_zero_div(msg: &str) -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(msg.to_string())),
    );
    MbValue::none()
}

/// Route a numeric binary op to arbitrary-precision arithmetic when either
/// operand is a heap BigInt. The inline arms in `mb_add`/`mb_sub`/… use
/// `as_int()`, which returns `None` for a heap BigInt, so a BigInt operand
/// would otherwise skip every numeric arm and fall through to a spurious
/// `None` (e.g. `sys.maxsize - 1`) or `unsupported operand` TypeError
/// (`sys.maxsize + 1`).
///
/// Returns `None` when neither operand is a BigInt (the inline hot path is
/// untouched) or an operand is non-numeric (so the caller's type-specific arms
/// — str/list/set/datetime/… — keep running). For `//`/`%` by zero it raises
/// ZeroDivisionError directly, matching the inline integer arms' messages.
fn bigint_numeric_binop(op: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    if !(is_bigint_value(a) || is_bigint_value(b)) {
        return None;
    }
    let num_like = |v: MbValue| v.is_int() || v.is_bool() || v.is_float() || is_bigint_value(v);
    if !(num_like(a) && num_like(b)) {
        return None;
    }
    // BigInt ⊕ float → float (CPython widens the integer operand to f64,
    // possibly to ±inf for very large magnitudes).
    if a.is_float() || b.is_float() {
        let as_f = |v: MbValue| -> f64 {
            v.as_float()
                .or_else(|| unsafe { super::bigint_ops::int_as_f64(v) })
                .unwrap_or(f64::NAN)
        };
        let (af, bf) = (as_f(a), as_f(b));
        return Some(match op {
            "+" => MbValue::from_float(af + bf),
            "-" => MbValue::from_float(af - bf),
            "*" => MbValue::from_float(af * bf),
            "**" => MbValue::from_float(af.powf(bf)),
            "//" => {
                if bf == 0.0 {
                    return Some(raise_zero_div("float floor division by zero"));
                }
                MbValue::from_float((af / bf).floor())
            }
            "%" => {
                if bf == 0.0 {
                    return Some(raise_zero_div("float modulo"));
                }
                let r = af % bf;
                MbValue::from_float(if r != 0.0 && r.signum() != bf.signum() {
                    r + bf
                } else {
                    r
                })
            }
            _ => return None,
        });
    }
    // Pure integer (inline / bool / BigInt) arithmetic.
    Some(unsafe {
        match op {
            "+" => super::bigint_ops::mb_int_add(a, b),
            "-" => super::bigint_ops::mb_int_sub(a, b),
            "*" => super::bigint_ops::mb_int_mul(a, b),
            "//" => super::bigint_ops::mb_int_floordiv(a, b)
                .unwrap_or_else(|| raise_zero_div("integer division or modulo by zero")),
            "%" => super::bigint_ops::mb_int_mod(a, b)
                .unwrap_or_else(|| raise_zero_div("integer modulo by zero")),
            "**" => match super::bigint_ops::mb_int_pow(a, b) {
                Some(r) => r,
                None => {
                    // Negative (or astronomically large) exponent → float.
                    let bf = super::bigint_ops::int_as_f64(a).unwrap_or(f64::NAN);
                    let ef = super::bigint_ops::int_as_f64(b).unwrap_or(f64::NAN);
                    MbValue::from_float(bf.powf(ef))
                }
            },
            _ => return None,
        }
    })
}

/// #2129/#961 guard: also called directly from `bigint_ops::mb_bigint_{add,sub,mul}`
/// (the `CheckedAdd`/`CheckedSub`/`CheckedMul` MIR lowering for statically
/// `Ty::Int` operands) so that `a + b` on a `fractions.Fraction` / `decimal.Decimal`
/// handle dispatches through the numeric-handle protocol instead of the raw
/// BigInt-aware int path — see the guard note on `mb_bigint_add`.
pub(crate) fn numeric_handle_binop(op: &str, a: MbValue, b: MbValue) -> Option<MbValue> {
    use super::stdlib::{decimal_mod, fractions_mod};
    if is_decimal_handle_value(a) || is_decimal_handle_value(b) {
        return Some(match op {
            "+" => decimal_mod::mb_decimal_add(a, b),
            "-" => decimal_mod::mb_decimal_sub(a, b),
            "*" => decimal_mod::mb_decimal_mul(a, b),
            "/" => decimal_mod::mb_decimal_truediv(a, b),
            "//" => decimal_mod::mb_decimal_floordiv(a, b),
            "%" => decimal_mod::mb_decimal_rem(a, b),
            "**" => decimal_mod::mb_decimal_pow(a, b),
            "divmod" => decimal_mod::mb_decimal_divmod(a, b),
            _ => return None,
        });
    }
    let a_frac = is_fraction_handle_value(a);
    let b_frac = is_fraction_handle_value(b);
    if !(a_frac || b_frac) {
        return None;
    }
    // Fraction ⊕ float → float (CPython converts the Fraction).
    if a.is_float() || b.is_float() {
        let to_f = |v: MbValue, frac: bool| {
            if frac {
                fraction_as_f64(v)
            } else {
                v.as_float()
            }
        };
        if let (Some(af), Some(bf)) = (to_f(a, a_frac), to_f(b, b_frac)) {
            return Some(match op {
                "+" => MbValue::from_float(af + bf),
                "-" => MbValue::from_float(af - bf),
                "*" => MbValue::from_float(af * bf),
                "/" => MbValue::from_float(af / bf),
                "//" => MbValue::from_float((af / bf).floor()),
                "%" => {
                    let r = af % bf;
                    MbValue::from_float(if r != 0.0 && r.signum() != bf.signum() {
                        r + bf
                    } else {
                        r
                    })
                }
                "**" => MbValue::from_float(af.powf(bf)),
                _ => return None,
            });
        }
    }
    Some(match op {
        "+" => fractions_mod::mb_fraction_add(a, b),
        "-" => fractions_mod::mb_fraction_sub(a, b),
        "*" => fractions_mod::mb_fraction_mul(a, b),
        "/" => fractions_mod::mb_fraction_truediv(a, b),
        "//" => fractions_mod::mb_fraction_floordiv(a, b),
        "%" => fractions_mod::mb_fraction_mod(a, b),
        "**" => fractions_mod::mb_fraction_pow(a, b),
        "divmod" => fractions_mod::mb_fraction_divmod(a, b),
        _ => return None,
    })
}

pub fn mb_add(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__add__") {
        return mb_add(na, nb);
    }
    if is_array_handle_value(a) || is_array_handle_value(b) {
        return super::stdlib::array_mod::mb_array_concat(a, b);
    }
    if let Some(r) = numeric_handle_binop("+", a, b) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("+", a, b) {
        return r;
    }
    // UserList / UserString concatenation: unwrap to the backing payloads,
    // add, then re-wrap with the LEFT operand's wrapper class (CPython:
    // `UserString + str` stays a UserString; `ul + [6]` stays a UserList).
    {
        let aw = super::stdlib::collections_mod::user_wrapper_data(a);
        let bw = super::stdlib::collections_mod::user_wrapper_data(b);
        if aw.is_some() || bw.is_some() {
            let av = aw.map(|(_, d)| d).unwrap_or(a);
            let bv = bw.map(|(_, d)| d).unwrap_or(b);
            let raw = mb_add(av, bv);
            if aw.is_some() {
                return super::stdlib::collections_mod::user_wrapper_rewrap_like(a, raw);
            }
            return super::stdlib::collections_mod::user_wrapper_rewrap_like(b, raw);
        }
    }

    // bool is an int subclass in Python (True + 1.0 == 2.0), so coerce bool→int
    // via as_int_pyint; plain as_int returns None for a NaN-boxed bool, which made
    // `True + 1.0` fall through to None.
    match (a.as_int_pyint(), b.as_int_pyint()) {
        (Some(ai), Some(bi)) => MbValue::from_int(ai.wrapping_add(bi)),
        _ => {
            let af = a.as_int_pyint().map(|i| i as f64).or(a.as_float());
            let bf = b.as_int_pyint().map(|i| i as f64).or(b.as_float());
            match (af, bf) {
                (Some(af), Some(bf)) => MbValue::from_float(af + bf),
                _ => {
                    // Str + non-str where the right operand is an inline value
                    // (for example int/bool) does not enter the pointer-pair
                    // concat block below, but CPython still gives the
                    // sequence-specific TypeError instead of the generic
                    // binary-operator message.
                    if let Some(pa) = a.as_ptr() {
                        unsafe {
                            if matches!(&(*pa).data, ObjData::Str(_)) {
                                let b_is_str = b
                                    .as_ptr()
                                    .map_or(false, |p| matches!(&(*p).data, ObjData::Str(_)));
                                let b_is_instance = b.as_ptr().map_or(false, |p| {
                                    matches!(&(*p).data, ObjData::Instance { .. })
                                });
                                if !b_is_str && !b_is_instance {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(format!(
                                            "can only concatenate str (not \"{}\") to str",
                                            add_operand_type_name(b)
                                        ))),
                                    );
                                    return MbValue::none();
                                }
                            }
                        }
                    }
                    // List + List → concatenation (also covers Instance-wrapped
                    // list-SUBCLASS operands via list_like_operand; CPython's
                    // binary `+` always drops the subclass here — unlike
                    // `+=`, which mutates in place instead and DOES preserve
                    // it, see mb_inplace in class.rs — #1026).
                    if let (Some(la_v), Some(lb_v)) = (list_like_operand(a), list_like_operand(b)) {
                        if let (Some(pla), Some(plb)) = (la_v.as_ptr(), lb_v.as_ptr()) {
                            unsafe {
                                if let (ObjData::List(ref la), ObjData::List(ref lb)) =
                                    (&(*pla).data, &(*plb).data)
                                {
                                    let mut result = la.read().unwrap().clone();
                                    result.extend_from_slice(&lb.read().unwrap());
                                    return MbValue::from_ptr(MbObject::new_list_inline(result));
                                }
                            }
                        }
                    }
                    // Tuple + Tuple → concatenation (also Instance-wrapped
                    // tuple-SUBCLASS operands; CPython drops the subclass
                    // here too, #1026).
                    if let (Some(ta_v), Some(tb_v)) = (tuple_like_operand(a), tuple_like_operand(b))
                    {
                        if let (Some(pta), Some(ptb)) = (ta_v.as_ptr(), tb_v.as_ptr()) {
                            unsafe {
                                if let (ObjData::Tuple(ref ta), ObjData::Tuple(ref tb)) =
                                    (&(*pta).data, &(*ptb).data)
                                {
                                    let mut result = ta.clone();
                                    result.extend_from_slice(tb);
                                    return MbValue::from_ptr(MbObject::new_tuple(result));
                                }
                            }
                        }
                    }
                    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
                        unsafe {
                            // Str + Str → concatenation (fallback for Any-typed strings)
                            if let (ObjData::Str(ref sa), ObjData::Str(ref sb)) =
                                (&(*pa).data, &(*pb).data)
                            {
                                let result = format!("{}{}", sa, sb);
                                return MbValue::from_ptr(MbObject::new_str(result));
                            }
                            // bytes/bytearray + bytes/bytearray → concatenation.
                            // Result type follows the LEFT operand (CPython):
                            // bytes+bytearray → bytes, bytearray+bytes → bytearray.
                            let a_byteslike =
                                matches!(&(*pa).data, ObjData::Bytes(_) | ObjData::ByteArray(_));
                            if a_byteslike {
                                let bytes_of = |p: *mut MbObject| -> Option<Vec<u8>> {
                                    match &(*p).data {
                                        ObjData::Bytes(d) => Some(d.clone()),
                                        ObjData::ByteArray(lk) => Some(lk.read().unwrap().clone()),
                                        _ => None,
                                    }
                                };
                                if let (Some(va), Some(vb)) = (bytes_of(pa), bytes_of(pb)) {
                                    let mut result = va;
                                    result.extend_from_slice(&vb);
                                    return if matches!(&(*pa).data, ObjData::ByteArray(_)) {
                                        MbValue::from_ptr(MbObject::new_bytearray(result))
                                    } else {
                                        MbValue::from_ptr(MbObject::new_bytes(result))
                                    };
                                }
                            }
                            // datetime.datetime + datetime.timedelta → shifted datetime
                            if let (
                                ObjData::Instance { class_name: ca, .. },
                                ObjData::Instance { class_name: cb, .. },
                            ) = (&(*pa).data, &(*pb).data)
                            {
                                if ca == "datetime.datetime" && cb == "datetime.timedelta" {
                                    return super::stdlib::datetime_mod::mb_datetime_add_timedelta(
                                        a, b,
                                    );
                                }
                                if ca == "datetime.timedelta" && cb == "datetime.datetime" {
                                    return super::stdlib::datetime_mod::mb_datetime_add_timedelta(
                                        b, a,
                                    );
                                }
                                if ca == "datetime.timedelta" && cb == "datetime.timedelta" {
                                    if let (Some(ua), Some(ub)) = (
                                        super::stdlib::datetime_mod::timedelta_total_us(a),
                                        super::stdlib::datetime_mod::timedelta_total_us(b),
                                    ) {
                                        return super::stdlib::datetime_mod::timedelta_from_us(
                                            ua + ub,
                                        );
                                    }
                                }
                                // Counter + Counter — CPython multiset add. (#1636)
                                if ca == "collections.Counter" && cb == "collections.Counter" {
                                    return super::stdlib::collections_mod::mb_counter_add(a, b);
                                }
                            }
                            // str/bytes + incompatible operand → TypeError.
                            // Reached only after the same-type concat arms above
                            // declined, so `b` is the wrong type. Skip when `b` is
                            // a user Instance, which may define __radd__ (handled
                            // elsewhere / left as the None fallback). The int/float
                            // fast paths ran earlier, so hot arithmetic never hits
                            // this.
                            let b_is_instance = matches!(&(*pb).data, ObjData::Instance { .. });
                            if !b_is_instance {
                                match &(*pa).data {
                                    ObjData::Str(_) => {
                                        super::exception::mb_raise(
                                            MbValue::from_ptr(MbObject::new_str(
                                                "TypeError".to_string(),
                                            )),
                                            MbValue::from_ptr(MbObject::new_str(format!(
                                                "can only concatenate str (not \"{}\") to str",
                                                add_operand_type_name(b)
                                            ))),
                                        );
                                        return MbValue::none();
                                    }
                                    ObjData::Bytes(_) | ObjData::ByteArray(_) => {
                                        super::exception::mb_raise(
                                            MbValue::from_ptr(MbObject::new_str(
                                                "TypeError".to_string(),
                                            )),
                                            MbValue::from_ptr(MbObject::new_str(format!(
                                                "can't concat {} to bytes",
                                                add_operand_type_name(b)
                                            ))),
                                        );
                                        return MbValue::none();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    // Complex addition — promote int/float to complex when
                    // either operand is `ObjData::Complex`. (#1256)
                    if is_complex_obj(a) || is_complex_obj(b) {
                        if let (Some((ar, ai)), Some((br, bi))) =
                            (as_complex_pair(a), as_complex_pair(b))
                        {
                            return MbValue::from_ptr(MbObject::new_complex(ar + br, ai + bi));
                        }
                        // The non-complex operand didn't coerce. A BigInt too
                        // large for a C double raises OverflowError (CPython:
                        // `1j + 10**1000`); a representable BigInt folds in;
                        // anything else (None, str, …) is a TypeError instead of
                        // a silent None. Reached only as the primitive fallback
                        // after dunder dispatch (`__add__`/`__radd__`) misses.
                        let (cplx, other) = if is_complex_obj(a) { (a, b) } else { (b, a) };
                        if is_bigint_value(other) {
                            match unsafe { super::bigint_ops::int_as_f64(other) } {
                                Some(x) if x.is_finite() => {
                                    let (ar, ai) = as_complex_pair(cplx).unwrap_or((0.0, 0.0));
                                    return MbValue::from_ptr(MbObject::new_complex(ar + x, ai));
                                }
                                _ => {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "OverflowError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "int too large to convert to float".to_string(),
                                        )),
                                    );
                                    return MbValue::none();
                                }
                            }
                        }
                        raise_type_error(format!(
                            "unsupported operand type(s) for +: '{}' and '{}'",
                            value_type_name(a),
                            value_type_name(b)
                        ));
                        return MbValue::none();
                    }
                    // statistics.NormalDist translation / combination.
                    if let Some(r) = super::stdlib::statistics_mod::normaldist_binop("+", a, b) {
                        return r;
                    }
                    if raise_datetime_op_type_error("+", a, b) {
                        return MbValue::none();
                    }
                    // A builtin sequence subclass on the right that is not
                    // compatible with the left sequence type still gets the
                    // sequence-specific TypeError, unless it defines a real
                    // reflected add hook to take over (`tuple + RHS.__radd__`).
                    if let Some(kind) = a.as_ptr().and_then(|p| match unsafe { &(*p).data } {
                        ObjData::List(_) => Some("list"),
                        ObjData::Tuple(_) => Some("tuple"),
                        _ => None,
                    }) {
                        if let Some(pb) = b.as_ptr() {
                            unsafe {
                                if let ObjData::Instance { class_name, .. } = &(*pb).data {
                                    let is_sequence_subclass =
                                        ["list", "tuple"].iter().any(|base| {
                                            class_name == *base
                                                || super::class::check_class_hierarchy(
                                                    class_name, base,
                                                )
                                        });
                                    let is_same_sequence = class_name == kind
                                        || super::class::check_class_hierarchy(class_name, kind);
                                    let has_reflected_add =
                                        !super::class::lookup_method(class_name, "__radd__")
                                            .is_none();
                                    if is_sequence_subclass
                                        && !is_same_sequence
                                        && !has_reflected_add
                                    {
                                        super::exception::mb_raise(
                                            MbValue::from_ptr(MbObject::new_str(
                                                "TypeError".to_string(),
                                            )),
                                            MbValue::from_ptr(MbObject::new_str(format!(
                                                "can only concatenate {kind} (not \"{}\") to {kind}",
                                                value_type_name(b)
                                            ))),
                                        );
                                        return MbValue::none();
                                    }
                                }
                            }
                        }
                    }
                    // Genuinely unsupported operand pair (e.g. `1.0 + "x"`):
                    // CPython raises TypeError. Preserve the None fallback when
                    // an operand is a user Instance (whose __add__/__radd__ is
                    // resolved by the binop dispatcher — None = "not handled")
                    // OR None: mamba's `__file__`/missing-attr values surface as
                    // None and several stdlib paths (os.path.join(None,...),
                    // linecache) lean on `None + str` staying lenient until that
                    // is fixed properly.
                    let a_inst = a.is_none()
                        || a.as_ptr().map_or(false, |p| {
                            matches!(unsafe { &(*p).data }, ObjData::Instance { .. })
                        });
                    let b_inst = b.is_none()
                        || b.as_ptr().map_or(false, |p| {
                            matches!(unsafe { &(*p).data }, ObjData::Instance { .. })
                        });
                    if !a_inst && !b_inst {
                        // A list/tuple on the left with a non-matching right
                        // operand gets CPython's sequence-specific message
                        // ("can only concatenate list (not \"str\") to list")
                        // rather than the generic operand message. list+list /
                        // tuple+tuple already concatenate via __add__, so reaching
                        // here means the right operand is a different type.
                        let seq_kind = a.as_ptr().and_then(|p| match unsafe { &(*p).data } {
                            ObjData::List(_) => Some("list"),
                            ObjData::Tuple(_) => Some("tuple"),
                            _ => None,
                        });
                        if let Some(kind) = seq_kind {
                            super::exception::mb_raise(
                                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                                MbValue::from_ptr(MbObject::new_str(format!(
                                    "can only concatenate {kind} (not \"{}\") to {kind}",
                                    value_type_name(b)
                                ))),
                            );
                            return MbValue::none();
                        }
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "unsupported operand type(s) for +: '{}' and '{}'",
                                value_type_name(a),
                                value_type_name(b)
                            ))),
                        );
                    }
                    MbValue::none()
                }
            }
        }
    }
}

/// Unwrap a set/frozenset-SUBCLASS instance (`class Foo(set): pass`, stored
/// as `ObjData::Instance` carrying a hidden builtin-data payload field) to
/// its raw `ObjData::Set`/`FrozenSet` payload, for the `-`/`|`/`&`/`^`
/// operator dispatch below. Returns the value unchanged when it is already a
/// raw set/frozenset; `None` when it isn't set-like at all. Without this, a
/// set/frozenset-subclass operand was invisible to the `a_is_setlike`/
/// `b_is_setlike` checks in `mb_sub`/`mb_bitand`/`mb_bitor`/`mb_bitxor`
/// (which only matched the raw ObjData kind), so e.g. `plain_set -
/// SetSubclass(...)` raised "unsupported operand type(s)" even though
/// CPython performs the operation — CPython's binary set operators always
/// drop subclass typing, returning a plain set/frozenset keyed off the LEFT
/// operand's raw kind (`build_set_like_left`), so passing the unwrapped
/// payload through is correct here (#975).
fn set_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Set(_) | ObjData::FrozenSet(_)) {
                return Some(value);
            }
        }
    }
    match super::class::builtin_data_payload(value) {
        Some(("set", payload)) | Some(("frozenset", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a list-SUBCLASS instance (`class LS(list): pass`) to its raw
/// `ObjData::List` payload, for the `+`/`*` concatenation/repetition
/// fallbacks in `mb_add`/`mb_mul`. Returns the value unchanged when it is
/// already a raw list; `None` when it isn't list-like at all. Mirrors
/// `set_like_operand` (#975): without this, a list-subclass operand was
/// invisible to the raw `ObjData::List` match (it's `ObjData::Instance`
/// wrapping a hidden payload field), so e.g. `LS([1]) + [2]` produced None
/// instead of CPython's plain `[1, 2]` — CPython's binary list operators
/// always drop subclass typing here (unlike `+=`/`*=`, which mutate the
/// receiver in place and DO preserve it; see `mb_inplace` in class.rs).
/// (#1026)
fn list_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::List(_)) {
                return Some(value);
            }
        }
    }
    match super::class::builtin_data_payload(value) {
        Some(("list", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a tuple-SUBCLASS instance (`class TS(tuple): pass`) to its raw
/// `ObjData::Tuple` payload, for the `+`/`*` fallbacks in `mb_add`/`mb_mul`.
/// Tuple has no in-place mutating dunder at all, so `TS((1,)) += (2,)`
/// reaches this same unwrap via the ordinary `__add__` binop dispatch
/// (`mb_inplace` falls through when `builtin_inplace_update_fn` has no
/// `"tuple"` entry) — CPython drops the subclass there too, returning a
/// plain tuple. See `list_like_operand` above for the general shape. (#1026)
fn tuple_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Tuple(_)) {
                return Some(value);
            }
        }
    }
    match super::class::builtin_data_payload(value) {
        Some(("tuple", payload)) => Some(payload),
        _ => None,
    }
}

/// Unwrap a dict-SUBCLASS instance (`class DS(dict): pass`) to its raw
/// `ObjData::Dict` payload, for the `|` merge fallback in `mb_bitor`. See
/// `list_like_operand` above for the general shape. (#1026)
fn dict_like_operand(value: MbValue) -> Option<MbValue> {
    if let Some(p) = value.as_ptr() {
        unsafe {
            if matches!((*p).data, ObjData::Dict(_)) {
                return Some(value);
            }
        }
    }
    match super::class::builtin_data_payload(value) {
        Some(("dict", payload)) => Some(payload),
        _ => None,
    }
}

pub fn mb_sub(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030 — mirrors mb_add's
    // numeric_subclass_operands; without this `P(int)(7) - 2` raised
    // "unsupported operand type(s)" instead of CPython's plain-int `5`).
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__sub__") {
        return mb_sub(na, nb);
    }
    if let Some(r) = numeric_handle_binop("-", a, b) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("-", a, b) {
        return r;
    }
    if let Some(result) = super::dict_ops::dict_view_sub(a, b) {
        return result;
    }
    // Int fast path first — matches mb_add's ordering. fib_recursive and
    // every other int-arith hot loop runs `n - 1` through here on every
    // iteration; the set-difference dispatch was paying two as_ptr()
    // bit-tests plus an unsafe deref+matches before falling through.
    match (a.as_int_pyint(), b.as_int_pyint()) {
        (Some(ai), Some(bi)) => return MbValue::from_int(ai.wrapping_sub(bi)),
        _ => {}
    }
    // Float / mixed-numeric fallback.
    let af = a.as_int_pyint().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int_pyint().map(|i| i as f64).or(b.as_float());
    if let (Some(af), Some(bf)) = (af, bf) {
        return MbValue::from_float(af - bf);
    }
    // Set / frozenset difference: a - b
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            let a_set = set_like_operand(a);
            let b_set = set_like_operand(b);
            let a_is_setlike = a_set.is_some();
            let b_is_setlike = b_set.is_some();
            if let (Some(av), Some(bv)) = (a_set, b_set) {
                return super::set_ops::mb_set_difference(av, bv);
            }
            // `set - <non-set>` is unsupported (operator needs set operands;
            // `.difference()` accepts any iterable). Skip Instance RHS so the
            // Counter/datetime/WeakSet arms below still run. Mirror mb_bitand.
            {
                let other_is_instance = if a_is_setlike {
                    matches!(&(*pb).data, ObjData::Instance { .. })
                } else {
                    matches!(&(*pa).data, ObjData::Instance { .. })
                };
                if (a_is_setlike || b_is_setlike) && !other_is_instance {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(format!(
                            "unsupported operand type(s) for -: '{}' and '{}'",
                            value_type_name(a),
                            value_type_name(b)
                        ))),
                    );
                    return MbValue::none();
                }
            }
            // Counter - Counter — CPython multiset diff (drops <=0). (#1636)
            if let (
                ObjData::Instance { class_name: ca, .. },
                ObjData::Instance { class_name: cb, .. },
            ) = (&(*pa).data, &(*pb).data)
            {
                if ca == "collections.Counter" && cb == "collections.Counter" {
                    return super::stdlib::collections_mod::mb_counter_sub(a, b);
                }
                // datetime - datetime -> timedelta (microsecond-exact).
                if ca == "datetime.datetime" && cb == "datetime.datetime" {
                    return super::stdlib::datetime_mod::mb_datetime_sub_datetime(a, b);
                }
                // datetime - timedelta -> shifted datetime.
                if ca == "datetime.datetime" && cb == "datetime.timedelta" {
                    if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(b) {
                        let neg = super::stdlib::datetime_mod::timedelta_from_us(-us);
                        return super::stdlib::datetime_mod::mb_datetime_add_timedelta(a, neg);
                    }
                }
                // timedelta - timedelta.
                if ca == "datetime.timedelta" && cb == "datetime.timedelta" {
                    if let (Some(ua), Some(ub)) = (
                        super::stdlib::datetime_mod::timedelta_total_us(a),
                        super::stdlib::datetime_mod::timedelta_total_us(b),
                    ) {
                        return super::stdlib::datetime_mod::timedelta_from_us(ua - ub);
                    }
                }
            }
        }
    }
    // Complex subtraction — promote int/float to complex when either
    // operand is `ObjData::Complex`. (#1256)
    if is_complex_obj(a) || is_complex_obj(b) {
        if let (Some((ar, ai)), Some((br, bi))) = (as_complex_pair(a), as_complex_pair(b)) {
            return MbValue::from_ptr(MbObject::new_complex(ar - br, ai - bi));
        }
    }
    // statistics.NormalDist translation / combination.
    if let Some(r) = super::stdlib::statistics_mod::normaldist_binop("-", a, b) {
        return r;
    }
    if raise_datetime_op_type_error("-", a, b) {
        return MbValue::none();
    }
    if !a.is_none() && !b.is_none() {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "unsupported operand type(s) for -: '{}' and '{}'",
                value_type_name(a),
                value_type_name(b)
            ))),
        );
    }
    MbValue::none()
}

/// Bitwise OR — also handles set union, dict merge, and PEP 604 type unions.
pub fn mb_bitor(a: MbValue, b: MbValue) -> MbValue {
    // Flag member composition: Color.RED | Color.BLUE → cached composite.
    if let Some(r) =
        super::stdlib::enum_class::flag_binop(a, b, super::stdlib::enum_class::FlagOp::Or)
    {
        return r;
    }
    if let Some(result) = super::dict_ops::dict_view_or(a, b) {
        return result;
    }
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            let a_set = set_like_operand(a);
            let b_set = set_like_operand(b);
            let a_is_setlike = a_set.is_some();
            let b_is_setlike = b_set.is_some();
            if let (Some(av), Some(bv)) = (a_set, b_set) {
                return super::set_ops::mb_set_union(av, bv);
            }
            // Also covers Instance-wrapped dict-SUBCLASS operands via
            // dict_like_operand; CPython's binary `|` always drops the
            // subclass here — unlike `|=` on a subclass RECEIVER, which
            // mutates in place instead and DOES preserve it (mb_inplace's
            // builtin_inplace_update_fn in class.rs intercepts that case
            // before it ever reaches here). #1026
            if let (Some(av), Some(bv)) = (dict_like_operand(a), dict_like_operand(b)) {
                return super::dict_ops::mb_dict_or(av, bv);
            }
            // `dict | <non-dict>` reaches here only as the fallback for `|=`
            // (`mb_ior` → `mb_inplace` hands the dict receiver to `mb_bitor`,
            // since `dict` has no Instance `__ior__`). PEP 584's in-place merge
            // is as permissive as `dict.update`: it accepts any iterable of
            // key/value pairs and mutates the receiver in place. Route it to
            // `mb_dict_ior`, which validates and raises TypeError/ValueError to
            // match CPython.
            if matches!((*pa).data, ObjData::Dict(_)) {
                return super::dict_ops::mb_dict_ior(a, b);
            }
            // Counter | Counter — CPython multiset max. (#1636)
            if super::stdlib::collections_mod::is_counter_instance(a)
                && super::stdlib::collections_mod::is_counter_instance(b)
            {
                return super::stdlib::collections_mod::mb_counter_or(a, b);
            }
            // `set | <non-set>` is unsupported — the operator form requires set
            // operands, while `.union()` accepts any iterable (mirror mb_bitand).
            // Skip set-like Instances (weakref.WeakSet __or__/__ror__) and Counter.
            let other_is_instance = if a_is_setlike {
                matches!(&(*pb).data, ObjData::Instance { .. })
            } else {
                matches!(&(*pa).data, ObjData::Instance { .. })
            };
            if (a_is_setlike || b_is_setlike)
                && !other_is_instance
                && !super::stdlib::collections_mod::is_counter_instance(a)
                && !super::stdlib::collections_mod::is_counter_instance(b)
            {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "unsupported operand type(s) for |: '{}' and '{}'",
                        value_type_name(a),
                        value_type_name(b)
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // PEP 604: T1 | T2 on type values produces a UnionType instance.
    // `isinstance` / `issubclass` unwrap its __args__ alongside tuple types.
    match mb_bitor_type_union(a, b) {
        TypeUnionBuild::Value(union) => return union,
        TypeUnionBuild::InvalidOperand => {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "unsupported operand type(s) for |".to_string(),
                )),
            );
            return MbValue::none();
        }
        TypeUnionBuild::NotUnion => {}
    }
    // bool is an int subclass in Python, but CPython's bool.__or__ returns
    // an actual bool when BOTH operands are bool (`True | False is True`);
    // otherwise it returns int. Plain `as_int()` returns None for a
    // NaN-boxed bool, which silently dropped this dynamic-dispatch fallback
    // to None for `(x in y) | (x in z)`-shaped expressions (#1025).
    if let (Some(ab), Some(bb)) = (a.as_bool(), b.as_bool()) {
        return MbValue::from_bool(ab | bb);
    }
    // int-SUBCLASS operand unwrap (#1030) — last resort, after set/dict/Counter/
    // flag/bool special-casing above, so it never disturbs those paths.
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__or__") {
        return mb_bitor(na, nb);
    }
    // BigInt (>=2^47 magnitude) operand — as_int()/as_int_pyint() return None
    // for a heap BigInt, so this must be checked before the inline fallback
    // below or `sys.maxsize | 1`-shaped expressions silently drop to None
    // (#1025 TAG-complete doctrine; mirrors `bigint_numeric_binop`).
    if is_bigint_value(a) || is_bigint_value(b) {
        return unsafe { super::bigint_ops::mb_int_or(a, b) };
    }
    match (a.as_int_pyint(), b.as_int_pyint()) {
        (Some(ai), Some(bi)) => MbValue::from_int(ai | bi),
        _ => MbValue::none(),
    }
}

enum TypeUnionBuild {
    Value(MbValue),
    InvalidOperand,
    NotUnion,
}

/// PEP 604 `T1 | T2` — build a tuple representing the union if both
/// operands look like type values (built-in type-name strings, registered
/// user-class names, or an existing union tuple from a previous `|`).
/// Returns InvalidOperand when exactly one side is type-like, matching
/// CPython's TypeError for `int | 42`.
fn mb_bitor_type_union(a: MbValue, b: MbValue) -> TypeUnionBuild {
    let mut left: Vec<MbValue> = Vec::new();
    let mut right: Vec<MbValue> = Vec::new();
    let left_ok = collect_union_operand(a, &mut left);
    let right_ok = collect_union_operand(b, &mut right);
    match (left_ok, right_ok) {
        (true, true) => {
            left.extend(right);
            let parts = dedupe_union_parts(left);
            if parts.len() == 1 {
                TypeUnionBuild::Value(parts[0])
            } else {
                TypeUnionBuild::Value(make_union_type_value(parts))
            }
        }
        (true, false) | (false, true) => TypeUnionBuild::InvalidOperand,
        (false, false) => TypeUnionBuild::NotUnion,
    }
}

fn collect_union_operand(v: MbValue, out: &mut Vec<MbValue>) -> bool {
    // PEP 604 shorthand: `T | None` means `T | type(None)`. The literal
    // None operand stands in for NoneType in the union.
    if v.is_none() {
        out.push(make_type_object("NoneType"));
        return true;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    if is_type_name(s) {
                        out.push(make_type_object(s));
                        return true;
                    }
                    false
                }
                ObjData::Tuple(items) => {
                    for &item in items.iter() {
                        if !collect_union_operand(item, out) {
                            return false;
                        }
                    }
                    true
                }
                ObjData::Instance { class_name, fields } if class_name == "UnionType" => {
                    let args = fields.read().ok().and_then(|f| f.get("__args__").copied());
                    if let Some(args) = args {
                        return collect_union_operand(args, out);
                    }
                    false
                }
                ObjData::Instance { class_name, fields } if class_name == "type" => {
                    let name = fields
                        .read()
                        .ok()
                        .and_then(|f| f.get("__name__").and_then(|v| type_name_from_value(*v)));
                    match name {
                        Some(name) if is_type_name(&name) => {
                            out.push(v);
                            true
                        }
                        _ => false,
                    }
                }
                // PEP 695: TypeVars and TypeAliasTypes are valid `|` operands
                // (`type R = R | None`, `T | int` bounds) — they join the
                // union as themselves.
                ObjData::Instance { class_name, .. }
                    if super::pep695::is_pep695_class(class_name) =>
                {
                    out.push(v);
                    true
                }
                // PEP 585/604: a generic alias (`list[int]`, `list[T]`) is a
                // valid union operand — `list[T] | int` joins it as a member.
                ObjData::Instance { class_name, .. } if class_name == "typing.Alias" => {
                    out.push(v);
                    true
                }
                _ => false,
            }
        }
    } else {
        false
    }
}

pub(crate) fn make_union_type_value(parts: Vec<MbValue>) -> MbValue {
    // __parameters__: the free TypeVars in the members (computed before `parts`
    // is moved into the __args__ tuple).
    let params = super::stdlib::typing_mod::typevar_params_tuple(&parts);
    let args = MbValue::from_ptr(MbObject::new_tuple(parts));
    let inst = MbObject::new_instance("UnionType".to_string());
    unsafe {
        if let ObjData::Instance { fields, .. } = &(*inst).data {
            let mut f = fields.write().unwrap();
            f.insert("__args__".to_string(), args);
            f.insert("__parameters__".to_string(), params);
        }
    }
    MbValue::from_ptr(inst)
}

fn dedupe_union_parts(parts: Vec<MbValue>) -> Vec<MbValue> {
    let mut out = Vec::new();
    let mut seen = Vec::<String>::new();
    for part in parts {
        let key = type_name_from_value(part).unwrap_or_else(|| format!("{:016x}", part.to_bits()));
        if !seen.iter().any(|s| s == &key) {
            seen.push(key);
            out.push(part);
        }
    }
    out
}

fn type_name_from_value(v: MbValue) -> Option<String> {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => Some(s.clone()),
                ObjData::Instance { class_name, fields } if class_name == "type" => {
                    fields.read().ok().and_then(|f| {
                        f.get("__name__")
                            .and_then(|name| type_name_from_value(*name))
                    })
                }
                _ => None,
            }
        }
    } else {
        None
    }
}

fn union_type_args(v: MbValue) -> Option<Vec<MbValue>> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if class_name == "UnionType" {
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

pub(crate) fn union_type_repr(v: MbValue) -> String {
    let Some(args) = union_type_args(v) else {
        return "UnionType()".to_string();
    };
    args.iter()
        .map(|arg| match type_name_from_value(*arg).as_deref() {
            Some("NoneType") => "None".to_string(),
            Some(name) => name.to_string(),
            None => "...".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

pub(crate) fn is_type_name(s: &str) -> bool {
    matches!(
        s,
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
            | "NoneType"
            | "range"
            | "slice"
            | "super"
    ) || super::class::class_is_registered(s)
}

/// Bitwise AND — also handles set intersection.
pub fn mb_bitand(a: MbValue, b: MbValue) -> MbValue {
    // Flag member composition: Color.RED & composite → cached member.
    if let Some(r) =
        super::stdlib::enum_class::flag_binop(a, b, super::stdlib::enum_class::FlagOp::And)
    {
        return r;
    }
    if let Some(result) = super::dict_ops::dict_view_and(a, b) {
        return result;
    }
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            let a_set = set_like_operand(a);
            let b_set = set_like_operand(b);
            let a_is_setlike = a_set.is_some();
            let b_is_setlike = b_set.is_some();
            if let (Some(av), Some(bv)) = (a_set, b_set) {
                return super::set_ops::mb_set_intersection(av, bv);
            }
            // Counter & Counter — CPython multiset min. (#1636)
            if super::stdlib::collections_mod::is_counter_instance(a)
                && super::stdlib::collections_mod::is_counter_instance(b)
            {
                return super::stdlib::collections_mod::mb_counter_and(a, b);
            }
            // `set & <non-set>` (e.g. `{1,2} & [3]`) is unsupported — only
            // set/frozenset operands intersect (CPython raises TypeError).
            // Skip when the other operand is an Instance — it may be a set-like
            // wrapper with its own __and__/__rand__ (e.g. weakref.WeakSet),
            // which the dunder-dispatch / NotImplemented path handles; only a
            // plainly-incompatible builtin (list/tuple/dict/...) is rejected.
            let other_is_instance = if a_is_setlike {
                matches!(&(*pb).data, ObjData::Instance { .. })
            } else {
                matches!(&(*pa).data, ObjData::Instance { .. })
            };
            if (a_is_setlike || b_is_setlike)
                && !other_is_instance
                && !super::stdlib::collections_mod::is_counter_instance(a)
                && !super::stdlib::collections_mod::is_counter_instance(b)
            {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "unsupported operand type(s) for &: '{}' and '{}'",
                        value_type_name(a),
                        value_type_name(b)
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // bool is an int subclass in Python, but CPython's bool.__and__ returns
    // an actual bool when BOTH operands are bool (`True & True is True`);
    // otherwise it returns int. Plain `as_int()` returns None for a
    // NaN-boxed bool, which silently dropped this dynamic-dispatch fallback
    // to None for `(x in y) & (x in z)`-shaped expressions (#1025).
    if let (Some(ab), Some(bb)) = (a.as_bool(), b.as_bool()) {
        return MbValue::from_bool(ab & bb);
    }
    // int-SUBCLASS operand unwrap (#1030) — last resort, after set/Counter/
    // flag/bool special-casing above, so it never disturbs those paths.
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__and__") {
        return mb_bitand(na, nb);
    }
    // BigInt (>=2^47 magnitude) operand — see the note in `mb_bitor`.
    if is_bigint_value(a) || is_bigint_value(b) {
        return unsafe { super::bigint_ops::mb_int_and(a, b) };
    }
    match (a.as_int_pyint(), b.as_int_pyint()) {
        (Some(ai), Some(bi)) => MbValue::from_int(ai & bi),
        _ => MbValue::none(),
    }
}

/// Bitwise XOR — also handles set symmetric difference.
pub fn mb_bitxor(a: MbValue, b: MbValue) -> MbValue {
    // Flag member composition: Color.RED ^ Color.BLUE → cached composite.
    if let Some(r) =
        super::stdlib::enum_class::flag_binop(a, b, super::stdlib::enum_class::FlagOp::Xor)
    {
        return r;
    }
    if let Some(result) = super::dict_ops::dict_view_xor(a, b) {
        return result;
    }
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            let a_set = set_like_operand(a);
            let b_set = set_like_operand(b);
            let a_is_setlike = a_set.is_some();
            let b_is_setlike = b_set.is_some();
            if let (Some(av), Some(bv)) = (a_set, b_set) {
                return super::set_ops::mb_set_symmetric_difference(av, bv);
            }
            // `set ^ <non-set>` is unsupported (operator needs set operands;
            // `.symmetric_difference()` accepts any iterable). Mirror mb_bitand.
            let other_is_instance = if a_is_setlike {
                matches!(&(*pb).data, ObjData::Instance { .. })
            } else {
                matches!(&(*pa).data, ObjData::Instance { .. })
            };
            if (a_is_setlike || b_is_setlike) && !other_is_instance {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "unsupported operand type(s) for ^: '{}' and '{}'",
                        value_type_name(a),
                        value_type_name(b)
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // bool is an int subclass in Python, but CPython's bool.__xor__ returns
    // an actual bool when BOTH operands are bool (`True ^ True is False`);
    // otherwise it returns int. Plain `as_int()` returns None for a
    // NaN-boxed bool, which silently dropped this dynamic-dispatch fallback
    // to None for `(x in y) ^ (x in z)`-shaped expressions (#1025) — the
    // module-scope symptom was a stray `None` operand feeding the outer
    // `^`, previously misread as a stale-VReg-after-reassignment bug.
    if let (Some(ab), Some(bb)) = (a.as_bool(), b.as_bool()) {
        return MbValue::from_bool(ab ^ bb);
    }
    // int-SUBCLASS operand unwrap (#1030) — last resort, after set/bool
    // special-casing above, so it never disturbs those paths.
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__xor__") {
        return mb_bitxor(na, nb);
    }
    // BigInt (>=2^47 magnitude) operand — see the note in `mb_bitor`.
    if is_bigint_value(a) || is_bigint_value(b) {
        return unsafe { super::bigint_ops::mb_int_xor(a, b) };
    }
    match (a.as_int_pyint(), b.as_int_pyint()) {
        (Some(ai), Some(bi)) => MbValue::from_int(ai ^ bi),
        _ => MbValue::none(),
    }
}

/// Left shift `a << b` for dynamically-typed (Any/boxed) integer operands.
/// The static (Int, Int) path is handled inline by codegen; this is the
/// runtime fallback reached when either operand is `Any` (e.g. a list/tuple
/// element or an `enumerate` index), matching `mb_bitand`/`mb_bitor`. Without
/// it, `x << n` on a boxed int silently produced None (no `mb_lshift` was
/// registered, so `binop_to_runtime` returned None and codegen emitted a raw
/// shift over NaN-boxed bits).
pub fn mb_lshift(a: MbValue, b: MbValue) -> MbValue {
    // int-SUBCLASS operand unwrap (#1030) — no other Instance-special-casing
    // here, so this is safe to check first.
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__lshift__") {
        return mb_lshift(na, nb);
    }
    let bi = match b.as_int() {
        Some(x) if x >= 0 => x,
        _ => return MbValue::none(),
    };
    // Fast path: inline base whose shift result is recoverable in i64
    // (no bits shifted out). `int_from_i64` still promotes to BigInt when the
    // value exceeds the inline range, so `1 << 48` is exact.
    if let Some(ai) = a.as_int() {
        if bi < 63 {
            let shifted = ai.wrapping_shl(bi as u32);
            if (shifted >> bi) == ai {
                return super::bigint_ops::int_from_i64(shifted);
            }
        }
        if ai == 0 {
            return MbValue::from_int(0);
        }
    }
    // General path: arbitrary-precision shift (handles i64 overflow — e.g.
    // `1 << 64` must yield 2**64, not wrap to 1 — and BigInt bases).
    match unsafe { super::bigint_ops::to_bigint(a) } {
        Some(big) => super::bigint_ops::normalize_bigint(big << (bi as u64)),
        None => MbValue::none(),
    }
}

/// Right shift `a >> b` for dynamically-typed (Any/boxed) integer operands.
/// See `mb_lshift`.
pub fn mb_rshift(a: MbValue, b: MbValue) -> MbValue {
    // int-SUBCLASS operand unwrap (#1030) — no other Instance-special-casing
    // here, so this is safe to check first.
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__rshift__") {
        return mb_rshift(na, nb);
    }
    let bi = match b.as_int() {
        Some(x) if x >= 0 => x,
        _ => return MbValue::none(),
    };
    // Fast path: inline base, shift amount within i64's bit width.
    // `wrapping_shr` wraps the shift COUNT modulo 64 in Rust, which is wrong
    // once `bi >= 64` (e.g. `5 >> 64` must be `0`, not `5`) — an inline
    // (small-magnitude) base's arithmetic right shift by >=64 always
    // saturates to 0 (non-negative) or -1 (negative), so handle that
    // directly instead of calling into `wrapping_shr`.
    if let Some(ai) = a.as_int() {
        if bi < 64 {
            return MbValue::from_int(ai.wrapping_shr(bi as u32));
        }
        return MbValue::from_int(if ai < 0 { -1 } else { 0 });
    }
    // General path: BigInt base — arbitrary-precision arithmetic right shift,
    // floor toward -infinity like CPython's `>>` (#1085's
    // `(-9223372036854775808) >> 1` needs this; previously `a.as_int()`
    // returned `None` for the heap BigInt and this fell straight to
    // `MbValue::none()`, whose TAG_NONE bit pattern was then misread as a
    // function object downstream).
    match unsafe { super::bigint_ops::to_bigint(a) } {
        Some(big) => super::bigint_ops::normalize_bigint(big >> (bi as u64)),
        None => MbValue::none(),
    }
}

pub fn mb_mul(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030 — without this `P(int)(7) *
    // 2` silently produced None instead of CPython's plain-int `14`).
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__mul__") {
        return mb_mul(na, nb);
    }
    if let Some(r) = numeric_handle_binop("*", a, b) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("*", a, b) {
        return r;
    }
    // timedelta * int/float (either order) — exact microsecond scaling.
    if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(a) {
        if let Some(k) = b.as_int() {
            return super::stdlib::datetime_mod::timedelta_from_us(us * k as i128);
        }
        if let Some(f) = b.as_float() {
            return super::stdlib::datetime_mod::timedelta_from_us(
                ((us as f64) * f).round_ties_even() as i128,
            );
        }
    }
    if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(b) {
        if let Some(k) = a.as_int() {
            return super::stdlib::datetime_mod::timedelta_from_us(us * k as i128);
        }
        if let Some(f) = a.as_float() {
            return super::stdlib::datetime_mod::timedelta_from_us(
                ((us as f64) * f).round_ties_even() as i128,
            );
        }
    }
    let a_is_array = is_array_handle_value(a);
    let b_is_array = is_array_handle_value(b);
    if a_is_array {
        return super::stdlib::array_mod::mb_array_repeat(a, b);
    }
    if b_is_array {
        return super::stdlib::array_mod::mb_array_repeat(b, a);
    }

    match (a.as_int(), b.as_int()) {
        (Some(_), Some(_)) => unsafe { super::bigint_ops::mb_int_mul(a, b) },
        _ => {
            // List * Int or Int * List → repetition
            let (list_val, n) = if a.as_ptr().is_some() && b.as_int().is_some() {
                (a, b.as_int().unwrap())
            } else if b.as_ptr().is_some() && a.as_int().is_some() {
                (b, a.as_int().unwrap())
            } else {
                (MbValue::none(), 0)
            };
            // Unwrap an Instance-wrapped list/tuple-SUBCLASS operand (`class
            // LS(list): pass`) to its raw payload before the ObjData match
            // below — mirrors list_like_operand/tuple_like_operand's use in
            // mb_add; CPython's binary `*` drops the subclass here just like
            // `+` does. `tuple *=` reaches this same fallback (tuple has no
            // in-place `__imul__`), so it drops the subclass too; `list *=`
            // instead mutates in place via mb_inplace's
            // builtin_inplace_update_fn in class.rs, which intercepts it
            // before it ever reaches here — #1026.
            let list_val = list_like_operand(list_val)
                .or_else(|| tuple_like_operand(list_val))
                .unwrap_or(list_val);
            if let Some(ptr) = list_val.as_ptr() {
                unsafe {
                    match &(*ptr).data {
                        ObjData::List(ref lock) => {
                            let items = lock.read().unwrap();
                            let n = n.max(0) as usize;
                            let mut result = Vec::with_capacity(items.len() * n);
                            for _ in 0..n {
                                result.extend_from_slice(&items);
                            }
                            return MbValue::from_ptr(MbObject::new_list(result));
                        }
                        ObjData::Tuple(ref items) => {
                            let n = n.max(0) as usize;
                            let mut result = Vec::with_capacity(items.len() * n);
                            for _ in 0..n {
                                result.extend_from_slice(items);
                            }
                            return MbValue::from_ptr(MbObject::new_tuple(result));
                        }
                        ObjData::Str(ref s) => {
                            let n = n.max(0) as usize;
                            let result = s.repeat(n);
                            return MbValue::from_ptr(MbObject::new_str(result));
                        }
                        ObjData::Bytes(ref data) => {
                            let n = n.max(0) as usize;
                            let mut result = Vec::with_capacity(data.len() * n);
                            for _ in 0..n {
                                result.extend_from_slice(data);
                            }
                            return MbValue::from_ptr(MbObject::new_bytes(result));
                        }
                        _ => {}
                    }
                }
            }
            // Complex multiplication — promote when either operand is
            // ObjData::Complex. Formula: (a+bi)*(c+di)=(ac-bd)+(ad+bc)i.
            // (#1256)
            if is_complex_obj(a) || is_complex_obj(b) {
                if let (Some((ar, ai)), Some((br, bi))) = (as_complex_pair(a), as_complex_pair(b)) {
                    return MbValue::from_ptr(MbObject::new_complex(
                        ar * br - ai * bi,
                        ar * bi + ai * br,
                    ));
                }
                // complex * non-numeric (e.g. `1j * None`) → TypeError, not a
                // silent None. Reached only as the primitive fallback after
                // dunder dispatch (`__mul__`/`__rmul__`) misses.
                raise_type_error(format!(
                    "unsupported operand type(s) for *: '{}' and '{}'",
                    value_type_name(a),
                    value_type_name(b)
                ));
                return MbValue::none();
            }
            let af = a.as_int().map(|i| i as f64).or(a.as_float());
            let bf = b.as_int().map(|i| i as f64).or(b.as_float());
            match (af, bf) {
                (Some(af), Some(bf)) => MbValue::from_float(af * bf),
                _ => {
                    // statistics.NormalDist scaling by a constant.
                    if let Some(r) = super::stdlib::statistics_mod::normaldist_binop("*", a, b) {
                        return r;
                    }
                    if raise_datetime_op_type_error("*", a, b) {
                        return MbValue::none();
                    }
                    // A sequence (list/tuple/str/bytes) times a non-integer
                    // raises CPython's "can't multiply sequence by non-int of
                    // type 'X'". Valid sequence*int forms are handled above, so
                    // a sequence operand here means the other side isn't an int.
                    let is_seq = |v: MbValue| -> bool {
                        v.as_ptr().map_or(false, |p| {
                            matches!(
                                unsafe { &(*p).data },
                                ObjData::List(_)
                                    | ObjData::Tuple(_)
                                    | ObjData::Str(_)
                                    | ObjData::Bytes(_)
                            )
                        })
                    };
                    let other = if is_seq(a) {
                        Some(b)
                    } else if is_seq(b) {
                        Some(a)
                    } else {
                        None
                    };
                    if let Some(other) = other {
                        super::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                            MbValue::from_ptr(MbObject::new_str(format!(
                                "can't multiply sequence by non-int of type '{}'",
                                value_type_name(other)
                            ))),
                        );
                    }
                    MbValue::none()
                }
            }
        }
    }
}

/// Fallback guard for arithmetic on datetime.* instances: any combination
/// that no dedicated arm accepted is an unsupported-operand TypeError in
/// CPython (e.g. timedelta + 1, datetime + datetime, int // timedelta).
/// Raises and returns true when either operand is a datetime.* instance.
fn raise_datetime_op_type_error(op: &str, a: MbValue, b: MbValue) -> bool {
    fn dt_class(v: MbValue) -> Option<String> {
        let ptr = v.as_ptr()?;
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if class_name.starts_with("datetime.") {
                    return Some(class_name.clone());
                }
            }
        }
        None
    }
    let (ca, cb) = (dt_class(a), dt_class(b));
    if ca.is_none() && cb.is_none() {
        return false;
    }
    let na = ca.unwrap_or_else(|| add_operand_type_name(a).to_string());
    let nb = cb.unwrap_or_else(|| add_operand_type_name(b).to_string());
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "unsupported operand type(s) for {op}: '{na}' and '{nb}'"
        ))),
    );
    true
}

/// Python floor division/modulo on i128 (quotient rounds toward -inf,
/// remainder takes the divisor's sign).
pub(crate) fn floor_divmod_i128(a: i128, b: i128) -> (i128, i128) {
    let q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        (q - 1, r + b)
    } else {
        (q, r)
    }
}

pub fn mb_div(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__truediv__") {
        return mb_div(na, nb);
    }
    if let Some(r) = numeric_handle_binop("/", a, b) {
        return r;
    }
    // timedelta / timedelta -> float ratio; timedelta / number -> scaled timedelta.
    if let Some(ua) = super::stdlib::datetime_mod::timedelta_total_us(a) {
        if let Some(ub) = super::stdlib::datetime_mod::timedelta_total_us(b) {
            if ub == 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            return MbValue::from_float(ua as f64 / ub as f64);
        }
        if let Some(d) = b.as_int().map(|i| i as f64).or_else(|| b.as_float()) {
            if d == 0.0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            let scaled = (ua as f64 / d).round_ties_even() as i128;
            return super::stdlib::datetime_mod::timedelta_from_us(scaled);
        }
    }
    // Complex division — promote when either operand is ObjData::Complex.
    // Formula: (a+bi)/(c+di) = ((ac+bd)+(bc-ad)i)/(c²+d²). (#1256)
    if is_complex_obj(a) || is_complex_obj(b) {
        if let (Some((ar, ai)), Some((br, bi))) = (as_complex_pair(a), as_complex_pair(b)) {
            let denom = br * br + bi * bi;
            if denom == 0.0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("complex division by zero".to_string())),
                );
                return MbValue::none();
            }
            return MbValue::from_ptr(MbObject::new_complex(
                (ar * br + ai * bi) / denom,
                (ai * br - ar * bi) / denom,
            ));
        }
    }
    // Python division always returns float
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    match (af, bf) {
        (Some(af), Some(bf)) if bf != 0.0 => MbValue::from_float(af / bf),
        (Some(_), Some(_)) => {
            // CPython distinguishes float true-division ("float division by
            // zero") from int true-division ("division by zero") by whether
            // either operand is a float.
            let msg = if a.as_float().is_some() || b.as_float().is_some() {
                "float division by zero"
            } else {
                "division by zero"
            };
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str(msg.to_string())),
            );
            MbValue::none()
        }
        _ => {
            // statistics.NormalDist scaling by a constant.
            if let Some(r) = super::stdlib::statistics_mod::normaldist_binop("/", a, b) {
                return r;
            }
            if raise_datetime_op_type_error("/", a, b) {
                return MbValue::none();
            }
            MbValue::none()
        }
    }
}

pub fn mb_mod(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__mod__") {
        return mb_mod(na, nb);
    }
    if let Some(r) = numeric_handle_binop("%", a, b) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("%", a, b) {
        return r;
    }
    // complex doesn't support the numeric modulo operator (CPython TypeError).
    // Don't intercept `str % complex` — that's printf-style formatting, where
    // the left operand is a string — so only guard a complex left operand or a
    // complex right operand against a non-string (numeric) left.
    let a_is_str = unsafe {
        a.as_ptr()
            .map_or(false, |p| matches!(&(*p).data, ObjData::Str(_)))
    };
    if is_complex_obj(a) || (is_complex_obj(b) && !a_is_str) {
        raise_type_error(format!(
            "unsupported operand type(s) for %: '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return MbValue::none();
    }
    // timedelta % timedelta -> timedelta remainder (floor semantics).
    if let (Some(ua), Some(ub)) = (
        super::stdlib::datetime_mod::timedelta_total_us(a),
        super::stdlib::datetime_mod::timedelta_total_us(b),
    ) {
        if ub == 0 {
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
            );
            return MbValue::none();
        }
        return super::stdlib::datetime_mod::timedelta_from_us(floor_divmod_i128(ua, ub).1);
    }
    // Integer fast path — Python floor-division modulo: result has same sign as b
    if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
        if bi != 0 {
            let r = ai % bi; // C-style truncation remainder
            let result = if r != 0 && (r ^ bi) < 0 { r + bi } else { r };
            return MbValue::from_int(result);
        }
        // ZeroDivisionError: CPython's `%` message differs from `//`
        // ("integer modulo by zero" vs "integer division or modulo by zero").
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
            MbValue::from_ptr(MbObject::new_str("integer modulo by zero".to_string())),
        );
        return MbValue::none();
    }
    // Float path — Python floor-division modulo: result has same sign as b
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    match (af, bf) {
        (Some(af), Some(bf)) if bf != 0.0 => {
            let r = af % bf; // IEEE 754 remainder (C-style)
            let result = if r != 0.0 && r.signum() != bf.signum() {
                r + bf
            } else {
                r
            };
            return MbValue::from_float(result);
        }
        (Some(_), Some(_)) => {
            // ZeroDivisionError: float modulo by zero
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str("float modulo".to_string())),
            );
            return MbValue::none();
        }
        _ => {}
    }
    // `str % X` — printf-style formatting (fallback after numeric paths).
    if let Some(ptr) = a.as_ptr() {
        unsafe {
            if let ObjData::Str(ref tmpl) = (*ptr).data {
                return super::string_ops::mb_str_percent_format(tmpl.clone(), b);
            }
            if matches!(&(*ptr).data, ObjData::Bytes(_) | ObjData::ByteArray(_)) {
                return super::bytes_ops::mb_bytes_percent_format(a, b);
            }
        }
    }
    if raise_datetime_op_type_error("%", a, b) {
        return MbValue::none();
    }
    MbValue::none()
}

pub fn mb_neg(a: MbValue) -> MbValue {
    if is_decimal_handle_value(a) {
        return super::stdlib::decimal_mod::mb_decimal_neg(a);
    }
    if is_fraction_handle_value(a) {
        return super::stdlib::fractions_mod::mb_fraction_neg(a);
    }
    // Codegen routes ANY non-Int/Float/Bool unary Neg operand straight to
    // this function (hir_to_mir's Neg lowering), bypassing
    // `mb_dispatch_unaryop`'s usual dunder lookup — so a user-overridden
    // `__neg__` on a numeric-derived class must be invoked here directly,
    // before the subclass-payload unwrap below. `try_get_dunder` is a cheap
    // no-op for non-Instance (raw int/float) operands. (#1030)
    if let Some(method) = super::class::try_get_dunder(a, "__neg__") {
        return super::class::mb_call_method1(method, a);
    }
    // int/float-SUBCLASS operand unwrap (#1030 — without this `-P(int)(7)`
    // produced None instead of CPython's plain-int `-7`).
    if let Some(v) = numeric_subclass_unary_operand(a, "__neg__") {
        return mb_neg(v);
    }
    if let Some(i) = a.as_int() {
        MbValue::from_int(-i)
    } else if let Some(f) = a.as_float() {
        MbValue::from_float(-f)
    } else if let Some(ptr) = a.as_ptr() {
        // -complex → complex with both components negated. (#1256)
        unsafe {
            if let ObjData::Complex(re, im) = (*ptr).data {
                return MbValue::from_ptr(MbObject::new_complex(-re, -im));
            }
            // -bigint → negated big integer. Without this, `-(2**63)` leaks the
            // BigInt pointer bits as a bogus small int (breaks every negative
            // out-of-48-bit literal, e.g. plistlib's signed-int range checks).
            if let ObjData::BigInt(ref big) = (*ptr).data {
                let neg = -big.clone();
                // Re-narrow to an inline int when it fits (e.g. -(2**47)).
                use num_traits::ToPrimitive;
                if let Some(i) = neg.to_i64() {
                    if (-(1i64 << 47)..(1i64 << 47)).contains(&i) {
                        return MbValue::from_int(i);
                    }
                }
                return super::bigint_ops::bigint_from_big(neg);
            }
            // -Counter — flip every count, then drop the now-non-positive ones
            // (CPython multiset semantics). `+c` routes through the generic
            // unary dispatcher's Counter arm; `-c` is lowered straight to
            // mb_neg, so it needs the same handling here.
            if super::stdlib::collections_mod::is_counter_instance(a) {
                return super::stdlib::collections_mod::mb_counter_unary(a, true);
            }
            // -timedelta — negate the exact microsecond total.
            if let Some(us) = super::stdlib::datetime_mod::timedelta_total_us(a) {
                return super::stdlib::datetime_mod::timedelta_from_us(-us);
            }
            // -NormalDist — flipped mean, fresh object.
            if let Some(r) = super::stdlib::statistics_mod::normaldist_neg(a) {
                return r;
            }
        }
        MbValue::none()
    } else {
        MbValue::none()
    }
}

// HANDWRITE-BEGIN reason: Phase 1.5 cross-cutting fix (#11) — Python's
// bytes/bytearray/memoryview/array('B') interconvert under `==`. Mamba's
// runtime stores these four representations differently (Bytes, ByteArray,
// Instance(memoryview, _buffer=...), Dict{__class__: "array", data: List[int]}),
// so structural equality must unify them before falling through to
// dispatch. Codegen has no section type for buffer-protocol coercion yet
// — convert to CODEGEN once the standardize sweep grows one.

/// Coerce a Python bytes-like MbValue into a byte vector.
///
/// Returns `Some` for:
///   * `bytes`              -> direct copy
///   * `bytearray`          -> read-lock copy of the contained Vec<u8>
///   * `memoryview(bb)`     -> recurse into the Instance's `_buffer` field
///   * `array('B'|'b', xs)` -> Dict-flavoured array whose `data` list holds
///                             ints; truncated to u8 bytes.
///
/// Returns `None` for anything else (caller can fall through to regular
/// equality / dispatch). Side-effect-free, safe to call from comparison
/// paths.
pub fn try_bytes_like(v: MbValue) -> Option<Vec<u8>> {
    if let Some(id) = v.as_int() {
        if super::stdlib::array_mod::is_array_handle(id as u64) {
            return try_bytes_like(super::stdlib::array_mod::mb_array_tobytes(v));
        }
    }
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Bytes(b) => Some(b.clone()),
            ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
            ObjData::Instance { class_name, fields } => {
                if class_name != "memoryview" {
                    return None;
                }
                let buf = fields.read().unwrap().get("_buffer").copied()?;
                try_bytes_like(buf)
            }
            ObjData::Dict(lock) => {
                let map = lock.read().unwrap();
                let class_v = map.get("__class__").copied()?;
                let cp = class_v.as_ptr()?;
                let is_array = matches!(&(*cp).data, ObjData::Str(s) if s == "array");
                if !is_array {
                    return None;
                }
                let typecode_v = map.get("typecode").copied();
                let typecode = typecode_v
                    .and_then(|tv| tv.as_ptr())
                    .and_then(|tp| match &(*tp).data {
                        ObjData::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                if typecode != "B" && typecode != "b" {
                    return None;
                }
                let data_v = map.get("data").copied()?;
                let dp = data_v.as_ptr()?;
                let ObjData::List(items_lock) = &(*dp).data else {
                    return None;
                };
                let items = items_lock.read().unwrap();
                let mut out = Vec::with_capacity(items.len());
                for item in items.iter() {
                    let i = item.as_int()?;
                    out.push(i as u8);
                }
                Some(out)
            }
            _ => None,
        }
    }
}
// HANDWRITE-END

pub fn mb_eq(a: MbValue, b: MbValue) -> MbValue {
    MbValue::from_bool(mb_values_eq(a, b))
}

pub fn mb_match_bool_literal(subject: MbValue, expected: i64) -> MbValue {
    MbValue::from_bool(subject.as_bool() == Some(expected != 0))
}

/// CPython `PyObject_RichCompareBool(a, b, Py_EQ)`: identity-first, then value
/// equality. Container comparisons (list/tuple/set/dict `==`, `in`, count,
/// subset/superset) use this so a self-unequal element such as NaN still
/// matches when the SAME object appears on both sides — `[nan] == [nan]` is True
/// for one shared NaN. Scalar `==` (mb_eq) deliberately does NOT add this, so
/// `nan == nan` stays False.
fn mb_richcmp_eq(a: MbValue, b: MbValue) -> bool {
    mb_values_identical(a, b) || mb_values_eq(a, b)
}

fn cell_values_eq(a: MbValue, b: MbValue) -> Option<bool> {
    use super::closure::CellCompareValue::{Empty, NotACell, Value};

    match (
        super::closure::mb_cell_compare_value(a),
        super::closure::mb_cell_compare_value(b),
    ) {
        (NotACell, NotACell) => None,
        (Empty, Empty) => Some(true),
        (Empty, _) | (_, Empty) => Some(false),
        (Value(av), Value(bv)) => {
            if let (Some(ai), Some(bf)) = (av.as_int_pyint(), bv.as_float()) {
                return Some((ai as f64) == bf);
            }
            if let (Some(af), Some(bi)) = (av.as_float(), bv.as_int_pyint()) {
                return Some(af == (bi as f64));
            }
            if let (Some(ai), Some(bi)) = (av.as_int_pyint(), bv.as_int_pyint()) {
                return Some(ai == bi);
            }
            Some(mb_values_eq(av, bv))
        }
        (Value(av), NotACell) => Some(mb_values_eq(av, b)),
        (NotACell, Value(bv)) => Some(mb_values_eq(a, bv)),
    }
}

fn cell_values_lt(a: MbValue, b: MbValue) -> Option<bool> {
    use super::closure::CellCompareValue::{Empty, NotACell, Value};

    match (
        super::closure::mb_cell_compare_value(a),
        super::closure::mb_cell_compare_value(b),
    ) {
        (NotACell, NotACell) => None,
        (Empty, Empty) => Some(false),
        (Empty, _) => Some(true),
        (_, Empty) => Some(false),
        (Value(av), Value(bv)) => {
            if let (Some(ai), Some(bf)) = (av.as_int_pyint(), bv.as_float()) {
                return Some((ai as f64) < bf);
            }
            if let (Some(af), Some(bi)) = (av.as_float(), bv.as_int_pyint()) {
                return Some(af < (bi as f64));
            }
            if let (Some(ai), Some(bi)) = (av.as_int_pyint(), bv.as_int_pyint()) {
                return Some(ai < bi);
            }
            Some(mb_values_lt(av, bv))
        }
        (Value(av), NotACell) => Some(mb_values_lt(av, b)),
        (NotACell, Value(bv)) => Some(mb_values_lt(a, bv)),
    }
}

fn generic_alias_origin_args(
    class_name: &str,
    fields: &super::rc::MbRwLock<super::rc::InstanceFields>,
) -> Option<(MbValue, Vec<MbValue>)> {
    let (origin_key, args_key) = match class_name {
        "GenericAlias" => ("__origin__", "__args__"),
        "types.GenericAlias" => ("_origin", "_args"),
        _ => return None,
    };
    let (origin, args_val) = {
        let g = fields.read().ok()?;
        (*g.get(origin_key)?, *g.get(args_key)?)
    };
    let args = args_val
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => items.to_vec(),
                ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                _ => vec![args_val],
            }
        })
        .unwrap_or_else(|| vec![args_val]);
    Some((origin, args))
}

fn type_object_display_name(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "type" => fields
                .read()
                .ok()
                .and_then(|f| f.get("__name__").copied())
                .and_then(str_value),
            _ => None,
        }
    }
}

fn str_value(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
}

fn generic_alias_repr_part(v: MbValue) -> String {
    if let Some(name) = pep695_display_name(v) {
        return name;
    }
    if let Some(name) = type_object_display_name(v) {
        return name;
    }
    str_value(mb_repr(v)).unwrap_or_default()
}

fn generic_alias_repr_string(
    class_name: &str,
    fields: &super::rc::MbRwLock<super::rc::InstanceFields>,
) -> Option<String> {
    let (origin, args) = generic_alias_origin_args(class_name, fields)?;
    let name = generic_alias_repr_part(origin);
    let parts = args
        .into_iter()
        .map(generic_alias_repr_part)
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{name}[{parts}]"))
}

/// Try the reflected __eq__ on `obj` (i.e. obj.__eq__(other)).
/// Returns true/false if the reflected op gives a definitive answer, false if not.
fn try_reflected_eq(obj: MbValue, other: MbValue) -> bool {
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            if let ObjData::Instance { class_name, .. } = &(*ptr).data {
                let eq_method = super::class::lookup_method(class_name, "__eq__");
                if !eq_method.is_none() {
                    let eq_name = MbValue::from_ptr(MbObject::new_str("__eq__".to_string()));
                    let args = MbValue::from_ptr(MbObject::new_list(vec![other]));
                    let result = super::class::mb_call_method(obj, eq_name, args);
                    if result.is_not_implemented() {
                        return false;
                    }
                    if let Some(bv) = result.as_bool() {
                        return bv;
                    }
                    if let Some(iv) = result.as_int() {
                        return iv != 0;
                    }
                }
            }
        }
    }
    false
}

enum RichcmpDunderCall {
    Missing,
    NotImplemented,
    Value(bool),
}

fn instance_class_name_for_richcmp(v: MbValue) -> Option<String> {
    v.as_ptr().and_then(|p| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*p).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

fn class_has_method(class_name: &str, method: &str) -> bool {
    !super::class::lookup_method(class_name, method).is_none()
}

fn call_instance_richcmp_dunder(
    inst: MbValue,
    class_name: &str,
    method: &str,
    other: MbValue,
) -> RichcmpDunderCall {
    if !class_has_method(class_name, method) {
        return RichcmpDunderCall::Missing;
    }
    let method_name = MbValue::from_ptr(MbObject::new_str(method.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![other]));
    let result = super::class::mb_call_method(inst, method_name, args);
    if result.is_not_implemented() {
        return RichcmpDunderCall::NotImplemented;
    }
    if let Some(bv) = result.as_bool() {
        return RichcmpDunderCall::Value(bv);
    }
    if let Some(iv) = result.as_int() {
        return RichcmpDunderCall::Value(iv != 0);
    }
    RichcmpDunderCall::Value(false)
}

fn instance_rhs_has_priority(lhs_class: &str, rhs_class: &str) -> bool {
    lhs_class != rhs_class && super::class::class_mro_any(rhs_class, |name| name == lhs_class)
}

fn mb_instance_ne(a: MbValue, b: MbValue) -> Option<bool> {
    let a_class = instance_class_name_for_richcmp(a);
    let b_class = instance_class_name_for_richcmp(b);
    let a_ne = a_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__ne__"));
    let a_eq = a_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__eq__"));
    let b_ne = b_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__ne__"));
    let b_eq = b_class
        .as_deref()
        .is_some_and(|class_name| class_has_method(class_name, "__eq__"));
    if !a_ne && !a_eq && !b_ne && !b_eq {
        return None;
    }

    let rhs_priority = match (a_class.as_deref(), b_class.as_deref()) {
        (Some(lhs), Some(rhs)) => instance_rhs_has_priority(lhs, rhs),
        _ => false,
    };

    if rhs_priority && b_ne {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__ne__", a)
            {
                return Some(v);
            }
        }
        if !a_ne {
            if let Some(class_name) = a_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(a, class_name, "__eq__", b)
                {
                    return Some(!v);
                }
            }
            return Some(true);
        }
    }

    if a_ne {
        if let Some(class_name) = a_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(a, class_name, "__ne__", b)
            {
                return Some(v);
            }
        }
        if !rhs_priority && b_ne {
            if let Some(class_name) = b_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(b, class_name, "__ne__", a)
                {
                    return Some(v);
                }
            }
            return Some(true);
        }
        if !b_ne && b_eq {
            if let Some(class_name) = b_class.as_deref() {
                if let RichcmpDunderCall::Value(v) =
                    call_instance_richcmp_dunder(b, class_name, "__eq__", a)
                {
                    return Some(!v);
                }
            }
        }
        return Some(true);
    }

    if a_eq {
        if let Some(class_name) = a_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(a, class_name, "__eq__", b)
            {
                return Some(!v);
            }
        }
    }

    if !rhs_priority && b_ne {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__ne__", a)
            {
                return Some(v);
            }
        }
        return Some(true);
    }

    if b_eq {
        if let Some(class_name) = b_class.as_deref() {
            if let RichcmpDunderCall::Value(v) =
                call_instance_richcmp_dunder(b, class_name, "__eq__", a)
            {
                return Some(!v);
            }
        }
        return Some(true);
    }

    if a_eq {
        return Some(true);
    }
    None
}

/// If `v` is a `slice` instance, return its (start, stop, step) as a tuple
/// MbValue so the value-comparison paths can compare/hash slices the way
/// CPython does (slices compare and hash as the 3-tuple of their fields).
fn slice_as_tuple(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "slice" {
                let g = fields.read().unwrap();
                let get = |k: &str| g.get(k).copied().unwrap_or_else(MbValue::none);
                return Some(MbValue::from_ptr(MbObject::new_tuple(vec![
                    get("start"),
                    get("stop"),
                    get("step"),
                ])));
            }
        }
    }
    None
}

fn mappingproxy_mapping(v: MbValue) -> Option<MbValue> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "mappingproxy" {
                return fields.read().unwrap().get("_mapping").copied();
            }
        }
    }
    None
}

fn bound_method_parts(v: MbValue) -> Option<(MbValue, MbValue)> {
    let ptr = v.as_ptr()?;
    unsafe {
        if let ObjData::Instance {
            ref class_name,
            ref fields,
        } = (*ptr).data
        {
            if class_name == "method" {
                let guard = fields.read().unwrap();
                let func = guard.get("__func__").copied().unwrap_or_else(MbValue::none);
                let recv = guard.get("__self__").copied().unwrap_or_else(MbValue::none);
                return Some((func, recv));
            }
        }
    }
    None
}

/// Deep structural equality for MbValues.
fn mb_values_eq(a: MbValue, b: MbValue) -> bool {
    if let Some(eq) = cell_values_eq(a, b) {
        return eq;
    }
    if let Some(eq) = super::stdlib::xml_mod::qname_eq_value(a, b) {
        return eq;
    }
    // UserList / UserDict / UserString compare by their backing payload, so
    // `UserList([0,1]) == [0,1]` and `UserList(x) == UserList(x)` hold (CPython
    // forwards __eq__ to self.data). Unwrap any wrapper operand, then recurse.
    let ua = super::stdlib::collections_mod::user_wrapper_data(a);
    let ub = super::stdlib::collections_mod::user_wrapper_data(b);
    if ua.is_some() || ub.is_some() {
        let la = ua.map(|(_, d)| d).unwrap_or(a);
        let lb = ub.map(|(_, d)| d).unwrap_or(b);
        return mb_values_eq(la, lb);
    }
    let da = super::class::unwrap_dictlike_data(a);
    let db = super::class::unwrap_dictlike_data(b);
    if da.is_some() || db.is_some() {
        let la = da.unwrap_or(a);
        let lb = db.unwrap_or(b);
        return mb_values_eq(la, lb);
    }
    // PEP 604 union cross-representation equality: a `X | Y` UnionType equals
    // the typing.Union[X, Y] alias (and is order-insensitive) by member set.
    if let Some(eq) = super::stdlib::typing_mod::union_values_equal(a, b) {
        return eq;
    }
    // slice == slice compares (start, stop, step) structurally (CPython).
    if let (Some(ta), Some(tb)) = (slice_as_tuple(a), slice_as_tuple(b)) {
        return mb_values_eq(ta, tb);
    }
    // complex vs complex/int/float/bool: equal iff both components match, so a
    // real-valued complex equals the matching real number (`(1+0j) == 1`).
    if is_complex_obj(a) || is_complex_obj(b) {
        if let (Some((ar, ai)), Some((br, bi))) = (as_complex_pair(a), as_complex_pair(b)) {
            return ar == br && ai == bi;
        }
    }
    // functools.cmp_to_key key objects compare via the wrapped cmp (sign == 0).
    if super::stdlib::functools_mod::is_cmp_to_key_obj(a)
        || super::stdlib::functools_mod::is_cmp_to_key_obj(b)
    {
        if let Some(r) = super::stdlib::functools_mod::mb_functools_cmp_to_key_richcmp(a, b, "eq") {
            return r;
        }
        if let Some(r) = super::stdlib::functools_mod::mb_functools_cmp_to_key_richcmp(b, a, "eq") {
            return r;
        }
    }
    if is_array_handle_value(a) || is_array_handle_value(b) {
        return super::stdlib::array_mod::mb_array_eq_bool(a, b).unwrap_or(false);
    }
    // Decimal / Fraction handles compare by exact numeric value (#2129);
    // non-numeric counterparts are simply unequal (CPython False).
    if is_decimal_handle_value(a)
        || is_decimal_handle_value(b)
        || is_fraction_handle_value(a)
        || is_fraction_handle_value(b)
    {
        return super::stdlib::decimal_mod::mb_numeric_handle_eq(a, b).unwrap_or(false);
    }
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__eq__") {
        return mb_values_eq(na, nb);
    }
    // NaN check: Python float NaN != NaN (IEEE 754). Must check before bit comparison.
    if let (Some(fa), Some(fb)) = (a.as_float(), b.as_float()) {
        return fa == fb; // IEEE 754: NaN == NaN is false
    }
    // Fast path: identical bits (safe for non-float types)
    if a.to_bits() == b.to_bits() {
        return true;
    }
    if let Some(eq) = super::dict_ops::dict_view_eq(a, b) {
        return eq;
    }
    if let Some(ma) = mappingproxy_mapping(a) {
        let rhs = mappingproxy_mapping(b).unwrap_or(b);
        return mb_values_eq(ma, rhs);
    }
    if let Some(mb) = mappingproxy_mapping(b) {
        return mb_values_eq(a, mb);
    }
    match (bound_method_parts(a), bound_method_parts(b)) {
        (Some((af, aself)), Some((bf, bself))) => {
            return af.to_bits() == bf.to_bits() && aself.to_bits() == bself.to_bits();
        }
        (Some(_), None) | (None, Some(_)) => return false,
        (None, None) => {}
    }
    // Bool/int cross-comparison: Python `True == 1` and `False == 0` (#827)
    if a.is_bool() && b.is_int() {
        let ai = if a.as_bool() == Some(true) {
            1i64
        } else {
            0i64
        };
        return ai == b.as_int().unwrap_or(i64::MIN);
    }
    if a.is_int() && b.is_bool() {
        let bi = if b.as_bool() == Some(true) {
            1i64
        } else {
            0i64
        };
        return a.as_int().unwrap_or(i64::MIN) == bi;
    }
    // Int-float cross-comparison
    if let (Some(ai), Some(bf)) = (a.as_int(), b.as_float()) {
        return (ai as f64) == bf;
    }
    if let (Some(af), Some(bi)) = (a.as_float(), b.as_int()) {
        return af == (bi as f64);
    }
    // BigInt value equality: two heap big integers (or a heap big int and an
    // inline int) compare by value, not pointer identity. Without this,
    // `2**64 == 2**64` is False (distinct allocations) — breaking plistlib's
    // `data >= _UID_MAX`/`== _UID_MAX` range checks and any big-int equality.
    {
        let a_big = unsafe { super::bigint_ops::extract_bigint(a).is_some() };
        let b_big = unsafe { super::bigint_ops::extract_bigint(b).is_some() };
        if a_big || b_big {
            if let (Some(ab), Some(bb)) = unsafe {
                (
                    super::bigint_ops::to_bigint(a),
                    super::bigint_ops::to_bigint(b),
                )
            } {
                return ab == bb;
            }
        }
    }
    // Class-body enum members: singleton identity between members; raw-value
    // equality ONLY for data-type mixins (IntFlag == int, StrEnum == str);
    // Plain/Flag members never equal raw values (`Suit.CLUBS != 1`). Must
    // run BEFORE the generic Instance-with-value-field rule below, which
    // would otherwise equate a plain Enum member with its raw value.
    if let Some(eq) = super::stdlib::enum_class::members_eq_override(a, b) {
        return eq;
    }
    // IntEnum-like Instance vs int comparison: when an Instance carries a
    // `value` int field (e.g. HTTPStatus.OK) compare against the other side
    // by value so `HTTPStatus.OK == 200` mirrors CPython's IntEnum semantics.
    let instance_value = |v: MbValue| -> Option<i64> {
        v.as_ptr().and_then(|p| unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*p).data
            {
                // A user-defined __eq__ takes precedence over the int-value
                // shortcut (e.g. an int subclass whose __eq__ always returns
                // True must NOT be compared by its raw value).
                if !super::class::lookup_method(class_name, "__eq__").is_none() {
                    return None;
                }
                fields
                    .read()
                    .unwrap()
                    .get("value")
                    .and_then(|fv| fv.as_int())
            } else {
                None
            }
        })
    };
    if let (Some(av), Some(bi)) = (instance_value(a), b.as_int()) {
        return av == bi;
    }
    if let (Some(ai), Some(bv)) = (a.as_int(), instance_value(b)) {
        return ai == bv;
    }
    // HANDWRITE-BEGIN reason: Phase 1.5 cross-cutting fix (#11) — Python
    // bytes/bytearray/memoryview/array('B') compare equal whenever their
    // underlying byte sequences match. Coerce both sides up-front so a
    // single buffer compare handles all 16 combinations instead of
    // enumerating pairs in the match arm below.
    if let (Some(av), Some(bv)) = (try_bytes_like(a), try_bytes_like(b)) {
        return av == bv;
    }
    // HANDWRITE-END

    // Range/range structural equality. Both `range()` results are NaN-boxed
    // iterator handles tagged as TAG_INT, so the int-int fast path above
    // already returns true for byte-identical handles. Different handles
    // pointing at equivalent Range iterators (e.g. `range(5) == range(5)`)
    // need a kind-aware compare. CPython rule: equal iff same length, and
    // when length >= 1 the starts match, and when length >= 2 the steps
    // also match. Other iterator kinds remain comparable only by identity.
    if super::iter::is_iter_handle(a) && super::iter::is_iter_handle(b) {
        if let Some(eq) = super::iter::ranges_value_eq(a, b) {
            return eq;
        }
    }
    // Counter == Counter — multiset equality: missing keys count as zero
    // (CPython Counter.__eq__). Counter vs plain dict falls back to exact
    // dict equality on the backing data (dict.__eq__ semantics).
    {
        let a_counter = super::stdlib::collections_mod::is_counter_instance(a);
        let b_counter = super::stdlib::collections_mod::is_counter_instance(b);
        if a_counter && b_counter {
            return super::stdlib::collections_mod::counter_eq(a, b);
        }
        if a_counter || b_counter {
            let (cnt, other) = if a_counter { (a, b) } else { (b, a) };
            let is_plain_dict = other
                .as_ptr()
                .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Dict(_)) });
            if is_plain_dict {
                if let Some(backing) = super::class::unwrap_dictlike_data(cnt) {
                    return mb_values_eq(backing, other);
                }
            }
        }
    }

    // namedtuple instances behave like tuple subclasses: compare element-wise
    // against another namedtuple or against a plain tuple of equal values.
    {
        let a_nt = super::stdlib::collections_mod::namedtuple_values(a);
        let b_nt = super::stdlib::collections_mod::namedtuple_values(b);
        let tuple_items = |v: MbValue| -> Option<Vec<MbValue>> {
            v.as_ptr().and_then(|p| unsafe {
                if let ObjData::Tuple(ref items) = (*p).data {
                    Some(items.to_vec())
                } else {
                    None
                }
            })
        };
        let pair = match (a_nt, b_nt) {
            (Some(va), Some(vb)) => Some((va, vb)),
            (Some(va), None) => tuple_items(b).map(|vb| (va, vb)),
            (None, Some(vb)) => tuple_items(a).map(|va| (va, vb)),
            (None, None) => None,
        };
        if let Some((va, vb)) = pair {
            return va.len() == vb.len()
                && va.iter().zip(vb.iter()).all(|(x, y)| mb_richcmp_eq(*x, *y));
        }
    }

    // Instance with a user __eq__ vs a primitive (non-pointer int/float/bool/
    // None): dispatch the instance's __eq__, matching CPython `x == y`. The
    // both-ptr match below requires BOTH operands to be heap pointers, so
    // without this `MyClass() == 1` and `obj in [1, 2, 3]` (via mb_eq) wrongly
    // returned False even when __eq__ would match. (The `==` operator path
    // dispatches via invoke_binop_method and already worked; this aligns the
    // value-equality path used by contains / dict / set.)
    {
        let dispatch_inst_eq = |inst: MbValue, other: MbValue| -> Option<bool> {
            // None / bool are matched by identity, not __eq__, in the contexts
            // that reach value-equality (notably `case None` / `case True`
            // pattern matching, which must NOT consult a custom __eq__). The
            // `==` operator keeps full __eq__ semantics via invoke_binop_method.
            if other.is_none() || other.is_bool() {
                return None;
            }
            let cn = inst.as_ptr().and_then(|p| unsafe {
                if let ObjData::Instance { ref class_name, .. } = (*p).data {
                    Some(class_name.clone())
                } else {
                    None
                }
            })?;
            if super::class::lookup_method(&cn, "__eq__").is_none() {
                return None;
            }
            let eq_name = MbValue::from_ptr(MbObject::new_str("__eq__".to_string()));
            let args = MbValue::from_ptr(MbObject::new_list(vec![other]));
            let result = super::class::mb_call_method(inst, eq_name, args);
            if result.is_not_implemented() {
                return Some(false);
            }
            if let Some(bv) = result.as_bool() {
                return Some(bv);
            }
            if let Some(iv) = result.as_int() {
                return Some(iv != 0);
            }
            Some(false)
        };
        if a.as_ptr().is_some() && b.as_ptr().is_none() {
            if let Some(r) = dispatch_inst_eq(a, b) {
                return r;
            }
        } else if b.as_ptr().is_some() && a.as_ptr().is_none() {
            if let Some(r) = dispatch_inst_eq(b, a) {
                return r;
            }
        }
    }

    // DATA-payload builtin subclasses compare through their hidden str/list/dict
    // payload only when neither side supplies an explicit __eq__ method.
    {
        let ap = super::class::builtin_data_payload_if_unoverridden(a, "__eq__")
            .map(|(_, payload)| payload);
        let bp = super::class::builtin_data_payload_if_unoverridden(b, "__eq__")
            .map(|(_, payload)| payload);
        if ap.is_some() || bp.is_some() {
            return mb_values_eq(ap.unwrap_or(a), bp.unwrap_or(b));
        }
    }

    // Structural equality for heap objects
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            return match (&(*pa).data, &(*pb).data) {
                (ObjData::List(la), ObjData::List(lb)) => {
                    let a = super::list_ops::retained_list_snapshot(la);
                    let b = super::list_ops::retained_list_snapshot(lb);
                    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| mb_richcmp_eq(*x, *y))
                }
                (ObjData::Tuple(a), ObjData::Tuple(b)) => {
                    a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| mb_richcmp_eq(*x, *y))
                }
                (ObjData::Str(sa), ObjData::Str(sb)) => {
                    super::string_ops::string_values_equal_if_surrogate(a, b).unwrap_or(sa == sb)
                }
                (ObjData::Set(la), ObjData::Set(lb)) => {
                    let a = la.read().unwrap();
                    let b = lb.read().unwrap();
                    if a.len() != b.len() {
                        return false;
                    }
                    a.iter().all(|x| b.iter().any(|y| mb_richcmp_eq(*x, *y)))
                }
                (ObjData::FrozenSet(a_items), ObjData::FrozenSet(b_items)) => {
                    if a_items.len() != b_items.len() {
                        return false;
                    }
                    a_items
                        .iter()
                        .all(|x| b_items.iter().any(|y| mb_richcmp_eq(*x, *y)))
                }
                (ObjData::Set(la), ObjData::FrozenSet(b_items)) => {
                    let a = la.read().unwrap();
                    if a.len() != b_items.len() {
                        return false;
                    }
                    a.iter()
                        .all(|x| b_items.iter().any(|y| mb_richcmp_eq(*x, *y)))
                }
                (ObjData::FrozenSet(a_items), ObjData::Set(lb)) => {
                    let b = lb.read().unwrap();
                    if a_items.len() != b.len() {
                        return false;
                    }
                    a_items
                        .iter()
                        .all(|x| b.iter().any(|y| mb_richcmp_eq(*x, *y)))
                }
                (ObjData::Dict(la), ObjData::Dict(lb)) => {
                    let a_items: Vec<(super::dict_ops::DictKey, MbValue)> = {
                        let a = la.read().unwrap();
                        a.iter().map(|(k, &v)| (k.clone(), v)).collect()
                    };
                    if a_items.len() != lb.read().unwrap().len() {
                        return false;
                    }
                    a_items.iter().all(|(k, v)| {
                        let key = super::dict_ops::dict_key_to_mbvalue(k);
                        if super::dict_ops::mb_dict_contains(b, key).as_bool() != Some(true) {
                            return false;
                        }
                        let found = super::dict_ops::mb_dict_getitem(b, key);
                        mb_richcmp_eq(*v, found)
                    })
                }
                (ObjData::Bytes(a), ObjData::Bytes(b)) => a == b,
                (ObjData::Complex(ar, ai), ObjData::Complex(br, bi)) => {
                    // IEEE 754 component equality (NaN != NaN). (#1256)
                    ar == br && ai == bi
                }
                (ObjData::ByteArray(la), ObjData::ByteArray(lb)) => {
                    *la.read().unwrap() == *lb.read().unwrap()
                }
                (ObjData::Bytes(a), ObjData::ByteArray(lb)) => *a == *lb.read().unwrap(),
                (ObjData::ByteArray(la), ObjData::Bytes(b)) => *la.read().unwrap() == *b,
                // types.SimpleNamespace value equality: two SimpleNamespace
                // objects compare equal iff their attribute dicts (__dict__)
                // are equal — same keys mapping to equal values. The native
                // constructor produces a plain Instance with no user __eq__,
                // so without this they compare by identity. CPython: a
                // SimpleNamespace is *not* equal to any non-SimpleNamespace,
                // so this only fires when both sides are SimpleNamespace.
                // Unbound builtin-method wrappers compare by identity of the
                // (type, method) pair — CPython slot wrappers are singletons,
                // so `tuple.__getitem__ == tuple.__getitem__` is True.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == "__unbound_method__" && cb == "__unbound_method__" => {
                    let ga = fa.read().unwrap();
                    let gb = fb.read().unwrap();
                    let pair = |g: &rustc_hash::FxHashMap<String, MbValue>| {
                        (
                            g.get("__type__").copied().unwrap_or(MbValue::none()),
                            g.get("__method__").copied().unwrap_or(MbValue::none()),
                        )
                    };
                    let (ta, ma) = pair(&ga);
                    let (tb, mb) = pair(&gb);
                    return mb_values_eq(ta, tb) && mb_values_eq(ma, mb);
                }
                // PEP 604/695: UnionType value equality — same member set
                // (order-insensitive, like CPython's frozenset-based eq).
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == "UnionType" && cb == "UnionType" => {
                    let aa = fa.read().unwrap().get("__args__").copied();
                    let bb = fb.read().unwrap().get("__args__").copied();
                    let items = |v: Option<MbValue>| -> Option<Vec<MbValue>> {
                        v.and_then(|t| t.as_ptr()).and_then(|p| match &(*p).data {
                            ObjData::Tuple(ref it) => Some(it.to_vec()),
                            _ => None,
                        })
                    };
                    return match (items(aa), items(bb)) {
                        (Some(xs), Some(ys)) => {
                            xs.len() == ys.len()
                                && xs.iter().all(|x| ys.iter().any(|y| mb_richcmp_eq(*x, *y)))
                        }
                        _ => false,
                    };
                }
                // Type objects compare by the class they name: `type(x)`
                // allocates a fresh Instance per call, but all type objects
                // naming the same class are the same class object.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == "type" && cb == "type" => {
                    let name_of = |f: &super::rc::MbRwLock<super::rc::InstanceFields>| {
                        f.read().ok().and_then(|g| {
                            g.get("__name__").and_then(|v| v.as_ptr()).and_then(|p| {
                                match &(*p).data {
                                    ObjData::Str(s) => Some(s.clone()),
                                    _ => None,
                                }
                            })
                        })
                    };
                    return match (name_of(fa), name_of(fb)) {
                        (Some(na), Some(nb)) => na == nb,
                        _ => false,
                    };
                }
                // PEP 585 / types.GenericAlias equality: two generic aliases
                // compare by origin and args, not heap identity. This covers
                // lazy bound expressions such as Sequence[S] rebuilt on demand.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if matches!(ca.as_str(), "GenericAlias" | "types.GenericAlias")
                    || matches!(cb.as_str(), "GenericAlias" | "types.GenericAlias") =>
                {
                    let Some((oa, aa)) = generic_alias_origin_args(ca, fa) else {
                        return false;
                    };
                    let Some((ob, ab)) = generic_alias_origin_args(cb, fb) else {
                        return false;
                    };
                    return mb_richcmp_eq(oa, ob)
                        && aa.len() == ab.len()
                        && aa.iter().zip(ab.iter()).all(|(x, y)| mb_richcmp_eq(*x, *y));
                }
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == "SimpleNamespace" && cb == "SimpleNamespace" => {
                    let ga = fa.read().unwrap();
                    let gb = fb.read().unwrap();
                    if ga.len() != gb.len() {
                        return false;
                    }
                    return ga
                        .iter()
                        .all(|(k, v)| gb.get(k).map_or(false, |v2| mb_richcmp_eq(*v, *v2)));
                }
                // statistics.NormalDist value equality: two NormalDists are
                // equal iff their (mu, sigma) pairs are equal. The constructor
                // produces a plain Instance with `mu`/`sigma` fields and no
                // user `__eq__`, so without this they would compare by
                // identity. Only fires when *both* sides are NormalDist;
                // mismatched types fall through to the generic dunder path.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == "NormalDist" && cb == "NormalDist" => {
                    let ga = fa.read().unwrap();
                    let gb = fb.read().unwrap();
                    let mua = ga
                        .get("mu")
                        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)));
                    let mub = gb
                        .get("mu")
                        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)));
                    let siga = ga
                        .get("sigma")
                        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)));
                    let sigb = gb
                        .get("sigma")
                        .and_then(|v| v.as_float().or_else(|| v.as_int().map(|i| i as f64)));
                    return mua == mub && siga == sigb;
                }
                // PEP 557: dataclass __eq__ — same-class instances compare by
                // their compare=True field values (declaration order). Only
                // fires when the class is a registered dataclass with eq=True
                // and defines no user __eq__; cross-class comparisons fall
                // through to the dunder path (→ False), matching CPython's
                // NotImplemented behavior.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == cb
                    && super::class::lookup_method(ca, "__eq__").is_none()
                    && super::stdlib::dataclasses_mod::dc_eq_field_names(ca).is_some() =>
                {
                    let names =
                        super::stdlib::dataclasses_mod::dc_eq_field_names(ca).unwrap_or_default();
                    let (av, bv): (Vec<MbValue>, Vec<MbValue>) = {
                        let ga = fa.read().unwrap();
                        let gb = fb.read().unwrap();
                        (
                            names
                                .iter()
                                .map(|n| ga.get(n).copied().unwrap_or_else(MbValue::none))
                                .collect(),
                            names
                                .iter()
                                .map(|n| gb.get(n).copied().unwrap_or_else(MbValue::none))
                                .collect(),
                        )
                    };
                    return av.iter().zip(bv.iter()).all(|(x, y)| mb_richcmp_eq(*x, *y));
                }
                // Instance: dispatch __eq__ dunder with NotImplemented fallback
                (ObjData::Instance { class_name, .. }, _) => {
                    let eq_method = super::class::lookup_method(class_name, "__eq__");
                    if !eq_method.is_none() {
                        let eq_name = MbValue::from_ptr(MbObject::new_str("__eq__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
                        let result = super::class::mb_call_method(a, eq_name, args);
                        // If __eq__ returns NotImplemented, try reflected op on b
                        if result.is_not_implemented() {
                            return try_reflected_eq(b, a);
                        }
                        if let Some(bv) = result.as_bool() {
                            return bv;
                        }
                        if let Some(iv) = result.as_int() {
                            return iv != 0;
                        }
                    } else {
                        return try_reflected_eq(b, a);
                    }
                    false
                }
                // b is Instance, a is not — try b.__eq__(a) directly
                (_, ObjData::Instance { class_name, .. }) => {
                    let eq_method = super::class::lookup_method(class_name, "__eq__");
                    if !eq_method.is_none() {
                        let eq_name = MbValue::from_ptr(MbObject::new_str("__eq__".to_string()));
                        let args = MbValue::from_ptr(MbObject::new_list(vec![a]));
                        let result = super::class::mb_call_method(b, eq_name, args);
                        if result.is_not_implemented() {
                            return false;
                        }
                        if let Some(bv) = result.as_bool() {
                            return bv;
                        }
                        if let Some(iv) = result.as_int() {
                            return iv != 0;
                        }
                    }
                    false
                }
                _ => false,
            };
        }
    }
    false
}

fn mb_values_identical(a: MbValue, b: MbValue) -> bool {
    if a.to_bits() == b.to_bits() {
        return true;
    }
    let user_iter_a = super::iter::mb_userdefined_iterator_target(a);
    let user_iter_b = super::iter::mb_userdefined_iterator_target(b);
    if user_iter_a.is_some_and(|target| target.to_bits() == b.to_bits())
        || user_iter_b.is_some_and(|target| target.to_bits() == a.to_bits())
        || user_iter_a
            .zip(user_iter_b)
            .is_some_and(|(target_a, target_b)| target_a.to_bits() == target_b.to_bits())
    {
        return true;
    }
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            let str_value = |v: MbValue| -> Option<String> {
                v.as_ptr().and_then(|p| {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
            };
            let unbound_key = |p: *mut MbObject| -> Option<(String, String)> {
                if let ObjData::Instance {
                    ref class_name,
                    ref fields,
                } = (*p).data
                {
                    if class_name == "__unbound_method__" {
                        let f = fields.read().unwrap();
                        return Some((
                            str_value(f.get("__type__").copied().unwrap_or(MbValue::none()))?,
                            str_value(f.get("__method__").copied().unwrap_or(MbValue::none()))?,
                        ));
                    }
                }
                None
            };
            if let (Some((ta, ma)), Some((tb, mb))) = (unbound_key(pa), unbound_key(pb)) {
                if ta == tb && ma == mb {
                    return true;
                }
            }
        }
    }
    // Class identity: a class is represented at runtime as its (uniquely-naming)
    // class-name string, so two distinct string allocations that both name the
    // same registered class refer to the same class object. This makes
    // `cls is SomeClass` work (e.g. ABCMeta `__subclasshook__` identity guards)
    // without interning ordinary strings.
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            if let (ObjData::Str(ref sa), ObjData::Str(ref sb)) = (&(*pa).data, &(*pb).data) {
                if sa == sb
                    && (super::class::class_is_registered(sa)
                        || super::exception::is_builtin_exception_name(sa))
                {
                    return true;
                }
            }
            // Type-object identity: `type(x)` allocates a fresh Instance
            // (class_name "type", __name__=N) per call, but all type objects
            // naming the same registered class ARE the same class object —
            // as is the class-name string itself. Makes `type(obj) is Cls`
            // and `type(a) is type(b)` behave like CPython for registered
            // classes and builtin type names.
            let type_obj_name = |p: *mut MbObject| -> Option<String> {
                if let ObjData::Instance {
                    ref class_name,
                    ref fields,
                } = (*p).data
                {
                    if class_name == "type" {
                        return fields.read().ok().and_then(|f| {
                            f.get("__name__").and_then(|v| {
                                if let Some(np) = v.as_ptr() {
                                    if let ObjData::Str(ref n) = (*np).data {
                                        return Some(n.clone());
                                    }
                                }
                                None
                            })
                        });
                    }
                }
                None
            };
            let resolves_to_type_name = |p: *mut MbObject, name: &str| -> bool {
                match &(*p).data {
                    ObjData::Str(ref s) => s == name,
                    _ => type_obj_name(p).as_deref() == Some(name),
                }
            };
            if let Some(na) = type_obj_name(pa) {
                if (super::class::class_is_registered(&na)
                    || is_type_name(&na)
                    || super::exception::is_builtin_exception_name(&na))
                    && resolves_to_type_name(pb, &na)
                {
                    return true;
                }
            } else if let Some(nb) = type_obj_name(pb) {
                if (super::class::class_is_registered(&nb)
                    || is_type_name(&nb)
                    || super::exception::is_builtin_exception_name(&nb))
                    && resolves_to_type_name(pa, &nb)
                {
                    return true;
                }
            }
        }
    }
    // Native constructor dispatchers used as class objects (e.g.
    // collections.ChainMap, io.StringIO): a TAG_FUNC with a recorded type
    // name IS the class object, so `type(x) is ChainMap` must hold when the
    // other side is a type object / class-name string naming the same type.
    {
        let native_name = |v: MbValue| -> Option<String> {
            v.as_func().and_then(|addr| {
                super::module::NATIVE_TYPE_NAMES.with(|m| m.borrow().get(&(addr as u64)).cloned())
            })
        };
        let value_type_name = |v: MbValue| -> Option<String> {
            v.as_ptr().and_then(|p| unsafe {
                match &(*p).data {
                    ObjData::Str(ref s) => Some(s.clone()),
                    ObjData::Instance {
                        ref class_name,
                        ref fields,
                    } if class_name == "type" => fields.read().ok().and_then(|f| {
                        f.get("__name__").and_then(|v| v.as_ptr()).and_then(|np| {
                            if let ObjData::Str(ref n) = (*np).data {
                                Some(n.clone())
                            } else {
                                None
                            }
                        })
                    }),
                    _ => None,
                }
            })
        };
        if let (Some(n), Some(t)) = (native_name(a), value_type_name(b)) {
            if n == t {
                return true;
            }
        }
        if let (Some(n), Some(t)) = (native_name(b), value_type_name(a)) {
            if n == t {
                return true;
            }
        }
    }
    false
}

/// Ordering on complex is undefined in Python: raise the CPython-exact
/// TypeError when either operand is a complex object. Returns true when an
/// exception was raised (caller must bail with a dummy value).
fn complex_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if is_complex_obj(a) || is_complex_obj(b) {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

/// Plain `Enum` / non-int `Flag` members don't support ordering in CPython;
/// raise the exact TypeError when either operand is one. IntEnum / IntFlag /
/// StrEnum members compare via their raw int/str value and are NOT guarded.
fn enum_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if super::stdlib::enum_class::member_is_plain_unorderable(a)
        || super::stdlib::enum_class::member_is_plain_unorderable(b)
    {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

fn range_ordering_guard(a: MbValue, b: MbValue, op: &str) -> bool {
    if super::iter::is_range_handle(a) || super::iter::is_range_handle(b) {
        raise_type_error(format!(
            "'{op}' not supported between instances of '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return true;
    }
    false
}

/// The elements of a set-like value (`set` or `frozenset`), or None for any
/// other value. Used so subset/superset comparisons treat the two types
/// interchangeably, matching CPython.
fn setlike_items(v: MbValue) -> Option<Vec<MbValue>> {
    v.as_ptr().and_then(|p| unsafe {
        match &(*p).data {
            ObjData::Set(lock) => Some(lock.read().unwrap().iter().copied().collect()),
            ObjData::FrozenSet(items) => Some(items.iter().copied().collect()),
            _ => None,
        }
    })
}

fn is_instance_value(v: MbValue) -> bool {
    v.as_ptr()
        .is_some_and(|p| unsafe { matches!((*p).data, ObjData::Instance { .. }) })
}

fn same_datetime_instances(a: MbValue, b: MbValue) -> bool {
    match (a.as_ptr(), b.as_ptr()) {
        (Some(pa), Some(pb)) => unsafe {
            matches!(
                (&(*pa).data, &(*pb).data),
                (
                    ObjData::Instance { class_name: ca, .. },
                    ObjData::Instance { class_name: cb, .. },
                ) if ca == "datetime.datetime" && cb == "datetime.datetime"
            )
        },
        _ => false,
    }
}

fn same_ordered_dataclass_instances(a: MbValue, b: MbValue) -> bool {
    match (a.as_ptr(), b.as_ptr()) {
        (Some(pa), Some(pb)) => unsafe {
            matches!(
                (&(*pa).data, &(*pb).data),
                (
                    ObjData::Instance { class_name: ca, .. },
                    ObjData::Instance { class_name: cb, .. },
                ) if ca == cb && super::stdlib::dataclasses_mod::dc_order_field_names(ca).is_some()
            )
        },
        _ => false,
    }
}

fn can_derive_ordering_from_lt_eq(a: MbValue, b: MbValue) -> bool {
    if !is_instance_value(a) && !is_instance_value(b) {
        return true;
    }
    if super::stdlib::collections_mod::is_counter_instance(a)
        && super::stdlib::collections_mod::is_counter_instance(b)
    {
        return true;
    }
    same_datetime_instances(a, b) || same_ordered_dataclass_instances(a, b)
}

fn unsupported_ordering_bool(a: MbValue, b: MbValue, op: &str) -> MbValue {
    raise_type_error(format!(
        "'{op}' not supported between instances of '{}' and '{}'",
        value_type_name(a),
        value_type_name(b)
    ));
    MbValue::from_bool(false)
}

pub fn mb_lt(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    if range_ordering_guard(a, b, "<") {
        return MbValue::from_bool(false);
    }
    if enum_ordering_guard(a, b, "<") {
        return MbValue::from_bool(false);
    }
    MbValue::from_bool(mb_values_lt(a, b))
}

/// Ordering comparison: supports int, float, mixed int/float, lists, tuples, strings.
fn mb_values_lt(a: MbValue, b: MbValue) -> bool {
    if let Some(lt) = cell_values_lt(a, b) {
        return lt;
    }
    // slice < slice orders by (start, stop, step) like the equivalent tuple.
    if let (Some(ta), Some(tb)) = (slice_as_tuple(a), slice_as_tuple(b)) {
        return mb_values_lt(ta, tb);
    }
    // complex < complex (or complex vs anything) is a TypeError in CPython —
    // raise instead of silently answering False. Catches `<` directly and the
    // sort/min/max paths that funnel through mb_values_lt.
    if complex_ordering_guard(a, b, "<") {
        return false;
    }
    if range_ordering_guard(a, b, "<") {
        return false;
    }
    // functools.cmp_to_key key objects delegate ordering to the wrapped cmp.
    if super::stdlib::functools_mod::is_cmp_to_key_obj(a) {
        if let Some(r) = super::stdlib::functools_mod::mb_functools_cmp_to_key_richcmp(a, b, "lt") {
            return r;
        }
    }
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__lt__") {
        return mb_values_lt(na, nb);
    }
    if let Some((na, nb)) = str_subclass_ordering_operands(a, b, "__lt__") {
        return mb_values_lt(na, nb);
    }
    // functools.total_ordering: derive __lt__ from the class's seed op.
    if super::stdlib::functools_mod::is_total_ordering_instance(a) {
        if let Some(r) =
            super::stdlib::functools_mod::mb_functools_total_ordering_richcmp(a, b, "lt")
        {
            return r;
        }
    }
    if is_array_handle_value(a) || is_array_handle_value(b) {
        return super::stdlib::array_mod::mb_array_lt_bool(a, b).unwrap_or(false);
    }
    // Decimal / Fraction handles order by exact numeric value (#2129).
    if is_decimal_handle_value(a)
        || is_decimal_handle_value(b)
        || is_fraction_handle_value(a)
        || is_fraction_handle_value(b)
    {
        return super::stdlib::decimal_mod::mb_numeric_handle_lt(a, b).unwrap_or(false);
    }
    // Counter < Counter — strict multiset inclusion (CPython 3.12:
    // `__lt__ = self <= other and self != other`, counts compared
    // elementwise with missing keys as zero). `>`, `<=`, `>=` all derive
    // from this via mb_gt/mb_le/mb_ge's lt/eq composition.
    if super::stdlib::collections_mod::is_counter_instance(a)
        && super::stdlib::collections_mod::is_counter_instance(b)
    {
        return super::stdlib::collections_mod::counter_le_multiset(a, b)
            && !super::stdlib::collections_mod::counter_eq(a, b);
    }
    // Int comparison (bool ≤ int: True < 2 must order numerically).
    if let (Some(ai), Some(bi)) = (a.as_int_pyint(), b.as_int_pyint()) {
        return ai < bi;
    }
    // BigInt comparison: when either operand is a heap big integer (literals
    // beyond the 48-bit inline range, e.g. 2**64), compare as arbitrary-
    // precision integers. Without this, `1 < 2**64` and `2**64 > 2**63`
    // fall through to `false`. Only engages when at least one side is a heap
    // BigInt, so inline-int and float paths above are unaffected.
    let a_is_big = unsafe { super::bigint_ops::extract_bigint(a).is_some() };
    let b_is_big = unsafe { super::bigint_ops::extract_bigint(b).is_some() };
    if a_is_big || b_is_big {
        if let (Some(ab), Some(bb)) = unsafe {
            (
                super::bigint_ops::to_bigint(a),
                super::bigint_ops::to_bigint(b),
            )
        } {
            return ab < bb;
        }
    }
    // Float/mixed numeric comparison (bool coerces like int).
    let af = a.as_int_pyint().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int_pyint().map(|i| i as f64).or(b.as_float());
    if let (Some(af), Some(bf)) = (af, bf) {
        return af < bf;
    }
    // Strict subset for set/frozenset (either side may be either type):
    // `a < b` iff every element of a is in b AND |a| < |b|. CPython treats
    // set and frozenset interchangeably for subset/superset comparisons.
    if let (Some(sa), Some(sb)) = (setlike_items(a), setlike_items(b)) {
        return sa.len() < sb.len() && sa.iter().all(|x| sb.iter().any(|y| mb_richcmp_eq(*x, *y)));
    }
    // Lexicographic comparison for sequences
    if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
        unsafe {
            return match (&(*pa).data, &(*pb).data) {
                (ObjData::List(la), ObjData::List(lb)) => {
                    let a = super::list_ops::retained_list_snapshot(la);
                    let b = super::list_ops::retained_list_snapshot(lb);
                    seq_lt(&a, &b)
                }
                (ObjData::Tuple(a), ObjData::Tuple(b)) => seq_lt(a, b),
                (ObjData::Str(a), ObjData::Str(b)) => a < b,
                // bytes/bytearray lexicographic ordering (all four combos).
                (ObjData::Bytes(x), ObjData::Bytes(y)) => x.as_slice() < y.as_slice(),
                (ObjData::ByteArray(x), ObjData::ByteArray(y)) => {
                    *x.read().unwrap() < *y.read().unwrap()
                }
                (ObjData::Bytes(x), ObjData::ByteArray(y)) => {
                    x.as_slice() < y.read().unwrap().as_slice()
                }
                (ObjData::ByteArray(x), ObjData::Bytes(y)) => {
                    x.read().unwrap().as_slice() < y.as_slice()
                }
                (
                    ObjData::Instance { class_name: ca, .. },
                    ObjData::Instance { class_name: cb, .. },
                ) if ca == "datetime.datetime" && cb == "datetime.datetime" => {
                    let na = super::stdlib::datetime_mod::instance_to_naive(a);
                    let nb = super::stdlib::datetime_mod::instance_to_naive(b);
                    match (na, nb) {
                        (Some(x), Some(y)) => x < y,
                        _ => false,
                    }
                }
                // REQ: R4 — strict subset: a < b iff a ⊂ b (every element of a is in b AND |a| < |b|)
                (ObjData::Set(la), ObjData::Set(lb)) => {
                    let a_items = la.read().unwrap();
                    let b_items = lb.read().unwrap();
                    if a_items.len() >= b_items.len() {
                        return false;
                    }
                    a_items
                        .iter()
                        .all(|x| b_items.iter().any(|y| mb_richcmp_eq(*x, *y)))
                }
                // PEP 557: @dataclass(order=True) — same-class instances order
                // lexicographically by their compare=True field tuples. Only
                // fires when no user __lt__ is defined.
                (
                    ObjData::Instance {
                        class_name: ca,
                        fields: fa,
                    },
                    ObjData::Instance {
                        class_name: cb,
                        fields: fb,
                    },
                ) if ca == cb
                    && super::class::lookup_method(ca, "__lt__").is_none()
                    && super::stdlib::dataclasses_mod::dc_order_field_names(ca).is_some() =>
                {
                    let names = super::stdlib::dataclasses_mod::dc_order_field_names(ca)
                        .unwrap_or_default();
                    let (av, bv): (Vec<MbValue>, Vec<MbValue>) = {
                        let ga = fa.read().unwrap();
                        let gb = fb.read().unwrap();
                        (
                            names
                                .iter()
                                .map(|n| ga.get(n).copied().unwrap_or_else(MbValue::none))
                                .collect(),
                            names
                                .iter()
                                .map(|n| gb.get(n).copied().unwrap_or_else(MbValue::none))
                                .collect(),
                        )
                    };
                    seq_lt(&av, &bv)
                }
                // Instance: dispatch __lt__ dunder
                (ObjData::Instance { class_name, .. }, _) => {
                    dispatch_richcmp_dunder_result(a, b, class_name, "__lt__")
                        .unwrap_or_else(|| values_lt_fallback(a, b))
                }
                _ => values_lt_fallback(a, b),
            };
        }
    }
    values_lt_fallback(a, b)
}

/// Final fallthrough for `<` when no native ordering applies: try the
/// reflected `__gt__` when the right operand is an instance (CPython's
/// `a < b` → `type(b).__gt__(b, a)` fallback), then raise the CPython
/// unorderable TypeError instead of silently answering False.
fn values_lt_fallback(a: MbValue, b: MbValue) -> bool {
    if let Some(pb) = b.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*pb).data {
                if !super::class::lookup_method(class_name, "__gt__").is_none() {
                    return dispatch_richcmp_dunder(b, a, class_name, "__gt__");
                }
            }
        }
    }
    raise_type_error(format!(
        "'<' not supported between instances of '{}' and '{}'",
        value_type_name(a),
        value_type_name(b)
    ));
    false
}

/// Dispatch a rich comparison dunder (__lt__, __le__, __gt__, __ge__) on an Instance.
fn dispatch_richcmp_dunder_result(
    a: MbValue,
    b: MbValue,
    class_name: &str,
    dunder: &str,
) -> Option<bool> {
    let method = super::class::lookup_method(class_name, dunder);
    if method.is_none() {
        return None;
    }
    let method_name = MbValue::from_ptr(MbObject::new_str(dunder.to_string()));
    let args = MbValue::from_ptr(MbObject::new_list(vec![b]));
    let result = super::class::mb_call_method(a, method_name, args);
    if result.is_not_implemented() {
        return None;
    }
    if let Some(bv) = result.as_bool() {
        return Some(bv);
    }
    if let Some(iv) = result.as_int() {
        return Some(iv != 0);
    }
    Some(false)
}

fn dispatch_richcmp_dunder(a: MbValue, b: MbValue, class_name: &str, dunder: &str) -> bool {
    dispatch_richcmp_dunder_result(a, b, class_name, dunder).unwrap_or(false)
}

/// Lexicographic less-than for MbValue sequences.
pub fn seq_lt(a: &[MbValue], b: &[MbValue]) -> bool {
    // CPython compares sequences element-wise: find the first position where
    // the elements are unequal, then decide with `<` there. Probing equality
    // first (rather than `x < y` / `y < x`) means an equal but unorderable
    // head — e.g. `(None, 2) < (None, 1)` — never triggers `None < None`; only
    // a genuinely differing, unorderable pair raises TypeError (as it should).
    for (x, y) in a.iter().zip(b.iter()) {
        if !mb_values_eq(*x, *y) {
            return mb_values_lt(*x, *y);
        }
    }
    a.len() < b.len()
}

// ── Aggregation builtins (#378) ──

/// CPython: a lone scalar argument to min/max/sum must be iterable.
/// Returns true (and raises TypeError) when `args` cannot be iterated.
fn raise_if_not_iterable(args: MbValue) -> bool {
    if args.as_ptr().is_none()
        && !super::iter::mb_is_iterator_handle(args)
        && !super::generator::is_known_generator(args)
    {
        raise_type_error(format!(
            "'{}' object is not iterable",
            value_type_name(args)
        ));
        return true;
    }
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                if !super::class::lookup_method(class_name, "__iter__").is_none() {
                    return false;
                }
                let iter = super::iter::mb_iter(args);
                if iter.is_none() {
                    return true;
                }
                if super::iter::is_iter_handle(iter) {
                    super::iter::mb_iter_release(iter);
                }
            }
        }
    }
    false
}

/// min(iterable) or min(a, b) — return the smallest value.
pub fn mb_min(args: MbValue) -> MbValue {
    if raise_if_not_iterable(args) {
        return MbValue::none();
    }
    let items = extract_items(args);
    if items.is_empty() {
        // CPython: min(()) raises ValueError (no silent None default).
        raise_value_error("min() arg is an empty sequence".to_string());
        return MbValue::none();
    }
    items
        .into_iter()
        .reduce(|a, b| if compare_values(a, b) { a } else { b })
        .unwrap_or(MbValue::none())
}

/// max(iterable) or max(a, b) — return the largest value.
pub fn mb_max(args: MbValue) -> MbValue {
    if raise_if_not_iterable(args) {
        return MbValue::none();
    }
    let items = extract_items(args);
    if items.is_empty() {
        // CPython: max(()) raises ValueError (no silent None default).
        raise_value_error("max() iterable argument is empty".to_string());
        return MbValue::none();
    }
    items
        .into_iter()
        .reduce(|a, b| if compare_values(b, a) { a } else { b })
        .unwrap_or(MbValue::none())
}

/// sum(iterable) — sum all numeric values.
pub fn mb_sum(args: MbValue) -> MbValue {
    sum_from(args, MbValue::from_int(0))
}

/// One sum() fold step. mb_add covers numerics, sequences, and the stdlib
/// handle types; instances dispatch __add__/__radd__; anything mb_add
/// declines (returns None without a pending exception) is a TypeError.
fn sum_fold_add(acc: MbValue, item: MbValue) -> MbValue {
    let r = mb_add(acc, item);
    if !r.is_none() || eval_pending() {
        return r;
    }
    for (recv, arg, dunder) in [(acc, item, "__add__"), (item, acc, "__radd__")] {
        if let Some(ptr) = recv.as_ptr() {
            if let ObjData::Instance { ref class_name, .. } = unsafe { &(*ptr).data } {
                let method = super::class::lookup_method(class_name, dunder);
                if !method.is_none() {
                    let name = MbValue::from_ptr(MbObject::new_str(dunder.to_string()));
                    let call_args = MbValue::from_ptr(MbObject::new_list(vec![arg]));
                    let out = super::class::mb_call_method(recv, name, call_args);
                    if !out.is_none() || eval_pending() {
                        return out;
                    }
                }
            }
        }
    }
    raise_type_error(format!(
        "unsupported operand type(s) for +: '{}' and '{}'",
        value_type_name(acc),
        value_type_name(item)
    ));
    MbValue::none()
}

/// One Neumaier (improved Kahan–Babuška) compensated-summation step.
/// CPython 3.12's `sum()` uses this for the float path so that, e.g.,
/// `sum([1.0, 1e101, 1.0, -1e101]) == 2.0` and `sum([0.1]*10) == 1.0`.
#[inline]
fn neumaier_step(ftotal: f64, c: f64, x: f64) -> (f64, f64) {
    let t = ftotal + x;
    let c = if ftotal.abs() >= x.abs() {
        c + ((ftotal - t) + x)
    } else {
        c + ((x - t) + ftotal)
    };
    (t, c)
}

fn sum_from(args: MbValue, start: MbValue) -> MbValue {
    if raise_if_not_iterable(args) {
        return MbValue::none();
    }
    let items = extract_items(args);
    // Fast path: numeric start and all-numeric items (i128 accumulator so
    // inline-int sums can never overflow; result normalizes to BigInt).
    let numeric_start = start.as_int_pyint().is_some() || start.is_float();
    if numeric_start
        && items
            .iter()
            .all(|v| v.as_int_pyint().is_some() || v.is_float())
    {
        let mut total: i128 = start.as_int_pyint().unwrap_or(0) as i128;
        let mut is_float = start.is_float();
        let mut ftotal: f64 = start.as_float().unwrap_or(0.0);
        // Neumaier compensation term — only meaningful once is_float is set.
        let mut c: f64 = 0.0;
        for item in &items {
            if let Some(i) = item.as_int_pyint() {
                if is_float {
                    let (nt, nc) = neumaier_step(ftotal, c, i as f64);
                    ftotal = nt;
                    c = nc;
                } else {
                    total += i as i128;
                }
            } else if let Some(f) = item.as_float() {
                if !is_float {
                    ftotal = total as f64;
                    is_float = true;
                    c = 0.0;
                }
                let (nt, nc) = neumaier_step(ftotal, c, f);
                ftotal = nt;
                c = nc;
            }
        }
        if is_float {
            return MbValue::from_float(ftotal + c);
        }
        if let Ok(small) = i64::try_from(total) {
            return super::bigint_ops::int_from_i64(small);
        }
        return super::bigint_ops::bigint_from_i128(total);
    }
    // Generic fold: covers BigInt, str-in-list TypeError, instances, etc.
    let mut acc = start;
    for item in items {
        acc = sum_fold_add(acc, item);
        if eval_pending() {
            return MbValue::none();
        }
    }
    acc
}

/// sorted(iterable, reverse=False) — return a new sorted list.
#[inline]
fn stable_order_for_reverse(order: std::cmp::Ordering, do_reverse: bool) -> std::cmp::Ordering {
    if do_reverse {
        order.reverse()
    } else {
        order
    }
}

pub fn mb_sorted(iterable: MbValue, reverse: MbValue) -> MbValue {
    if iterable.as_ptr().is_none()
        && !super::iter::mb_is_iterator_handle(iterable)
        && !super::generator::is_known_generator(iterable)
    {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "'{}' object is not iterable",
                value_type_name(iterable)
            ))),
        );
        return MbValue::none();
    }
    let mut items = extract_items(iterable);
    let do_reverse = reverse.as_bool() == Some(true) || reverse.as_int() == Some(1);

    // Type-specialized sort: detect homogeneous lists and use a cheaper comparator.
    // For all-int lists (common: sorted(range(N))), use direct i64 comparison
    // instead of the generic mb_value_cmp which converts int→f64 per comparison.
    if !items.is_empty() {
        let first_is_int = items[0].is_int();
        let first_is_float = !first_is_int && items[0].is_float();

        if first_is_int && items.iter().all(|v| v.is_int()) {
            items.sort_by(|a, b| {
                stable_order_for_reverse(
                    a.as_int_unchecked().cmp(&b.as_int_unchecked()),
                    do_reverse,
                )
            });
        } else if (first_is_int || first_is_float)
            && items.iter().all(|v| v.is_int() || v.is_float())
        {
            items.sort_by(|a, b| {
                let af = a.as_int().map(|i| i as f64).or(a.as_float()).unwrap_or(0.0);
                let bf = b.as_int().map(|i| i as f64).or(b.as_float()).unwrap_or(0.0);
                stable_order_for_reverse(
                    af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal),
                    do_reverse,
                )
            });
        } else {
            items.sort_by(|a, b| stable_order_for_reverse(mb_value_cmp(*a, *b), do_reverse));
        }
    }

    // Items are borrowed from the source container via extract_items — retain them.
    MbValue::from_ptr(MbObject::new_list_borrowed(items))
}

///// Public wrapper for cross-module sorting.
pub fn mb_value_cmp_pub(a: MbValue, b: MbValue) -> std::cmp::Ordering {
    mb_value_cmp(a, b)
}

/// Public wrapper for cross-module callable resolution.
pub fn resolve_callable_pub(func: MbValue) -> Option<usize> {
    resolve_callable(func)
}

/// General-purpose comparison for sorting: int/float → numeric, str → lexicographic, tuple → element-wise.
fn mb_value_cmp(a: MbValue, b: MbValue) -> std::cmp::Ordering {
    // Fast path: both ints — direct i64 comparison (no float conversion).
    if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
        return ai.cmp(&bi);
    }
    // Try numeric comparison (int/float mix)
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    if let (Some(af), Some(bf)) = (af, bf) {
        return af.partial_cmp(&bf).unwrap_or(std::cmp::Ordering::Equal);
    }
    // Try pointer-based comparison (str, tuple, list, instances with __lt__)
    unsafe {
        if let (Some(pa), Some(pb)) = (a.as_ptr(), b.as_ptr()) {
            match (&(*pa).data, &(*pb).data) {
                (ObjData::Str(ref sa), ObjData::Str(ref sb)) => return sa.cmp(sb),
                (ObjData::Tuple(ref ta), ObjData::Tuple(ref tb)) => {
                    for (ea, eb) in ta.iter().zip(tb.iter()) {
                        let cmp = mb_value_cmp(*ea, *eb);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    return ta.len().cmp(&tb.len());
                }
                (ObjData::List(ref la), ObjData::List(ref lb)) => {
                    let la = super::list_ops::retained_list_snapshot(la);
                    let lb = super::list_ops::retained_list_snapshot(lb);
                    for (ea, eb) in la.iter().zip(lb.iter()) {
                        let cmp = mb_value_cmp(*ea, *eb);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    return la.len().cmp(&lb.len());
                }
                (ObjData::Instance { .. }, _) | (_, ObjData::Instance { .. }) => {
                    // Dispatch __lt__ in both directions so sorted()/list.sort()
                    // on user-class values without a key= respects rich comparison.
                    if mb_values_lt(a, b) {
                        return std::cmp::Ordering::Less;
                    }
                    if mb_values_lt(b, a) {
                        return std::cmp::Ordering::Greater;
                    }
                    return std::cmp::Ordering::Equal;
                }
                _ => {}
            }
        }
    }
    // Equal values — including equal-but-unorderable ones like None — compare
    // Equal without ever invoking `<`, so an element-wise tuple/list sort whose
    // heads are equal (`sorted([(None, 2), (None, 1)])`) doesn't choke on
    // `None < None`. Only a genuinely differing, unorderable pair falls through
    // to mb_values_lt, which raises CPython's exact unorderable-types message.
    if mb_values_eq(a, b) {
        return std::cmp::Ordering::Equal;
    }
    if mb_values_lt(b, a) {
        return std::cmp::Ordering::Greater;
    }
    if mb_values_lt(a, b) {
        return std::cmp::Ordering::Less;
    }
    std::cmp::Ordering::Equal
}

/// Helper: extract items from a list, tuple, set, frozenset, or string (as chars).
pub fn extract_items(val: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::List(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::Tuple(items) => return items.clone(),
                ObjData::Set(ref lock) => return lock.read().unwrap().to_vec(),
                ObjData::FrozenSet(items) => return items.clone(),
                ObjData::Str(s) => {
                    // Class-body enum classes iterate canonical members.
                    if let Some(members) = super::stdlib::enum_class::class_canonical_members(s) {
                        return members;
                    }
                    // Iterate over characters like Python does: "abc" → ['a', 'b', 'c']
                    return s
                        .chars()
                        .map(|c| MbValue::from_ptr(MbObject::new_str(c.to_string())))
                        .collect();
                }
                ObjData::Dict(ref lock) => {
                    // Iterating a dict yields its keys.
                    return lock
                        .read()
                        .unwrap()
                        .keys()
                        .map(|k| super::dict_ops::dict_key_to_mbvalue(k))
                        .collect();
                }
                // bytes/bytearray iterate as their integer byte values.
                ObjData::Bytes(ref data) => {
                    return data.iter().map(|&b| MbValue::from_int(b as i64)).collect();
                }
                ObjData::ByteArray(ref lock) => {
                    return lock
                        .read()
                        .unwrap()
                        .iter()
                        .map(|&b| MbValue::from_int(b as i64))
                        .collect();
                }
                ObjData::Instance { .. } => {
                    if let Some(items) = super::dict_ops::dict_view_elements(val) {
                        return items;
                    }
                    if let Some((_base, payload)) =
                        super::class::builtin_data_payload_if_unoverridden(val, "__iter__")
                    {
                        return extract_items(payload);
                    }
                    // User iterable: go through iterator protocol.
                    // Fall through to the iterator handling below.
                }
                _ => return vec![],
            }
        }
    }
    // Handle iterator handles (generators, user iterators, iter() results)
    // via the iterator protocol.
    let iter_handle = super::iter::mb_iter(val);
    if iter_handle.is_none() {
        return vec![];
    }
    // Fast path: drain the iterator batch (avoids per-element HashMap lookups).
    if let Some(items) = super::iter::drain_iter_to_vec(iter_handle) {
        return items;
    }
    // Fallback: standard iterator protocol.
    let mut items = Vec::new();
    // A generator/iterator that raises mid-iteration leaves a pending
    // exception; stop and let the caller (set()/list()/heapq.merge/sorted/…)
    // propagate it rather than swallowing it as end-of-iteration. StopIteration
    // is the normal exhaustion signal and must not be treated as an error.
    let raised = || {
        super::exception::mb_has_exception().as_bool() == Some(true)
            && super::exception::current_exception_type().as_deref() != Some("StopIteration")
    };
    loop {
        if super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        if raised() {
            break;
        }
        let item = super::iter::mb_next(iter_handle);
        if raised() {
            break;
        }
        if item.is_none() && super::iter::mb_has_next(iter_handle).as_bool() == Some(false) {
            break;
        }
        items.push(item);
    }
    items
}

/// Helper: compare_values returns true if a < b.
fn compare_values(a: MbValue, b: MbValue) -> bool {
    // Route through mb_values_lt so min()/max() honors every type the
    // `<` operator does — including Instance values with __lt__ and
    // tuples/lists that compare element-wise.
    mb_values_lt(a, b)
}

// ── Conversion builtins (#378) ──

/// repr(value) — return string representation.
pub fn mb_repr(val: MbValue) -> MbValue {
    if let Some(name) = pep695_display_name(val) {
        return MbValue::from_ptr(MbObject::new_str(name));
    }
    let s = if let Some(i) = val.as_int() {
        // UUID handles render as `UUID('<canonical>')` (CPython parity)
        // instead of the i64 handle ID (#1475).
        if super::stdlib::uuid_mod::is_uuid_handle(i as u64) {
            let canon_val = super::stdlib::uuid_mod::mb_uuid_str(val);
            let canon = if let Some(ptr) = canon_val.as_ptr() {
                unsafe {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        s.clone()
                    } else {
                        String::new()
                    }
                }
            } else {
                String::new()
            };
            return MbValue::from_ptr(MbObject::new_str(format!("UUID('{canon}')")));
        }
        // Decimal / Fraction handles render as their constructor reprs (#2129).
        if super::stdlib::decimal_mod::is_decimal_handle(i as u64) {
            return super::stdlib::decimal_mod::mb_decimal_repr(val);
        }
        if super::stdlib::fractions_mod::is_fraction_handle(i as u64) {
            return super::stdlib::fractions_mod::mb_fraction_repr(val);
        }
        // Named itertools iterators (repeat, …) have a CPython-style repr
        // rather than their raw handle id.
        if let Some(r) = super::iter::mb_iter_repr(val) {
            return MbValue::from_ptr(MbObject::new_str(r));
        }
        if super::async_rt::is_known_coroutine(val) {
            return MbValue::from_ptr(MbObject::new_str(format!(
                "<coroutine object at 0x{:x}>",
                i
            )));
        }
        format!("{i}")
    } else if let Some(f) = val.as_float() {
        super::string_ops::python_float_repr(f)
    } else if let Some(b) = val.as_bool() {
        (if b { "True" } else { "False" }).to_string()
    } else if val.is_none() {
        "None".to_string()
    } else if val.is_not_implemented() {
        "NotImplemented".to_string()
    } else if val.is_ellipsis() {
        "Ellipsis".to_string()
    } else if let Some(addr) = val.as_func() {
        // TAG_FUNC: produce CPython-style `<function NAME at 0xADDR>`.
        // The FUNC_NAMES registry is primed at module init for every
        // user-defined function (and lambdas register their own names),
        // so look up the name; fall back to `<lambda>` for anonymous
        // values that never registered.
        let name_val = super::closure::mb_func_get_name(val);
        let name = if let Some(ptr) = name_val.as_ptr() {
            unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    s.clone()
                } else {
                    "<lambda>".to_string()
                }
            }
        } else {
            "<lambda>".to_string()
        };
        format!("<function {name} at 0x{addr:x}>")
    } else if let Some(ptr) = val.as_ptr() {
        if let Some(codepoints) = super::string_ops::surrogate_codepoints(val) {
            return MbValue::from_ptr(MbObject::new_str(
                super::string_ops::repr_string_from_codepoints(&codepoints),
            ));
        }
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => super::string_ops::repr_string(s),
                ObjData::Bytes(data) => format!("b{}", format_bytes_inner(data)),
                ObjData::ByteArray(ref lock) => {
                    let data = lock.read().unwrap();
                    format!("bytearray(b{})", format_bytes_inner(&data))
                }
                ObjData::Tuple(items) => {
                    let mut parts = Vec::with_capacity(items.len());
                    for item in items {
                        let r = mb_repr(*item);
                        let s = r
                            .as_ptr()
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_else(|| "None".to_string());
                        parts.push(s);
                    }
                    let body = parts.join(", ");
                    if items.len() == 1 {
                        format!("({body},)")
                    } else {
                        format!("({body})")
                    }
                }
                ObjData::Instance {
                    class_name,
                    ref fields,
                } => {
                    if matches!(class_name.as_str(), "GenericAlias" | "types.GenericAlias") {
                        if let Some(s) = generic_alias_repr_string(class_name, fields) {
                            return MbValue::from_ptr(MbObject::new_str(s));
                        }
                    }
                    if class_name == "coroutine_wrapper" {
                        return MbValue::from_ptr(MbObject::new_str(format!(
                            "<coroutine_wrapper object at 0x{:x}>",
                            ptr as usize
                        )));
                    }
                    if class_name == "UnionType" {
                        return MbValue::from_ptr(MbObject::new_str(union_type_repr(val)));
                    }
                    // zoneinfo.ZoneInfo(key='America/New_York')
                    if class_name == "ZoneInfo" {
                        if let Some(k) = fields.read().ok().and_then(|f| f.get("key").copied()) {
                            let key_repr = mb_repr(k);
                            if let Some(p) = key_repr.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    return MbValue::from_ptr(MbObject::new_str(format!(
                                        "zoneinfo.ZoneInfo(key={s})"
                                    )));
                                }
                            }
                        }
                    }
                    // A bare object() instance reprs as `<object object at 0xADDR>`
                    // (CPython's object.__repr__). User subclasses that don't
                    // override __repr__ get the same shape with their class name,
                    // handled by the generic fallback below.
                    if class_name == "object" {
                        return MbValue::from_ptr(MbObject::new_str(format!(
                            "<object object at 0x{:x}>",
                            ptr as usize
                        )));
                    }
                    // Type objects (make_type_object: a "type"-class instance
                    // carrying __name__): repr is `<class 'name'>`, matching
                    // CPython and the print path. Without this, `repr(int)` /
                    // `repr(defaultdict(int))`'s factory showed `<type instance>`.
                    if class_name == "type" {
                        let name = fields
                            .read()
                            .ok()
                            .and_then(|f| f.get("__name__").copied())
                            .and_then(|v| v.as_ptr())
                            .and_then(|p| {
                                if let ObjData::Str(ref s) = (*p).data {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            });
                        if let Some(name) = name {
                            return MbValue::from_ptr(MbObject::new_str(format!(
                                "<class '{name}'>"
                            )));
                        }
                    }
                    // Class-body enum member without a user __repr__:
                    // repr(Color.RED) → "<Color.RED: 1>".
                    if let Some(s) = super::stdlib::enum_class::member_repr(val) {
                        return MbValue::from_ptr(MbObject::new_str(s));
                    }
                    // weakref.ref CPython-style repr: `<weakref at 0x..; to
                    // 'CLASS' at 0x..>`, naming the referent's class (gh-99184).
                    if class_name == "ReferenceType" {
                        let r = super::stdlib::weakref_mod::reference_repr(val);
                        if !r.is_none() {
                            return r;
                        }
                    }
                    // Counter has its own CPython-style repr: sort by count
                    // descending, ties by insertion order. (#1638)
                    if class_name == "collections.Counter" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::collections_mod::counter_repr(val),
                        ));
                    }
                    // statistics.NormalDist value-type repr round-trips the
                    // constructor: `NormalDist(mu=37.5, sigma=5.625)`.
                    if class_name == "NormalDist" {
                        if let Some(s) = super::stdlib::statistics_mod::normaldist_repr(val) {
                            return MbValue::from_ptr(MbObject::new_str(s));
                        }
                    }
                    // defaultdict / deque also need CPython-style repr. (#1640)
                    if class_name == "collections.defaultdict" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::collections_mod::defaultdict_repr(val),
                        ));
                    }
                    if class_name == "collections.deque" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::collections_mod::deque_repr(val),
                        ));
                    }
                    if class_name == "collections.OrderedDict" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::collections_mod::ordereddict_repr(val),
                        ));
                    }
                    if class_name == "mappingproxy" {
                        let mapping = fields.read().unwrap().get("_mapping").copied();
                        if let Some(mapping) = mapping {
                            let inner = mb_repr(mapping);
                            if let Some(ip) = inner.as_ptr() {
                                if let ObjData::Str(ref s) = (*ip).data {
                                    return MbValue::from_ptr(MbObject::new_str(format!(
                                        "mappingproxy({s})"
                                    )));
                                }
                            }
                        }
                    }
                    // re.Match / re.Pattern CPython-style repr. (#1642)
                    if class_name == "re.Match" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::re_mod::match_repr(val),
                        ));
                    }
                    if class_name == "re.Pattern" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::re_mod::pattern_repr(val),
                        ));
                    }
                    // datetime / timedelta CPython-style repr. (#1644)
                    if class_name == "datetime.datetime" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::datetime_mod::datetime_repr(val),
                        ));
                    }
                    if class_name == "datetime.timedelta" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::datetime_mod::timedelta_repr(val),
                        ));
                    }
                    if class_name == "datetime.time" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::datetime_mod::time_repr(val),
                        ));
                    }
                    if class_name == "datetime.timezone" {
                        return MbValue::from_ptr(MbObject::new_str(
                            super::stdlib::datetime_mod::timezone_repr(val),
                        ));
                    }
                    // namedtuple instances: Point(x=1, y=2). (#1648)
                    if let Some(s) = super::stdlib::collections_mod::namedtuple_repr(val) {
                        return MbValue::from_ptr(MbObject::new_str(s));
                    }
                    if class_name == "slice" {
                        let guard = fields.read().unwrap();
                        let s = guard.get("start").copied().unwrap_or(MbValue::none());
                        let e = guard.get("stop").copied().unwrap_or(MbValue::none());
                        let st = guard.get("step").copied().unwrap_or(MbValue::none());
                        drop(guard);
                        let part = |v: MbValue| -> String {
                            let r = mb_repr(v);
                            if let Some(p) = r.as_ptr() {
                                if let ObjData::Str(ref s) = (*p).data {
                                    return s.clone();
                                }
                            }
                            "None".to_string()
                        };
                        return MbValue::from_ptr(MbObject::new_str(format!(
                            "slice({}, {}, {})",
                            part(s),
                            part(e),
                            part(st)
                        )));
                    }
                    // Exception classes default to `ClassName(repr(args[0]))`
                    // when no custom __repr__ is defined — mirrors CPython's
                    // BaseException.__repr__.
                    let is_exception_class =
                        super::exception::is_subclass_of(class_name, "Exception")
                            || class_name == "BaseException";
                    // __repr__ dunder dispatch
                    let repr_method = super::class::lookup_method(class_name, "__repr__");
                    if !repr_method.is_none() {
                        let result = super::class::mb_call_method1(repr_method, val);
                        if let Some(ptr) = result.as_ptr() {
                            if let ObjData::Str(ref s) = (*ptr).data {
                                return MbValue::from_ptr(MbObject::new_str(s.clone()));
                            }
                        }
                    }
                    if let Some((base, payload)) =
                        super::class::builtin_data_payload_if_unoverridden(val, "__repr__")
                    {
                        // set/frozenset subclasses prefix the class name
                        // onto the literal body — `ClassName({...})` /
                        // `ClassName()` when empty — matching CPython's
                        // set.__repr__ non-exact-type special case
                        // (dict/list/tuple subclasses stay unprefixed
                        // per CPython). (#1051)
                        if base == "set" || base == "frozenset" {
                            if let Some(items) = set_family_payload_items(payload) {
                                if items.is_empty() {
                                    return MbValue::from_ptr(MbObject::new_str(format!(
                                        "{class_name}()"
                                    )));
                                }
                                let mut body = String::new();
                                for (i, item) in items.iter().enumerate() {
                                    if i > 0 {
                                        body.push_str(", ");
                                    }
                                    let r = mb_repr(*item);
                                    if let Some(rp) = r.as_ptr() {
                                        if let ObjData::Str(ref s) = (*rp).data {
                                            body.push_str(s);
                                        }
                                    }
                                }
                                return MbValue::from_ptr(MbObject::new_str(format!(
                                    "{class_name}({{{body}}})"
                                )));
                            }
                        }
                        return mb_repr(payload);
                    }
                    // PEP 557: dataclass synthesized __repr__ —
                    // `Cls(f1=v1, f2=v2)` over repr=True fields in declaration
                    // order. Only reached when no user __repr__ is defined.
                    if let Some(s) = super::stdlib::dataclasses_mod::dc_repr_string(val, class_name)
                    {
                        return MbValue::from_ptr(MbObject::new_str(s));
                    }
                    if class_name == "SimpleNamespace"
                        || super::class::check_class_hierarchy(class_name, "SimpleNamespace")
                    {
                        // CPython renders `namespace(field=repr(value), ...)` in
                        // INSERTION order (tracked in the hidden `__ns_order__`
                        // list), with a direct self-reference shown as
                        // `namespace(...)`. Subclasses use their class name as
                        // the repr prefix.
                        let self_ptr = val.as_ptr();
                        let guard = fields.read().unwrap();
                        // Preferred order from `__ns_order__`; any field missing
                        // from it (defensive) is appended in map order.
                        let order: Vec<String> = guard
                            .get("__ns_order__")
                            .and_then(|v| v.as_ptr())
                            .map(|p| unsafe {
                                if let ObjData::List(ref lk) = (*p).data {
                                    lk.read()
                                        .unwrap()
                                        .iter()
                                        .filter_map(|k| {
                                            k.as_ptr().and_then(|kp| {
                                                if let ObjData::Str(ref s) = (*kp).data {
                                                    Some(s.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                        })
                                        .collect()
                                } else {
                                    Vec::new()
                                }
                            })
                            .unwrap_or_default();
                        let mut keys: Vec<String> = order
                            .iter()
                            .filter(|k| guard.contains_key(*k))
                            .cloned()
                            .collect();
                        for k in guard.keys() {
                            if k != "__ns_order__" && !keys.contains(k) {
                                keys.push(k.clone());
                            }
                        }
                        let repr_prefix = if class_name == "SimpleNamespace" {
                            "namespace"
                        } else {
                            class_name.as_str()
                        };
                        let parts: Vec<String> = keys
                            .iter()
                            .filter_map(|k| {
                                let v = *guard.get(k)?;
                                Some(if v.as_ptr().is_some() && v.as_ptr() == self_ptr {
                                    format!("{k}={repr_prefix}(...)")
                                } else {
                                    let r = mb_repr(v);
                                    let rs = r
                                        .as_ptr()
                                        .and_then(|p| {
                                            if let ObjData::Str(ref s) = (*p).data {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_default();
                                    format!("{k}={rs}")
                                })
                            })
                            .collect();
                        drop(guard);
                        return MbValue::from_ptr(MbObject::new_str(format!(
                            "{repr_prefix}({})",
                            parts.join(", ")
                        )));
                    }
                    // PEP 654 ExceptionGroup repr: `ClassName('message', [child
                    // reprs])` — recursive over the `.exceptions` tuple. Gated to
                    // EG-shaped instances (subclass of BaseExceptionGroup with an
                    // `exceptions` tuple field) so plain exceptions are unaffected.
                    if super::exception::is_subclass_of(class_name, "BaseExceptionGroup")
                        || super::exception::is_subclass_of(class_name, "ExceptionGroup")
                        || class_name == "BaseExceptionGroup"
                        || class_name == "ExceptionGroup"
                    {
                        let guard = fields.read().unwrap();
                        let msg_v = guard.get("message").copied();
                        let exc_v = guard.get("exceptions").copied();
                        drop(guard);
                        if let (Some(msg_v), Some(exc_v)) = (msg_v, exc_v) {
                            let children: Option<Vec<MbValue>> = exc_v.as_ptr().and_then(|p| {
                                if let ObjData::Tuple(ref t) = (*p).data {
                                    Some(t.clone())
                                } else {
                                    None
                                }
                            });
                            if let Some(children) = children {
                                let repr_s = |v: MbValue| -> String {
                                    mb_repr(v)
                                        .as_ptr()
                                        .and_then(|p| {
                                            if let ObjData::Str(ref s) = (*p).data {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| "None".to_string())
                                };
                                let msg_r = repr_s(msg_v);
                                let kids: Vec<String> =
                                    children.iter().map(|c| repr_s(*c)).collect();
                                return MbValue::from_ptr(MbObject::new_str(format!(
                                    "{class_name}({msg_r}, [{}])",
                                    kids.join(", ")
                                )));
                            }
                        }
                    }
                    if is_exception_class {
                        // CPython repr: ClassName(repr(arg0), repr(arg1), ...)
                        // over the full `args` tuple. Falls back to the
                        // legacy `message` path when `args` isn't present
                        // (defensive — exception.rs always populates it). (#1652)
                        let guard = fields.read().unwrap();
                        let args_val = guard.get("args").copied();
                        let msg_val = guard.get("message").copied();
                        drop(guard);
                        let arg_items: Option<Vec<MbValue>> =
                            args_val.and_then(|v| v.as_ptr()).and_then(|p| {
                                if let ObjData::Tuple(ref items) = (*p).data {
                                    Some(items.clone())
                                } else {
                                    None
                                }
                            });
                        if let Some(items) = arg_items {
                            if items.is_empty() {
                                return MbValue::from_ptr(MbObject::new_str(format!(
                                    "{class_name}()"
                                )));
                            }
                            let parts: Vec<String> = items
                                .iter()
                                .map(|v| {
                                    let r = mb_repr(*v);
                                    r.as_ptr()
                                        .and_then(|p| {
                                            if let ObjData::Str(ref s) = (*p).data {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .unwrap_or_else(|| "None".to_string())
                                })
                                .collect();
                            return MbValue::from_ptr(MbObject::new_str(format!(
                                "{class_name}({})",
                                parts.join(", ")
                            )));
                        }
                        if let Some(msg) = msg_val {
                            let inner = mb_repr(msg);
                            if let Some(ptr) = inner.as_ptr() {
                                if let ObjData::Str(ref s) = (*ptr).data {
                                    return MbValue::from_ptr(MbObject::new_str(format!(
                                        "{class_name}({s})"
                                    )));
                                }
                            }
                        }
                        return MbValue::from_ptr(MbObject::new_str(format!("{class_name}()")));
                    }
                    // functools.partial repr: `functools.partial(func, a, b, k=v)`.
                    if class_name == "functools.partial" {
                        return super::stdlib::functools_mod::mb_functools_partial_repr(val);
                    }
                    super::string_ops::value_to_string(val)
                }
                _ => super::string_ops::value_to_string(val),
            }
        }
    } else {
        String::new()
    };
    MbValue::from_ptr(MbObject::new_str(s))
}

/// Raise `ZeroDivisionError: 0.0 cannot be raised to a negative power` — the
/// CPython error for `0 ** -n` / `0.0 ** -n`.
fn raise_zero_neg_pow() -> MbValue {
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "0.0 cannot be raised to a negative power".to_string(),
        )),
    );
    MbValue::none()
}

/// pow(base, exp) — power operator.
pub fn mb_pow(base: MbValue, exp: MbValue) -> MbValue {
    let base = int_enum_like_value(base).unwrap_or(base);
    let exp = int_enum_like_value(exp).unwrap_or(exp);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((nb, ne)) = numeric_subclass_operands(base, exp, "__pow__") {
        return mb_pow(nb, ne);
    }
    if let Some(r) = numeric_handle_binop("**", base, exp) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("**", base, exp) {
        return r;
    }
    // Complex base: route through complex pow so `complex(3,4) ** 2` works.
    // Either operand being `ObjData::Complex` promotes the whole op to
    // complex. (#1256 sub-priority 3 — complex arithmetic)
    if is_complex_obj(base) || is_complex_obj(exp) {
        if let (Some((br, bi)), Some((er, ei))) = (as_complex_pair(base), as_complex_pair(exp)) {
            // Integer exponent on a complex base — exact via repeated
            // multiplication (avoids polar-form precision loss for the
            // common `c**2`, `c**3` cases).
            if ei == 0.0 && er.fract() == 0.0 && er.abs() < (i32::MAX as f64) {
                let n = er as i32;
                let (mut rr, mut ri) = if n < 0 {
                    // 1/(a+bi) = (a-bi)/(a²+b²)
                    let denom = br * br + bi * bi;
                    if denom == 0.0 {
                        return MbValue::none();
                    }
                    (br / denom, -bi / denom)
                } else {
                    (br, bi)
                };
                let count = n.unsigned_abs();
                if count == 0 {
                    return MbValue::from_ptr(MbObject::new_complex(1.0, 0.0));
                }
                let (sr, si) = (rr, ri);
                for _ in 1..count {
                    let new_r = rr * sr - ri * si;
                    let new_i = rr * si + ri * sr;
                    rr = new_r;
                    ri = new_i;
                }
                return MbValue::from_ptr(MbObject::new_complex(rr, ri));
            }
            // General complex pow via polar form: c**e = exp(e * log c)
            // where log(a+bi) = ln(r) + i*θ.
            let r = (br * br + bi * bi).sqrt();
            if r == 0.0 {
                return MbValue::from_ptr(MbObject::new_complex(0.0, 0.0));
            }
            let theta = bi.atan2(br);
            let ln_r = r.ln();
            // (er + ei*i) * (ln_r + theta*i) = (er*ln_r - ei*theta) + (er*theta + ei*ln_r)i
            let real_part = er * ln_r - ei * theta;
            let imag_part = er * theta + ei * ln_r;
            let mag = real_part.exp();
            return MbValue::from_ptr(MbObject::new_complex(
                mag * imag_part.cos(),
                mag * imag_part.sin(),
            ));
        }
        return MbValue::none();
    }
    match (base.as_int(), exp.as_int()) {
        (Some(b), Some(e)) => {
            if e >= 0 {
                // Promote out-of-payload results to BigInt (2**64 must not
                // wrap to 0 in the 48-bit NaN-boxed int payload).
                use num_bigint::BigInt;
                let big = BigInt::from(b).pow(e as u32);
                let fits = big >= BigInt::from(-(1i64 << 47)) && big < BigInt::from(1i64 << 47);
                if fits {
                    use num_traits::ToPrimitive;
                    MbValue::from_int(big.to_i64().unwrap_or(0))
                } else {
                    super::bigint_ops::bigint_from_big(big)
                }
            } else {
                // 0 ** -n: zero to a negative power has no finite value.
                if b == 0 {
                    return raise_zero_neg_pow();
                }
                MbValue::from_float((b as f64).powi(e as i32))
            }
        }
        _ => {
            let bf = base.as_int().map(|i| i as f64).or(base.as_float());
            let ef = exp.as_int().map(|i| i as f64).or(exp.as_float());
            match (bf, ef) {
                (Some(b), Some(e)) => {
                    // 0.0 ** -n raises ZeroDivisionError in CPython rather than
                    // returning inf.
                    if b == 0.0 && e < 0.0 {
                        return raise_zero_neg_pow();
                    }
                    MbValue::from_float(b.powf(e))
                }
                _ => {
                    super::exception::mb_raise(
                        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
                        MbValue::from_ptr(MbObject::new_str(
                            "unsupported operand type(s) for pow()".to_string(),
                        )),
                    );
                    MbValue::none()
                }
            }
        }
    }
}

/// Modular multiplicative inverse of `a` modulo `m` via the extended
/// Euclidean algorithm, returning `x` in `[0, m)` such that `a*x ≡ 1 (mod m)`.
/// Returns `None` if `gcd(a, m) != 1` (CPython raises `ValueError` here).
fn mod_inverse_i128(a: i128, m: i128) -> Option<i128> {
    if m == 0 {
        return None;
    }
    let m_abs = m.abs();
    let (mut old_r, mut r) = (a.rem_euclid(m_abs), m_abs);
    let (mut old_s, mut s) = (1i128, 0i128);
    while r != 0 {
        let q = old_r / r;
        let nr = old_r - q * r;
        old_r = r;
        r = nr;
        let ns = old_s - q * s;
        old_s = s;
        s = ns;
    }
    if old_r != 1 {
        return None;
    }
    Some(old_s.rem_euclid(m_abs))
}

/// pow(base, exp, mod) — modular exponentiation.
/// CPython 3.8+: when `exp < 0`, computes the modular inverse of `base`
/// then raises it to `-exp`; valid only when `gcd(base, mod) == 1`.
pub fn mb_pow_mod(base: MbValue, exp: MbValue, modulus: MbValue) -> MbValue {
    match (
        base.as_int_pyint(),
        exp.as_int_pyint(),
        modulus.as_int_pyint(),
    ) {
        (Some(b), Some(e), Some(m)) => {
            if m == 0 {
                raise_value_error("pow() 3rd argument cannot be 0".to_string());
                return MbValue::none();
            }
            let m128 = m as i128;
            let (mut base_val, exp_pos): (i128, u64) = if e < 0 {
                match mod_inverse_i128(b as i128, m128) {
                    Some(inv) => (inv, (-e) as u64),
                    None => {
                        raise_value_error(
                            "base is not invertible for the given modulus".to_string(),
                        );
                        return MbValue::none();
                    }
                }
            } else {
                ((b as i128).rem_euclid(m128), e as u64)
            };
            let mut result: i128 = 1 % m128;
            let mut exp_val = exp_pos;
            while exp_val > 0 {
                if exp_val & 1 == 1 {
                    result = (result * base_val).rem_euclid(m128);
                }
                exp_val >>= 1;
                base_val = (base_val * base_val).rem_euclid(m128);
            }
            MbValue::from_int(result as i64)
        }
        _ => {
            raise_type_error(
                "pow() 3rd argument not allowed unless all arguments are integers".to_string(),
            );
            MbValue::none()
        }
    }
}

/// int(value, base) — convert string to integer with given base; raises ValueError on bad input.
pub fn mb_int_base(val: MbValue, base: MbValue) -> MbValue {
    // base accepts any SupportsIndex (int / bool / object with __index__),
    // e.g. `int("ff", Indexable(16))`.
    let Some(base_int) = resolve_index_value(base) else {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "int() base must be an integer".to_string(),
            )),
        );
        return MbValue::none();
    };
    // CPython: base is 0 (prefix auto-detect) or 2..=36; anything else raises
    // ValueError. Rust's from_str_radix panics on a radix outside 2..=36, and a
    // negative base would wrap when cast to u32 — so reject up front.
    if base_int != 0 && !(2..=36).contains(&base_int) {
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "int() base must be >= 2 and <= 36, or 0".to_string(),
            )),
        );
        return MbValue::none();
    }
    let base_num = base_int as u32;
    if let Some(ptr) = val.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                let full = s.clone();
                let trimmed = s.trim();
                // Returns (sign, effective_base, stripped_digits) on a valid
                // parse so the caller can build an inline i64 or, on overflow,
                // a heap BigInt. Validation of digits is deferred to the
                // numeric conversion step (i64/BigInt both reject bad digits).
                let try_parse = |t: &str| -> Option<(i64, u32, String)> {
                    // Pull off optional sign first so the radix prefix
                    // detection sees `0x`/`0o`/`0b` rather than `-0x`.
                    let (sign, rest) = match t.as_bytes().first() {
                        Some(b'-') => (-1i64, &t[1..]),
                        Some(b'+') => (1, &t[1..]),
                        _ => (1, t),
                    };
                    // base == 0: auto-detect from prefix (CPython behaviour).
                    // `0x`/`0X` → 16, `0o`/`0O` → 8, `0b`/`0B` → 2, else 10.
                    // Without a prefix the value cannot have leading zeros
                    // (CPython raises on `int("010", 0)`); we replicate that.
                    let (effective_base, digits, prefix_stripped) = if base_num == 0 {
                        if let Some(d) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X"))
                        {
                            (16u32, d, true)
                        } else if let Some(d) =
                            rest.strip_prefix("0o").or_else(|| rest.strip_prefix("0O"))
                        {
                            (8, d, true)
                        } else if let Some(d) =
                            rest.strip_prefix("0b").or_else(|| rest.strip_prefix("0B"))
                        {
                            (2, d, true)
                        } else {
                            // Decimal: forbid leading zeros except the literal
                            // `0` (or `0_0`, etc.) — match CPython.
                            let bare = rest.trim_start_matches('+');
                            let nonzero = bare.trim_start_matches('0').trim_start_matches('_');
                            if !bare.is_empty()
                                && bare != "0"
                                && nonzero != bare
                                && !nonzero.is_empty()
                            {
                                return None;
                            }
                            (10, rest, false)
                        }
                    } else if base_num == 16 {
                        rest.strip_prefix("0x")
                            .or_else(|| rest.strip_prefix("0X"))
                            .map(|d| (16, d, true))
                            .unwrap_or((16, rest, false))
                    } else if base_num == 8 {
                        rest.strip_prefix("0o")
                            .or_else(|| rest.strip_prefix("0O"))
                            .map(|d| (8, d, true))
                            .unwrap_or((8, rest, false))
                    } else if base_num == 2 {
                        rest.strip_prefix("0b")
                            .or_else(|| rest.strip_prefix("0B"))
                            .map(|d| (2, d, true))
                            .unwrap_or((2, rest, false))
                    } else {
                        (base_num, rest, false)
                    };
                    // PEP 515: a single underscore is allowed immediately
                    // after a radix prefix (e.g. `0x_FF`). Otherwise no
                    // leading/trailing/consecutive underscores.
                    let stripped = strip_pep515_underscores(digits, prefix_stripped)?;
                    if stripped.is_empty() {
                        return None;
                    }
                    Some((sign, effective_base, stripped))
                };
                if let Some((sign, effective_base, stripped)) = try_parse(trimmed) {
                    // Fast path: fits in i64 inline range.
                    if let Ok(mag) = i64::from_str_radix(&stripped, effective_base) {
                        if super::bigint_ops::fits_inline(sign * mag) {
                            return MbValue::from_int(sign * mag);
                        }
                    }
                    // Overflow path: parse as an arbitrary-precision BigInt so
                    // values beyond the 48-bit inline range (e.g. a 128-bit
                    // `int(uuid.hex, 16)`) round-trip exactly.
                    if let Some(big) =
                        num_bigint::BigInt::parse_bytes(stripped.as_bytes(), effective_base)
                    {
                        let signed = if sign < 0 { -big } else { big };
                        return super::bigint_ops::bigint_from_big(signed);
                    }
                }
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                    MbValue::from_ptr(MbObject::new_str(format!(
                        "invalid literal for int() with base {base_num}: '{full}'"
                    ))),
                );
                return MbValue::none();
            }
        }
    }
    // base given but the value is bytes-like: parse its ASCII like a string
    // (`int(b"ff", 16) == 255`).
    if let Some(ptr) = val.as_ptr() {
        let bytes_text: Option<Vec<u8>> = unsafe {
            match &(*ptr).data {
                ObjData::Bytes(b) => Some(b.clone()),
                ObjData::ByteArray(lock) => Some(lock.read().unwrap().clone()),
                _ => None,
            }
        };
        if let Some(raw) = bytes_text {
            let text = String::from_utf8_lossy(&raw).into_owned();
            let s_obj = MbValue::from_ptr(MbObject::new_str(text));
            return mb_int_base(s_obj, base);
        }
    }
    // An explicit base requires a string/bytes value (CPython: `int(123, 10)`
    // raises TypeError, not a silent 0).
    super::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(
            "int() can't convert non-string with explicit base".to_string(),
        )),
    );
    MbValue::none()
}

/// print(*args, sep=' ', end='\n') — print with kwargs.
pub fn mb_print_kwargs(args_list: MbValue, sep: MbValue, end: MbValue) -> MbValue {
    // Extract separator string (default " ")
    let sep_str = if sep.is_none() {
        " ".to_string()
    } else if let Some(ptr) = sep.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                s.clone()
            } else {
                " ".to_string()
            }
        }
    } else {
        " ".to_string()
    };
    // Extract end string (default "\n")
    let end_str = if end.is_none() {
        "\n".to_string()
    } else if let Some(ptr) = end.as_ptr() {
        unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                s.clone()
            } else {
                "\n".to_string()
            }
        }
    } else {
        "\n".to_string()
    };
    // Print items separated by sep, ending with end
    if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        mb_out!("{}", sep_str);
                    }
                    print_value_str(*item);
                }
                mb_out!("{}", end_str);
                return MbValue::none();
            }
        }
    }
    MbValue::none()
}

/// True if `file` is `sys.stderr` — either the legacy dict stub
/// (`{'name': '<stderr>'}`) or the current `sys._Stream` Instance carrying
/// `name == "<stderr>"`.
fn is_stderr_file(file: MbValue) -> bool {
    if let Some(ptr) = file.as_ptr() {
        unsafe {
            let name_val = match &(*ptr).data {
                ObjData::Dict(ref lock) => lock
                    .read()
                    .unwrap()
                    .get(&super::dict_ops::DictKey::Str("name".to_string()))
                    .copied(),
                ObjData::Instance { ref fields, .. } => fields.read().unwrap().get("name").copied(),
                _ => None,
            };
            if let Some(v) = name_val {
                if let Some(sp) = v.as_ptr() {
                    if let ObjData::Str(ref s) = (*sp).data {
                        return s == "<stderr>";
                    }
                }
            }
        }
    }
    false
}

/// `print(*args, sep=, end=, file=)` — like mb_print_kwargs but honors the
/// `file=` kwarg. Routes to stderr when `file is sys.stderr`; all other files
/// (None, sys.stdout, unrecognized) keep the existing stdout behavior.
pub fn mb_print_kwargs_file(
    args_list: MbValue,
    sep: MbValue,
    end: MbValue,
    file: MbValue,
) -> MbValue {
    if !is_stderr_file(file) {
        // stdout — unchanged behavior.
        return mb_print_kwargs(args_list, sep, end);
    }
    // stderr — build the line via str() of each item and write to stderr.
    let sep_str = sep
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| " ".to_string());
    let end_str = if end.is_none() {
        "\n".to_string()
    } else {
        end.as_ptr()
            .and_then(|p| unsafe {
                if let ObjData::Str(ref s) = (*p).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "\n".to_string())
    };
    let mut line = String::new();
    if let Some(ptr) = args_list.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        line.push_str(&sep_str);
                    }
                    let s = mb_str(*item);
                    if let Some(sp) = s.as_ptr() {
                        if let ObjData::Str(ref st) = (*sp).data {
                            line.push_str(st);
                        }
                    }
                }
            }
        }
    }
    line.push_str(&end_str);
    // contextlib.redirect_stderr: route into the active redirect target.
    if super::output::try_write_stderr_redirect(&line) {
        return MbValue::none();
    }
    eprint!("{line}");
    MbValue::none()
}

/// sorted(iterable, key=None, reverse=False) — sort with key function and reverse flag.
/// Validate that `func` can be called with a single positional argument (the
/// sort/min/max key contract). Raises and returns true when it declares more
/// than one REQUIRED positional parameter — CPython: "<lambda>() missing 1
/// required positional argument: 'y'". A native callable (no recorded params)
/// or a callable with `*args` is left unchecked.
pub fn key_unary_arity_error(func: MbValue) -> bool {
    let params = match super::closure::func_params(func) {
        Some(p) => p,
        None => return false,
    };
    if params.iter().any(|p| p.kind == 2) {
        return false; // *args absorbs extra/missing positionals
    }
    let required: Vec<String> = params
        .iter()
        .filter(|p| p.kind <= 1 && !p.has_default)
        .map(|p| p.name.clone())
        .collect();
    if required.len() <= 1 {
        return false;
    }
    let missing = &required[1..];
    let n = missing.len();
    let names = if n == 1 {
        format!("'{}'", missing[0])
    } else {
        let head = missing[..n - 1]
            .iter()
            .map(|x| format!("'{x}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{head} and '{}'", missing[n - 1])
    };
    let fname = super::closure::mb_func_get_name(func)
        .as_ptr()
        .and_then(|p| unsafe {
            if let ObjData::Str(ref s) = (*p).data {
                Some(s.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "<lambda>".to_string());
    raise_type_error(format!(
        "{fname}() missing {n} required positional argument{}: {names}",
        if n == 1 { "" } else { "s" }
    ));
    true
}

pub fn mb_sorted_kwargs(iterable: MbValue, key: MbValue, reverse: MbValue) -> MbValue {
    let items = extract_items(iterable);
    let do_reverse = reverse.as_bool() == Some(true) || reverse.as_int() == Some(1);
    let has_key = !key.is_none();

    if has_key {
        // The key must be callable (CPython: `sorted(xs, key=42)` →
        // "'int' object is not callable"). Callables are functions, named
        // builtins (Str), or instances with __call__; a bare scalar/container
        // is rejected up front rather than silently producing None keys.
        let key_callable = resolve_callable(key).is_some()
            || key.as_ptr().map_or(false, |p| unsafe {
                matches!(&(*p).data, ObjData::Str(_) | ObjData::Instance { .. })
            });
        if !key_callable {
            raise_type_error(format!("'{}' object is not callable", value_type_name(key)));
            return MbValue::none();
        }
        // The key is invoked with exactly one argument; a key declaring >1
        // required positional param raises TypeError before any sorting.
        if key_unary_arity_error(key) {
            return MbValue::none();
        }
    }

    if has_key {
        // Apply key function to each element, sort by key result
        let key_fn_addr = resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };

        let mut indexed: Vec<(MbValue, MbValue)> = Vec::with_capacity(items.len());
        for &item in &items {
            let k = if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                // Instance-based callables (unbound method wrappers,
                // functools.partial, @dataclass callables, __call__ protocol,
                // ...) — route through the dynamic 1-arg dispatcher.
                super::class::mb_call1_val(key, item)
            } else {
                item
            };
            // A key function that raises aborts the sort (CPython propagates the
            // exception rather than swallowing it and sorting partial keys).
            if super::exception::mb_has_exception().as_bool() == Some(true) {
                return MbValue::none();
            }
            indexed.push((item, k));
        }

        indexed.sort_by(|a, b| stable_order_for_reverse(mb_value_cmp(a.1, b.1), do_reverse));
        let sorted_items: Vec<MbValue> = indexed.into_iter().map(|(v, _)| v).collect();
        // Items borrowed from source container — retain.
        MbValue::from_ptr(MbObject::new_list_borrowed(sorted_items))
    } else {
        let mut sorted_items = items;
        // Type-specialized sort for no-key case (same logic as mb_sorted).
        if !sorted_items.is_empty()
            && sorted_items[0].is_int()
            && sorted_items.iter().all(|v| v.is_int())
        {
            sorted_items.sort_by(|a, b| {
                stable_order_for_reverse(
                    a.as_int().unwrap_or(0).cmp(&b.as_int().unwrap_or(0)),
                    do_reverse,
                )
            });
        } else {
            sorted_items.sort_by(|a, b| stable_order_for_reverse(mb_value_cmp(*a, *b), do_reverse));
        }
        // Items borrowed from source container — retain.
        MbValue::from_ptr(MbObject::new_list_borrowed(sorted_items))
    }
}

/// min(iterable, key=None, default=None) — min with key and default.
pub fn mb_min_kwargs(args: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let items = extract_items(args);
    if items.is_empty() {
        return if default.is_none() {
            MbValue::none()
        } else {
            default
        };
    }
    let has_key = !key.is_none();
    let result = if has_key {
        let key_fn_addr = resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let apply_key = |item: MbValue| -> MbValue {
            if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                super::class::mb_call1_val(key, item)
            } else {
                item
            }
        };
        items
            .into_iter()
            .reduce(|a, b| {
                if compare_values(apply_key(a), apply_key(b)) {
                    a
                } else {
                    b
                }
            })
            .unwrap_or(default)
    } else {
        items
            .into_iter()
            .reduce(|a, b| if compare_values(a, b) { a } else { b })
            .unwrap_or(default)
    };
    // Honor NEW-contract: returned value must be independently owned —
    // the iterable still holds its own ref, so retain on the way out.
    unsafe {
        super::rc::retain_if_ptr(result);
    }
    result
}

/// max(iterable, key=None, default=None) — max with key and default.
pub fn mb_max_kwargs(args: MbValue, key: MbValue, default: MbValue) -> MbValue {
    let items = extract_items(args);
    if items.is_empty() {
        return if default.is_none() {
            MbValue::none()
        } else {
            default
        };
    }
    let has_key = !key.is_none();
    let result = if has_key {
        let key_fn_addr = resolve_callable(key);
        let named_key = if key_fn_addr.is_none() {
            key.as_ptr().and_then(|ptr| unsafe {
                if let ObjData::Str(ref s) = (*ptr).data {
                    Some(s.clone())
                } else {
                    None
                }
            })
        } else {
            None
        };
        let apply_key = |item: MbValue| -> MbValue {
            if let Some(addr) = key_fn_addr {
                let _ = addr;
                super::class::mb_call1_val(key, item)
            } else if let Some(ref name) = named_key {
                call_named_callable(name, item).unwrap_or(item)
            } else if key.as_ptr().is_some() {
                super::class::mb_call1_val(key, item)
            } else {
                item
            }
        };
        items
            .into_iter()
            .reduce(|a, b| {
                if compare_values(apply_key(b), apply_key(a)) {
                    a
                } else {
                    b
                }
            })
            .unwrap_or(default)
    } else {
        items
            .into_iter()
            .reduce(|a, b| if compare_values(b, a) { a } else { b })
            .unwrap_or(default)
    };
    unsafe {
        super::rc::retain_if_ptr(result);
    }
    result
}

/// callable(obj) — return True if the object appears callable.
pub fn mb_callable(obj: MbValue) -> MbValue {
    // TAG_FUNC values + closure handles (TAG_INT carrying a closure id):
    // resolve_callable recognises both kinds of compiled-function references,
    // so user-defined `def`s and `lambda`s round-trip correctly.
    if resolve_callable(obj).is_some() {
        return MbValue::from_bool(true);
    }
    if let Some(ptr) = obj.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Instance { class_name, .. } => {
                    if super::stdlib::enum_mod::is_functional_enum_class(obj) {
                        return MbValue::from_bool(true);
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
                    let method = super::class::mb_lookup_dunder(
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
                    if super::class::class_is_registered(s) {
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

/// sum(iterable, start) — sum with an initial value.
pub fn mb_sum_with_start(iterable: MbValue, start: MbValue) -> MbValue {
    // CPython rejects text/bytes starts up front with a dedicated message.
    if let Some(ptr) = start.as_ptr() {
        let reject = unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => Some("strings [use ''.join(seq) instead]"),
                ObjData::Bytes(_) => Some("bytes [use b''.join(seq) instead]"),
                ObjData::ByteArray(_) => Some("bytearray [use b''.join(seq) instead]"),
                _ => None,
            }
        };
        if let Some(kind) = reject {
            raise_type_error(format!("sum() can't sum {kind}"));
            return MbValue::none();
        }
    }
    sum_from(iterable, start)
}

/// Resolve a callable MbValue to a raw function pointer address (usize).
///
/// Handles two cases:
/// - TAG_FUNC value (from ExternFuncRef / compiled lambda FuncRef): extract directly.
/// - TAG_INT value (closure ID from mb_closure_new): look up closure's `func` field.
fn resolve_callable(func: MbValue) -> Option<usize> {
    // Direct function pointer (TAG_FUNC = 4)
    if let Some(addr) = func.as_func() {
        if addr > 4096 {
            return Some(addr);
        }
    }
    // Closure handle (TAG_INT): look up embedded function pointer
    if func.as_int().is_some() {
        let fn_val = super::closure::mb_closure_get_func(func);
        if let Some(addr) = fn_val.as_func() {
            if addr > 4096 {
                return Some(addr);
            }
        }
    }
    None
}

/// call_spread: call func with elements of args_list as positional arguments.
///
/// Used for `f(*args)` splat-in-call syntax. Supports 0–8 arguments.
/// The function pointer is transmuted to the matching arity; callers must ensure
/// args_list.len() matches the actual parameter count of func.
/// Native extern functions (`extern "C" fn(*const MbValue, usize) -> MbValue`)
/// are detected via `is_native_func` and dispatched with the correct ABI (#1132).
fn unbound_builtin_classmethod_target(type_name: &str, method_name: &str) -> bool {
    let is_builtin_type = matches!(
        type_name,
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
        method_name,
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
    is_builtin_type && is_classmethod_name
}

pub fn mb_call_spread(func: MbValue, args_list: MbValue) -> MbValue {
    super::closure::with_callable_module(func, || mb_call_spread_impl(func, args_list))
}

fn mb_call_spread_impl(func: MbValue, args_list: MbValue) -> MbValue {
    if super::stdlib::ast_mod::mb_ast_is_node_init_func(func) {
        let items = extract_items(args_list);
        let self_v = items.first().copied().unwrap_or_else(MbValue::none);
        return super::stdlib::ast_mod::mb_ast_init_bound_method_kwargs(
            func,
            self_v,
            &items[usize::from(!items.is_empty())..],
            MbValue::none(),
        )
        .unwrap_or_else(MbValue::none);
    }
    // Fast path: native flat-args dispatcher + List/Tuple args. Hot-loop
    // module-level wrapper calls (e.g. `operator.eq(a, b)`, `math.fma(...)`)
    // are dominated by the per-call `Vec<MbValue>` clone in `extract_items`.
    // Borrow the args storage directly and hand the raw slice to the FFI
    // dispatcher, which consumes it during the synchronous call only.
    if let Some(addr) = resolve_callable(func) {
        if super::module::is_native_func(addr as u64) {
            if let Some(ptr) = args_list.as_ptr() {
                unsafe {
                    match &(*ptr).data {
                        ObjData::List(lock) => {
                            if let Ok(guard) = lock.try_read() {
                                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                    std::mem::transmute(addr);
                                return f(guard.as_ptr(), guard.len());
                            }
                        }
                        ObjData::Tuple(items) => {
                            let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                std::mem::transmute(addr);
                            return f(items.as_ptr(), items.len());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    let items = extract_items(args_list);
    // Instance-based callables: functools.partial, namedtuple factory.
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            // `str.lower(x)` / `list.append(xs, v)` — unbound method wrapper
            // produced by mb_getattr when invoked on a type-name string.
            // Dispatch as `args[0].method(args[1:])`.
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // Bound native method (`f = gen.uniform; f(10, 10)`): the
                // Instance carries the receiver + method name; dispatch as a
                // normal method call.
                if class_name == "__exec_function__" {
                    return mb_exec_function_call(func, items);
                }
                if class_name == "__bound_native_method__" {
                    let g = fields.read().unwrap();
                    let recv = g.get("__self__").copied().unwrap_or_else(MbValue::none);
                    let mname = g.get("__method__").copied().unwrap_or_else(MbValue::none);
                    drop(g);
                    // new_list_borrowed retains the elements; the release
                    // below frees this internal repackaging temp and returns
                    // those refs (#1091: `new_list` left it permanently
                    // rc>=1 and gc-tracked, so every generically-dispatched
                    // bound-native-method call — e.g. `s.startswith(...)` —
                    // leaked one List forever, growing the GC tracked set
                    // unboundedly and making `collect()` superlinear).
                    let rest = MbValue::from_ptr(MbObject::new_list_borrowed(items));
                    let result = super::class::mb_call_method(recv, mname, rest);
                    super::rc::release_if_ptr(rest);
                    return result;
                }
                if class_name == "method" {
                    let g = fields.read().unwrap();
                    let func_v = g.get("__func__").copied().unwrap_or_else(MbValue::none);
                    let self_v = g.get("__self__").copied().unwrap_or_else(MbValue::none);
                    drop(g);
                    if let Some(result) = super::stdlib::ast_mod::mb_ast_init_bound_method_kwargs(
                        func_v,
                        self_v,
                        &items,
                        MbValue::none(),
                    ) {
                        return result;
                    }
                    let mut all_args = Vec::with_capacity(items.len() + 1);
                    all_args.push(self_v);
                    all_args.extend(items);
                    super::class::append_missing_method_defaults(func_v, &mut all_args, 1);
                    // Same leak as __bound_native_method__ above (#1091): use
                    // the borrowed constructor + explicit release so this
                    // internal repackaging temp doesn't leak on every bound
                    // user-method call.
                    let args_list = MbValue::from_ptr(MbObject::new_list_borrowed(all_args));
                    let result = mb_call_spread(func_v, args_list);
                    super::rc::release_if_ptr(args_list);
                    return result;
                }
                if class_name == "functools._singledispatchmethod_bound" {
                    return super::stdlib::functools_mod::mb_singledispatchmethod_call_bound(
                        func, items,
                    );
                }
                if class_name == "__unbound_method__" {
                    let guard = fields.read().unwrap();
                    let method_name = guard
                        .get("__method__")
                        .copied()
                        .unwrap_or_else(MbValue::none);
                    let type_name = guard
                        .get("__type__")
                        .and_then(|v| v.as_ptr())
                        .and_then(|p| match &(*p).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    drop(guard);
                    let method_str = method_name
                        .as_ptr()
                        .and_then(|p| match &(*p).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    if let Some(result) = super::stdlib::ast_mod::mb_ast_init_unbound_method_kwargs(
                        &type_name,
                        &method_str,
                        &items,
                        MbValue::none(),
                    ) {
                        return result;
                    }
                    if method_str == "__new__" {
                        if let Some(result) =
                            super::class::builtin_type_new_unbound(&type_name, &items)
                        {
                            return result;
                        }
                    }
                    if type_name == "type" && method_str == "__init__" {
                        return super::class::type_init_unbound(&items);
                    }
                    if type_name == "collections.Counter" && method_str == "fromkeys" {
                        return super::class::mb_counter_fromkeys_not_implemented();
                    }
                    if type_name == "dict" && method_str == "fromkeys" {
                        let iterable = items.first().copied().unwrap_or_else(MbValue::none);
                        let value = items.get(1).copied().unwrap_or_else(MbValue::none);
                        return super::dict_ops::mb_dict_fromkeys(iterable, value);
                    }
                    if type_name == "object" && method_str == "__new__" {
                        return super::class::object_new_unbound(&items);
                    }
                    if type_name == "object" && method_str == "__init__" {
                        return super::class::object_init_unbound(&items);
                    }
                    if type_name == "object" {
                        match method_str.as_str() {
                            "__setattr__" => return super::class::object_setattr_unbound(&items),
                            "__delattr__" => return super::class::object_delattr_unbound(&items),
                            "__getattribute__" => {
                                return super::class::object_getattribute_unbound(&items)
                            }
                            _ => {}
                        }
                    }
                    if items.is_empty() {
                        if let Some(result) =
                            super::stdlib::string_constants_mod::static_no_self_error(
                                &type_name,
                                &method_str,
                            )
                        {
                            return result;
                        }
                    }
                    if matches!(method_str.as_str(), "extract" | "from_list")
                        && (type_name == "StackSummary"
                            || super::class::class_mro_any(&type_name, |name| {
                                name == "StackSummary"
                            }))
                    {
                        let m = super::class::lookup_method(&type_name, &method_str);
                        if let Some(addr) = m.as_func() {
                            if super::module::is_native_func(addr as u64) {
                                let mut all_items = Vec::with_capacity(items.len() + 1);
                                all_items
                                    .push(MbValue::from_ptr(MbObject::new_str(type_name.clone())));
                                all_items.extend(items.iter().copied());
                                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                    std::mem::transmute(addr);
                                return f(all_items.as_ptr(), all_items.len());
                            }
                        }
                    }
                    // complex comparison dunders accessed unbound
                    // (`complex.__eq__(a, b)` etc.). __eq__/__ne__ return a bool
                    // when the other operand is numeric and NotImplemented
                    // otherwise; ordering dunders are always NotImplemented
                    // (complex has no ordering). The arg slab is [a, b].
                    if type_name == "complex" {
                        let method_str = method_name
                            .as_ptr()
                            .and_then(|p| match &(*p).data {
                                ObjData::Str(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        let a = items.first().copied().unwrap_or_else(MbValue::none);
                        let b = items.get(1).copied().unwrap_or_else(MbValue::none);
                        if let Some(result) = complex_cmp_dunder(&method_str, a, b) {
                            return result;
                        }
                    }
                    // Native classmethods (`datetime.date.today`,
                    // `datetime.datetime.fromordinal`): the registered method
                    // value is itself a raw `(args_ptr, nargs)` dispatcher, so
                    // the call args are NOT receiver + rest — pass them through
                    // whole. Gated to the date/datetime class tables; every
                    // other type keeps receiver dispatch.
                    if matches!(
                        type_name.as_str(),
                        "date"
                            | "datetime"
                            | "datetime.time"
                            | "StackSummary"
                            | "TracebackException"
                            | "patch"
                            | "zipfile.ZipInfo"
                            | "chain"
                    ) {
                        let method_str = method_name
                            .as_ptr()
                            .and_then(|p| match &(*p).data {
                                ObjData::Str(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        let m = super::class::lookup_method(&type_name, &method_str);
                        if let Some(addr) = m.as_func() {
                            if super::module::is_native_func(addr as u64) {
                                let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                                    std::mem::transmute(addr);
                                return f(items.as_ptr(), items.len());
                            }
                        }
                    }
                    if (type_name == "bytes" || type_name == "bytearray")
                        && method_str == "maketrans"
                    {
                        return super::bytes_ops::mb_bytes_maketrans(
                            items.first().copied().unwrap_or_else(MbValue::none),
                            items.get(1).copied().unwrap_or_else(MbValue::none),
                        );
                    }
                    if type_name == "str" && method_str == "maketrans" {
                        return super::string_ops::mb_str_maketrans(
                            items.first().copied().unwrap_or_else(MbValue::none),
                            items.get(1).copied().unwrap_or_else(MbValue::none),
                            items.get(2).copied().unwrap_or_else(MbValue::none),
                        );
                    }
                    if unbound_builtin_classmethod_target(&type_name, &method_str) {
                        let type_recv = MbValue::from_ptr(MbObject::new_str(type_name.clone()));
                        let args = MbValue::from_ptr(MbObject::new_list_borrowed(items.clone()));
                        let result = super::class::mb_call_method(type_recv, method_name, args);
                        super::rc::release_if_ptr(type_recv);
                        super::rc::release_if_ptr(args);
                        return result;
                    }
                    // `random.Random.getrandbits(self, n)` — unbound calls on
                    // the Random base must reach the NATIVE generator (the
                    // user's override delegates here; re-dispatching on self
                    // would recurse forever). Resolve self to its handle and
                    // go through the handle protocol.
                    if (type_name == "Random" || type_name == "SystemRandom") && !items.is_empty() {
                        let recv = items[0];
                        let handle = if recv.is_int() {
                            recv
                        } else {
                            super::stdlib::random_mod::handle_for_instance(recv)
                        };
                        if handle.is_int() {
                            let rest = MbValue::from_ptr(MbObject::new_list(items[1..].to_vec()));
                            return super::class::mb_call_method(handle, method_name, rest);
                        }
                    }
                    // Pathlib classmethods (`Path.cwd()` / `Path.home()`)
                    // arrive with NO receiver: the registered methods are
                    // variadic `fn(self, args_list)` that derive the flavour
                    // from the receiver when present and default to the host
                    // concrete flavour for a None receiver. Dispatch instead
                    // of falling into the generic empty-args None bail.
                    if items.is_empty()
                        && matches!(
                            type_name.as_str(),
                            "Path"
                                | "PosixPath"
                                | "WindowsPath"
                                | "PurePath"
                                | "PurePosixPath"
                                | "PureWindowsPath"
                        )
                    {
                        let method_str = method_name
                            .as_ptr()
                            .and_then(|p| match &(*p).data {
                                ObjData::Str(s) => Some(s.clone()),
                                _ => None,
                            })
                            .unwrap_or_default();
                        if matches!(method_str.as_str(), "cwd" | "home") {
                            let m = super::class::lookup_method(&type_name, &method_str);
                            if let Some(addr) = m.as_func() {
                                if super::module::is_variadic_func(addr as u64) {
                                    let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                                        std::mem::transmute(addr);
                                    let empty = MbValue::from_ptr(MbObject::new_list(Vec::new()));
                                    return f(MbValue::none(), empty);
                                }
                            }
                        }
                    }
                    if items.is_empty() {
                        return MbValue::none();
                    }
                    let receiver = items[0];
                    let rest_list = MbValue::from_ptr(MbObject::new_list(items[1..].to_vec()));
                    return super::class::mb_call_method(receiver, method_name, rest_list);
                }
                if class_name == "collections.abc._register_bound"
                    || class_name == "abc._user_register_bound"
                {
                    let is_user = class_name == "abc._user_register_bound";
                    let parent_name = fields
                        .read()
                        .unwrap()
                        .get("_abc_parent")
                        .and_then(|v| v.as_ptr())
                        .and_then(|p| match &(*p).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    let child = items.first().copied().unwrap_or_else(MbValue::none);
                    if is_user {
                        return super::class::mb_user_abc_register(&parent_name, child);
                    }
                    return super::class::mb_collections_abc_register(&parent_name, child);
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
            }
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // `weakref.ref(obj[, cb])` — the `ref` attribute is a type
                // stub (class_name="type", __name__="ReferenceType"); calling
                // it constructs a new ReferenceType instance.
                if class_name == "type" {
                    let name = fields
                        .read()
                        .unwrap()
                        .get("__name__")
                        .and_then(|v| {
                            v.as_ptr().and_then(|p| match &(*p).data {
                                super::rc::ObjData::Str(ref s) => Some(s.clone()),
                                _ => None,
                            })
                        })
                        .unwrap_or_default();
                    if let Some(result) =
                        super::class::mb_collections_abc_reject_abstract_instantiation(&name)
                    {
                        return result;
                    }
                    if let Some(result) =
                        super::class::mb_user_abc_reject_abstract_instantiation(&name)
                    {
                        return result;
                    }
                    if let Some(result) =
                        super::class::mb_contextlib_abc_reject_abstract_instantiation(&name)
                    {
                        return result;
                    }
                    if name == "ReferenceType" {
                        let obj_arg = items.first().copied().unwrap_or_else(MbValue::none);
                        let cb_arg = items.get(1).copied().unwrap_or_else(MbValue::none);
                        return super::stdlib::weakref_mod::mb_weakref_ref(obj_arg, cb_arg);
                    }
                    // Builtin type constructors: int(), str(), bool(), float(), etc.
                    // Route to the real constructor rather than creating a stub Instance.
                    {
                        let result = match name.as_str() {
                            // Singleton types: `type(None)()` / `type(...)()` /
                            // `type(NotImplemented)()` with no args return the
                            // singleton itself; any argument is a TypeError.
                            "NoneType" | "ellipsis" | "NotImplementedType" => {
                                if !items.is_empty() {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(format!(
                                            "{name} takes no arguments"
                                        ))),
                                    );
                                    Some(MbValue::none())
                                } else {
                                    Some(match name.as_str() {
                                        "NoneType" => MbValue::none(),
                                        "ellipsis" => MbValue::ellipsis(),
                                        _ => MbValue::not_implemented(),
                                    })
                                }
                            }
                            "int" => Some(match items.len() {
                                0 => MbValue::from_int(0),
                                1 => mb_int(items[0]),
                                2 => {
                                    // #942: a keyword call routed through this
                                    // type-object constructor path (e.g.
                                    // `int("ff", base=16)` called dynamically
                                    // because it carries a kwarg) arrives here
                                    // with the spread kwargs dict appended as
                                    // `items[1]` instead of a literal base
                                    // value (see mb_call_spread_kwargs's
                                    // native-target fallback). Pull `base`
                                    // back out by name so it binds to the
                                    // base slot instead of being fed to
                                    // `mb_int_base` as a dict.
                                    let is_kwargs_dict = items[1]
                                        .as_ptr()
                                        .is_some_and(|p| matches!((*p).data, ObjData::Dict(_)));
                                    let base = if is_kwargs_dict {
                                        kwargs_dict_pairs(items[1])
                                            .into_iter()
                                            .find(|(k, _)| k == "base")
                                            .map(|(_, v)| v)
                                            .unwrap_or(items[1])
                                    } else {
                                        items[1]
                                    };
                                    mb_int_base(items[0], base)
                                }
                                n => type_error_value(format!(
                                    "int() takes at most 2 arguments ({n} given)"
                                )),
                            }),
                            "float" => Some(match items.len() {
                                0 => MbValue::from_float(0.0),
                                1 => mb_float(items[0]),
                                n => type_error_value(format!(
                                    "float expected at most 1 argument, got {n}"
                                )),
                            }),
                            "str" => Some(match items.len() {
                                0 => MbValue::from_ptr(MbObject::new_str(String::new())),
                                1 => mb_str(items[0]),
                                2 => mb_str_construct(items[0], items[1], MbValue::none()),
                                3 => mb_str_construct(items[0], items[1], items[2]),
                                n => type_error_value(format!(
                                    "str() takes at most 3 arguments ({n} given)"
                                )),
                            }),
                            "bool" => Some(match items.len() {
                                0 => MbValue::from_bool(false),
                                1 => mb_bool(items[0]),
                                n => type_error_value(format!(
                                    "bool expected at most 1 argument, got {n}"
                                )),
                            }),
                            "list" => Some(match items.len() {
                                0 => super::list_ops::mb_list_new(),
                                1 => super::list_ops::mb_list_from_iterable(items[0]),
                                n => type_error_value(format!(
                                    "list expected at most 1 argument, got {n}"
                                )),
                            }),
                            "tuple" => Some(match items.len() {
                                0 => super::tuple_ops::mb_tuple_new(),
                                1 => super::tuple_ops::mb_tuple_from_iterable(items[0]),
                                n => type_error_value(format!(
                                    "tuple expected at most 1 argument, got {n}"
                                )),
                            }),
                            "dict" => Some(match items.len() {
                                0 => super::dict_ops::mb_dict_new(),
                                1 => super::dict_ops::mb_dict_from_pairs(items[0]),
                                n => type_error_value(format!(
                                    "dict expected at most 1 argument, got {n}"
                                )),
                            }),
                            "set" => Some(match items.len() {
                                0 => super::set_ops::mb_set_new(),
                                1 => mb_set_from_iterable(items[0]),
                                n => type_error_value(format!(
                                    "set expected at most 1 argument, got {n}"
                                )),
                            }),
                            "frozenset" => Some(match items.len() {
                                0 => mb_frozenset_new(MbValue::none()),
                                1 => mb_frozenset_new(items[0]),
                                n => type_error_value(format!(
                                    "frozenset expected at most 1 argument, got {n}"
                                )),
                            }),
                            "mappingproxy" => {
                                if items.len() != 1 {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "mappingproxy() takes exactly one argument".to_string(),
                                        )),
                                    );
                                    Some(MbValue::none())
                                } else {
                                    let mut fields = rustc_hash::FxHashMap::default();
                                    super::rc::retain_if_ptr(items[0]);
                                    fields.insert("_mapping".to_string(), items[0]);
                                    let obj = Box::new(MbObject {
                                        header: super::rc::MbObjectHeader {
                                            rc: std::sync::atomic::AtomicU32::new(1),
                                            kind: super::rc::ObjKind::Instance,
                                        },
                                        data: ObjData::Instance {
                                            class_name: "mappingproxy".to_string(),
                                            fields: crate::runtime::rc::MbRwLock::new(fields),
                                        },
                                    });
                                    Some(MbValue::from_ptr(Box::into_raw(obj)))
                                }
                            }
                            "complex" => Some(match items.len() {
                                0 => mb_complex(MbValue::from_int(0), MbValue::from_int(0)),
                                1 => mb_complex(items[0], MbValue::from_int(0)),
                                2 => mb_complex(items[0], items[1]),
                                n => type_error_value(format!(
                                    "complex() takes at most 2 arguments ({n} given)"
                                )),
                            }),
                            // types.CodeType(...) — the 18-arg 3.12 positional
                            // constructor; zips positionals onto the co_* field
                            // order and returns a real code object.
                            "code" => Some(super::class::make_code_object_from_ctor_args(&items)),
                            // types.MethodType(func, obj) — a real bound method:
                            // calling it prepends __self__ to the args.
                            "method" if items.len() >= 2 => {
                                let mut fields = rustc_hash::FxHashMap::default();
                                unsafe {
                                    super::rc::retain_if_ptr(items[0]);
                                    super::rc::retain_if_ptr(items[1]);
                                }
                                fields.insert("__func__".to_string(), items[0]);
                                fields.insert("__self__".to_string(), items[1]);
                                let obj = Box::new(MbObject {
                                    header: super::rc::MbObjectHeader {
                                        rc: std::sync::atomic::AtomicU32::new(1),
                                        kind: super::rc::ObjKind::Instance,
                                    },
                                    data: ObjData::Instance {
                                        class_name: "method".to_string(),
                                        fields: crate::runtime::rc::MbRwLock::new(fields),
                                    },
                                });
                                Some(MbValue::from_ptr(Box::into_raw(obj)))
                            }
                            "type" => Some(match items.len() {
                                1 => mb_type(items[0]),
                                3 => mb_type3(items[0], items[1], items[2]),
                                _ => {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "type() takes 1 or 3 arguments".to_string(),
                                        )),
                                    );
                                    MbValue::none()
                                }
                            }),
                            "range" => Some(match items.len() {
                                0 => {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "TypeError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(
                                            "range expected at least 1 argument, got 0".to_string(),
                                        )),
                                    );
                                    MbValue::none()
                                }
                                1 => mb_range(items[0]),
                                2 => mb_range_2(items[0], items[1]),
                                3 => mb_range_3(items[0], items[1], items[2]),
                                n => type_error_value(format!(
                                    "range expected at most 3 arguments, got {n}"
                                )),
                            }),
                            "enumerate" => Some(match items.len() {
                                0 => type_error_value(
                                    "enumerate() missing required argument 'iterable' (pos 1)",
                                ),
                                1 => super::iter::mb_enumerate(items[0], MbValue::from_int(0)),
                                2 => super::iter::mb_enumerate(items[0], items[1]),
                                n => type_error_value(format!(
                                    "enumerate() takes at most 2 arguments ({n} given)"
                                )),
                            }),
                            "zip" => Some(if items.len() == 2 {
                                super::iter::mb_zip(items[0], items[1])
                            } else {
                                super::iter::mb_zip_n(MbValue::from_ptr(MbObject::new_list(
                                    items.clone(),
                                )))
                            }),
                            "map" => Some(mb_map(
                                items.first().copied().unwrap_or_else(MbValue::none),
                                items.get(1).copied().unwrap_or_else(MbValue::none),
                            )),
                            "filter" => Some(match items.len() {
                                2 => mb_filter(items[0], items[1]),
                                n => type_error_value(format!(
                                    "filter expected 2 arguments, got {n}"
                                )),
                            }),
                            "reversed" => Some(match items.len() {
                                1 => super::iter::mb_reversed(items[0]),
                                n => type_error_value(format!(
                                    "reversed expected 1 argument, got {n}"
                                )),
                            }),
                            "memoryview" => Some(match items.len() {
                                1 => mb_memoryview(items[0]),
                                n if n > 1 => type_error_value(format!(
                                    "memoryview() takes at most 1 argument ({n} given)"
                                )),
                                _ => mb_memoryview(MbValue::none()),
                            }),
                            "slice" => Some(match items.len() {
                                0 => mb_slice_no_args(),
                                1 => mb_slice(MbValue::none(), items[0], MbValue::none()),
                                2 => mb_slice(items[0], items[1], MbValue::none()),
                                3 => mb_slice(items[0], items[1], items[2]),
                                n => type_error_value(format!(
                                    "slice expected at most 3 arguments, got {n}"
                                )),
                            }),
                            "object" => Some(if items.is_empty() {
                                super::class::mb_instance_new(
                                    MbValue::from_ptr(MbObject::new_str("object".to_string())),
                                    MbValue::none(),
                                )
                            } else {
                                type_error_value("object() takes no arguments")
                            }),
                            "property" => Some(super::class::mb_property_construct(&items)),
                            "classmethod" => Some(match items.len() {
                                1 => super::class::mb_classmethod_new(items[0]),
                                n => type_error_value(format!(
                                    "classmethod expected 1 argument, got {n}"
                                )),
                            }),
                            "staticmethod" => Some(match items.len() {
                                1 => super::class::mb_staticmethod_new(items[0]),
                                n => type_error_value(format!(
                                    "staticmethod expected 1 argument, got {n}"
                                )),
                            }),
                            "bytes" => Some(match items.len() {
                                0 => super::bytes_ops::mb_bytes_new_checked(MbValue::none()),
                                1 => super::bytes_ops::mb_bytes_new_checked(items[0]),
                                2 | 3 => super::bytes_ops::mb_bytes_new_encoded(items[0], items[1]),
                                n => type_error_value(format!(
                                    "bytes() takes at most 3 arguments ({n} given)"
                                )),
                            }),
                            "bytearray" => Some(match items.len() {
                                0 => super::bytes_ops::mb_bytearray_new_checked(MbValue::none()),
                                1 => super::bytes_ops::mb_bytearray_new_checked(items[0]),
                                2 | 3 => {
                                    super::bytes_ops::mb_bytearray_new_encoded(items[0], items[1])
                                }
                                n => type_error_value(format!(
                                    "bytearray() takes at most 3 arguments ({n} given)"
                                )),
                            }),
                            _ => None,
                        };
                        if let Some(v) = result {
                            return v;
                        }
                    }
                    if !name.is_empty() {
                        // zoneinfo.ZoneInfo(key): mamba has no IANA tz database
                        // (only UTC). Validate the key like CPython: a path-
                        // traversal/absolute key is a ValueError; any unknown
                        // zone is a ZoneInfoNotFoundError (a KeyError subclass).
                        if name == "ZoneInfo" {
                            let zi_key: Option<String> = items
                                .first()
                                .and_then(|v| v.as_ptr())
                                .and_then(|p| match &(*p).data {
                                    super::rc::ObjData::Str(ref s) => Some(s.clone()),
                                    _ => None,
                                });
                            if let Some(key) = zi_key {
                                if key.starts_with('/') || key.split('/').any(|c| c == "..") {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str("ValueError".to_string())),
                                        MbValue::from_ptr(MbObject::new_str(format!(
                                            "ZoneInfo key {key:?} is not a valid IANA time zone name"
                                        ))),
                                    );
                                    return MbValue::none();
                                }
                                if !super::stdlib::long_tail3_mod::is_known_zone(&key) {
                                    super::exception::mb_raise(
                                        MbValue::from_ptr(MbObject::new_str(
                                            "ZoneInfoNotFoundError".to_string(),
                                        )),
                                        MbValue::from_ptr(MbObject::new_str(format!(
                                            "No time zone found with key {key}"
                                        ))),
                                    );
                                    return MbValue::none();
                                }
                                // Strong cache: repeated ZoneInfo(key) returns the
                                // SAME instance (CPython). no_cache() bypasses this.
                                return super::stdlib::long_tail3_mod::zoneinfo_cached(&key);
                            }
                        }
                        if let Some(result) = reject_non_constructible_type_object(&name) {
                            return result;
                        }
                        // If `name` is a registered native class that has a
                        // registered `__init__`, run the REAL constructor so
                        // the instance is properly initialised (e.g. unittest
                        // TestSuite/TestResult/TestLoader seed their lists,
                        // TestCase stores its method name) rather than the bare
                        // `_argN` stub below. Classes registered WITHOUT an
                        // `__init__` still fall through to the generic stub,
                        // preserving the ZoneInfo/Path/... `_arg0`/`key` shape.
                        // User-defined classes (`class Foo: ...` bodies, tracked
                        // in USER_CLASSES) always route through the real
                        // constructor too — this is what makes a class called
                        // through a variable/lambda/callback (`cls = Foo;
                        // cls(v)`) construct identically to the literal `Foo(v)`
                        // call, honoring a custom `__new__`'s attribute writes
                        // instead of falling through to the generic `_argN`
                        // stub below (#1035). Native pseudo-classes registered
                        // directly via `mb_class_register` (mmap/select/ctypes
                        // shells) are never inserted into USER_CLASSES, so they
                        // keep their existing stub/init routing untouched.
                        if super::class::class_is_registered(&name)
                            && (!super::class::lookup_method(&name, "__init__").is_none()
                                || super::class::class_is_user_defined(&name)
                                || super::stdlib::dataclasses_mod::dc_has_synth_init(&name)
                                || super::class::class_has_builtin_data_payload_base(&name)
                                || super::class::check_class_hierarchy(&name, "int"))
                        {
                            // Keep the long-standing trailing-dict kwargs
                            // convention consistent for registered classes as
                            // well as generic type stubs: a constructor value
                            // held in a variable (`ctor = Cls; ctor(**kw)`)
                            // can still reach this positional spread path, so
                            // peel the final kwargs dict back out before
                            // routing through the real constructor.
                            if let Some(kwargs) = items.last().copied().filter(|v| {
                                v.as_ptr().is_some_and(|p| {
                                    matches!((*p).data, super::rc::ObjData::Dict(_))
                                })
                            }) {
                                let pos = MbValue::from_ptr(MbObject::new_list(
                                    items[..items.len().saturating_sub(1)].to_vec(),
                                ));
                                return super::class::mb_instance_new_with_init_kwargs(
                                    MbValue::from_ptr(MbObject::new_str(name.clone())),
                                    pos,
                                    kwargs,
                                );
                            }
                            let args_list = MbValue::from_ptr(MbObject::new_list(items.clone()));
                            return super::class::mb_instance_new_with_init(
                                MbValue::from_ptr(MbObject::new_str(name.clone())),
                                args_list,
                            );
                        }
                        // Generic type-stub construction: produce a fresh
                        // Instance whose class_name matches the type's
                        // `__name__`. Constructor positional args are stored
                        // as `_arg0`, `_arg1`, ... so simple consumers
                        // (`ZoneInfo("UTC").key`) can read them back without
                        // a real __init__. A trailing dict (kwargs lowering)
                        // is splat-copied as named fields so `Cls(name="x")`
                        // round-trips through `inst.name`.
                        let inst = MbObject::new_instance(name.clone());
                        let kwargs_idx = items.iter().rposition(|v| {
                            v.as_ptr()
                                .is_some_and(|p| matches!((*p).data, super::rc::ObjData::Dict(_)))
                        });
                        let trailing_dict_is_kwargs = kwargs_idx
                            .filter(|&i| i + 1 == items.len())
                            .and_then(|i| items[i].as_ptr().map(|p| (i, p)));
                        if let super::rc::ObjData::Instance {
                            fields: ref iflds, ..
                        } = (*inst).data
                        {
                            let mut g = iflds.write().unwrap();
                            let last_pos = trailing_dict_is_kwargs
                                .map(|(i, _)| i)
                                .unwrap_or(items.len());
                            for (i, arg) in items.iter().take(last_pos).enumerate() {
                                g.insert(format!("_arg{i}"), *arg);
                                super::rc::retain_if_ptr(*arg);
                            }
                            // Common "primary identifier" alias used by
                            // stdlib classes whose first arg names the
                            // resource (ZoneInfo.key, Path, Connection...).
                            if let Some(arg0) = items.first().copied() {
                                if matches!(
                                    arg0.as_ptr().map(|p| &(*p).data),
                                    Some(super::rc::ObjData::Str(_))
                                ) {
                                    g.insert("key".to_string(), arg0);
                                }
                            }
                            if let Some((_, p)) = trailing_dict_is_kwargs {
                                if let super::rc::ObjData::Dict(ref lock) = (*p).data {
                                    let dict = lock.read().unwrap();
                                    for (k, v) in dict.iter() {
                                        if let super::dict_ops::DictKey::Str(ref ks) = k {
                                            g.insert(ks.clone(), *v);
                                            super::rc::retain_if_ptr(*v);
                                        }
                                    }
                                }
                            }
                        }
                        return MbValue::from_ptr(inst);
                    }
                }
            }
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                // traceback.TracebackException.format() — bound shell.
                if class_name == "traceback._TracebackException_format_bound" {
                    let f = fields.read().unwrap();
                    let receiver = f.get("_receiver").copied().unwrap_or_else(MbValue::none);
                    drop(f);
                    return super::stdlib::traceback_mod::mb_traceback_exception_format(receiver);
                }
                // weakref.ref(obj)() — return the live target, or None after expiry.
                if class_name == "ReferenceType" {
                    return super::stdlib::weakref_mod::reference_target(func);
                }
                if class_name == "collections.namedtuple_factory" {
                    return super::stdlib::collections_mod::mb_namedtuple_create(func, &items);
                }
                if class_name == "HTTPStatus" {
                    let arg = items.first().copied().unwrap_or_else(MbValue::none);
                    return super::stdlib::http_mod::mb_httpstatus_call(arg);
                }
                // Functional-API enum class objects reached through dotted
                // module-call lowering, e.g. uuid.SafeUUID(0).
                if super::stdlib::enum_mod::is_functional_enum_class(func) {
                    let arg = items.first().copied().unwrap_or_else(MbValue::none);
                    return super::stdlib::enum_mod::mb_functional_enum_call(func, arg);
                }
                // functools.lru_cache wrapper: look up cache or invoke inner.
                if class_name == "functools.lru_cache_wrapper" {
                    let _ = fields;
                    return super::stdlib::functools_mod::mb_functools_lru_cache_invoke(
                        func, items,
                    );
                }
                // functools.lru_cache factory: one arg (the callable) → wrap.
                if class_name == "functools.lru_cache_factory" {
                    return super::stdlib::functools_mod::mb_functools_lru_cache_factory_apply(
                        func, items,
                    );
                }
                // functools._lru_bound_method: dispatch cache_info / cache_clear.
                if class_name == "functools._lru_bound_method" {
                    let f = fields.read().unwrap();
                    let method_name = f
                        .get("_method")
                        .and_then(|v| v.as_ptr())
                        .and_then(|p| {
                            if let ObjData::Str(ref s) = (*p).data {
                                Some(s.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let wrapper = f.get("_wrapper").copied().unwrap_or_else(MbValue::none);
                    drop(f);
                    return match method_name.as_str() {
                        "cache_info" => {
                            super::stdlib::functools_mod::mb_functools_lru_cache_info(wrapper)
                        }
                        "cache_clear" => {
                            super::stdlib::functools_mod::mb_functools_lru_cache_clear(wrapper)
                        }
                        _ => MbValue::none(),
                    };
                }
                if class_name == "functools.partial" {
                    let _ = fields; // partial fields read inside the kwargs path
                                    // Route through the kwargs-aware path so a partial's
                                    // stored keyword arguments forward to the wrapped callable
                                    // (this call site carries only positional args).
                    let pos_list = MbValue::from_ptr(MbObject::new_list(items));
                    let empty_kw = MbValue::from_ptr(MbObject::new_dict());
                    return mb_call_spread_kwargs(func, pos_list, empty_kw);
                }
                // functools.cmp_to_key(mycmp)(value) → build a key object.
                if class_name == "functools.cmp_to_key" {
                    let _ = fields;
                    return super::stdlib::functools_mod::mb_functools_cmp_to_key_apply(
                        func, items,
                    );
                }
                // Calling a singledispatch wrapper dispatches on arg[0]'s type.
                if class_name == "functools.singledispatch" {
                    let _ = fields;
                    return super::stdlib::functools_mod::mb_singledispatch_call(func, items);
                }
                // `@f.register(type)` decorator instance applied to an impl.
                if class_name == "functools._sd_register" {
                    let _ = fields;
                    let impl_val = items.first().copied().unwrap_or_else(MbValue::none);
                    return super::stdlib::functools_mod::mb_singledispatch_register_apply(
                        func, impl_val,
                    );
                }
                // __call__ dunder dispatch for callable instances
                let call_method = super::class::lookup_method(class_name, "__call__");
                if !call_method.is_none() {
                    let method_name = MbValue::from_ptr(MbObject::new_str("__call__".to_string()));
                    let args_val = MbValue::from_ptr(MbObject::new_list(items));
                    return super::class::mb_call_method(func, method_name, args_val);
                }
            }
            // A bare class-name string that names a registered user class is a
            // constructor when called indirectly (through a variable, a dict
            // slot, or an imported module attr — e.g. `F = Foo; F(5)` or
            // `plistlib.UID(1)`). The direct `ClassName(args)` call site lowers
            // to mb_instance_new_with_init, but the indirect value path lands
            // here; without this it returns None and __init__ never fires.
            if let ObjData::Str(ref s) = (*ptr).data {
                if let Some(result) = super::stdlib::ast_mod::mb_ast_construct_marker(s, &items) {
                    return result;
                }
                if super::class::class_is_registered(s) {
                    let args_val = MbValue::from_ptr(MbObject::new_list(items));
                    return super::class::mb_instance_new_with_init(func, args_val);
                }
                // A module attribute that is an exception/warning class-name
                // string (e.g. zoneinfo.InvalidTZPathWarning) called
                // indirectly (not via mb_call_method's dict-method dispatch)
                // builds an exception instance, same as a bare `ValueError(...)`.
                // Mirrors the kwargs variant's check in mb_call_spread_kwargs.
                if super::exception::is_builtin_exception_name(s)
                    || super::class::class_mro_any(s, super::exception::is_builtin_exception_name)
                {
                    let args_val = MbValue::from_ptr(MbObject::new_list(items));
                    return super::exception::mb_exception_new_with_args(func, args_val);
                }
            }
        }
    }
    if let Some(raw_addr) = resolve_callable(func) {
        // An any/object-returning callee hands back an already-boxed MbValue
        // (e.g. a float, whose untagged bits lack a NaN-prefix), so the re-box
        // steps below must pass it through rather than mis-boxing it as an int.
        let is_boxed_ret = super::module::is_boxed_return_func(raw_addr as u64);
        // *args/**kwargs presence: addr-registry (native / struct-seq) with a
        // func_params fallback for user JIT functions, which register these flags
        // by symbol-id rather than address.
        let (has_star, has_kwargs) = detect_star_kw(func, Some(raw_addr));
        // Declared-signature defaults fill: when the call supplies fewer
        // positional args than the target declares, fill the omitted
        // trailing ones from FUNC_PARAMS' recorded defaults, or raise
        // CPython's "missing N required positional argument(s)" TypeError
        // when a still-unfilled parameter has none (#1007, #1008).
        // FUNC_PARAMS is keyed on whatever value `func` actually is — the raw
        // FuncRef for a plain top-level/module function, or the closure
        // handle for a capturing nested function — so this covers a
        // cross-module `mod.func(a)` call reached via mb_getattr just as well
        // as a same-module call. The `closure_arity`/`closure_defaults`
        // fallback below only recognises an INT closure handle and silently
        // no-ops for a plain function value, leaving an omitted trailing
        // default as uninitialized call-register garbage, or letting a
        // genuinely missing required argument through with no error at all.
        // Skipped for variadic / native / kwargs functions which manage their
        // own arg packing below.
        let mut items = items;
        if !super::module::is_native_func(raw_addr as u64) && !has_star && !has_kwargs {
            if let Some(params) = super::closure::func_params(func) {
                let positional: Vec<_> =
                    params.iter().filter(|p| matches!(p.kind, 0 | 1)).collect();
                if items.len() < positional.len() {
                    let tail = &positional[items.len()..];
                    let missing: Vec<&str> = tail
                        .iter()
                        .filter(|p| !p.has_default)
                        .map(|p| p.name.as_str())
                        .collect();
                    if !missing.is_empty() {
                        raise_type_error(missing_positional_args_message(
                            &callable_display_name(func),
                            &missing,
                        ));
                        return MbValue::none();
                    }
                    for p in tail {
                        items.push(p.default);
                    }
                }
            } else {
                let arity = super::closure::closure_arity(func);
                if arity > items.len() {
                    let defaults = super::closure::closure_defaults(func);
                    let needed = arity - items.len();
                    if defaults.len() >= needed {
                        let take_from = defaults.len() - needed;
                        items.extend_from_slice(&defaults[take_from..]);
                    }
                }
            }
        }
        if super::closure::func_has_boxed_params(func)
            && !super::module::is_native_func(raw_addr as u64)
            && !has_star
            && !has_kwargs
        {
            if let Some(frame) = bind_declared_call_frame(func, &items, &[]) {
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                items = frame;
            }
        }
        // Native extern functions use (args_ptr, nargs) convention (#1132).
        if super::module::is_native_func(raw_addr as u64) {
            let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
                unsafe { std::mem::transmute(raw_addr) };
            return unsafe { f(items.as_ptr(), items.len()) };
        }
        // Variadic / **kwargs JIT functions. The entry ABI lists every declared
        // parameter in order, so it is f(regular_0, .., regular_{R-1},
        // [args_list], [kwargs_dict]) — a function with leading regular params
        // receives them as individual arguments BEFORE the packed *args list
        // (`def full(a, b=2, *args, **kwargs)` → f(a, b, args_list, kwargs_dict)).
        // Build that frame: regular slots (filling trailing defaults when the
        // spread is short), *args collects the positional overflow, **kwargs is
        // an empty dict (a positional spread carries no keywords). Then fall
        // through to the fixed-arity dispatch.
        if has_star || has_kwargs {
            let empty_kw: Vec<(String, MbValue)> = Vec::new();
            if let Some(frame) = bind_declared_call_frame(func, &items, &empty_kw) {
                if super::exception::current_exception_type().is_some() {
                    return MbValue::none();
                }
                items = frame;
            } else {
                let mut frame: Vec<MbValue> = Vec::new();
                if has_star {
                    frame.push(MbValue::from_ptr(MbObject::new_list(items)));
                }
                if has_kwargs {
                    frame.push(MbValue::from_ptr(MbObject::new_dict()));
                }
                items = frame;
            }
        }
        // SAFETY: the function was compiled with the matching arity.
        // JIT-compiled functions use SystemV/C calling convention and may return
        // unboxed raw i64 values (CheckedAdd unboxes inline ints for perf),
        // so we re-box the result via mb_box_int.
        // REQ: extern "C" ABI required — JIT emits SystemV, not Rust ABI.
        let raw_result: MbValue = super::closure::with_closure_cells(func, || unsafe {
            match items.len() {
                0 => {
                    let f: extern "C" fn() -> MbValue = std::mem::transmute(raw_addr);
                    f()
                }
                1 => {
                    let f: extern "C" fn(MbValue) -> MbValue = std::mem::transmute(raw_addr);
                    f(items[0])
                }
                2 => {
                    let f: extern "C" fn(MbValue, MbValue) -> MbValue =
                        std::mem::transmute(raw_addr);
                    f(items[0], items[1])
                }
                3 => {
                    let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                        std::mem::transmute(raw_addr);
                    f(items[0], items[1], items[2])
                }
                4 => {
                    let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                        std::mem::transmute(raw_addr);
                    f(items[0], items[1], items[2], items[3])
                }
                5 => {
                    let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue, MbValue) -> MbValue =
                        std::mem::transmute(raw_addr);
                    f(items[0], items[1], items[2], items[3], items[4])
                }
                6 => {
                    let f: extern "C" fn(
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                    ) -> MbValue = std::mem::transmute(raw_addr);
                    f(items[0], items[1], items[2], items[3], items[4], items[5])
                }
                7 => {
                    let f: extern "C" fn(
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                    ) -> MbValue = std::mem::transmute(raw_addr);
                    f(
                        items[0], items[1], items[2], items[3], items[4], items[5], items[6],
                    )
                }
                8 => {
                    let f: extern "C" fn(
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                        MbValue,
                    ) -> MbValue = std::mem::transmute(raw_addr);
                    f(
                        items[0], items[1], items[2], items[3], items[4], items[5], items[6],
                        items[7],
                    )
                }
                _ => MbValue::none(),
            }
        });
        // Re-box primitive returns through mb_box_int. It preserves real
        // NaN-boxed values but also correctly boxes raw negative i64 values,
        // whose high bits otherwise look like a NaN-box prefix.
        let result = if is_boxed_ret {
            raw_result
        } else {
            mb_box_int(raw_result.to_bits() as i64)
        };
        if super::stdlib::types_mod::is_coroutine_generator(result) {
            result
        } else {
            super::stdlib::types_mod::mark_coroutine_result(func, result)
        }
    } else {
        // Nothing above recognized `func` as callable (no TAG_FUNC/closure
        // handle, no Instance-based __call__/partial/singledispatch, no
        // Str-based class-name constructor). CPython raises TypeError here
        // (issue #1037, e.g. `(42)()`). This was left as a permissive
        // `None`-returning fallback while the native-class bound-method-
        // synthesis gap table was closed per-class (collections.Counter,
        // UserDict/UserList/UserString, tempfile.NamedTemporaryFile,
        // contextlib.ExitStack, lzma.LZMADecompressor, string.Template,
        // threading.Condition/Event/Lock/RLock/Semaphore, pipes.Template,
        // re.Match/Pattern, int/float — #1037/#1067/#1068/#1069).
        //
        // The remaining `super().__init__()` gap is handled in
        // class.rs::mb_super_getattr now, so a truly non-callable value here
        // is a real TypeError again (`(42)(3)`, `None()`, ...).
        raise_type_error(format!(
            "'{}' object is not callable",
            value_type_name(func)
        ));
        MbValue::none()
    }
}

/// Read a kwargs `ObjData::Dict` into ordered (name, value) pairs (Str keys
/// only — keyword names are always strings).
fn kwargs_dict_pairs(dict: MbValue) -> Vec<(String, MbValue)> {
    let mut out = Vec::new();
    if let Some(ptr) = dict.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                for (k, v) in lock.read().unwrap().iter() {
                    if let super::dict_ops::DictKey::Str(ref s) = k {
                        out.push((s.clone(), *v));
                    }
                }
            }
        }
    }
    out
}

/// Merge two kwargs dicts into a fresh dict; `b` (call-time) wins on key
/// collisions.
fn merge_kwargs_dicts(a: MbValue, b: MbValue) -> MbValue {
    let out = super::dict_ops::mb_dict_new();
    for src in [a, b] {
        for (k, v) in kwargs_dict_pairs(src) {
            unsafe {
                super::rc::retain_if_ptr(v);
            }
            super::dict_ops::mb_dict_setitem(out, MbValue::from_ptr(MbObject::new_str(k)), v);
        }
    }
    out
}

fn callable_display_name(func: MbValue) -> String {
    super::closure::mb_func_get_name(func)
        .as_ptr()
        .and_then(|fp| unsafe {
            match &(*fp).data {
                ObjData::Str(ref s) => Some(s.clone()),
                _ => None,
            }
        })
        .unwrap_or_default()
}

/// Build CPython's exact "missing N required positional argument(s): ..."
/// TypeError message, including the Oxford comma before the final name for
/// N >= 3 (CPython: `f() missing 3 required positional arguments: 'b', 'c',
/// and 'd'` vs the plain `'b' and 'c'` join for N == 2). Used by the
/// cross-module positional-call binder in `mb_call_spread` (#1008).
fn missing_positional_args_message(fname: &str, missing: &[&str]) -> String {
    let n = missing.len();
    let word = if n == 1 { "argument" } else { "arguments" };
    let names = match missing {
        [only] => format!("'{only}'"),
        [first, second] => format!("'{first}' and '{second}'"),
        _ => {
            let head = missing[..n - 1]
                .iter()
                .map(|m| format!("'{m}'"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{head}, and '{}'", missing[n - 1])
        }
    };
    format!("{fname}() missing {n} required positional {word}: {names}")
}

fn bind_declared_call_frame(
    func: MbValue,
    pos: &[MbValue],
    kw_pairs: &[(String, MbValue)],
) -> Option<Vec<MbValue>> {
    let params = super::closure::func_params(func)?;
    let has_varargs = params.iter().any(|p| p.kind == 2);
    let has_varkw = params.iter().any(|p| p.kind == 4);
    let mut frame: Vec<MbValue> = Vec::with_capacity(params.len());
    let mut pos_idx = 0usize;
    let mut used: std::collections::HashSet<String> = std::collections::HashSet::new();
    let fname = callable_display_name(func);
    for p in &params {
        match p.kind {
            0 => {
                if pos_idx < pos.len() {
                    frame.push(pos[pos_idx]);
                    pos_idx += 1;
                } else if p.has_default {
                    frame.push(p.default);
                } else {
                    frame.push(MbValue::none());
                }
            }
            1 => {
                if pos_idx < pos.len() {
                    if kw_pairs.iter().any(|(k, _)| *k == p.name) {
                        raise_type_error(format!(
                            "{fname}() got multiple values for argument '{}'",
                            p.name
                        ));
                        return Some(Vec::new());
                    }
                    frame.push(pos[pos_idx]);
                    pos_idx += 1;
                } else if let Some((k, v)) = kw_pairs.iter().find(|(k, _)| *k == p.name) {
                    used.insert(k.clone());
                    frame.push(*v);
                } else if p.has_default {
                    frame.push(p.default);
                } else {
                    frame.push(MbValue::none());
                }
            }
            2 => {
                let rest = if pos_idx < pos.len() {
                    pos[pos_idx..].to_vec()
                } else {
                    Vec::new()
                };
                pos_idx = pos.len();
                frame.push(MbValue::from_ptr(MbObject::new_list(rest)));
            }
            3 => {
                if let Some((k, v)) = kw_pairs.iter().find(|(k, _)| *k == p.name) {
                    used.insert(k.clone());
                    frame.push(*v);
                } else if p.has_default {
                    frame.push(p.default);
                } else {
                    frame.push(MbValue::none());
                }
            }
            4 => {
                let extra = super::dict_ops::mb_dict_new();
                for (k, v) in kw_pairs {
                    if !used.contains(k) {
                        unsafe {
                            super::rc::retain_if_ptr(*v);
                        }
                        super::dict_ops::mb_dict_setitem(
                            extra,
                            MbValue::from_ptr(MbObject::new_str(k.clone())),
                            *v,
                        );
                    }
                }
                frame.push(extra);
            }
            _ => {}
        }
    }
    if !has_varargs && pos_idx < pos.len() {
        raise_type_error(format!(
            "{fname}() takes {} positional arguments but {} were given",
            params.iter().filter(|p| matches!(p.kind, 0 | 1)).count(),
            pos.len()
        ));
        return Some(Vec::new());
    }
    if !has_varkw {
        if let Some((bad, _)) = kw_pairs.iter().find(|(k, _)| !used.contains(k.as_str())) {
            raise_type_error(format!(
                "{fname}() got an unexpected keyword argument '{bad}'"
            ));
            return Some(Vec::new());
        }
    }
    Some(frame)
}

/// Dispatch a JIT-compiled function by its exact frame arity (SystemV/C ABI),
/// reboxing a raw-int return unless the callee is `any`/object-returning.
/// Shared by the variadic spread + kwargs binding paths so the entry ABI
/// `f(regular_0, .., args_list, kwargs_dict)` is honoured uniformly.
fn dispatch_jit_frame(raw_addr: usize, items: &[MbValue], is_boxed_ret: bool) -> MbValue {
    let raw_result: MbValue = unsafe {
        match items.len() {
            0 => {
                let f: extern "C" fn() -> MbValue = std::mem::transmute(raw_addr);
                f()
            }
            1 => {
                let f: extern "C" fn(MbValue) -> MbValue = std::mem::transmute(raw_addr);
                f(items[0])
            }
            2 => {
                let f: extern "C" fn(MbValue, MbValue) -> MbValue = std::mem::transmute(raw_addr);
                f(items[0], items[1])
            }
            3 => {
                let f: extern "C" fn(MbValue, MbValue, MbValue) -> MbValue =
                    std::mem::transmute(raw_addr);
                f(items[0], items[1], items[2])
            }
            4 => {
                let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue) -> MbValue =
                    std::mem::transmute(raw_addr);
                f(items[0], items[1], items[2], items[3])
            }
            5 => {
                let f: extern "C" fn(MbValue, MbValue, MbValue, MbValue, MbValue) -> MbValue =
                    std::mem::transmute(raw_addr);
                f(items[0], items[1], items[2], items[3], items[4])
            }
            6 => {
                let f: extern "C" fn(
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                ) -> MbValue = std::mem::transmute(raw_addr);
                f(items[0], items[1], items[2], items[3], items[4], items[5])
            }
            7 => {
                let f: extern "C" fn(
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                ) -> MbValue = std::mem::transmute(raw_addr);
                f(
                    items[0], items[1], items[2], items[3], items[4], items[5], items[6],
                )
            }
            8 => {
                let f: extern "C" fn(
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                    MbValue,
                ) -> MbValue = std::mem::transmute(raw_addr);
                f(
                    items[0], items[1], items[2], items[3], items[4], items[5], items[6], items[7],
                )
            }
            _ => MbValue::none(),
        }
    };
    if is_boxed_ret {
        return raw_result;
    }
    mb_box_int(raw_result.to_bits() as i64)
}

/// Detect `(*args, **kwargs)` presence for a callable. The addr-keyed
/// registries (`is_variadic_func` / `is_kwargs_func`) only cover native and
/// struct-seq targets; user JIT functions register their declared params but
/// not addr flags, so fall back to the params registry (kind 2 = VAR_POSITIONAL,
/// 4 = VAR_KEYWORD — inspect.Parameter ordinals). Without this, a `f(**d)` call
/// to a `def f(*a, **k)` user function failed the has_star/has_kw gate and lost
/// its keyword bindings.
fn detect_star_kw(func: MbValue, addr: Option<usize>) -> (bool, bool) {
    let mut has_star = addr
        .map(|a| super::module::is_variadic_func(a as u64))
        .unwrap_or(false);
    let mut has_kw = addr
        .map(|a| super::module::is_kwargs_func(a as u64))
        .unwrap_or(false);
    if !has_star || !has_kw {
        if let Some(params) = super::closure::func_params(func) {
            if params.iter().any(|p| p.kind == 2) {
                has_star = true;
            }
            if params.iter().any(|p| p.kind == 4) {
                has_kw = true;
            }
        }
    }
    (has_star, has_kw)
}

/// Invoke `func` with a structurally-separated positional args list and
/// kwargs dict, honoring the compiled variadic/kwargs ABI. Used by
/// `mb_call_spread_kwargs` for `(*args, **kw)` targets.
fn invoke_args_kwargs(func: MbValue, args_list: MbValue, kwargs_dict: MbValue) -> MbValue {
    let Some(raw_addr) = resolve_callable(func) else {
        return mb_call_spread(func, args_list);
    };
    // Native dispatchers take (args_ptr, nargs); append kwargs as a trailing
    // dict (the established native-kwargs convention).
    if super::module::is_native_func(raw_addr as u64) {
        let mut items = extract_items(args_list);
        items.push(kwargs_dict);
        let f: unsafe extern "C" fn(*const MbValue, usize) -> MbValue =
            unsafe { std::mem::transmute(raw_addr) };
        return unsafe { f(items.as_ptr(), items.len()) };
    }
    let is_boxed_ret = super::module::is_boxed_return_func(raw_addr as u64);
    let (has_star, has_kwargs) = detect_star_kw(func, Some(raw_addr));
    if !has_star && !has_kwargs {
        // No declared variadic/kwargs slot: fall back to positional spread.
        return mb_call_spread(func, args_list);
    }
    // Build the variadic entry frame: f(regular_0, .., regular_{R-1},
    // [args_list], [kwargs_dict]). Regular params bind from positionals first,
    // then from keyword args by name (filling defaults); positional overflow
    // packs into *args and unmatched keywords into **kwargs. Without the regular
    // split, `full(**{"a":1,"b":2,"k":99})` to `def full(a,b=2,*args,**kwargs)`
    // passed the kwargs dict as `a` and read garbage for the trailing slots.
    let pos = extract_items(args_list);
    let kw_pairs = kwargs_dict_pairs(kwargs_dict);
    if let Some(frame) = bind_declared_call_frame(func, &pos, &kw_pairs) {
        if super::exception::current_exception_type().is_some() {
            return MbValue::none();
        }
        return dispatch_jit_frame(raw_addr, &frame, is_boxed_ret);
    }
    let mut frame = Vec::new();
    if has_star {
        frame.push(MbValue::from_ptr(MbObject::new_list(pos)));
    }
    if has_kwargs {
        frame.push(kwargs_dict);
    }
    dispatch_jit_frame(raw_addr, &frame, is_boxed_ret)
}

/// Dynamic call with positional args AND keyword args kept structurally
/// separate. Closes the "kwargs dropped for dynamically-dispatched callables"
/// gap: a value held in a variable (functools.partial, a closure, a method
/// reference) called with `kw=v` now binds those keywords to the target's
/// named parameters at runtime via the FUNC_PARAMS registry, instead of
/// flattening keyword values into positionals and losing the names.
pub fn mb_call_spread_kwargs(func: MbValue, pos_list: MbValue, kwargs_dict: MbValue) -> MbValue {
    let pos = extract_items(pos_list);
    // 1. functools.partial: prepend stored args, merge stored keywords
    //    (call-time wins), and forward to the wrapped callable.
    if let Some(ptr) = func.as_ptr() {
        let nested = unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "functools.partial" {
                    let f = fields.read().unwrap();
                    Some((
                        f.get("func").copied().unwrap_or_else(MbValue::none),
                        f.get("args").copied().unwrap_or_else(MbValue::none),
                        f.get("keywords").copied().unwrap_or_else(MbValue::none),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some((inner, bound_args, bound_kw)) = nested {
            let mut combined = extract_items(bound_args);
            combined.extend(pos);
            let merged_kw = merge_kwargs_dicts(bound_kw, kwargs_dict);
            return mb_call_spread_kwargs(
                inner,
                MbValue::from_ptr(MbObject::new_list(combined)),
                merged_kw,
            );
        }
    }
    let kw_pairs = kwargs_dict_pairs(kwargs_dict);
    // 2. No keywords → plain positional spread.
    if kw_pairs.is_empty() {
        return mb_call_spread(func, MbValue::from_ptr(MbObject::new_list(pos)));
    }
    if super::stdlib::ast_mod::mb_ast_is_node_init_func(func) {
        let self_v = pos.first().copied().unwrap_or_else(MbValue::none);
        return super::stdlib::ast_mod::mb_ast_init_bound_method_kwargs(
            func,
            self_v,
            &pos[usize::from(!pos.is_empty())..],
            kwargs_dict,
        )
        .unwrap_or_else(MbValue::none);
    }
    let native_type_ctor = func
        .as_func()
        .filter(|addr| super::module::is_native_func(*addr as u64))
        .and_then(|_| super::class::resolve_class_name(func));
    if let Some(type_name) = native_type_ctor.clone().or_else(|| {
        func.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }) {
        if type_name == "list" {
            raise_type_error("list() takes no keyword arguments".to_string());
            return MbValue::none();
        }
        // float() takes no keyword arguments at all (unlike int(), it has no
        // `base` keyword). Mirrors the list() check above. (#1057)
        if type_name == "float" {
            raise_type_error("float() takes no keyword arguments".to_string());
            return MbValue::none();
        }
        // int() accepts only `base` as a keyword; any other keyword name is
        // rejected with CPython's exact per-key message. (#1057)
        if type_name == "int" {
            if let Some((bad_key, _)) = kw_pairs.iter().find(|(k, _)| *k != "base") {
                raise_type_error(format!(
                    "'{bad_key}' is an invalid keyword argument for int()"
                ));
                return MbValue::none();
            }
        }
    }
    if let Some(ptr) = func.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "method" {
                    let guard = fields.read().unwrap();
                    let func_v = guard.get("__func__").copied().unwrap_or_else(MbValue::none);
                    let self_v = guard.get("__self__").copied().unwrap_or_else(MbValue::none);
                    drop(guard);
                    if let Some(result) = super::stdlib::ast_mod::mb_ast_init_bound_method_kwargs(
                        func_v,
                        self_v,
                        &pos,
                        kwargs_dict,
                    ) {
                        return result;
                    }
                    let addr = resolve_callable(func_v);
                    let is_native = addr
                        .map(|a| super::module::is_native_func(a as u64))
                        .unwrap_or(false);
                    let (has_star, has_kw) = if is_native {
                        (
                            addr.map(|a| super::module::is_variadic_func(a as u64))
                                .unwrap_or(false),
                            addr.map(|a| super::module::is_kwargs_func(a as u64))
                                .unwrap_or(false),
                        )
                    } else {
                        detect_star_kw(func_v, addr)
                    };
                    if !has_star && !is_native {
                        if let Some(params) = super::closure::func_params(func_v) {
                            let regulars: Vec<&super::closure::MbParamInfo> =
                                params.iter().filter(|p| p.kind <= 1).collect();
                            let mut items: Vec<MbValue> =
                                Vec::with_capacity(regulars.len() + pos.len() + 1);
                            let mut used = std::collections::HashSet::new();
                            let mut pos_iter = pos.iter().copied();
                            for p in &regulars {
                                if let Some(v) = pos_iter.next() {
                                    if kw_pairs.iter().any(|(k, _)| *k == p.name) {
                                        let fname = super::closure::mb_func_get_name(func_v)
                                            .as_ptr()
                                            .and_then(|fp| unsafe {
                                                match &(*fp).data {
                                                    ObjData::Str(ref s) => Some(s.clone()),
                                                    _ => None,
                                                }
                                            })
                                            .unwrap_or_default();
                                        raise_type_error(format!(
                                            "{fname}() got multiple values for argument '{}'",
                                            p.name
                                        ));
                                        return MbValue::none();
                                    }
                                    items.push(v);
                                } else if let Some((k, v)) =
                                    kw_pairs.iter().find(|(k, _)| *k == p.name)
                                {
                                    used.insert(k.clone());
                                    items.push(*v);
                                } else if p.has_default {
                                    items.push(p.default);
                                } else {
                                    items.push(MbValue::none());
                                }
                            }
                            items.extend(pos_iter);
                            if !has_kw {
                                let declared: std::collections::HashSet<String> = params
                                    .iter()
                                    .filter(|p| p.kind <= 3)
                                    .map(|p| p.name.clone())
                                    .collect();
                                if let Some((bad, _)) = kw_pairs
                                    .iter()
                                    .find(|(k, _)| !used.contains(k) && !declared.contains(k))
                                {
                                    let fname = super::closure::mb_func_get_name(func_v)
                                        .as_ptr()
                                        .and_then(|fp| unsafe {
                                            match &(*fp).data {
                                                ObjData::Str(ref s) => Some(s.clone()),
                                                _ => None,
                                            }
                                        })
                                        .unwrap_or_default();
                                    raise_type_error(format!(
                                        "{fname}() got an unexpected keyword argument '{bad}'"
                                    ));
                                    return MbValue::none();
                                }
                            }
                            if has_kw {
                                let extra = super::dict_ops::mb_dict_new();
                                for (k, v) in &kw_pairs {
                                    if !used.contains(k) {
                                        unsafe {
                                            super::rc::retain_if_ptr(*v);
                                        }
                                        super::dict_ops::mb_dict_setitem(
                                            extra,
                                            MbValue::from_ptr(MbObject::new_str(k.clone())),
                                            *v,
                                        );
                                    }
                                }
                                items.push(extra);
                            }
                            let mut bound_items = Vec::with_capacity(items.len() + 1);
                            bound_items.push(self_v);
                            bound_items.extend(items);
                            return mb_call_spread(
                                func_v,
                                MbValue::from_ptr(MbObject::new_list(bound_items)),
                            );
                        }
                    }
                    let mut combined = Vec::with_capacity(pos.len() + 1);
                    combined.push(self_v);
                    combined.extend(pos.iter().copied());
                    return mb_call_spread(func_v, MbValue::from_ptr(MbObject::new_list(combined)));
                }
                if class_name == "__unbound_method__" {
                    let guard = fields.read().unwrap();
                    let method_name = guard
                        .get("__method__")
                        .copied()
                        .unwrap_or_else(MbValue::none);
                    let type_name = guard
                        .get("__type__")
                        .and_then(|v| v.as_ptr())
                        .and_then(|p| match &(*p).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    drop(guard);
                    let method_str = method_name
                        .as_ptr()
                        .and_then(|p| match &(*p).data {
                            ObjData::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .unwrap_or_default();
                    if let Some(result) = super::stdlib::ast_mod::mb_ast_init_unbound_method_kwargs(
                        &type_name,
                        &method_str,
                        &pos,
                        kwargs_dict,
                    ) {
                        return result;
                    }
                }
                if class_name == "__bound_native_method__" {
                    let guard = fields.read().unwrap();
                    let recv = guard.get("__self__").copied().unwrap_or_else(MbValue::none);
                    let method = guard
                        .get("__method__")
                        .copied()
                        .unwrap_or_else(MbValue::none);
                    drop(guard);
                    return super::class::mb_call_method_kwargs(
                        recv,
                        method,
                        MbValue::from_ptr(MbObject::new_list(pos)),
                        kwargs_dict,
                    );
                }
            }
        }
    }
    if let Some(type_name) = super::class::resolve_class_name(func).or_else(|| {
        func.as_ptr().and_then(|ptr| unsafe {
            if let ObjData::Str(ref s) = (*ptr).data {
                Some(s.clone())
            } else {
                None
            }
        })
    }) {
        if super::exception::is_builtin_exception_name(&type_name) {
            return super::exception::mb_exception_new_with_args_and_kwargs(
                func,
                MbValue::from_ptr(MbObject::new_list(pos)),
                kwargs_dict,
            );
        }
        if native_type_ctor.is_none() && super::class::class_is_registered(&type_name) {
            return super::class::mb_instance_new_with_init_kwargs(
                MbValue::from_ptr(MbObject::new_str(type_name)),
                MbValue::from_ptr(MbObject::new_list(pos)),
                kwargs_dict,
            );
        }
    }
    let addr = resolve_callable(func);
    let is_native = addr
        .map(|a| super::module::is_native_func(a as u64))
        .unwrap_or(false);
    let (has_star, has_kw) = if is_native {
        // Native targets keep the addr-registry answer (the flattened-positional
        // convention in step 5 depends on it); skip the params fallback.
        (
            addr.map(|a| super::module::is_variadic_func(a as u64))
                .unwrap_or(false),
            addr.map(|a| super::module::is_kwargs_func(a as u64))
                .unwrap_or(false),
        )
    } else {
        detect_star_kw(func, addr)
    };

    // 3. `*args`-style user target: all positionals pack into one args list;
    //    the keywords pack into the kwargs dict. (Leading regular params
    //    before `*args` are uncommon and not bound here — the args list
    //    already carries them positionally.) Native dispatchers are excluded:
    //    they expect the flattened-positional convention (step 5).
    if has_star && !is_native {
        return invoke_args_kwargs(
            func,
            MbValue::from_ptr(MbObject::new_list(pos)),
            kwargs_dict,
        );
    }
    if has_kw && !is_native {
        return invoke_args_kwargs(
            func,
            MbValue::from_ptr(MbObject::new_list(pos)),
            kwargs_dict,
        );
    }

    // 4. Regular / keyword-only user target with declared parameters: bind
    //    keyword args to their named positional slots, filling defaults.
    if !has_star && !is_native {
        if let Some(params) = super::closure::func_params(func) {
            let callable_params: Vec<&super::closure::MbParamInfo> =
                params.iter().filter(|p| p.kind <= 3).collect();
            let mut items: Vec<MbValue> = Vec::with_capacity(callable_params.len() + pos.len());
            let mut used = std::collections::HashSet::new();
            let mut pos_iter = pos.iter().copied();
            for p in &callable_params {
                if p.kind <= 1 {
                    if let Some(v) = pos_iter.next() {
                        if kw_pairs.iter().any(|(k, _)| *k == p.name) {
                            let fname = super::closure::mb_func_get_name(func)
                                .as_ptr()
                                .and_then(|fp| unsafe {
                                    match &(*fp).data {
                                        ObjData::Str(ref s) => Some(s.clone()),
                                        _ => None,
                                    }
                                })
                                .unwrap_or_default();
                            raise_type_error(format!(
                                "{fname}() got multiple values for argument '{}'",
                                p.name
                            ));
                            return MbValue::none();
                        }
                        items.push(v);
                    } else if let Some((k, v)) = kw_pairs.iter().find(|(k, _)| *k == p.name) {
                        used.insert(k.clone());
                        items.push(*v);
                    } else if p.has_default {
                        items.push(p.default);
                    } else {
                        items.push(MbValue::none());
                    }
                } else if let Some((k, v)) = kw_pairs.iter().find(|(k, _)| *k == p.name) {
                    used.insert(k.clone());
                    items.push(*v);
                } else if p.has_default {
                    items.push(p.default);
                } else {
                    items.push(MbValue::none());
                }
            }
            items.extend(pos_iter);
            if !has_kw {
                let declared: std::collections::HashSet<String> = params
                    .iter()
                    .filter(|p| p.kind <= 3)
                    .map(|p| p.name.clone())
                    .collect();
                if let Some((bad, _)) = kw_pairs
                    .iter()
                    .find(|(k, _)| !used.contains(k) && !declared.contains(k))
                {
                    let fname = super::closure::mb_func_get_name(func)
                        .as_ptr()
                        .and_then(|fp| unsafe {
                            match &(*fp).data {
                                ObjData::Str(ref s) => Some(s.clone()),
                                _ => None,
                            }
                        })
                        .unwrap_or_default();
                    raise_type_error(format!(
                        "{fname}() got an unexpected keyword argument '{bad}'"
                    ));
                    return MbValue::none();
                }
            }
            if has_kw {
                let extra = super::dict_ops::mb_dict_new();
                for (k, v) in &kw_pairs {
                    if !used.contains(k) {
                        unsafe {
                            super::rc::retain_if_ptr(*v);
                        }
                        super::dict_ops::mb_dict_setitem(
                            extra,
                            MbValue::from_ptr(MbObject::new_str(k.clone())),
                            *v,
                        );
                    }
                }
                items.push(extra);
                // ABI for (regulars..., **kw): regulars individual + trailing
                // kwargs dict. Invoke positionally including the dict slot.
                return mb_call_spread(func, MbValue::from_ptr(MbObject::new_list(items)));
            }
            return mb_call_spread(func, MbValue::from_ptr(MbObject::new_list(items)));
        }
    }

    // 5. Fallback (native / unknown target): append the kwargs dict as a
    //    trailing positional — the established native-dispatcher convention
    //    (`split_kwargs` recovers it). This matches what the prior lowering
    //    produced for a `**mapping` splat, so native module functions reached
    //    as bare idents (`textwrap.shorten(t, w, **kw)`) keep working. The
    //    keyword-binding path above already handles user functions/partials.
    let mut items = pos;
    items.push(kwargs_dict);
    mb_call_spread(func, MbValue::from_ptr(MbObject::new_list(items)))
}

/// floor division: a // b
pub fn mb_floordiv(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // int/float-SUBCLASS operand unwrap (#1030).
    if let Some((na, nb)) = numeric_subclass_operands(a, b, "__floordiv__") {
        return mb_floordiv(na, nb);
    }
    if let Some(r) = numeric_handle_binop("//", a, b) {
        return r;
    }
    if let Some(r) = bigint_numeric_binop("//", a, b) {
        return r;
    }
    // complex doesn't support floor division (CPython TypeError).
    if is_complex_obj(a) || is_complex_obj(b) {
        raise_type_error(format!(
            "unsupported operand type(s) for //: '{}' and '{}'",
            value_type_name(a),
            value_type_name(b)
        ));
        return MbValue::none();
    }
    // timedelta // timedelta -> int; timedelta // int -> timedelta.
    if let Some(ua) = super::stdlib::datetime_mod::timedelta_total_us(a) {
        if let Some(ub) = super::stdlib::datetime_mod::timedelta_total_us(b) {
            if ub == 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            return MbValue::from_int(floor_divmod_i128(ua, ub).0 as i64);
        }
        if let Some(d) = b.as_int() {
            if d == 0 {
                super::exception::mb_raise(
                    MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                    MbValue::from_ptr(MbObject::new_str("division by zero".to_string())),
                );
                return MbValue::none();
            }
            return super::stdlib::datetime_mod::timedelta_from_us(
                floor_divmod_i128(ua, d as i128).0,
            );
        }
    }
    // Integer fast path — Python floor division (round towards -∞)
    if let (Some(ai), Some(bi)) = (a.as_int(), b.as_int()) {
        if bi != 0 {
            let d = ai / bi;
            let r = ai % bi;
            // Adjust: if remainder is non-zero and signs of remainder and divisor differ,
            // subtract 1 to get floor division (rounds towards -∞, not towards 0).
            let floored = if r != 0 && ((r ^ bi) < 0) { d - 1 } else { d };
            return MbValue::from_int(floored);
        }
        // ZeroDivisionError: integer division or modulo by zero
        super::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
            MbValue::from_ptr(MbObject::new_str(
                "integer division or modulo by zero".to_string(),
            )),
        );
        return MbValue::none();
    }
    // Float path
    let af = a.as_int().map(|i| i as f64).or(a.as_float());
    let bf = b.as_int().map(|i| i as f64).or(b.as_float());
    match (af, bf) {
        (Some(af), Some(bf)) if bf != 0.0 => MbValue::from_float((af / bf).floor()),
        (Some(_), Some(_)) => {
            // ZeroDivisionError: float floor division by zero
            super::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("ZeroDivisionError".to_string())),
                MbValue::from_ptr(MbObject::new_str(
                    "float floor division by zero".to_string(),
                )),
            );
            MbValue::none()
        }
        _ => {
            if raise_datetime_op_type_error("//", a, b) {
                return MbValue::none();
            }
            MbValue::none()
        }
    }
}

/// gt comparison: a > b
pub fn mb_gt(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // complex ordering raises in CPython — guard here so the message carries
    // '>' (delegating to mb_lt(b, a) would mislabel the operator).
    if complex_ordering_guard(a, b, ">") {
        return MbValue::from_bool(false);
    }
    if range_ordering_guard(a, b, ">") {
        return MbValue::from_bool(false);
    }
    if enum_ordering_guard(a, b, ">") {
        return MbValue::from_bool(false);
    }
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__gt__") {
        return mb_lt(nb, na);
    }
    if let Some((na, nb)) = str_subclass_ordering_operands(a, b, "__gt__") {
        return mb_lt(nb, na);
    }
    // Try __gt__ dunder on a first
    if let Some(pa) = a.as_ptr() {
        unsafe {
            if let ObjData::Instance { class_name, .. } = &(*pa).data {
                let method = super::class::lookup_method(class_name, "__gt__");
                if !method.is_none() {
                    return MbValue::from_bool(dispatch_richcmp_dunder(a, b, class_name, "__gt__"));
                }
            }
        }
    }
    // functools.total_ordering: derive __gt__ from the class's seed op.
    if super::stdlib::functools_mod::is_total_ordering_instance(a) {
        if let Some(r) =
            super::stdlib::functools_mod::mb_functools_total_ordering_richcmp(a, b, "gt")
        {
            return MbValue::from_bool(r);
        }
    }
    // Reflected fallback: `a > b` where `b` is an instance defining __lt__ and
    // `a` did not handle __gt__ (e.g. `5 > Num(2)` with a plain int on the left)
    // → b.__lt__(a), matching CPython's reflected-comparison rule. Without this,
    // the `mb_lt(b, a)` tail mishandles an instance-vs-inline-primitive pair
    // (mb_values_lt dispatches __lt__ only when both operands are heap objects)
    // and wrongly answers False. (mb_ge needs no such arm because its mb_le(b,a)
    // tail dispatches the instance's __le__ directly.)
    if let Some(pb) = b.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*pb).data {
                if !super::class::lookup_method(class_name, "__lt__").is_none() {
                    if let Some(r) = dispatch_richcmp_dunder_result(b, a, class_name, "__lt__") {
                        return MbValue::from_bool(r);
                    }
                }
            }
        }
    }
    if !can_derive_ordering_from_lt_eq(a, b) {
        return unsupported_ordering_bool(a, b, ">");
    }
    mb_lt(b, a)
}

/// le comparison: a <= b
pub fn mb_le(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // complex ordering raises in CPython — guard with the '<=' op symbol.
    if complex_ordering_guard(a, b, "<=") {
        return MbValue::from_bool(false);
    }
    if range_ordering_guard(a, b, "<=") {
        return MbValue::from_bool(false);
    }
    if enum_ordering_guard(a, b, "<=") {
        return MbValue::from_bool(false);
    }
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__le__") {
        let lt_result = mb_lt(na, nb);
        let eq_result = mb_eq(na, nb);
        return MbValue::from_bool(
            lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false),
        );
    }
    if let Some((na, nb)) = str_subclass_ordering_operands(a, b, "__le__") {
        let lt_result = mb_lt(na, nb);
        let eq_result = mb_eq(na, nb);
        return MbValue::from_bool(
            lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false),
        );
    }
    // Try __le__ dunder on a first
    if let Some(pa) = a.as_ptr() {
        unsafe {
            if let ObjData::Instance { class_name, .. } = &(*pa).data {
                let method = super::class::lookup_method(class_name, "__le__");
                if !method.is_none() {
                    return MbValue::from_bool(dispatch_richcmp_dunder(a, b, class_name, "__le__"));
                }
            }
        }
    }
    // functools.total_ordering: derive __le__ from the class's seed op.
    if super::stdlib::functools_mod::is_total_ordering_instance(a) {
        if let Some(r) =
            super::stdlib::functools_mod::mb_functools_total_ordering_richcmp(a, b, "le")
        {
            return MbValue::from_bool(r);
        }
    }
    // Reflected fallback: `a <= b` where `b` is an instance defining __ge__
    // and `a` did not handle __le__ (e.g. `{1} <= Probe()` with a set on the left)
    // → b.__ge__(a), matching CPython's reflected-comparison rule.
    if let Some(pb) = b.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*pb).data {
                if !super::class::lookup_method(class_name, "__ge__").is_none() {
                    if let Some(r) = dispatch_richcmp_dunder_result(b, a, class_name, "__ge__") {
                        return MbValue::from_bool(r);
                    }
                }
            }
        }
    }
    if !can_derive_ordering_from_lt_eq(a, b) {
        return unsupported_ordering_bool(a, b, "<=");
    }
    let lt_result = mb_lt(a, b);
    let eq_result = mb_eq(a, b);
    MbValue::from_bool(lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false))
}

/// ge comparison: a >= b
pub fn mb_ge(a: MbValue, b: MbValue) -> MbValue {
    let a = int_enum_like_value(a).unwrap_or(a);
    let b = int_enum_like_value(b).unwrap_or(b);
    // complex ordering raises in CPython — guard with the '>=' op symbol.
    if complex_ordering_guard(a, b, ">=") {
        return MbValue::from_bool(false);
    }
    if range_ordering_guard(a, b, ">=") {
        return MbValue::from_bool(false);
    }
    if enum_ordering_guard(a, b, ">=") {
        return MbValue::from_bool(false);
    }
    if let Some((na, nb)) = int_subclass_numeric_operands(a, b, "__ge__") {
        let lt_result = mb_lt(nb, na);
        let eq_result = mb_eq(na, nb);
        return MbValue::from_bool(
            lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false),
        );
    }
    if let Some((na, nb)) = str_subclass_ordering_operands(a, b, "__ge__") {
        let lt_result = mb_lt(nb, na);
        let eq_result = mb_eq(na, nb);
        return MbValue::from_bool(
            lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false),
        );
    }
    // Try __ge__ dunder on a first
    if let Some(pa) = a.as_ptr() {
        unsafe {
            if let ObjData::Instance { class_name, .. } = &(*pa).data {
                let method = super::class::lookup_method(class_name, "__ge__");
                if !method.is_none() {
                    return MbValue::from_bool(dispatch_richcmp_dunder(a, b, class_name, "__ge__"));
                }
            }
        }
    }
    // functools.total_ordering: derive __ge__ from the class's seed op.
    if super::stdlib::functools_mod::is_total_ordering_instance(a) {
        if let Some(r) =
            super::stdlib::functools_mod::mb_functools_total_ordering_richcmp(a, b, "ge")
        {
            return MbValue::from_bool(r);
        }
    }
    if let Some(pb) = b.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref class_name, .. } = (*pb).data {
                if !super::class::lookup_method(class_name, "__le__").is_none() {
                    if let Some(r) = dispatch_richcmp_dunder_result(b, a, class_name, "__le__") {
                        return MbValue::from_bool(r);
                    }
                }
            }
        }
    }
    if !can_derive_ordering_from_lt_eq(a, b) {
        return unsupported_ordering_bool(a, b, ">=");
    }
    let lt_result = mb_lt(b, a);
    let eq_result = mb_eq(a, b);
    MbValue::from_bool(lt_result.as_bool().unwrap_or(false) || eq_result.as_bool().unwrap_or(false))
}

/// ne comparison: a != b
/// Must use !mb_values_eq (not raw bit comparison) because NaN != NaN is True in Python.
pub fn mb_ne(a: MbValue, b: MbValue) -> MbValue {
    if let Some(ne) = mb_instance_ne(a, b) {
        return MbValue::from_bool(ne);
    }
    MbValue::from_bool(!mb_values_eq(a, b))
}

#[cfg(test)]
mod tests {
    use super::super::rc::mb_release;
    use super::*;

    extern "C" fn kwonly_sum_test_fn(a: MbValue, b: MbValue) -> MbValue {
        MbValue::from_int(a.as_int().unwrap() + b.as_int().unwrap())
    }

    fn param_sig(name: &str, kind: i64) -> MbValue {
        MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_ptr(MbObject::new_str(name.to_string())),
            MbValue::from_int(kind),
            MbValue::from_int(0),
            MbValue::none(),
            MbValue::none(),
        ]))
    }

    #[test]
    fn test_call_spread_kwargs_binds_keyword_only_params() {
        let addr = kwonly_sum_test_fn as *const () as usize;
        super::super::module::register_boxed_return_func(addr as u64);
        let func = MbValue::from_func(addr);
        let params = MbValue::from_ptr(MbObject::new_list(vec![
            param_sig("a", 3),
            param_sig("b", 3),
        ]));
        super::super::closure::mb_func_set_params(func, params);

        let kwargs = super::super::dict_ops::mb_dict_new();
        super::super::dict_ops::mb_dict_setitem(
            kwargs,
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_int(2),
        );
        super::super::dict_ops::mb_dict_setitem(
            kwargs,
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
            MbValue::from_int(3),
        );

        let result = mb_call_spread_kwargs(
            func,
            MbValue::from_ptr(MbObject::new_list(Vec::new())),
            kwargs,
        );
        assert_eq!(result.as_int(), Some(5));
    }

    #[test]
    fn test_arithmetic() {
        let a = MbValue::from_int(10);
        let b = MbValue::from_int(3);

        assert_eq!(mb_add(a, b).as_int(), Some(13));
        assert_eq!(mb_sub(a, b).as_int(), Some(7));
        assert_eq!(mb_mul(a, b).as_int(), Some(30));
        assert!(unsafe {
            crate::runtime::bigint_ops::extract_bigint(mb_mul(
                MbValue::from_int(1_103_515_245),
                MbValue::from_int(1_104_988_548),
            ))
            .is_some()
        });
        assert_eq!(mb_mod(a, b).as_int(), Some(1));
        assert_eq!(mb_neg(a).as_int(), Some(-10));
        assert_eq!(
            mb_sub(MbValue::from_bool(true), MbValue::from_bool(false)).as_int(),
            Some(1)
        );
        assert_eq!(
            mb_sub(MbValue::from_bool(true), MbValue::from_float(0.5)).as_float(),
            Some(0.5)
        );
    }

    #[test]
    fn test_division_returns_float() {
        let a = MbValue::from_int(7);
        let b = MbValue::from_int(2);
        let result = mb_div(a, b);
        assert!(result.is_float());
        assert_eq!(result.as_float(), Some(3.5));
    }

    #[test]
    fn test_comparisons() {
        let a = MbValue::from_int(3);
        let b = MbValue::from_int(5);
        assert_eq!(mb_eq(a, a).as_bool(), Some(true));
        assert_eq!(mb_eq(a, b).as_bool(), Some(false));
        assert_eq!(mb_lt(a, b).as_bool(), Some(true));
        assert_eq!(mb_lt(b, a).as_bool(), Some(false));
    }

    fn generic_alias_instance(
        class_name: &str,
        origin_field: &str,
        args_field: &str,
        origin: MbValue,
        args: Vec<MbValue>,
    ) -> MbValue {
        let ptr = MbObject::new_instance(class_name.to_string());
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let mut g = fields.write().unwrap();
                g.insert(origin_field.to_string(), origin);
                g.insert(
                    args_field.to_string(),
                    MbValue::from_ptr(MbObject::new_tuple(args)),
                );
            }
        }
        MbValue::from_ptr(ptr)
    }

    #[test]
    fn test_generic_alias_equality_uses_origin_and_args() {
        let origin = make_type_object("list");
        let typevar = MbValue::from_ptr(MbObject::new_instance("TypeVar".to_string()));
        let alias_a = generic_alias_instance(
            "GenericAlias",
            "__origin__",
            "__args__",
            origin,
            vec![typevar],
        );
        let alias_b = generic_alias_instance(
            "GenericAlias",
            "__origin__",
            "__args__",
            origin,
            vec![typevar],
        );
        let alias_c = generic_alias_instance(
            "GenericAlias",
            "__origin__",
            "__args__",
            origin,
            vec![make_type_object("int")],
        );
        let types_alias = generic_alias_instance(
            "types.GenericAlias",
            "_origin",
            "_args",
            origin,
            vec![typevar],
        );

        assert_eq!(mb_eq(alias_a, alias_b).as_bool(), Some(true));
        assert_eq!(mb_eq(alias_a, alias_c).as_bool(), Some(false));
        assert_eq!(mb_eq(alias_a, types_alias).as_bool(), Some(true));
    }

    #[test]
    fn test_conversions() {
        assert_eq!(mb_int(MbValue::from_float(3.7)).as_int(), Some(3));
        assert_eq!(mb_float(MbValue::from_int(42)).as_float(), Some(42.0));
        assert_eq!(mb_bool(MbValue::from_int(0)).as_bool(), Some(false));
        assert_eq!(mb_bool(MbValue::from_int(1)).as_bool(), Some(true));
    }

    #[test]
    fn test_abs() {
        assert_eq!(mb_abs(MbValue::from_int(-5)).as_int(), Some(5));
        assert_eq!(mb_abs(MbValue::from_int(5)).as_int(), Some(5));
    }

    #[test]
    fn test_range() {
        // CPython `range(3)` is a lazy range; mb_range returns an iterator
        // handle (int id) that yields 0, 1, 2 then exhausts.
        let handle = mb_range(MbValue::from_int(3));
        assert!(
            handle.as_int().is_some(),
            "range() returns a lazy handle id"
        );
        assert_eq!(super::super::iter::mb_next(handle).as_int(), Some(0));
        assert_eq!(super::super::iter::mb_next(handle).as_int(), Some(1));
        assert_eq!(super::super::iter::mb_next(handle).as_int(), Some(2));
        assert!(
            super::super::iter::mb_next(handle).is_none(),
            "range(3) exhausted"
        );
    }

    #[test]
    fn test_range_zero() {
        // `range(0)` yields nothing.
        let handle = mb_range(MbValue::from_int(0));
        assert!(
            handle.as_int().is_some(),
            "range() returns a lazy handle id"
        );
        assert!(
            super::super::iter::mb_next(handle).is_none(),
            "range(0) is empty"
        );
    }

    #[test]
    fn test_range_non_int() {
        // CPython 3.12: `range(3.0)` raises
        // TypeError: 'float' object cannot be interpreted as an integer.
        // mamba is type-strict and must raise too (never silently coerce).
        super::super::exception::mb_clear_exception();
        let result = mb_range(MbValue::from_float(3.0));
        assert!(result.is_none());
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("TypeError"),
        );
        super::super::exception::mb_clear_exception();
    }

    #[test]
    fn test_is_none() {
        assert_eq!(mb_is_none(MbValue::none()).as_bool(), Some(true));
        assert_eq!(mb_is_none(MbValue::from_int(0)).as_bool(), Some(false));
    }

    #[test]
    fn test_div_by_zero() {
        let result = mb_div(MbValue::from_int(1), MbValue::from_int(0));
        assert!(result.is_none());
    }

    #[test]
    fn test_mod_by_zero() {
        let result = mb_mod(MbValue::from_int(5), MbValue::from_int(0));
        assert!(result.is_none());
    }

    #[test]
    fn test_float_arithmetic() {
        let a = MbValue::from_float(2.5);
        let b = MbValue::from_float(1.5);
        assert_eq!(mb_add(a, b).as_float(), Some(4.0));
        assert_eq!(mb_sub(a, b).as_float(), Some(1.0));
        assert_eq!(mb_mul(a, b).as_float(), Some(3.75));
    }

    #[test]
    fn test_neg_float() {
        assert_eq!(mb_neg(MbValue::from_float(3.5)).as_float(), Some(-3.5));
    }

    #[test]
    fn test_neg_none() {
        assert!(mb_neg(MbValue::none()).is_none());
    }

    #[test]
    fn test_abs_float() {
        assert_eq!(mb_abs(MbValue::from_float(-2.5)).as_float(), Some(2.5));
    }

    #[test]
    fn test_abs_non_numeric() {
        // abs() on non-numeric (None/bool) returns 0 to avoid JIT double-free UAF.
        let v = MbValue::none();
        assert_eq!(mb_abs(v).as_int(), Some(0));
    }

    #[test]
    fn test_int_from_bool() {
        assert_eq!(mb_int(MbValue::from_bool(true)).as_int(), Some(1));
        assert_eq!(mb_int(MbValue::from_bool(false)).as_int(), Some(0));
    }

    #[test]
    fn test_int_from_none_raises_type_error() {
        // CPython: int(None) raises TypeError; the runtime entry returns
        // None after routing through the exception machinery.
        assert!(mb_int(MbValue::none()).is_none());
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_float_from_bool() {
        assert_eq!(mb_float(MbValue::from_bool(true)).as_float(), Some(1.0));
        assert_eq!(mb_float(MbValue::from_bool(false)).as_float(), Some(0.0));
    }

    #[test]
    fn test_float_from_none_raises_type_error() {
        // CPython: float(None) raises TypeError; the runtime entry returns
        // None after routing through the exception machinery.
        assert!(mb_float(MbValue::none()).is_none());
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_float_passthrough() {
        let f = MbValue::from_float(3.14);
        assert_eq!(mb_float(f).as_float(), Some(3.14));
    }

    #[test]
    fn test_complex_from_complex_and_imag_preserves_real_part() {
        let value = mb_complex(
            MbValue::from_ptr(MbObject::new_complex(1.0, 2.0)),
            MbValue::from_int(3),
        );
        let (real, imag) = value
            .as_ptr()
            .map(|ptr| unsafe {
                if let ObjData::Complex(re, im) = (*ptr).data {
                    (re, im)
                } else {
                    panic!("expected complex result")
                }
            })
            .expect("expected complex pointer");
        assert_eq!((real, imag), (1.0, 5.0));
    }

    #[test]
    fn test_int_passthrough() {
        let i = MbValue::from_int(42);
        assert_eq!(mb_int(i).as_int(), Some(42));
    }

    #[test]
    fn test_bool_none() {
        assert_eq!(mb_bool(MbValue::none()).as_bool(), Some(false));
    }

    #[test]
    fn test_bool_float() {
        assert_eq!(mb_bool(MbValue::from_float(0.0)).as_bool(), Some(false));
        assert_eq!(mb_bool(MbValue::from_float(1.0)).as_bool(), Some(true));
    }

    #[test]
    fn test_bool_string() {
        let empty = MbValue::from_ptr(MbObject::new_str(String::new()));
        assert_eq!(mb_bool(empty).as_bool(), Some(false));
        let nonempty = MbValue::from_ptr(MbObject::new_str("hi".to_string()));
        assert_eq!(mb_bool(nonempty).as_bool(), Some(true));
    }

    #[test]
    fn test_bool_list() {
        let empty = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert_eq!(mb_bool(empty).as_bool(), Some(false));
        let nonempty = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        assert_eq!(mb_bool(nonempty).as_bool(), Some(true));
    }

    #[test]
    fn test_bool_tuple() {
        let empty = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        assert_eq!(mb_bool(empty).as_bool(), Some(false));
        let nonempty = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(1)]));
        assert_eq!(mb_bool(nonempty).as_bool(), Some(true));
    }

    #[test]
    fn test_bool_bool() {
        assert_eq!(mb_bool(MbValue::from_bool(true)).as_bool(), Some(true));
        assert_eq!(mb_bool(MbValue::from_bool(false)).as_bool(), Some(false));
    }

    #[test]
    fn test_len_str() {
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        assert_eq!(mb_len(s).as_int(), Some(5));
    }

    #[test]
    fn test_len_list() {
        let l = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        assert_eq!(mb_len(l).as_int(), Some(2));
    }

    #[test]
    fn test_len_tuple() {
        let t = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_int(1)]));
        assert_eq!(mb_len(t).as_int(), Some(1));
    }

    #[test]
    fn test_len_dict() {
        let d = MbValue::from_ptr(MbObject::new_dict());
        assert_eq!(mb_len(d).as_int(), Some(0));
    }

    #[test]
    fn test_len_non_collection() {
        assert!(mb_len(MbValue::from_int(42)).is_none());
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("TypeError")
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_str_from_int() {
        let s = mb_str(MbValue::from_int(42));
        unsafe {
            if let ObjData::Str(ref text) = (*s.as_ptr().unwrap()).data {
                assert_eq!(text, "42");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_str_from_bool() {
        let s = mb_str(MbValue::from_bool(true));
        unsafe {
            if let ObjData::Str(ref text) = (*s.as_ptr().unwrap()).data {
                assert_eq!(text, "True");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_str_from_none() {
        let s = mb_str(MbValue::none());
        unsafe {
            if let ObjData::Str(ref text) = (*s.as_ptr().unwrap()).data {
                assert_eq!(text, "None");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_str_passthrough() {
        // str(str_val) returns a NEW object (owned ref) so JIT can independently
        // release input and output VRegs without double-free UAF. The content must
        // match the original string, but the pointer identity may differ.
        let original = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let result = mb_str(original);
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "hello");
            } else {
                panic!("expected str object");
            }
        }
    }

    #[test]
    fn test_str_construct_decodes_bytes_with_encoding_and_errors() {
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![0x63, 0x61, 0x66, 0xc3, 0xa9]));
        let encoding = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
        let decoded = mb_str_construct(bytes, encoding, MbValue::none());
        unsafe {
            if let ObjData::Str(ref s) = (*decoded.as_ptr().unwrap()).data {
                assert_eq!(s, "café");
            } else {
                panic!("expected decoded str object");
            }
        }

        let invalid = MbValue::from_ptr(MbObject::new_bytes(vec![0x63, 0x61, 0x66, 0xff]));
        let encoding = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));
        let errors = MbValue::from_ptr(MbObject::new_str("ignore".to_string()));
        let decoded = mb_str_construct(invalid, encoding, errors);
        unsafe {
            if let ObjData::Str(ref s) = (*decoded.as_ptr().unwrap()).data {
                assert_eq!(s, "caf");
            } else {
                panic!("expected decoded str object");
            }
        }
    }

    #[test]
    fn test_str_construct_reports_cpython_argument_type_errors() {
        let bytes = MbValue::from_ptr(MbObject::new_bytes(vec![0x78]));
        let utf8 = MbValue::from_ptr(MbObject::new_str("utf-8".to_string()));

        crate::runtime::exception::mb_clear_exception();
        assert!(mb_str_construct(MbValue::from_int(1), utf8, MbValue::none()).is_none());
        let exc = crate::runtime::exception::mb_get_exception();
        assert_eq!(
            crate::runtime::exception::get_exception_message_pub(exc).as_deref(),
            Some("decoding to str: need a bytes-like object, int found")
        );

        crate::runtime::exception::mb_clear_exception();
        assert!(mb_str_construct(bytes, MbValue::from_int(1), MbValue::none()).is_none());
        let exc = crate::runtime::exception::mb_get_exception();
        assert_eq!(
            crate::runtime::exception::get_exception_message_pub(exc).as_deref(),
            Some("str() argument 'encoding' must be str, not int")
        );

        crate::runtime::exception::mb_clear_exception();
        assert!(mb_str_construct(bytes, utf8, MbValue::from_int(1)).is_none());
        let exc = crate::runtime::exception::mb_get_exception();
        assert_eq!(
            crate::runtime::exception::get_exception_message_pub(exc).as_deref(),
            Some("str() argument 'errors' must be str, not int")
        );

        crate::runtime::exception::mb_clear_exception();
    }

    /// Helper: extract __name__ from a type object returned by mb_type.
    fn type_name(t: MbValue) -> String {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*t.as_ptr().unwrap()).data {
                let fields = fields.read().unwrap();
                if let Some(name_val) = fields.get("__name__") {
                    if let Some(ptr) = name_val.as_ptr() {
                        if let ObjData::Str(ref s) = (*ptr).data {
                            return s.clone();
                        }
                    }
                }
            }
            panic!("expected type object with __name__");
        }
    }

    #[test]
    fn test_type_int() {
        assert_eq!(type_name(mb_type(MbValue::from_int(42))), "int");
    }

    #[test]
    fn test_type_float() {
        assert_eq!(type_name(mb_type(MbValue::from_float(1.0))), "float");
    }

    #[test]
    fn test_type_bool() {
        assert_eq!(type_name(mb_type(MbValue::from_bool(true))), "bool");
    }

    #[test]
    fn test_type_none() {
        assert_eq!(type_name(mb_type(MbValue::none())), "NoneType");
    }

    #[test]
    fn test_type_str() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_str(
                "hi".to_string()
            )))),
            "str"
        );
    }

    #[test]
    fn test_type_list() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_list(vec![])))),
            "list"
        );
    }

    #[test]
    fn test_type_tuple() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_tuple(vec![])))),
            "tuple"
        );
    }

    #[test]
    fn test_builtin_type_object_survives_return_release() {
        let name = MbValue::from_ptr(MbObject::new_str("TypeReleaseSentinel".to_string()));
        let type_obj = mb_builtin_type_obj(name);
        unsafe {
            mb_release(type_obj.as_ptr().unwrap());
        }

        let name_again = MbValue::from_ptr(MbObject::new_str("TypeReleaseSentinel".to_string()));
        let type_obj_again = mb_builtin_type_obj(name_again);
        assert_eq!(type_name(type_obj_again), "TypeReleaseSentinel");
    }

    #[test]
    fn test_unbound_builtin_classmethod_int_from_bytes_keeps_type_receiver() {
        crate::runtime::exception::mb_clear_exception();
        let int_type = mb_builtin_type_obj(MbValue::from_ptr(MbObject::new_str("int".to_string())));
        let method = super::super::class::mb_getattr(
            int_type,
            MbValue::from_ptr(MbObject::new_str("from_bytes".to_string())),
        );
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_bytes(vec![0x01, 0x02])),
            MbValue::from_ptr(MbObject::new_str("big".to_string())),
        ]));
        let result = mb_call_spread(method, args);
        assert_eq!(result.as_int(), Some(258));
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
    }

    #[test]
    fn test_unbound_builtin_classmethod_dict_fromkeys_keeps_type_receiver() {
        crate::runtime::exception::mb_clear_exception();
        let dict_type =
            mb_builtin_type_obj(MbValue::from_ptr(MbObject::new_str("dict".to_string())));
        let method = super::super::class::mb_getattr(
            dict_type,
            MbValue::from_ptr(MbObject::new_str("fromkeys".to_string())),
        );
        let keys = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
        ]));
        let result = mb_call_spread(
            method,
            MbValue::from_ptr(MbObject::new_list(vec![keys, MbValue::from_int(7)])),
        );
        assert_eq!(
            super::super::dict_ops::mb_dict_getitem(
                result,
                MbValue::from_ptr(MbObject::new_str("a".to_string()))
            )
            .as_int(),
            Some(7)
        );
        assert_eq!(
            super::super::dict_ops::mb_dict_getitem(
                result,
                MbValue::from_ptr(MbObject::new_str("b".to_string()))
            )
            .as_int(),
            Some(7)
        );
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
    }

    #[test]
    fn test_unbound_builtin_instance_method_still_uses_first_arg_as_receiver() {
        crate::runtime::exception::mb_clear_exception();
        let str_type = mb_builtin_type_obj(MbValue::from_ptr(MbObject::new_str("str".to_string())));
        let method = super::super::class::mb_getattr(
            str_type,
            MbValue::from_ptr(MbObject::new_str("startswith".to_string())),
        );
        let result = mb_call_spread(
            method,
            MbValue::from_ptr(MbObject::new_list(vec![
                MbValue::from_ptr(MbObject::new_str("abc".to_string())),
                MbValue::from_ptr(MbObject::new_str("a".to_string())),
            ])),
        );
        assert_eq!(result.as_bool(), Some(true));
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
    }

    #[test]
    fn test_type_dict() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_dict()))),
            "dict"
        );
    }

    #[test]
    fn test_type_set() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_set(vec![])))),
            "set"
        );
    }

    #[test]
    fn test_type_frozenset() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_frozenset(vec![])))),
            "frozenset"
        );
    }

    #[test]
    fn test_type_bytes() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_bytes(vec![1, 2])))),
            "bytes"
        );
    }

    #[test]
    fn test_type_bytearray() {
        assert_eq!(
            type_name(mb_type(MbValue::from_ptr(MbObject::new_bytearray(vec![1])))),
            "bytearray"
        );
    }

    #[test]
    fn test_lt_float() {
        assert_eq!(
            mb_lt(MbValue::from_float(1.0), MbValue::from_float(2.0)).as_bool(),
            Some(true)
        );
        assert_eq!(
            mb_lt(MbValue::from_float(2.0), MbValue::from_float(1.0)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_lt_non_comparable() {
        assert_eq!(
            mb_lt(MbValue::none(), MbValue::from_int(1)).as_bool(),
            Some(false)
        );
    }

    #[test]
    fn test_gt_le_ge_ne() {
        let a = MbValue::from_int(3);
        let b = MbValue::from_int(5);
        assert_eq!(mb_gt(b, a).as_bool(), Some(true));
        assert_eq!(mb_le(a, b).as_bool(), Some(true));
        assert_eq!(mb_le(a, a).as_bool(), Some(true));
        assert_eq!(mb_ge(b, a).as_bool(), Some(true));
        assert_eq!(mb_ge(a, a).as_bool(), Some(true));
        assert_eq!(mb_ne(a, b).as_bool(), Some(true));
        assert_eq!(mb_ne(a, a).as_bool(), Some(false));
    }

    #[test]
    fn test_not() {
        assert_eq!(mb_not(MbValue::from_bool(true)).as_bool(), Some(false));
        assert_eq!(mb_not(MbValue::from_bool(false)).as_bool(), Some(true));
        assert_eq!(mb_not(MbValue::from_int(0)).as_bool(), Some(true));
        assert_eq!(mb_not(MbValue::from_int(1)).as_bool(), Some(false));
    }

    #[test]
    fn test_min() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(3),
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        assert_eq!(mb_min(list).as_int(), Some(1));
    }

    #[test]
    fn test_max() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(3),
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        assert_eq!(mb_max(list).as_int(), Some(3));
    }

    #[test]
    fn test_min_max_empty() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert!(mb_min(list).is_none());
        let list2 = MbValue::from_ptr(MbObject::new_list(vec![]));
        assert!(mb_max(list2).is_none());
    }

    #[test]
    fn test_sum_ints() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        assert_eq!(mb_sum(list).as_int(), Some(6));
    }

    #[test]
    fn test_sum_floats() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_float(1.5),
            MbValue::from_float(2.5),
        ]));
        assert_eq!(mb_sum(list).as_float(), Some(4.0));
    }

    #[test]
    fn test_sum_mixed() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_float(2.5),
        ]));
        assert_eq!(mb_sum(list).as_float(), Some(3.5));
    }

    #[test]
    fn test_sorted() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(3),
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let sorted = mb_sorted(list, MbValue::none());
        unsafe {
            let ptr = sorted.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items[0].as_int(), Some(1));
                assert_eq!(items[1].as_int(), Some(2));
                assert_eq!(items[2].as_int(), Some(3));
            }
        }
    }

    #[test]
    fn test_any_all() {
        let truthy = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
        ]));
        assert_eq!(mb_any(truthy).as_bool(), Some(true));

        let all_false = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(0),
            MbValue::from_int(0),
        ]));
        assert_eq!(mb_any(all_false).as_bool(), Some(false));

        let all_true = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        assert_eq!(mb_all(all_true).as_bool(), Some(true));

        let not_all = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(0),
        ]));
        assert_eq!(mb_all(not_all).as_bool(), Some(false));
    }

    #[test]
    fn test_repr_int() {
        let r = mb_repr(MbValue::from_int(42));
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "42");
            }
        }
    }

    #[test]
    fn test_repr_str() {
        let val = MbValue::from_ptr(MbObject::new_str("hi".to_string()));
        let r = mb_repr(val);
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "'hi'");
            }
        }

        let printable = MbValue::from_ptr(MbObject::new_str("café".to_string()));
        let r = mb_repr(printable);
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "'café'");
            }
        }

        let unassigned = MbValue::from_ptr(MbObject::new_str("\u{0378}".to_string()));
        let r = mb_repr(unassigned);
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "'\\u0378'");
            }
        }
    }

    #[test]
    fn test_repr_none() {
        let r = mb_repr(MbValue::none());
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "None");
            }
        }
    }

    #[test]
    fn test_repr_bool() {
        let r = mb_repr(MbValue::from_bool(true));
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "True");
            }
        }
    }

    #[test]
    fn test_pep695_type_alias_repr_and_str_use_alias_name() {
        let alias = super::super::pep695::mb_pep695_type_alias(
            MbValue::from_ptr(MbObject::new_str("Alias".to_string())),
            MbValue::none(),
            MbValue::from_ptr(MbObject::new_tuple(vec![])),
        );

        let repr = mb_repr(alias);
        let text = mb_str(alias);
        unsafe {
            match &(*repr.as_ptr().unwrap()).data {
                ObjData::Str(s) => assert_eq!(s, "Alias"),
                _ => panic!("expected repr string"),
            }
            match &(*text.as_ptr().unwrap()).data {
                ObjData::Str(s) => assert_eq!(s, "Alias"),
                _ => panic!("expected str string"),
            }
            match &(*alias.as_ptr().unwrap()).data {
                ObjData::Instance { fields, .. } => {
                    let guard = fields.read().unwrap();
                    let name = guard.get("__name__").copied().unwrap();
                    let params = guard.get("__type_params__").copied().unwrap();
                    assert!(guard.get("__mb_value_thunk__").is_some());
                    match &(*name.as_ptr().unwrap()).data {
                        ObjData::Str(s) => assert_eq!(s, "Alias"),
                        _ => panic!("expected __name__ string"),
                    }
                    match &(*params.as_ptr().unwrap()).data {
                        ObjData::Tuple(items) => assert!(items.is_empty()),
                        _ => panic!("expected empty __type_params__"),
                    }
                }
                _ => panic!("expected alias instance"),
            }
        }
    }

    #[test]
    fn test_pep695_type_params_repr_and_str_use_param_name() {
        for (kind, class_name) in [(0, "TypeVar"), (1, "TypeVarTuple"), (2, "ParamSpec")] {
            let param = super::super::pep695::mb_pep695_typevar(
                MbValue::from_ptr(MbObject::new_str("T".to_string())),
                MbValue::from_int(kind),
                MbValue::none(),
                MbValue::none(),
                MbValue::none(),
            );
            let repr = mb_repr(param);
            let text = mb_str(param);
            let tuple_repr = mb_repr(MbValue::from_ptr(MbObject::new_tuple(vec![param])));
            unsafe {
                match &(*repr.as_ptr().unwrap()).data {
                    ObjData::Str(s) => assert_eq!(s, "T"),
                    _ => panic!("expected repr string"),
                }
                match &(*text.as_ptr().unwrap()).data {
                    ObjData::Str(s) => assert_eq!(s, "T"),
                    _ => panic!("expected str string"),
                }
                match &(*tuple_repr.as_ptr().unwrap()).data {
                    ObjData::Str(s) => assert_eq!(s, "(T,)"),
                    _ => panic!("expected tuple repr string"),
                }
                match &(*param.as_ptr().unwrap()).data {
                    ObjData::Instance {
                        class_name: actual,
                        fields,
                    } => {
                        assert_eq!(actual, class_name);
                        let guard = fields.read().unwrap();
                        let name = guard.get("__name__").copied().unwrap();
                        match &(*name.as_ptr().unwrap()).data {
                            ObjData::Str(s) => assert_eq!(s, "T"),
                            _ => panic!("expected __name__ string"),
                        }
                    }
                    _ => panic!("expected type param instance"),
                }
            }
        }
    }

    #[test]
    fn test_hash_int() {
        assert_eq!(mb_hash(MbValue::from_int(42)).as_int(), Some(42));
        // CPython remaps hash(-1) to -2
        assert_eq!(mb_hash(MbValue::from_int(-1)).as_int(), Some(-2));
    }

    #[test]
    fn test_hash_non_integral_float_fits_mamba_int() {
        let hash = mb_hash(MbValue::from_float(1.5));
        let value = hash.as_int().expect("hash(float) should return int");
        assert!((-(1i64 << 47)..(1i64 << 47)).contains(&value));
    }

    #[test]
    fn test_hash_bool() {
        assert_eq!(mb_hash(MbValue::from_bool(true)).as_int(), Some(1));
        assert_eq!(mb_hash(MbValue::from_bool(false)).as_int(), Some(0));
    }

    #[test]
    fn test_hash_none() {
        assert_eq!(mb_hash(MbValue::none()).as_int(), Some(0));
    }

    #[test]
    fn test_chr() {
        let c = mb_chr(MbValue::from_int(65));
        unsafe {
            if let ObjData::Str(ref s) = (*c.as_ptr().unwrap()).data {
                assert_eq!(s, "A");
            }
        }
    }

    #[test]
    fn test_chr_invalid() {
        assert!(mb_chr(MbValue::none()).is_none());
    }

    #[test]
    fn test_ord() {
        let s = MbValue::from_ptr(MbObject::new_str("A".to_string()));
        assert_eq!(mb_ord(s).as_int(), Some(65));
    }

    #[test]
    fn test_ord_multi_char() {
        let s = MbValue::from_ptr(MbObject::new_str("AB".to_string()));
        assert!(mb_ord(s).is_none());
    }

    #[test]
    fn test_hex() {
        let h = mb_hex(MbValue::from_int(255));
        unsafe {
            if let ObjData::Str(ref s) = (*h.as_ptr().unwrap()).data {
                assert_eq!(s, "0xff");
            }
        }
    }

    #[test]
    fn test_hex_negative() {
        let h = mb_hex(MbValue::from_int(-10));
        unsafe {
            if let ObjData::Str(ref s) = (*h.as_ptr().unwrap()).data {
                assert_eq!(s, "-0xa");
            }
        }
    }

    #[test]
    fn test_oct() {
        let o = mb_oct(MbValue::from_int(8));
        unsafe {
            if let ObjData::Str(ref s) = (*o.as_ptr().unwrap()).data {
                assert_eq!(s, "0o10");
            }
        }
    }

    #[test]
    fn test_bin() {
        let b = mb_bin(MbValue::from_int(5));
        unsafe {
            if let ObjData::Str(ref s) = (*b.as_ptr().unwrap()).data {
                assert_eq!(s, "0b101");
            }
        }
    }

    #[test]
    fn test_hex_oct_bin_non_int() {
        assert!(mb_hex(MbValue::none()).is_none());
        assert!(mb_oct(MbValue::none()).is_none());
        assert!(mb_bin(MbValue::none()).is_none());
    }

    #[test]
    fn test_pow_int() {
        assert_eq!(
            mb_pow(MbValue::from_int(2), MbValue::from_int(10)).as_int(),
            Some(1024)
        );
    }

    #[test]
    fn test_pow_negative_exp() {
        let result = mb_pow(MbValue::from_int(2), MbValue::from_int(-1));
        assert_eq!(result.as_float(), Some(0.5));
    }

    #[test]
    fn test_pow_float() {
        let result = mb_pow(MbValue::from_float(2.0), MbValue::from_float(0.5));
        let f = result.as_float().unwrap();
        assert!((f - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn test_round_float() {
        // CPython: `round(f, n)` (ndigits given) preserves the float type,
        // even when n == 0. Only `round(f)` with no ndigits returns int.
        assert_eq!(
            mb_round(MbValue::from_float(3.7), MbValue::from_int(0)).as_float(),
            Some(4.0)
        );
        assert_eq!(
            mb_round(MbValue::from_float(3.14159), MbValue::from_int(2)).as_float(),
            Some(3.14)
        );
        assert_eq!(
            mb_round(MbValue::from_float(3.7), MbValue::none()).as_int(),
            Some(4)
        );
    }

    #[test]
    fn test_round_int() {
        assert_eq!(
            mb_round(MbValue::from_int(42), MbValue::from_int(0)).as_int(),
            Some(42)
        );
    }

    #[test]
    fn test_round_negative_ndigits() {
        assert_eq!(
            mb_round(MbValue::from_int(1234), MbValue::from_int(-2)).as_int(),
            Some(1200)
        );
    }

    #[test]
    fn test_divmod() {
        let result = mb_divmod(MbValue::from_int(7), MbValue::from_int(3));
        unsafe {
            if let ObjData::Tuple(ref items) = (*result.as_ptr().unwrap()).data {
                assert_eq!(items[0].as_int(), Some(2));
                assert_eq!(items[1].as_int(), Some(1));
            } else {
                panic!("expected tuple");
            }
        }
    }

    #[test]
    fn test_divmod_zero() {
        // divmod by zero must raise a catchable ZeroDivisionError (CPython parity),
        // not silently return None.
        super::super::exception::clear_current_exception();
        let r = mb_divmod(MbValue::from_int(5), MbValue::from_int(0));
        assert!(r.is_none());
        assert_eq!(
            super::super::exception::current_exception_type().as_deref(),
            Some("ZeroDivisionError")
        );
        super::super::exception::clear_current_exception();
    }

    #[test]
    fn test_floordiv_int() {
        assert_eq!(
            mb_floordiv(MbValue::from_int(7), MbValue::from_int(2)).as_int(),
            Some(3)
        );
    }

    #[test]
    fn test_floordiv_float() {
        assert_eq!(
            mb_floordiv(MbValue::from_float(7.0), MbValue::from_float(2.0)).as_float(),
            Some(3.0)
        );
    }

    #[test]
    fn test_floordiv_zero() {
        crate::runtime::exception::mb_clear_exception();
        assert!(mb_floordiv(MbValue::from_int(5), MbValue::from_int(0)).is_none());
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_floordiv_zero_raises_zerodivision_error() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_floordiv(MbValue::from_int(5), MbValue::from_int(0));
        assert!(
            result.is_none(),
            "mb_floordiv by zero should return the none sentinel"
        );
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "ZeroDivisionError should be raised on integer floor div by zero",
        );
        let exc = crate::runtime::exception::mb_get_exception();
        let exc_type = crate::runtime::exception::get_exception_type_pub(exc);
        assert_eq!(exc_type.as_deref(), Some("ZeroDivisionError"));
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_frozenset_new() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(1),
        ]));
        let fs = mb_frozenset_new(list);
        unsafe {
            if let ObjData::FrozenSet(ref items) = (*fs.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 2);
            } else {
                panic!("expected frozenset");
            }
        }
    }

    #[test]
    fn test_frozenset_empty() {
        let fs = mb_frozenset_new(MbValue::none());
        unsafe {
            if let ObjData::FrozenSet(ref items) = (*fs.as_ptr().unwrap()).data {
                assert_eq!(items.len(), 0);
            } else {
                panic!("expected frozenset");
            }
        }
    }

    #[test]
    fn test_set_from_iterable() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(2),
        ]));
        let s = mb_set_from_iterable(list);
        unsafe {
            if let ObjData::Set(ref lock) = (*s.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            } else {
                panic!("expected set");
            }
        }
    }

    #[test]
    fn test_set_from_iterable_empty() {
        let s = mb_set_from_iterable(MbValue::none());
        unsafe {
            if let ObjData::Set(ref lock) = (*s.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 0);
            } else {
                panic!("expected set");
            }
        }
    }

    #[test]
    fn test_assertion_error_none() {
        mb_assertion_error(MbValue::none());
        let has = crate::runtime::exception::mb_has_exception();
        assert!(has.as_bool() == Some(true), "exception must be set");
    }

    #[test]
    fn test_assertion_error_int() {
        mb_assertion_error(MbValue::from_int(42));
        let has = crate::runtime::exception::mb_has_exception();
        assert!(has.as_bool() == Some(true), "exception must be set");
    }

    #[test]
    fn test_assertion_error_str() {
        mb_assertion_error(MbValue::from_ptr(MbObject::new_str("oops".to_string())));
        let has = crate::runtime::exception::mb_has_exception();
        assert!(has.as_bool() == Some(true), "exception must be set");
    }

    #[test]
    fn test_assertion_error_no_msg() {
        mb_assertion_error_no_msg();
        let has = crate::runtime::exception::mb_has_exception();
        assert!(has.as_bool() == Some(true), "exception must be set");
    }

    #[test]
    fn test_eval_int() {
        let expr = MbValue::from_ptr(MbObject::new_str("42".to_string()));
        assert_eq!(mb_eval(expr).as_int(), Some(42));
    }

    #[test]
    fn test_eval_float() {
        let expr = MbValue::from_ptr(MbObject::new_str("3.14".to_string()));
        assert_eq!(mb_eval(expr).as_float(), Some(3.14));
    }

    #[test]
    fn test_eval_bool() {
        let expr = MbValue::from_ptr(MbObject::new_str("True".to_string()));
        assert_eq!(mb_eval(expr).as_bool(), Some(true));
        let expr2 = MbValue::from_ptr(MbObject::new_str("False".to_string()));
        assert_eq!(mb_eval(expr2).as_bool(), Some(false));
    }

    #[test]
    fn test_eval_none() {
        let expr = MbValue::from_ptr(MbObject::new_str("None".to_string()));
        assert!(mb_eval(expr).is_none());
    }

    #[test]
    fn test_eval_non_str() {
        assert!(mb_eval(MbValue::from_int(1)).is_none());
    }

    #[test]
    fn test_exec_returns_none() {
        assert!(mb_exec(MbValue::none()).is_none());
    }

    #[test]
    fn test_exec_with_globals_star_import_uses_all() {
        let mut attrs = std::collections::HashMap::new();
        attrs.insert("public_a".to_string(), MbValue::from_int(1));
        attrs.insert("public_extra".to_string(), MbValue::from_int(2));
        attrs.insert("_private_listed".to_string(), MbValue::from_int(3));
        attrs.insert("_private_one".to_string(), MbValue::from_int(4));
        attrs.insert(
            "__all__".to_string(),
            MbValue::from_ptr(MbObject::new_list(vec![
                MbValue::from_ptr(MbObject::new_str("public_a".to_string())),
                MbValue::from_ptr(MbObject::new_str("_private_listed".to_string())),
            ])),
        );
        crate::runtime::module::mb_module_register("mamba_exec_star_test_mod", attrs);

        let ns = crate::runtime::dict_ops::mb_dict_new();
        let code = MbValue::from_ptr(MbObject::new_str(
            "from mamba_exec_star_test_mod import *".to_string(),
        ));
        assert!(mb_exec_with_globals(code, ns).is_none());

        let public_a = crate::runtime::dict_ops::mb_dict_get(
            ns,
            MbValue::from_ptr(MbObject::new_str("public_a".to_string())),
            MbValue::none(),
        );
        let private_listed = crate::runtime::dict_ops::mb_dict_get(
            ns,
            MbValue::from_ptr(MbObject::new_str("_private_listed".to_string())),
            MbValue::none(),
        );
        let public_extra = crate::runtime::dict_ops::mb_dict_get(
            ns,
            MbValue::from_ptr(MbObject::new_str("public_extra".to_string())),
            MbValue::none(),
        );
        let private_one = crate::runtime::dict_ops::mb_dict_get(
            ns,
            MbValue::from_ptr(MbObject::new_str("_private_one".to_string())),
            MbValue::none(),
        );

        assert_eq!(public_a.as_int(), Some(1));
        assert_eq!(private_listed.as_int(), Some(3));
        assert!(public_extra.is_none());
        assert!(private_one.is_none());
    }

    #[test]
    fn test_exec_with_globals_try_finally_flow() {
        crate::runtime::exception::mb_clear_exception();
        let ns = crate::runtime::dict_ops::mb_dict_new();
        let trace = MbValue::from_ptr(MbObject::new_list(Vec::new()));
        crate::runtime::dict_ops::mb_dict_setitem(
            ns,
            MbValue::from_ptr(MbObject::new_str("TRACE".to_string())),
            trace,
        );
        let code = MbValue::from_ptr(MbObject::new_str(
            r#"
def inner():
    try:
        TRACE.append("try-before-return")
        return "try-return"
    finally:
        TRACE.append("finally-overrides")
        return "finally-return"
result = inner()
for i in range(3):
    try:
        TRACE.append(f"try:{i}")
        if i == 1:
            raise ValueError("trigger")
    except ValueError:
        TRACE.append(f"except:{i}")
    finally:
        TRACE.append(f"finally:{i}")
        if i == 1:
            break
TRACE.append("after-loop")
"#
            .to_string(),
        ));

        assert!(mb_exec_with_globals(code, ns).is_none());
        assert_eq!(
            eval_str_value(crate::runtime::dict_ops::mb_dict_get(
                ns,
                MbValue::from_ptr(MbObject::new_str("result".to_string())),
                MbValue::none(),
            ))
            .as_deref(),
            Some("finally-return")
        );
        let got: Vec<String> = extract_items(trace)
            .into_iter()
            .map(|value| eval_str_value(value).unwrap_or_default())
            .collect();
        assert_eq!(
            got,
            vec![
                "try-before-return",
                "finally-overrides",
                "try:0",
                "finally:0",
                "try:1",
                "except:1",
                "finally:1",
                "after-loop",
            ]
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_exec_with_globals_decorator_errors_at_definition_time() {
        fn assert_exec_exception(ns: MbValue, source: &str, expected: &str) {
            crate::runtime::exception::mb_clear_exception();
            let code = MbValue::from_ptr(MbObject::new_str(source.to_string()));

            assert!(mb_exec_with_globals(code, ns).is_none());
            assert_eq!(
                crate::runtime::exception::current_exception_type().as_deref(),
                Some(expected)
            );

            crate::runtime::exception::mb_clear_exception();
        }

        let ns = crate::runtime::dict_ops::mb_dict_new();
        assert_exec_exception(ns, "@missing\ndef f():\n    return 1\n", "NameError");

        crate::runtime::dict_ops::mb_dict_setitem(
            ns,
            MbValue::from_ptr(MbObject::new_str("nullval".to_string())),
            MbValue::none(),
        );
        assert_exec_exception(ns, "@nullval\ndef f():\n    return 1\n", "TypeError");
        assert_exec_exception(
            ns,
            "@nullval.attr\ndef f():\n    return 1\n",
            "AttributeError",
        );

        assert_exec_exception(
            ns,
            r#"
def needs_one(n):
    def deco(func):
        return func
    return deco
@needs_one()
def f():
    return 1
"#,
            "TypeError",
        );
    }

    #[test]
    fn test_eval_expression_arith() {
        let s = MbValue::from_ptr(MbObject::new_str("1+2".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(3));
        let s = MbValue::from_ptr(MbObject::new_str("3*4-1".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(11));
        let s = MbValue::from_ptr(MbObject::new_str("2**10".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(1024));
        let s = MbValue::from_ptr(MbObject::new_str("10//3".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(3));
        let s = MbValue::from_ptr(MbObject::new_str("10%3".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(1));
    }

    #[test]
    fn test_eval_expression_unary() {
        let s = MbValue::from_ptr(MbObject::new_str("-5".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(-5));
        let s = MbValue::from_ptr(MbObject::new_str("not True".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(false));
        let s = MbValue::from_ptr(MbObject::new_str("not False".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(true));
        let s = MbValue::from_ptr(MbObject::new_str("~0".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(-1));
    }

    #[test]
    fn test_eval_expression_comparison() {
        let s = MbValue::from_ptr(MbObject::new_str("1 < 2".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(true));
        let s = MbValue::from_ptr(MbObject::new_str("2 == 2".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(true));
        let s = MbValue::from_ptr(MbObject::new_str("1 < 2 < 3".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(true));
        let s = MbValue::from_ptr(MbObject::new_str("1 < 2 < 1".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(false));
    }

    #[test]
    fn test_eval_expression_bitwise() {
        let s = MbValue::from_ptr(MbObject::new_str("0xff & 0x0f".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(0x0f));
        let s = MbValue::from_ptr(MbObject::new_str("1 | 4".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(5));
        let s = MbValue::from_ptr(MbObject::new_str("1 << 4".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(16));
    }

    #[test]
    fn test_eval_expression_logical_shortcircuit() {
        let s = MbValue::from_ptr(MbObject::new_str("True and 5".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(5));
        let s = MbValue::from_ptr(MbObject::new_str("False and 5".to_string()));
        assert_eq!(mb_eval(s).as_bool(), Some(false));
        let s = MbValue::from_ptr(MbObject::new_str("False or 7".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(7));
    }

    #[test]
    fn test_eval_expression_ternary() {
        let s = MbValue::from_ptr(MbObject::new_str("1 if True else 2".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(1));
        let s = MbValue::from_ptr(MbObject::new_str("1 if False else 2".to_string()));
        assert_eq!(mb_eval(s).as_int(), Some(2));
    }

    #[test]
    fn test_eval_with_globals_resolves_names() {
        crate::runtime::exception::mb_clear_exception();
        let globals = crate::runtime::dict_ops::mb_dict_new();
        crate::runtime::dict_ops::mb_dict_setitem(globals, make_str("x"), MbValue::from_int(41));
        let expr = make_str("x + 1");
        assert_eq!(mb_eval_with_globals(expr, globals).as_int(), Some(42));
    }

    #[test]
    fn test_eval_with_namespaces_prefers_locals() {
        crate::runtime::exception::mb_clear_exception();
        let globals = crate::runtime::dict_ops::mb_dict_new();
        let locals = crate::runtime::dict_ops::mb_dict_new();
        crate::runtime::dict_ops::mb_dict_setitem(globals, make_str("x"), MbValue::from_int(1));
        crate::runtime::dict_ops::mb_dict_setitem(locals, make_str("x"), MbValue::from_int(41));
        let expr = make_str("x + 1");
        assert_eq!(
            mb_eval_with_namespaces(expr, globals, locals).as_int(),
            Some(42)
        );
    }

    #[test]
    fn test_exec_with_globals_locals_stores_in_locals() {
        crate::runtime::exception::mb_clear_exception();
        let globals = crate::runtime::dict_ops::mb_dict_new();
        let locals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals_locals(make_str("p = 10\nq = p * 2"), globals, locals);
        let p = crate::runtime::dict_ops::mb_dict_get(locals, make_str("p"), MbValue::none());
        let q = crate::runtime::dict_ops::mb_dict_get(locals, make_str("q"), MbValue::none());
        assert_eq!(p.as_int(), Some(10));
        assert_eq!(q.as_int(), Some(20));
    }

    #[test]
    fn test_eval_expression_unresolved_returns_none() {
        // Identifier resolution is out of scope -- preserves the prior
        // literal-only fallback's behavior for unsupported expressions.
        let s = MbValue::from_ptr(MbObject::new_str("undefined_var".to_string()));
        assert!(mb_eval(s).is_none());
        let s = MbValue::from_ptr(MbObject::new_str("f(1)".to_string()));
        assert!(mb_eval(s).is_none());
    }

    // ── compile() tests (#976) ──────────────────────────────────────────────

    fn make_str(s: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(s.to_string()))
    }

    /// AC1: compile("1+2", "<test>", "eval") returns a non-None code object.
    #[test]
    fn test_compile_eval_returns_code_object() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str("1+2"), make_str("<test>"), make_str("eval"));
        assert!(!result.is_none(), "eval should return a code object");
        assert!(result.is_ptr(), "result should be a pointer");
        unsafe {
            if let ObjData::CodeObject {
                ref mode,
                ref filename,
                ..
            } = (*result.as_ptr().unwrap()).data
            {
                assert_eq!(mode, "eval");
                assert_eq!(filename, "<test>");
            } else {
                panic!("expected CodeObject data");
            }
        }
    }

    #[test]
    fn test_eval_runs_compile_eval_code_object() {
        crate::runtime::exception::mb_clear_exception();
        let code = mb_compile(make_str("40 + 2"), make_str("<test>"), make_str("eval"));
        assert_eq!(mb_eval(code).as_int(), Some(42));
    }

    /// AC2a: compile("x = 1\ny = 2", "<test>", "exec") succeeds.
    #[test]
    fn test_compile_exec_multi_stmt_ok() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(
            make_str("x = 1\ny = 2"),
            make_str("<test>"),
            make_str("exec"),
        );
        assert!(
            !result.is_none(),
            "exec should succeed for multi-statement source"
        );
        assert!(result.is_ptr());
    }

    #[test]
    fn test_exec_runs_compile_exec_code_object() {
        crate::runtime::exception::mb_clear_exception();
        let code = mb_compile(
            make_str("x = 5\ny = x * 3"),
            make_str("<test>"),
            make_str("exec"),
        );
        let globals = crate::runtime::dict_ops::mb_dict_new();
        mb_exec_with_globals(code, globals);
        let x = crate::runtime::dict_ops::mb_dict_get(globals, make_str("x"), MbValue::none());
        let y = crate::runtime::dict_ops::mb_dict_get(globals, make_str("y"), MbValue::none());
        assert_eq!(x.as_int(), Some(5));
        assert_eq!(y.as_int(), Some(15));
    }

    /// AC2b: compile("x = 1", "<test>", "eval") raises SyntaxError.
    #[test]
    fn test_compile_eval_rejects_statement() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str("x = 1"), make_str("<test>"), make_str("eval"));
        assert!(result.is_none(), "eval mode should reject a statement");
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "SyntaxError should be raised"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    /// AC3: compile("1 +", "<test>", "eval") raises SyntaxError with location info.
    #[test]
    fn test_compile_eval_syntax_error() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str("1 +"), make_str("<test>"), make_str("eval"));
        assert!(result.is_none(), "invalid syntax should raise SyntaxError");
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "SyntaxError should be raised"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    /// AC5: compile("x = 1", "<test>", "single") succeeds for single statement.
    #[test]
    fn test_compile_single_one_stmt_ok() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str("x = 1"), make_str("<test>"), make_str("single"));
        assert!(!result.is_none(), "single mode should accept one statement");
        assert!(result.is_ptr());
    }

    /// AC5: compile("x = 1\ny = 2", "<test>", "single") raises SyntaxError.
    #[test]
    fn test_compile_single_multi_stmt_error() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(
            make_str("x = 1\ny = 2"),
            make_str("<test>"),
            make_str("single"),
        );
        assert!(
            result.is_none(),
            "single mode should reject multi-statement"
        );
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "SyntaxError should be raised"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_compile_invalid_nonlocal_raises_syntax_error() {
        crate::runtime::exception::mb_clear_exception();
        let src = "def outer():\n    def inner():\n        nonlocal missing\n        missing = 1\n";
        let result = mb_compile(make_str(src), make_str("<test>"), make_str("exec"));
        assert!(result.is_none(), "invalid nonlocal should fail");
        let exc = crate::runtime::exception::mb_catch_exception();
        assert_eq!(
            crate::runtime::exception::get_exception_type_pub(exc).as_deref(),
            Some("SyntaxError")
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_compile_exec_rejects_unlexed_trailing_operator() {
        crate::runtime::exception::mb_clear_exception();
        let src = "def fact(x):\n\treturn x!\n";
        let result = mb_compile(make_str(src), make_str("?"), make_str("exec"));
        assert!(result.is_none(), "invalid trailing token should fail");
        let exc = crate::runtime::class::mb_catch_exception_instance();
        assert_eq!(
            crate::runtime::exception::get_exception_type_pub(exc).as_deref(),
            Some("SyntaxError")
        );
        let offset = crate::runtime::class::mb_getattr(exc, make_str("offset"));
        assert_eq!(offset.as_int(), Some(10));
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_compile_exec_rejects_unexpected_indent_as_indentation_error() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str(" print(2)"), make_str("?"), make_str("exec"));
        assert!(result.is_none(), "unexpected indent should fail");
        let exc = crate::runtime::class::mb_catch_exception_instance();
        assert_eq!(
            crate::runtime::exception::get_exception_type_pub(exc).as_deref(),
            Some("IndentationError")
        );
        let offset = crate::runtime::class::mb_getattr(exc, make_str("offset"));
        assert!(
            offset.is_none(),
            "unexpected indent should not synthesize a caret"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_compile_exec_rejects_mismatched_unindent_as_indentation_error() {
        crate::runtime::exception::mb_clear_exception();
        let src = "def spam():\n  print(1)\n print(2)";
        let result = mb_compile(make_str(src), make_str("?"), make_str("exec"));
        assert!(result.is_none(), "mismatched unindent should fail");
        let exc = crate::runtime::class::mb_catch_exception_instance();
        assert_eq!(
            crate::runtime::exception::get_exception_type_pub(exc).as_deref(),
            Some("IndentationError")
        );
        let offset = crate::runtime::class::mb_getattr(exc, make_str("offset"));
        assert_eq!(offset.as_int(), Some(10));
        crate::runtime::exception::mb_clear_exception();
    }

    /// R2: unknown mode raises ValueError.
    #[test]
    fn test_compile_invalid_mode() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile(make_str("1+2"), make_str("<test>"), make_str("badmode"));
        assert!(result.is_none(), "invalid mode should raise ValueError");
        assert_eq!(
            crate::runtime::exception::mb_has_exception().as_bool(),
            Some(true),
            "ValueError should be raised"
        );
        crate::runtime::exception::mb_clear_exception();
    }

    /// R6: source as bytes.
    #[test]
    fn test_compile_bytes_source() {
        crate::runtime::exception::mb_clear_exception();
        let src = MbValue::from_ptr(MbObject::new_bytes(b"1+2".to_vec()));
        let result = mb_compile(src, make_str("<test>"), make_str("eval"));
        assert!(!result.is_none(), "bytes source should work in eval mode");
        assert!(result.is_ptr());
    }

    /// R5: 5-arg form with flags and dont_inherit accepted.
    #[test]
    fn test_compile_5_arg_form() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_compile_5(
            make_str("1+2"),
            make_str("<test>"),
            make_str("eval"),
            MbValue::from_int(0),
            MbValue::from_bool(false),
        );
        assert!(!result.is_none(), "5-arg compile should work");
        assert!(result.is_ptr());
    }

    #[test]
    fn test_compile_ast_name_constant_raises_value_error() {
        for constant in ["True", "False", "None"] {
            crate::runtime::exception::mb_clear_exception();
            let load =
                super::super::stdlib::ast_mod::mb_ast_construct_marker("mb_ast_node_Load", &[])
                    .expect("Load");
            let name = super::super::stdlib::ast_mod::mb_ast_construct_marker(
                "mb_ast_node_Name",
                &[
                    MbValue::from_ptr(MbObject::new_str(constant.to_string())),
                    load,
                ],
            )
            .expect("Name");
            let expr = super::super::stdlib::ast_mod::mb_ast_construct_marker(
                "mb_ast_node_Expression",
                &[name],
            )
            .expect("Expression");
            let expr = super::super::stdlib::ast_mod::mb_ast_fix_missing_locations(expr);
            let result = mb_compile(expr, make_str("<test>"), make_str("eval"));
            assert!(result.is_none(), "{constant} should fail AST validation");
            assert_eq!(
                crate::runtime::exception::current_exception_type().as_deref(),
                Some("ValueError")
            );
            let exc = crate::runtime::exception::mb_get_exception();
            assert_eq!(
                crate::runtime::exception::get_exception_message_pub(exc).as_deref(),
                Some(format!("identifier field can't represent '{constant}' constant").as_str())
            );
        }
        crate::runtime::exception::mb_clear_exception();
    }

    #[test]
    fn test_compile_ast_import_from_level_none_executes() {
        crate::runtime::exception::mb_clear_exception();
        let empty_args = MbValue::from_ptr(MbObject::new_list(vec![]));
        let alias_kwargs = crate::runtime::dict_ops::mb_dict_new();
        crate::runtime::dict_ops::mb_dict_setitem(
            alias_kwargs,
            make_str("name"),
            make_str("sleep"),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            alias_kwargs,
            make_str("asname"),
            MbValue::none(),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            alias_kwargs,
            make_str("lineno"),
            MbValue::from_int(0),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            alias_kwargs,
            make_str("col_offset"),
            MbValue::from_int(0),
        );
        let alias = mb_call_spread_kwargs(make_str("alias"), empty_args, alias_kwargs);
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
        let import_from_kwargs = crate::runtime::dict_ops::mb_dict_new();
        crate::runtime::dict_ops::mb_dict_setitem(
            import_from_kwargs,
            make_str("module"),
            make_str("time"),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            import_from_kwargs,
            make_str("names"),
            MbValue::from_ptr(MbObject::new_list(vec![alias])),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            import_from_kwargs,
            make_str("level"),
            MbValue::none(),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            import_from_kwargs,
            make_str("lineno"),
            MbValue::from_int(0),
        );
        crate::runtime::dict_ops::mb_dict_setitem(
            import_from_kwargs,
            make_str("col_offset"),
            MbValue::from_int(0),
        );
        let import_from = mb_call_spread_kwargs(
            make_str("ImportFrom"),
            MbValue::from_ptr(MbObject::new_list(vec![])),
            import_from_kwargs,
        );
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
        let module = super::super::stdlib::ast_mod::mb_ast_construct_marker(
            "mb_ast_node_Module",
            &[
                MbValue::from_ptr(MbObject::new_list(vec![import_from])),
                MbValue::from_ptr(MbObject::new_list(vec![])),
            ],
        )
        .expect("Module node");
        assert!(!module.is_none(), "constructor should build a Module node");
        let body = crate::runtime::class::mb_getattr(module, make_str("body"));
        assert_eq!(crate::runtime::builtins::extract_items(body).len(), 1);
        let import_from = crate::runtime::builtins::extract_items(body)[0];
        assert_eq!(
            crate::runtime::class::mb_getattr(import_from, make_str("module"))
                .as_ptr()
                .and_then(|ptr| unsafe {
                    if let ObjData::Str(ref s) = (*ptr).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .as_deref(),
            Some("time")
        );
        assert!(
            crate::runtime::class::mb_getattr(import_from, make_str("level")).is_none(),
            "level=None should remain None on the AST node"
        );

        let code = mb_compile(module, make_str("<test>"), make_str("exec"));
        assert!(!code.is_none(), "AST Module compile should succeed");
        assert_eq!(crate::runtime::exception::current_exception_type(), None);
        unsafe {
            match &(*code.as_ptr().expect("code object")).data {
                ObjData::CodeObject {
                    source, mode, ast, ..
                } => {
                    assert_eq!(mode, "exec");
                    assert_eq!(source, "from time import sleep");
                    assert!(matches!(
                        ast.stmts.first().map(|stmt| &stmt.node),
                        Some(crate::parser::ast::Stmt::Import { names: Some(names), .. })
                            if names.len() == 1 && names[0].0 == "sleep"
                    ));
                }
                _ => panic!("expected code object"),
            }
        }
    }

    #[test]
    fn test_globals_locals_empty() {
        let g = mb_globals();
        let l = mb_locals();
        assert_eq!(mb_len(g).as_int(), Some(0));
        assert_eq!(mb_len(l).as_int(), Some(0));
    }

    #[test]
    fn test_filter_none_func() {
        // CPython `filter(None, [0,1,0,2])` lazily yields the truthy elements
        // 1, 2. mb_filter returns an iterator handle (int id) driven by mb_next.
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(0),
            MbValue::from_int(1),
            MbValue::from_int(0),
            MbValue::from_int(2),
        ]));
        let handle = mb_filter(MbValue::none(), list);
        assert!(
            handle.as_int().is_some(),
            "filter() returns a lazy handle id"
        );
        assert_eq!(super::super::iter::mb_next(handle).as_int(), Some(1));
        assert_eq!(super::super::iter::mb_next(handle).as_int(), Some(2));
        assert!(
            super::super::iter::mb_next(handle).is_none(),
            "filter exhausted"
        );
    }

    #[test]
    fn test_min_max_from_tuple() {
        let tup = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(5),
            MbValue::from_int(2),
            MbValue::from_int(8),
        ]));
        assert_eq!(mb_min(tup).as_int(), Some(2));
        let tup2 = MbValue::from_ptr(MbObject::new_tuple(vec![
            MbValue::from_int(5),
            MbValue::from_int(2),
            MbValue::from_int(8),
        ]));
        assert_eq!(mb_max(tup2).as_int(), Some(8));
    }

    #[test]
    fn test_callable_int() {
        // Integers are not callable in Python
        assert_eq!(mb_callable(MbValue::from_int(42)).as_bool(), Some(false));
    }

    #[test]
    fn test_callable_none() {
        assert_eq!(mb_callable(MbValue::none()).as_bool(), Some(false));
    }

    #[test]
    fn test_call_spread_non_callable_int_raises_type_error() {
        crate::runtime::exception::mb_clear_exception();
        let result = mb_call_spread(
            MbValue::from_int(42),
            MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(3)])),
        );
        assert!(result.is_none());
        assert_eq!(
            crate::runtime::exception::current_exception_type().as_deref(),
            Some("TypeError")
        );
        let exc = crate::runtime::exception::mb_get_exception();
        assert_eq!(
            crate::runtime::exception::get_exception_message_pub(exc).as_deref(),
            Some("'int' object is not callable")
        );
        crate::runtime::exception::mb_clear_exception();
    }

    // --- mb_print return-value tests (builtins fix) ---

    use super::super::output::{begin_capture, end_capture};

    /// mb_print must return MbValue::none(), not MbValue::from_int(0).
    /// Before the fix, the void return caused the JIT to see an undefined
    /// register that NaN-boxing decoded as TAG_INT(0), producing a spurious "0"
    /// in program output.
    #[test]
    fn test_mb_print_returns_none_for_int() {
        let prev = begin_capture();
        let ret = mb_print(MbValue::from_int(42));
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print must return MbValue::none(), got non-none for int input"
        );
        assert!(!ret.is_int(), "mb_print must not return TAG_INT(0)");
    }

    #[test]
    fn test_mb_print_returns_none_for_float() {
        let prev = begin_capture();
        let ret = mb_print(MbValue::from_float(3.14));
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print must return MbValue::none() for float input"
        );
    }

    #[test]
    fn test_mb_print_returns_none_for_bool() {
        let prev = begin_capture();
        let ret = mb_print(MbValue::from_bool(true));
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print must return MbValue::none() for bool input"
        );
    }

    #[test]
    fn test_mb_print_returns_none_for_none() {
        let prev = begin_capture();
        let ret = mb_print(MbValue::none());
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print must return MbValue::none() for None input"
        );
    }

    #[test]
    fn test_mb_print_returns_none_for_string() {
        let prev = begin_capture();
        let s = MbValue::from_ptr(MbObject::new_str("hello".to_string()));
        let ret = mb_print(s);
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print must return MbValue::none() for string input"
        );
        unsafe {
            mb_release(s.as_ptr().unwrap());
        }
    }

    /// mb_print_args must also return MbValue::none(), not TAG_INT(0).
    #[test]
    fn test_mb_print_args_returns_none_for_list() {
        let prev = begin_capture();
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let ret = mb_print_args(list);
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print_args must return MbValue::none() for list input"
        );
        assert!(!ret.is_int(), "mb_print_args must not return TAG_INT(0)");
        unsafe {
            mb_release(list.as_ptr().unwrap());
        }
    }

    #[test]
    fn test_mb_print_args_returns_none_for_fallback() {
        // When args_list is not a list, mb_print_args falls through to mb_print.
        let prev = begin_capture();
        let ret = mb_print_args(MbValue::from_int(99));
        let _ = end_capture(prev);
        assert!(
            ret.is_none(),
            "mb_print_args fallback must return MbValue::none()"
        );
    }

    /// breakpoint() must evaluate without erroring and return None,
    /// matching CPython's PEP 553 contract. The default mamba hook is
    /// the print fallback (no pdb wired in); PYTHONBREAKPOINT=0 still
    /// returns None. (#1256)
    #[test]
    fn test_mb_breakpoint_returns_none() {
        let ret = mb_breakpoint();
        assert!(ret.is_none(), "breakpoint() must return None");
    }

    /// Under PYTHONBREAKPOINT=0 the hook must be a silent no-op and
    /// still return None. We can't safely mutate env in parallel tests
    /// without isolation, so this test sets the var, calls the hook,
    /// then restores. (#1256)
    #[test]
    fn test_mb_breakpoint_disabled_returns_none() {
        let prev = std::env::var("PYTHONBREAKPOINT").ok();
        // SAFETY: tests run in-process; restoring below keeps other
        // tests in this module unaffected. The variable is only read
        // by `mb_breakpoint` itself.
        unsafe {
            std::env::set_var("PYTHONBREAKPOINT", "0");
        }
        let ret = mb_breakpoint();
        match prev {
            Some(v) => unsafe {
                std::env::set_var("PYTHONBREAKPOINT", v);
            },
            None => unsafe {
                std::env::remove_var("PYTHONBREAKPOINT");
            },
        }
        assert!(
            ret.is_none(),
            "breakpoint() with PYTHONBREAKPOINT=0 must return None"
        );
    }

    // --- mb_print output correctness tests ---

    #[test]
    fn test_mb_print_output_int() {
        let prev = begin_capture();
        mb_print(MbValue::from_int(42));
        let out = end_capture(prev);
        assert_eq!(out, "42\n");
    }

    #[test]
    fn test_mb_print_output_float_whole() {
        let prev = begin_capture();
        mb_print(MbValue::from_float(1.0));
        let out = end_capture(prev);
        // Python prints 1.0 not 1
        assert_eq!(out, "1.0\n");
    }

    #[test]
    fn test_mb_print_output_float_fractional() {
        let prev = begin_capture();
        mb_print(MbValue::from_float(3.14));
        let out = end_capture(prev);
        assert_eq!(out, "3.14\n");
    }

    #[test]
    fn test_mb_print_output_bool_true() {
        let prev = begin_capture();
        mb_print(MbValue::from_bool(true));
        let out = end_capture(prev);
        assert_eq!(out, "True\n");
    }

    #[test]
    fn test_mb_print_output_bool_false() {
        let prev = begin_capture();
        mb_print(MbValue::from_bool(false));
        let out = end_capture(prev);
        assert_eq!(out, "False\n");
    }

    #[test]
    fn test_mb_print_output_none() {
        let prev = begin_capture();
        mb_print(MbValue::none());
        let out = end_capture(prev);
        assert_eq!(out, "None\n");
    }

    #[test]
    fn test_mb_print_output_string() {
        let prev = begin_capture();
        let s = MbValue::from_ptr(MbObject::new_str("hello world".to_string()));
        mb_print(s);
        let out = end_capture(prev);
        assert_eq!(out, "hello world\n");
        unsafe {
            mb_release(s.as_ptr().unwrap());
        }
    }

    // --- mb_print_args output correctness tests ---

    #[test]
    fn test_mb_print_args_output_multi() {
        let prev = begin_capture();
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        mb_print_args(list);
        let out = end_capture(prev);
        assert_eq!(out, "1 2 3\n");
        unsafe {
            mb_release(list.as_ptr().unwrap());
        }
    }

    #[test]
    fn test_mb_print_args_output_single() {
        let prev = begin_capture();
        let list = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(42)]));
        mb_print_args(list);
        let out = end_capture(prev);
        assert_eq!(out, "42\n");
        unsafe {
            mb_release(list.as_ptr().unwrap());
        }
    }

    #[test]
    fn test_mb_print_args_output_empty_list() {
        let prev = begin_capture();
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        mb_print_args(list);
        let out = end_capture(prev);
        assert_eq!(out, "\n");
        unsafe {
            mb_release(list.as_ptr().unwrap());
        }
    }

    // ── R3: mb_print_kwargs tests (sep/end) ──

    #[test]
    fn test_print_kwargs_sep() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let sep = MbValue::from_ptr(MbObject::new_str("-".to_string()));
        mb_print_kwargs(args, sep, MbValue::none());
        let out = end_capture(prev);
        assert_eq!(out, "1-2-3\n");
    }

    #[test]
    fn test_print_kwargs_end() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("hello".to_string()),
        )]));
        let end = MbValue::from_ptr(MbObject::new_str("!!!\n".to_string()));
        mb_print_kwargs(args, MbValue::none(), end);
        let out = end_capture(prev);
        assert_eq!(out, "hello!!!\n");
    }

    #[test]
    fn test_print_kwargs_sep_and_end() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
        ]));
        let sep = MbValue::from_ptr(MbObject::new_str(", ".to_string()));
        let end = MbValue::from_ptr(MbObject::new_str(".\n".to_string()));
        mb_print_kwargs(args, sep, end);
        let out = end_capture(prev);
        assert_eq!(out, "a, b.\n");
    }

    #[test]
    fn test_print_kwargs_empty_end() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_ptr(
            MbObject::new_str("x".to_string()),
        )]));
        let end = MbValue::from_ptr(MbObject::new_str(String::new()));
        mb_print_kwargs(args, MbValue::none(), end);
        let out = end_capture(prev);
        assert_eq!(out, "x");
    }

    #[test]
    fn test_print_kwargs_defaults() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        mb_print_kwargs(args, MbValue::none(), MbValue::none());
        let out = end_capture(prev);
        assert_eq!(out, "1 2\n");
    }

    #[test]
    fn test_print_kwargs_returns_none() {
        let prev = begin_capture();
        let args = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(1)]));
        let ret = mb_print_kwargs(args, MbValue::none(), MbValue::none());
        let _ = end_capture(prev);
        assert!(ret.is_none(), "mb_print_kwargs must return None");
    }

    // ── R4: mb_sorted_kwargs tests (key/reverse) ──

    #[test]
    fn test_sorted_kwargs_reverse() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(3),
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let result = mb_sorted_kwargs(list, MbValue::none(), MbValue::from_bool(true));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items[0].as_int(), Some(3));
                assert_eq!(items[1].as_int(), Some(2));
                assert_eq!(items[2].as_int(), Some(1));
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_sorted_kwargs_reverse_preserves_equal_key_order() {
        let first = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(10),
        ]));
        let second = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(2),
            MbValue::from_int(20),
        ]));
        let short = MbValue::from_ptr(MbObject::new_list(vec![MbValue::from_int(3)]));
        let list = MbValue::from_ptr(MbObject::new_list(vec![first, second, short]));
        let key = MbValue::from_ptr(MbObject::new_str("len".to_string()));
        let result = mb_sorted_kwargs(list, key, MbValue::from_bool(true));
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items[0].to_bits(), first.to_bits());
                assert_eq!(items[1].to_bits(), second.to_bits());
                assert_eq!(items[2].to_bits(), short.to_bits());
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_sorted_kwargs_no_key_no_reverse() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(1),
            MbValue::from_int(3),
        ]));
        let result = mb_sorted_kwargs(list, MbValue::none(), MbValue::none());
        unsafe {
            let ptr = result.as_ptr().unwrap();
            if let ObjData::List(ref lock) = (*ptr).data {
                let items = lock.read().unwrap();
                assert_eq!(items[0].as_int(), Some(1));
                assert_eq!(items[1].as_int(), Some(3));
                assert_eq!(items[2].as_int(), Some(5));
            } else {
                panic!("expected list");
            }
        }
    }

    // ── R4: mb_min_kwargs / mb_max_kwargs tests ──

    #[test]
    fn test_min_kwargs_default_on_empty() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        let default = MbValue::from_ptr(MbObject::new_str("empty".to_string()));
        let result = mb_min_kwargs(list, MbValue::none(), default);
        // Should return the default value
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "empty");
            } else {
                panic!("expected str default");
            }
        }
    }

    #[test]
    fn test_max_kwargs_default_on_empty() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        let default = MbValue::from_ptr(MbObject::new_str("empty".to_string()));
        let result = mb_max_kwargs(list, MbValue::none(), default);
        assert!(result.is_ptr());
        unsafe {
            if let ObjData::Str(ref s) = (*result.as_ptr().unwrap()).data {
                assert_eq!(s, "empty");
            } else {
                panic!("expected str default");
            }
        }
    }

    #[test]
    fn test_min_kwargs_no_key() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(2),
            MbValue::from_int(8),
        ]));
        let result = mb_min_kwargs(list, MbValue::none(), MbValue::none());
        assert_eq!(result.as_int(), Some(2));
    }

    #[test]
    fn test_max_kwargs_no_key() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(5),
            MbValue::from_int(2),
            MbValue::from_int(8),
        ]));
        let result = mb_max_kwargs(list, MbValue::none(), MbValue::none());
        assert_eq!(result.as_int(), Some(8));
    }

    // ── R4: mb_sum_with_start tests ──

    #[test]
    fn test_sum_with_start_int() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
            MbValue::from_int(3),
        ]));
        let result = mb_sum_with_start(list, MbValue::from_int(10));
        assert_eq!(result.as_int(), Some(16));
    }

    #[test]
    fn test_sum_with_start_float() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_float(1.5),
            MbValue::from_float(2.5),
        ]));
        let result = mb_sum_with_start(list, MbValue::from_float(10.0));
        assert_eq!(result.as_float(), Some(14.0));
    }

    #[test]
    fn test_sum_with_start_mixed() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![
            MbValue::from_int(1),
            MbValue::from_int(2),
        ]));
        let result = mb_sum_with_start(list, MbValue::from_float(0.5));
        assert_eq!(result.as_float(), Some(3.5));
    }

    #[test]
    fn test_sum_with_start_empty() {
        let list = MbValue::from_ptr(MbObject::new_list(vec![]));
        let result = mb_sum_with_start(list, MbValue::from_int(100));
        assert_eq!(result.as_int(), Some(100));
    }

    // ── R5: mb_pow_mod (three-arg pow) tests ──

    #[test]
    fn test_pow_mod_basic() {
        // pow(2, 10, 1000) = 1024 % 1000 = 24
        let result = mb_pow_mod(
            MbValue::from_int(2),
            MbValue::from_int(10),
            MbValue::from_int(1000),
        );
        assert_eq!(result.as_int(), Some(24));
    }

    #[test]
    fn test_pow_mod_zero_exp() {
        // pow(5, 0, 3) = 1
        let result = mb_pow_mod(
            MbValue::from_int(5),
            MbValue::from_int(0),
            MbValue::from_int(3),
        );
        assert_eq!(result.as_int(), Some(1));
    }

    #[test]
    fn test_pow_mod_zero_modulus() {
        // pow(2, 3, 0) should return none (ValueError)
        let result = mb_pow_mod(
            MbValue::from_int(2),
            MbValue::from_int(3),
            MbValue::from_int(0),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_pow_mod_negative_exp() {
        // CPython 3.8+: pow(2, -1, 5) returns the modular inverse of 2 mod 5,
        // which is 3 (since 2 * 3 ≡ 1 mod 5). gcd(2, 5) == 1 so inverse exists.
        let result = mb_pow_mod(
            MbValue::from_int(2),
            MbValue::from_int(-1),
            MbValue::from_int(5),
        );
        assert_eq!(result.as_int(), Some(3));
    }

    #[test]
    fn test_pow_mod_negative_exp_no_inverse() {
        // gcd(2, 4) == 2, so 2 has no modular inverse mod 4 — CPython raises
        // ValueError; Mamba returns none.
        let result = mb_pow_mod(
            MbValue::from_int(2),
            MbValue::from_int(-1),
            MbValue::from_int(4),
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_pow_mod_large() {
        // pow(3, 100, 97) — Fermat's little theorem: 3^96 ≡ 1 (mod 97)
        // 3^100 = 3^96 * 3^4 ≡ 1 * 81 ≡ 81 (mod 97)
        let result = mb_pow_mod(
            MbValue::from_int(3),
            MbValue::from_int(100),
            MbValue::from_int(97),
        );
        assert_eq!(result.as_int(), Some(81));
    }

    // ── R5: mb_int_base tests ──

    #[test]
    fn test_int_base_hex() {
        let val = MbValue::from_ptr(MbObject::new_str("ff".to_string()));
        let result = mb_int_base(val, MbValue::from_int(16));
        assert_eq!(result.as_int(), Some(255));
    }

    #[test]
    fn test_int_base_hex_with_prefix() {
        let val = MbValue::from_ptr(MbObject::new_str("0xff".to_string()));
        let result = mb_int_base(val, MbValue::from_int(16));
        assert_eq!(result.as_int(), Some(255));
    }

    #[test]
    fn test_int_base_binary() {
        let val = MbValue::from_ptr(MbObject::new_str("1010".to_string()));
        let result = mb_int_base(val, MbValue::from_int(2));
        assert_eq!(result.as_int(), Some(10));
    }

    #[test]
    fn test_int_base_binary_with_prefix() {
        let val = MbValue::from_ptr(MbObject::new_str("0b1010".to_string()));
        let result = mb_int_base(val, MbValue::from_int(2));
        assert_eq!(result.as_int(), Some(10));
    }

    #[test]
    fn test_int_base_octal() {
        let val = MbValue::from_ptr(MbObject::new_str("77".to_string()));
        let result = mb_int_base(val, MbValue::from_int(8));
        assert_eq!(result.as_int(), Some(63));
    }

    #[test]
    fn test_int_base_octal_with_prefix() {
        let val = MbValue::from_ptr(MbObject::new_str("0o77".to_string()));
        let result = mb_int_base(val, MbValue::from_int(8));
        assert_eq!(result.as_int(), Some(63));
    }

    #[test]
    fn test_int_base_decimal() {
        let val = MbValue::from_ptr(MbObject::new_str("42".to_string()));
        let result = mb_int_base(val, MbValue::from_int(10));
        assert_eq!(result.as_int(), Some(42));
    }

    #[test]
    fn test_int_base_with_whitespace() {
        let val = MbValue::from_ptr(MbObject::new_str("  ff  ".to_string()));
        let result = mb_int_base(val, MbValue::from_int(16));
        assert_eq!(result.as_int(), Some(255));
    }

    // ── R6: mb_chr / mb_ord Unicode edge cases ──

    #[test]
    fn test_chr_unicode_emoji() {
        // chr(128522) = 😊 (U+1F60A SMILING FACE WITH SMILING EYES)
        let c = mb_chr(MbValue::from_int(128522));
        unsafe {
            if let ObjData::Str(ref s) = (*c.as_ptr().unwrap()).data {
                assert_eq!(s, "😊");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_chr_unicode_cjk() {
        // chr(20013) = '中'
        let c = mb_chr(MbValue::from_int(20013));
        unsafe {
            if let ObjData::Str(ref s) = (*c.as_ptr().unwrap()).data {
                assert_eq!(s, "中");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_chr_zero() {
        let c = mb_chr(MbValue::from_int(0));
        unsafe {
            if let ObjData::Str(ref s) = (*c.as_ptr().unwrap()).data {
                assert_eq!(s, "\0");
            } else {
                panic!("expected str");
            }
        }
    }

    #[test]
    fn test_chr_invalid_codepoint() {
        // 0x110000 is beyond the valid Unicode range
        let c = mb_chr(MbValue::from_int(0x110000));
        assert!(c.is_none());
    }

    #[test]
    fn test_chr_lone_surrogate_roundtrip() {
        let c = mb_chr(MbValue::from_int(0xD800));
        assert_eq!(mb_len(c).as_int(), Some(1));
        assert_eq!(mb_ord(c).as_int(), Some(0xD800));
        assert!(mb_richcmp_eq(c, mb_chr(MbValue::from_int(0xD800))));
        assert_eq!(
            mb_hash(c).as_int(),
            mb_hash(mb_chr(MbValue::from_int(0xD800))).as_int()
        );

        let r = mb_repr(c);
        let a = mb_ascii(c);
        unsafe {
            if let ObjData::Str(ref s) = (*r.as_ptr().unwrap()).data {
                assert_eq!(s, "'\\ud800'");
            } else {
                panic!("expected repr str");
            }
            if let ObjData::Str(ref s) = (*a.as_ptr().unwrap()).data {
                assert_eq!(s, "'\\ud800'");
            } else {
                panic!("expected ascii str");
            }
        }

        let dict = MbValue::from_ptr(MbObject::new_dict());
        super::super::dict_ops::mb_dict_setitem(dict, c, MbValue::from_int(42));
        assert_eq!(
            super::super::dict_ops::mb_dict_getitem(dict, mb_chr(MbValue::from_int(0xD800)))
                .as_int(),
            Some(42)
        );
    }

    #[test]
    fn test_ord_unicode_emoji() {
        let s = MbValue::from_ptr(MbObject::new_str("😊".to_string()));
        assert_eq!(mb_ord(s).as_int(), Some(128522));
    }

    #[test]
    fn test_ord_unicode_cjk() {
        let s = MbValue::from_ptr(MbObject::new_str("中".to_string()));
        assert_eq!(mb_ord(s).as_int(), Some(20013));
    }

    #[test]
    fn test_ord_empty_string() {
        let s = MbValue::from_ptr(MbObject::new_str(String::new()));
        assert!(mb_ord(s).is_none());
    }

    #[test]
    fn test_chr_ord_roundtrip() {
        // chr(ord(c)) == c for various codepoints
        for codepoint in [65, 233, 8364, 20013, 128522] {
            let ch = mb_chr(MbValue::from_int(codepoint));
            let ord_val = mb_ord(ch);
            assert_eq!(
                ord_val.as_int(),
                Some(codepoint),
                "chr/ord roundtrip failed for codepoint {codepoint}"
            );
        }
    }

    // ── type() 3-arg tests (#974) ──

    #[test]
    fn test_type3_basic_class_creation() {
        super::super::class::cleanup_all_classes();

        // type('MyClass', (object,), {'x': 42})
        let name = MbValue::from_ptr(MbObject::new_str("TestType3Basic".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str("object".to_string()),
        )]));
        let dict = super::super::dict_ops::mb_dict_new();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
            MbValue::from_int(42),
        );

        let type_obj = mb_type3(name, bases, dict);
        // Should return a type object (Instance with class_name="type")
        assert!(type_obj.as_ptr().is_some());
        unsafe {
            let ptr = type_obj.as_ptr().unwrap();
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                assert_eq!(class_name, "type");
                let f = fields.read().unwrap();
                let name_val = f.get("__name__").unwrap();
                let name_str = name_val.as_ptr().and_then(|p| {
                    if let ObjData::Str(ref s) = (*p).data {
                        Some(s.clone())
                    } else {
                        None
                    }
                });
                assert_eq!(name_str, Some("TestType3Basic".to_string()));
            } else {
                panic!("expected Instance with class_name='type'");
            }
        }

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_empty_class() {
        super::super::class::cleanup_all_classes();

        let name = MbValue::from_ptr(MbObject::new_str("TestType3Empty".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str("object".to_string()),
        )]));
        let dict = super::super::dict_ops::mb_dict_new();

        let type_obj = mb_type3(name, bases, dict);
        assert!(type_obj.as_ptr().is_some());

        // Instance should be creatable
        let cls_name_val = MbValue::from_ptr(MbObject::new_str("TestType3Empty".to_string()));
        let instance = super::super::class::mb_instance_new(cls_name_val, MbValue::none());
        assert!(instance.as_ptr().is_some());
        unsafe {
            let ptr = instance.as_ptr().unwrap();
            if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
                assert_eq!(class_name, "TestType3Empty");
            } else {
                panic!("expected Instance");
            }
        }

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_dunder_in_dict_classified_as_method() {
        super::super::class::cleanup_all_classes();

        // Dunder keys should be classified as methods even if value is not TAG_FUNC.
        // We use an int value as a stand-in; in real usage this would be a closure handle.
        let name = MbValue::from_ptr(MbObject::new_str("TestType3Dunder".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str("object".to_string()),
        )]));
        let dict = super::super::dict_ops::mb_dict_new();
        // __repr__ with a non-None value (int as placeholder)
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("__repr__".to_string())),
            MbValue::from_int(999),
        );
        // Regular attr 'x' should be accessible as class_attr
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
            MbValue::from_int(42),
        );

        let _type_obj = mb_type3(name, bases, dict);

        // Create instance and verify 'x' is accessible as class attr
        let cls_name_val = MbValue::from_ptr(MbObject::new_str("TestType3Dunder".to_string()));
        let instance = super::super::class::mb_instance_new(cls_name_val, MbValue::none());
        let x_attr = super::super::class::mb_getattr(
            instance,
            MbValue::from_ptr(MbObject::new_str("x".to_string())),
        );
        assert_eq!(x_attr.as_int(), Some(42), "class attr x should be 42");

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_isinstance_with_type_object() {
        super::super::class::cleanup_all_classes();

        let name = MbValue::from_ptr(MbObject::new_str("TestType3Inst".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str("object".to_string()),
        )]));
        let dict = super::super::dict_ops::mb_dict_new();
        let type_obj = mb_type3(name, bases, dict);

        // Create an instance
        let cls_name_val = MbValue::from_ptr(MbObject::new_str("TestType3Inst".to_string()));
        let instance = super::super::class::mb_instance_new(cls_name_val, MbValue::none());

        // isinstance(instance, type_obj) should be True
        let result = super::super::class::mb_isinstance(instance, type_obj);
        assert_eq!(result.as_bool(), Some(true));

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_empty_bases_defaults_to_object() {
        super::super::class::cleanup_all_classes();

        // Passing empty tuple for bases should default to (object,)
        let name = MbValue::from_ptr(MbObject::new_str("TestType3NoBases".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        let dict = super::super::dict_ops::mb_dict_new();
        let _type_obj = mb_type3(name, bases, dict);

        // Instance should be an instance of object
        let cls_name_val = MbValue::from_ptr(MbObject::new_str("TestType3NoBases".to_string()));
        let instance = super::super::class::mb_instance_new(cls_name_val, MbValue::none());
        let obj_name = MbValue::from_ptr(MbObject::new_str("object".to_string()));
        let result = super::super::class::mb_isinstance(instance, obj_name);
        assert_eq!(result.as_bool(), Some(true), "should be instance of object");

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_namespace_mappingproxy_preserves_order_and_metadata() {
        super::super::class::cleanup_all_classes();

        let name = MbValue::from_ptr(MbObject::new_str("TestType3Namespace".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![]));
        let dict = super::super::dict_ops::mb_dict_new();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("b".to_string())),
            MbValue::from_int(2),
        );
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("a".to_string())),
            MbValue::from_int(1),
        );
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("__qualname__".to_string())),
            MbValue::from_ptr(MbObject::new_str("Outer.TestType3Namespace".to_string())),
        );

        let type_obj = mb_type3(name, bases, dict);
        let qualname = super::super::class::mb_getattr(
            type_obj,
            MbValue::from_ptr(MbObject::new_str("__qualname__".to_string())),
        );
        assert_eq!(
            mb_str_value(qualname),
            Some("Outer.TestType3Namespace".to_string())
        );
        let doc = super::super::class::mb_getattr(
            type_obj,
            MbValue::from_ptr(MbObject::new_str("__doc__".to_string())),
        );
        assert!(doc.is_none(), "__doc__ should default to None");

        let proxy = super::super::class::mb_getattr(
            type_obj,
            MbValue::from_ptr(MbObject::new_str("__dict__".to_string())),
        );
        let namespace = super::super::dict_ops::mappingproxy_mapping(proxy)
            .expect("type __dict__ should expose namespace mapping");
        let entries = extract_items(super::super::dict_ops::mb_dict_items(namespace));
        let first = extract_items(entries[0]);
        let second = extract_items(entries[1]);
        assert_eq!(mb_str_value(first[0]).as_deref(), Some("b"));
        assert_eq!(first[1].as_int(), Some(2));
        assert_eq!(mb_str_value(second[0]).as_deref(), Some("a"));
        assert_eq!(second[1].as_int(), Some(1));

        super::super::class::cleanup_all_classes();
    }

    #[test]
    fn test_type3_with_inheritance() {
        super::super::class::cleanup_all_classes();

        // Register a base class first
        super::super::class::mb_class_register(
            "TestType3Base",
            vec![],
            std::collections::HashMap::new(),
        );

        // Create child via type() 3-arg
        let name = MbValue::from_ptr(MbObject::new_str("TestType3Child".to_string()));
        let bases = MbValue::from_ptr(MbObject::new_tuple(vec![MbValue::from_ptr(
            MbObject::new_str("TestType3Base".to_string()),
        )]));
        let dict = super::super::dict_ops::mb_dict_new();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("y".to_string())),
            MbValue::from_int(10),
        );
        let _type_obj = mb_type3(name, bases, dict);

        // isinstance(Child(), Base) should work via MRO
        let child_name = MbValue::from_ptr(MbObject::new_str("TestType3Child".to_string()));
        let instance = super::super::class::mb_instance_new(child_name, MbValue::none());
        let base_name = MbValue::from_ptr(MbObject::new_str("TestType3Base".to_string()));
        let result = super::super::class::mb_isinstance(instance, base_name);
        assert_eq!(result.as_bool(), Some(true));

        // Also isinstance with object
        let obj_name = MbValue::from_ptr(MbObject::new_str("object".to_string()));
        let result2 = super::super::class::mb_isinstance(instance, obj_name);
        assert_eq!(result2.as_bool(), Some(true));

        // Class attr 'y' should be accessible
        let y_attr = super::super::class::mb_getattr(
            instance,
            MbValue::from_ptr(MbObject::new_str("y".to_string())),
        );
        assert_eq!(y_attr.as_int(), Some(10));

        super::super::class::cleanup_all_classes();
    }

    // HANDWRITE-BEGIN reason: Phase 1.5 cross-cutting fix (#11) — pin
    // the bytes-like equality contract so the unblocker doesn't regress.
    // These exercise the four representations (bytes / bytearray /
    // memoryview / array('B')) that surface in CPython Lib/test/* via
    // `assertEqual`. Convert to CODEGEN once the standardize sweep grows
    // a "buffer-protocol equality" section type.
    #[test]
    fn test_eq_bytearray_bytearray() {
        let a = MbValue::from_ptr(MbObject::new_bytearray(b"abc".to_vec()));
        let b = MbValue::from_ptr(MbObject::new_bytearray(b"abc".to_vec()));
        assert_eq!(mb_eq(a, b).as_bool(), Some(true));

        let c = MbValue::from_ptr(MbObject::new_bytearray(b"abd".to_vec()));
        assert_eq!(mb_eq(a, c).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_bytearray_bytes_cross() {
        // CPython: bytearray(b"xyz") == b"xyz" is True
        let ba = MbValue::from_ptr(MbObject::new_bytearray(b"xyz".to_vec()));
        let bs = MbValue::from_ptr(MbObject::new_bytes(b"xyz".to_vec()));
        assert_eq!(mb_eq(ba, bs).as_bool(), Some(true));
        assert_eq!(mb_eq(bs, ba).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_memoryview_bytes() {
        let src = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let mv = super::mb_memoryview(src);
        let target = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        assert_eq!(mb_eq(mv, target).as_bool(), Some(true));
        assert_eq!(mb_eq(target, mv).as_bool(), Some(true));

        let mismatch = MbValue::from_ptr(MbObject::new_bytes(b"abd".to_vec()));
        assert_eq!(mb_eq(mv, mismatch).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_memoryview_memoryview() {
        let src_a = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let src_b = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let a = super::mb_memoryview(src_a);
        let b = super::mb_memoryview(src_b);
        assert_eq!(mb_eq(a, b).as_bool(), Some(true));
    }

    #[test]
    fn test_eq_memoryview_bytearray() {
        let ba = MbValue::from_ptr(MbObject::new_bytearray(b"abc".to_vec()));
        let mv_src = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        let mv = super::mb_memoryview(mv_src);
        assert_eq!(mb_eq(mv, ba).as_bool(), Some(true));
    }

    /// Build a Dict that mirrors mamba's `array.array('B', [...])`
    /// representation so the bytes-like coercion is exercised end-to-end.
    fn make_array_dict_b(data: &[u8]) -> MbValue {
        let dict = super::super::dict_ops::mb_dict_new();
        // __class__: "array"
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("__class__".to_string())),
            MbValue::from_ptr(MbObject::new_str("array".to_string())),
        );
        // typecode: "B"
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("typecode".to_string())),
            MbValue::from_ptr(MbObject::new_str("B".to_string())),
        );
        // data: [<int>, ...]
        let items: Vec<MbValue> = data.iter().map(|b| MbValue::from_int(*b as i64)).collect();
        super::super::dict_ops::mb_dict_setitem(
            dict,
            MbValue::from_ptr(MbObject::new_str("data".to_string())),
            MbValue::from_ptr(MbObject::new_list(items)),
        );
        dict
    }

    #[test]
    fn test_eq_array_b_array_b() {
        let a = make_array_dict_b(b"abc");
        let b = make_array_dict_b(b"abc");
        assert_eq!(mb_eq(a, b).as_bool(), Some(true));

        let c = make_array_dict_b(b"abd");
        assert_eq!(mb_eq(a, c).as_bool(), Some(false));
    }

    #[test]
    fn test_eq_array_b_bytes() {
        // array('B', b"abc") == b"abc" in CPython is False (array doesn't
        // compare bytes-equal to bytes), but our buffer-protocol coercion
        // unifies the byte-vector view. Pin the lifted semantics so we
        // notice if the unblocker is rolled back.
        let arr = make_array_dict_b(b"abc");
        let bs = MbValue::from_ptr(MbObject::new_bytes(b"abc".to_vec()));
        assert_eq!(mb_eq(arr, bs).as_bool(), Some(true));
    }

    #[test]
    fn test_try_bytes_like_rejects_non_bytes_like() {
        // Plain dict (no __class__: "array") must not coerce.
        let d = super::super::dict_ops::mb_dict_new();
        super::super::dict_ops::mb_dict_setitem(
            d,
            MbValue::from_ptr(MbObject::new_str("k".to_string())),
            MbValue::from_int(1),
        );
        assert!(super::try_bytes_like(d).is_none());

        // Plain Instance (not memoryview) must not coerce.
        let inst = MbValue::from_ptr(MbObject::new_instance("Foo".to_string()));
        assert!(super::try_bytes_like(inst).is_none());

        // int must not coerce.
        assert!(super::try_bytes_like(MbValue::from_int(42)).is_none());
    }
    // HANDWRITE-END
}
