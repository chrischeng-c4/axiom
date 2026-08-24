use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/695/double_generic_base_raises.py`.
#[test]
fn test_gen_errors_pep_695_double_generic_base_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "double_generic_base_raises"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = "class _[T](Generic[T]) does not raise the double-Generic TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: double_generic_base_raises (errors)."""
from typing import Generic

_raised = False
try:
    exec('class _D[T](Generic[T]):\n    ...')
except TypeError:
    _raised = True
assert _raised, "double_generic_base_raises: expected TypeError"
print("double_generic_base_raises OK")
"###);
    assert_output(&out, r###"double_generic_base_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/695/object_base_plus_generic_mro_raises.py`.
#[test]
fn test_gen_errors_pep_695_object_base_plus_generic_mro_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "object_base_plus_generic_mro_raises"
# subject = "typing.Generic"
# kind = "mechanical"
# xfail = "class _[X](object) does not raise the object+Generic MRO TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: object_base_plus_generic_mro_raises (errors)."""
from typing import Generic

_raised = False
try:
    exec('class _W[X](object):\n    ...')
except TypeError:
    _raised = True
assert _raised, "object_base_plus_generic_mro_raises: expected TypeError"
print("object_base_plus_generic_mro_raises OK")
"###);
    assert_output(&out, r###"object_base_plus_generic_mro_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/695/undeclared_typevar_in_base_raises.py`.
#[test]
fn test_gen_errors_pep_695_undeclared_typevar_in_base_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "errors"
# case = "undeclared_typevar_in_base_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = "an undeclared module-level TypeVar in a new-style generic base does not raise TypeError on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: undeclared_typevar_in_base_raises (errors)."""
from typing import TypeVar

_raised = False
try:
    exec('S = TypeVar("S")\nclass _M[T](dict[T, S]):\n    ...')
except TypeError:
    _raised = True
assert _raised, "undeclared_typevar_in_base_raises: expected TypeError"
print("undeclared_typevar_in_base_raises OK")
"###);
    assert_output(&out, r###"undeclared_typevar_in_base_raises OK
"###);
}
