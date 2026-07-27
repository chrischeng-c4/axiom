use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/spwd/getspnam__arg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_spwd_getspnam__arg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "spwd"
# dimension = "type"
# case = "getspnam__arg_as_str_wrong"
# subject = "spwd.getspnam(arg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/spwd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: spwd.getspnam(arg: str); call it with the wrong type.

typeshed contract: arg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from spwd import getspnam
try:
    getspnam(12345)  # arg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
