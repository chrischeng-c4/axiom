use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/fnmatch/filter__names_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_fnmatch_filter__names_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "type"
# case = "filter__names_as_Iterable_wrong"
# subject = "fnmatch.filter(names: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fnmatch.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fnmatch.filter(names: Iterable); call it with the wrong type.

typeshed contract: names is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fnmatch import filter
try:
    filter(_W(), None)  # names: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fnmatch/filterfalse__names_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_fnmatch_filterfalse__names_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "type"
# case = "filterfalse__names_as_Iterable_wrong"
# subject = "fnmatch.filterfalse(names: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fnmatch.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fnmatch.filterfalse(names: Iterable); call it with the wrong type.

typeshed contract: names is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from fnmatch import filterfalse
try:
    filterfalse(_W(), None)  # names: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/fnmatch/translate__pat_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_fnmatch_translate__pat_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "fnmatch"
# dimension = "type"
# case = "translate__pat_as_str_wrong"
# subject = "fnmatch.translate(pat: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/fnmatch.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: fnmatch.translate(pat: str); call it with the wrong type.

typeshed contract: pat is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from fnmatch import translate
try:
    translate(12345)  # pat: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
