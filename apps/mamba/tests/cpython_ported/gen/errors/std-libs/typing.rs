use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/typing/get_type_hints_bad_forward_ref_raises.py`.
#[test]
fn test_gen_errors_std_libs_typing_get_type_hints_bad_forward_ref_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "get_type_hints_bad_forward_ref_raises"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints on a function annotated with an unresolvable forward reference 'NoSuchType' raises NameError"""
import typing


def with_bad_hint(x: "NoSuchType") -> int:  # noqa: F821
    return x


_raised = False
try:
    typing.get_type_hints(with_bad_hint)
except NameError:
    _raised = True
assert _raised, "get_type_hints_bad_forward_ref_raises: expected NameError"
print("get_type_hints_bad_forward_ref_raises OK")
"###);
    assert_output(&out, r###"get_type_hints_bad_forward_ref_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/typing/non_generic_subscript_raises.py`.
#[test]
fn test_gen_errors_std_libs_typing_non_generic_subscript_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "non_generic_subscript_raises"
# subject = "int"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""int: non_generic_subscript_raises (errors)."""
import typing  # noqa: F401

_raised = False
try:
    int["x"]
except TypeError:
    _raised = True
assert _raised, "non_generic_subscript_raises: expected TypeError"
print("non_generic_subscript_raises OK")
"###);
    assert_output(&out, r###"non_generic_subscript_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/typing/typevar_bound_and_constraints_raises.py`.
#[test]
fn test_gen_errors_std_libs_typing_typevar_bound_and_constraints_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "errors"
# case = "typevar_bound_and_constraints_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: typevar_bound_and_constraints_raises (errors)."""
import typing

_raised = False
try:
    typing.TypeVar("T", int, str, bound=int)
except TypeError:
    _raised = True
assert _raised, "typevar_bound_and_constraints_raises: expected TypeError"
print("typevar_bound_and_constraints_raises OK")
"###);
    assert_output(&out, r###"typevar_bound_and_constraints_raises OK
"###);
}
