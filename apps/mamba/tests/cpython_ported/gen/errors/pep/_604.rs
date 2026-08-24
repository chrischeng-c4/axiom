use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/pep/604/calling_union_raises.py`.
#[test]
fn test_gen_errors_pep_604_calling_union_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "errors"
# case = "calling_union_raises"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = "`int | str` returns None on mamba so calling it does not raise TypeError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: calling_union_raises (errors)."""
import types

_raised = False
try:
    (int | str)(1)
except TypeError:
    _raised = True
assert _raised, "calling_union_raises: expected TypeError"
print("calling_union_raises OK")
"###);
    assert_output(&out, r###"calling_union_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/604/union_with_non_type_value_raises.py`.
#[test]
fn test_gen_errors_pep_604_union_with_non_type_value_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "errors"
# case = "union_with_non_type_value_raises"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = "`int | 42` returns None on mamba instead of raising TypeError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: union_with_non_type_value_raises (errors)."""
import types

_raised = False
try:
    int | 42
except TypeError:
    _raised = True
assert _raised, "union_with_non_type_value_raises: expected TypeError"
print("union_with_non_type_value_raises OK")
"###);
    assert_output(&out, r###"union_with_non_type_value_raises OK
"###);
}

/// Ported from `tests/cpython/errors/pep/604/union_with_string_value_raises.py`.
#[test]
fn test_gen_errors_pep_604_union_with_string_value_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "604"
# dimension = "errors"
# case = "union_with_string_value_raises"
# subject = "types.UnionType"
# kind = "mechanical"
# xfail = "`int | 'x'` returns None on mamba instead of raising TypeError (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""types.UnionType: union_with_string_value_raises (errors)."""
import types

_raised = False
try:
    int | 'not_a_type'
except TypeError:
    _raised = True
assert _raised, "union_with_string_value_raises: expected TypeError"
print("union_with_string_value_raises OK")
"###);
    assert_output(&out, r###"union_with_string_value_raises OK
"###);
}
