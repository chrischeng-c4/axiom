use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/container_element/dict_str_int_with_wrong_value.py`.
#[test]
fn test_gen_type_core_container_element_dict_str_int_with_wrong_value() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_element"
# dimension = "type"
# case = "dict_str_int_with_wrong_value"
# subject = "dict value annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: dict[str, int]-annotated var with wrong value.

CPython 3.12: dict contains the wrong-typed value; annotation
ignored at runtime.
Mamba: raises TypeError because a value type violates the
container annotation contract.
"""

try:
    d: dict[str, int] = {"a": "not_an_int"}
    print("no_typeerror:", repr(d))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/container_element/list_int_with_str_element.py`.
#[test]
fn test_gen_type_core_container_element_list_int_with_str_element() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "container_element"
# dimension = "type"
# case = "list_int_with_str_element"
# subject = "list element annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list[int]-annotated var with str element.

CPython 3.12: annotation is documentation, list contains the str.
Mamba: raises TypeError because the element type violates the
container annotation contract.
"""

try:
    xs: list[int] = [1, "two", 3]
    print("no_typeerror:", repr(xs))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
