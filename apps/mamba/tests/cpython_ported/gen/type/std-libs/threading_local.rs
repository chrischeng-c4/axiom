use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_threading_local/local____delattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__threading_local_local____delattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_threading_local"
# dimension = "type"
# case = "local____delattr____name_as_str_wrong"
# subject = "_threading_local.local.__delattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_threading_local.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _threading_local.local.__delattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _threading_local import local
obj = object.__new__(local)
try:
    obj.__delattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_threading_local/local____getattribute____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__threading_local_local____getattribute____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_threading_local"
# dimension = "type"
# case = "local____getattribute____name_as_str_wrong"
# subject = "_threading_local.local.__getattribute__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_threading_local.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _threading_local.local.__getattribute__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _threading_local import local
obj = object.__new__(local)
try:
    obj.__getattribute__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_threading_local/local____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__threading_local_local____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_threading_local"
# dimension = "type"
# case = "local____setattr____name_as_str_wrong"
# subject = "_threading_local.local.__setattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_threading_local.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _threading_local.local.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _threading_local import local
obj = object.__new__(local)
try:
    obj.__setattr__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
