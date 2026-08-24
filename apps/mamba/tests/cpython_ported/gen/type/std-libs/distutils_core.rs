use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_core/gen_usage__script_name_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_core_gen_usage__script_name_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_core"
# dimension = "type"
# case = "gen_usage__script_name_as_StrOrBytesPath_wrong"
# subject = "distutils.core.gen_usage(script_name: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/core.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.core.gen_usage(script_name: StrOrBytesPath); call it with the wrong type.

typeshed contract: script_name is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.core import gen_usage
try:
    gen_usage(_W())  # script_name: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_core/run_setup__script_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_core_run_setup__script_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_core"
# dimension = "type"
# case = "run_setup__script_name_as_str_wrong"
# subject = "distutils.core.run_setup(script_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/core.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.core.run_setup(script_name: str); call it with the wrong type.

typeshed contract: script_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.core import run_setup
try:
    run_setup(12345)  # script_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
