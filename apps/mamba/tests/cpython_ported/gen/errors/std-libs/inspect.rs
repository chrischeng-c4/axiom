use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/inspect/getclosurevars_class_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_getclosurevars_class_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getclosurevars_class_raises_typeerror"
# subject = "inspect.getclosurevars"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclosurevars: getclosurevars_class_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getclosurevars(list)
except TypeError:
    _raised = True
assert _raised, "getclosurevars_class_raises_typeerror: expected TypeError"
print("getclosurevars_class_raises_typeerror OK")
"###);
    assert_output(&out, r###"getclosurevars_class_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/getfile_builtin_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_getfile_builtin_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getfile_builtin_raises_typeerror"
# subject = "inspect.getfile"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getfile: getfile_builtin_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getfile(len)
except TypeError:
    _raised = True
assert _raised, "getfile_builtin_raises_typeerror: expected TypeError"
print("getfile_builtin_raises_typeerror OK")
"###);
    assert_output(&out, r###"getfile_builtin_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/getsource_builtin_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_getsource_builtin_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getsource_builtin_raises_typeerror"
# subject = "inspect.getsource"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getsource: getsource_builtin_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getsource(len)
except TypeError:
    _raised = True
assert _raised, "getsource_builtin_raises_typeerror: expected TypeError"
print("getsource_builtin_raises_typeerror OK")
"###);
    assert_output(&out, r###"getsource_builtin_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/getsourcelines_int_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_getsourcelines_int_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "getsourcelines_int_raises_typeerror"
# subject = "inspect.getsourcelines"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getsourcelines: getsourcelines_int_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    inspect.getsourcelines(int)
except TypeError:
    _raised = True
assert _raised, "getsourcelines_int_raises_typeerror: expected TypeError"
print("getsourcelines_int_raises_typeerror OK")
"###);
    assert_output(&out, r###"getsourcelines_int_raises_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/parameter_bad_kind_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_parameter_bad_kind_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "parameter_bad_kind_raises_valueerror"
# subject = "inspect.Parameter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: parameter_bad_kind_raises_valueerror (errors)."""
import inspect

_raised = False
try:
    inspect.Parameter('x', kind=999)
except ValueError:
    _raised = True
assert _raised, "parameter_bad_kind_raises_valueerror: expected ValueError"
print("parameter_bad_kind_raises_valueerror OK")
"###);
    assert_output(&out, r###"parameter_bad_kind_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/signature_type_raises_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_signature_type_raises_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "signature_type_raises_valueerror"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_type_raises_valueerror (errors)."""
import inspect

_raised = False
try:
    inspect.signature(type)
except ValueError:
    _raised = True
assert _raised, "signature_type_raises_valueerror: expected ValueError"
print("signature_type_raises_valueerror OK")
"###);
    assert_output(&out, r###"signature_type_raises_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/inspect/signature_unhashable_default_hash_raises_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_inspect_signature_unhashable_default_hash_raises_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "errors"
# case = "signature_unhashable_default_hash_raises_typeerror"
# subject = "inspect.signature"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature_unhashable_default_hash_raises_typeerror (errors)."""
import inspect

_raised = False
try:
    hash(inspect.signature(lambda a={}: None))
except TypeError:
    _raised = True
assert _raised, "signature_unhashable_default_hash_raises_typeerror: expected TypeError"
print("signature_unhashable_default_hash_raises_typeerror OK")
"###);
    assert_output(&out, r###"signature_unhashable_default_hash_raises_typeerror OK
"###);
}
