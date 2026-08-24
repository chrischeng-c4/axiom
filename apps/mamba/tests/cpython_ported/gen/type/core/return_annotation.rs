use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/return_annotation/func_int_return_returns_str.py`.
#[test]
fn test_gen_type_core_return_annotation_func_int_return_returns_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_int_return_returns_str"
# subject = "function return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: function annotated int -> int returns str.

CPython 3.12: accepts the str return.
Mamba: raises TypeError at return time (annotation is a contract).
"""


def a() -> int:
    return "not_an_int"


try:
    result = a()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/return_annotation/func_list_return_returns_int.py`.
#[test]
fn test_gen_type_core_return_annotation_func_list_return_returns_int() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_list_return_returns_int"
# subject = "function return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-return function returns int.

CPython 3.12: accepts the int return.
Mamba: raises TypeError at return time.
"""


def get() -> list:
    return 7


try:
    result = get()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/return_annotation/func_none_return_returns_int.py`.
#[test]
fn test_gen_type_core_return_annotation_func_none_return_returns_int() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "return_annotation"
# dimension = "type"
# case = "func_none_return_returns_int"
# subject = "function None return annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects returning a value from a function annotated as None."""


def a() -> None:
    return 7


try:
    result = a()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
