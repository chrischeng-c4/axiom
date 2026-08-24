use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/core/arg_annotation/default_int_arg_uses_str_default.py`.
#[test]
fn test_gen_type_core_arg_annotation_default_int_arg_uses_str_default() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "default_int_arg_uses_str_default"
# subject = "function default parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects an annotated int parameter whose default is a str."""


def requires_count(count: int = "3") -> int:
    return count


try:
    result = requires_count()
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/func_dict_arg_called_with_list.py`.
#[test]
fn test_gen_type_core_arg_annotation_func_dict_arg_called_with_list() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_dict_arg_called_with_list"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: dict-annotated arg called with list.

CPython 3.12: accepts; later .items() would fail but the call lands.
Mamba: raises TypeError at call time.
"""


def keys_of(d: dict) -> int:
    return len(d)


try:
    result = keys_of([1, 2, 3])
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/func_int_arg_called_with_str.py`.
#[test]
fn test_gen_type_core_arg_annotation_func_int_arg_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_int_arg_called_with_str"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: int-annotated arg called with str.

CPython 3.12: annotations are documentation, call succeeds.
Mamba:        annotations are contract, call raises TypeError.
"""


def a(i: int) -> int:
    return i


try:
    result = a("a")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/func_list_arg_called_with_int.py`.
#[test]
fn test_gen_type_core_arg_annotation_func_list_arg_called_with_int() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_list_arg_called_with_int"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: list-annotated arg called with int.

CPython 3.12: accepts; uses the int wherever the list would be used
(crash deferred until the body tries a list operation).
Mamba: raises TypeError at call time.
"""


def take(lst: list) -> int:
    # Don't touch lst — keep the body free of operations that raise
    # on the wrong-typed value. We're testing annotation enforcement
    # at call time, not body-level failure.
    return 0


try:
    result = take(7)
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/func_str_arg_called_with_bytes.py`.
#[test]
fn test_gen_type_core_arg_annotation_func_str_arg_called_with_bytes() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "func_str_arg_called_with_bytes"
# subject = "function positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba runtime-type enforcement: str-annotated arg called with bytes.

CPython 3.12: accepts.
Mamba: raises TypeError at call time.
"""


def upper(s: str) -> str:
    return s.upper() if isinstance(s, str) else "<not str>"


try:
    result = upper(b"hi")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:60])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/keyword_only_int_arg_called_with_str.py`.
#[test]
fn test_gen_type_core_arg_annotation_keyword_only_int_arg_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "keyword_only_int_arg_called_with_str"
# subject = "function keyword-only parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed keyword-only argument annotation."""


def requires_count(*, count: int) -> int:
    return count


try:
    result = requires_count(count="3")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/kwargs_int_arg_called_with_str.py`.
#[test]
fn test_gen_type_core_arg_annotation_kwargs_int_arg_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "kwargs_int_arg_called_with_str"
# subject = "function variadic keyword parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed variadic keyword argument annotation."""


def count_items(**items: int) -> int:
    return len(items)


try:
    result = count_items(count="3")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/positional_only_int_arg_called_with_str.py`.
#[test]
fn test_gen_type_core_arg_annotation_positional_only_int_arg_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "positional_only_int_arg_called_with_str"
# subject = "function positional-only parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed positional-only argument annotation."""


def requires_count(count: int, /) -> int:
    return count


try:
    result = requires_count("3")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/core/arg_annotation/varargs_int_arg_called_with_str.py`.
#[test]
fn test_gen_type_core_arg_annotation_varargs_int_arg_called_with_str() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "arg_annotation"
# dimension = "type"
# case = "varargs_int_arg_called_with_str"
# subject = "function variadic positional parameter annotation"
# kind = "semantic"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Mamba rejects a wrong-typed variadic positional argument annotation."""


def sum_items(*items: int) -> int:
    return len(items)


try:
    result = sum_items("3")
    print("no_typeerror:", repr(result))
except TypeError as e:
    print("typeerror:", type(e).__name__, str(e)[:80])
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
