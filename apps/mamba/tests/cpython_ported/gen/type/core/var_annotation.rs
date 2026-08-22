use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/var_annotation/int_var_assigned_str.py`.
#[test]
fn test_gen_type_core_var_annotation_int_var_assigned_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "var_annotation"
# dimension = "type"
# case = "int_var_assigned_str"
# subject = "variable annotation assignment"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: int-annotated var bound to str.

CPython 3.12: annotation is documentation, assignment succeeds.
Mamba: raises TypeError at assignment time.
"""

try:
    x: int = "abc"
    print("no_typeerror:", repr(x))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/var_annotation/list_var_assigned_int.py`.
#[test]
fn test_gen_type_core_var_annotation_list_var_assigned_int() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "var_annotation"
# dimension = "type"
# case = "list_var_assigned_int"
# subject = "variable annotation assignment"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-annotated var bound to int.

CPython 3.12: assignment succeeds.
Mamba: raises TypeError at assignment time.
"""

try:
    xs: list = 7
    print("no_typeerror:", repr(xs))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
