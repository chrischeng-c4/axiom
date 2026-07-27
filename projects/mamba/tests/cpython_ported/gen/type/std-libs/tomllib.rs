use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tomllib/load__fp_as_SupportsRead_wrong.py`.
#[test]
fn test_gen_type_std_libs_tomllib_load__fp_as_SupportsRead_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "type"
# case = "load__fp_as_SupportsRead_wrong"
# subject = "tomllib.load(fp: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tomllib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tomllib.load(fp: SupportsRead); call it with the wrong type.

typeshed contract: fp is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tomllib import load
try:
    load(_W())  # fp: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tomllib/loads__s_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tomllib_loads__s_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tomllib"
# dimension = "type"
# case = "loads__s_as_str_wrong"
# subject = "tomllib.loads(s: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tomllib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tomllib.loads(s: str); call it with the wrong type.

typeshed contract: s is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tomllib import loads
try:
    loads(12345)  # s: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
