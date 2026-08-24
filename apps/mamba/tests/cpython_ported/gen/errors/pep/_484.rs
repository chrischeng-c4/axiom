use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/484/generic_too_many_params_raises.py`.
#[test]
fn test_gen_errors_pep_484_generic_too_many_params_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "generic_too_many_params_raises"
# subject = "typing.List"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: generic_too_many_params_raises (errors)."""
import typing

_raised = False
try:
    typing.List[int, str, float]
except TypeError:
    _raised = True
assert _raised, "generic_too_many_params_raises: expected TypeError"
print("generic_too_many_params_raises OK")
"###);
    assert_output(&out, r###"generic_too_many_params_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/get_type_hints_bad_forward_ref_raises.py`.
#[test]
fn test_gen_errors_pep_484_get_type_hints_bad_forward_ref_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "get_type_hints_bad_forward_ref_raises"
# subject = "typing.get_type_hints"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints_bad_forward_ref_raises (errors)."""
import typing

def _with_bad_ref(x: "NoSuchType") -> int:
    return 1


_raised = False
try:
    typing.get_type_hints(_with_bad_ref)
except NameError:
    _raised = True
assert _raised, "get_type_hints_bad_forward_ref_raises: expected NameError"
print("get_type_hints_bad_forward_ref_raises OK")
"###);
    assert_output(&out, r###"get_type_hints_bad_forward_ref_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/namedtuple_list_form_with_kwargs_raises.py`.
#[test]
fn test_gen_errors_pep_484_namedtuple_list_form_with_kwargs_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "namedtuple_list_form_with_kwargs_raises"
# subject = "typing.NamedTuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: namedtuple_list_form_with_kwargs_raises (errors)."""
import typing

_raised = False
try:
    typing.NamedTuple('Bad', [('x', int)], y=str)
except TypeError:
    _raised = True
assert _raised, "namedtuple_list_form_with_kwargs_raises: expected TypeError"
print("namedtuple_list_form_with_kwargs_raises OK")
"###);
    assert_output(&out, r###"namedtuple_list_form_with_kwargs_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/non_subscriptable_type_raises.py`.
#[test]
fn test_gen_errors_pep_484_non_subscriptable_type_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "non_subscriptable_type_raises"
# subject = "typing"
# kind = "mechanical"
# xfail = "mamba does not raise on subscripting a non-subscriptable builtin type (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: non_subscriptable_type_raises (errors)."""
import typing

_raised = False
try:
    int['x']
except TypeError:
    _raised = True
assert _raised, "non_subscriptable_type_raises: expected TypeError"
print("non_subscriptable_type_raises OK")
"###);
    assert_output(&out, r###"non_subscriptable_type_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/overload_only_stub_call_raises.py`.
#[test]
fn test_gen_errors_pep_484_overload_only_stub_call_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "overload_only_stub_call_raises"
# subject = "typing.overload"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.overload: overload_only_stub_call_raises (errors)."""
import typing

@typing.overload
def _only_stub(x: int) -> int: ...
@typing.overload
def _only_stub(x: str) -> str: ...


_raised = False
try:
    _only_stub(1)
except NotImplementedError:
    _raised = True
assert _raised, "overload_only_stub_call_raises: expected NotImplementedError"
print("overload_only_stub_call_raises OK")
"###);
    assert_output(&out, r###"overload_only_stub_call_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/runtime_checkable_non_protocol_raises.py`.
#[test]
fn test_gen_errors_pep_484_runtime_checkable_non_protocol_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "runtime_checkable_non_protocol_raises"
# subject = "typing.runtime_checkable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.runtime_checkable: runtime_checkable_non_protocol_raises (errors)."""
import typing

class _PlainClass:
    pass


_raised = False
try:
    typing.runtime_checkable(_PlainClass)
except TypeError:
    _raised = True
assert _raised, "runtime_checkable_non_protocol_raises: expected TypeError"
print("runtime_checkable_non_protocol_raises OK")
"###);
    assert_output(&out, r###"runtime_checkable_non_protocol_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/typevar_bound_and_constraints_raises.py`.
#[test]
fn test_gen_errors_pep_484_typevar_bound_and_constraints_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "typevar_bound_and_constraints_raises"
# subject = "typing.TypeVar"
# kind = "mechanical"
# xfail = "mamba does not raise when a TypeVar mixes constraints with bound= (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: typevar_bound_and_constraints_raises (errors)."""
import typing

_raised = False
try:
    typing.TypeVar('T', int, str, bound=int)
except TypeError:
    _raised = True
assert _raised, "typevar_bound_and_constraints_raises: expected TypeError"
print("typevar_bound_and_constraints_raises OK")
"###);
    assert_output(&out, r###"typevar_bound_and_constraints_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/484/unhashable_annotated_metadata_raises.py`.
#[test]
fn test_gen_errors_pep_484_unhashable_annotated_metadata_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "errors"
# case = "unhashable_annotated_metadata_raises"
# subject = "typing.Annotated"
# kind = "mechanical"
# xfail = "mamba does not raise hashing an Annotated form with unhashable metadata (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Annotated: unhashable_annotated_metadata_raises (errors)."""
import typing

_raised = False
try:
    hash(typing.Annotated[int, []])
except TypeError:
    _raised = True
assert _raised, "unhashable_annotated_metadata_raises: expected TypeError"
print("unhashable_annotated_metadata_raises OK")
"###);
    assert_output(&out, r###"unhashable_annotated_metadata_raises OK
"###);
}
