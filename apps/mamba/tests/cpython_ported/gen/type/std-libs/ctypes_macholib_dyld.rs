use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/ctypes_macholib_dyld/dyld_find__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_macholib_dyld_dyld_find__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes_macholib_dyld"
# dimension = "type"
# case = "dyld_find__name_as_str_wrong"
# subject = "ctypes.macholib.dyld.dyld_find(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes/macholib/dyld.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.macholib.dyld.dyld_find(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes.macholib.dyld import dyld_find
try:
    dyld_find(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/ctypes_macholib_dyld/framework_find__fn_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_ctypes_macholib_dyld_framework_find__fn_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes_macholib_dyld"
# dimension = "type"
# case = "framework_find__fn_as_str_wrong"
# subject = "ctypes.macholib.dyld.framework_find(fn: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ctypes/macholib/dyld.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ctypes.macholib.dyld.framework_find(fn: str); call it with the wrong type.

typeshed contract: fn is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ctypes.macholib.dyld import framework_find
try:
    framework_find(12345)  # fn: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
