use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/types/coroutine_wrapped_arg_mismatch_raises.py`.
#[test]
fn test_gen_errors_std_libs_types_coroutine_wrapped_arg_mismatch_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "coroutine_wrapped_arg_mismatch_raises"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: coroutine() returns a wrapper that forwards the call to the wrapped plain function, so calling it with an extra positional arg the function does not accept raises TypeError"""
import types


def regular() -> int:
    return 1


# coroutine() wraps a plain (non-generator) function; the wrapper forwards the
# call, so passing an argument that regular() does not accept raises TypeError.
_raised = False
try:
    types.coroutine(regular)(1)
except TypeError:
    _raised = True
assert _raised, "coroutine_wrapped_arg_mismatch_raises: expected TypeError"

print("coroutine_wrapped_arg_mismatch_raises OK")
"###);
    assert_output(&out, r###"coroutine_wrapped_arg_mismatch_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/types/mappingproxy_delitem_raises.py`.
#[test]
fn test_gen_errors_std_libs_types_mappingproxy_delitem_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "mappingproxy_delitem_raises"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.MappingProxyType: mappingproxy_delitem_raises (errors)."""
import operator, types

_raised = False
try:
    operator.delitem(types.MappingProxyType({'a': 1}), 'a')
except TypeError:
    _raised = True
assert _raised, "mappingproxy_delitem_raises: expected TypeError"
print("mappingproxy_delitem_raises OK")
"###);
    assert_output(&out, r###"mappingproxy_delitem_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/types/mappingproxy_setitem_raises.py`.
#[test]
fn test_gen_errors_std_libs_types_mappingproxy_setitem_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "mappingproxy_setitem_raises"
# subject = "types.MappingProxyType"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.MappingProxyType: mappingproxy_setitem_raises (errors)."""
import operator, types

_raised = False
try:
    operator.setitem(types.MappingProxyType({'a': 1}), 'b', 2)
except TypeError:
    _raised = True
assert _raised, "mappingproxy_setitem_raises: expected TypeError"
print("mappingproxy_setitem_raises OK")
"###);
    assert_output(&out, r###"mappingproxy_setitem_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/types/new_class_bad_metaclass_raises.py`.
#[test]
fn test_gen_errors_std_libs_types_new_class_bad_metaclass_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "new_class_bad_metaclass_raises"
# subject = "types.new_class"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class_bad_metaclass_raises (errors)."""
import types

_raised = False
try:
    types.new_class('X', (object,), {'metaclass': 'not_a_class'})
except TypeError:
    _raised = True
assert _raised, "new_class_bad_metaclass_raises: expected TypeError"
print("new_class_bad_metaclass_raises OK")
"###);
    assert_output(&out, r###"new_class_bad_metaclass_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/types/union_wrong_arity_substitution_raises.py`.
#[test]
fn test_gen_errors_std_libs_types_union_wrong_arity_substitution_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "errors"
# case = "union_wrong_arity_substitution_raises"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: substituting the wrong number of parameters into a parameterized union (int | T)[int, str] raises TypeError"""
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
partial = int | T

_raised = False
try:
    partial[int, str]
except TypeError:
    _raised = True
assert _raised, "union_wrong_arity_substitution_raises: expected TypeError"

print("union_wrong_arity_substitution_raises OK")
"###);
    assert_output(&out, r###"union_wrong_arity_substitution_raises OK
"###);
}
