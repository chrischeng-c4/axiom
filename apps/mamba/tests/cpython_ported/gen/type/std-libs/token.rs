use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/token/ISEOF__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_token_ISEOF__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "type"
# case = "ISEOF__x_as_int_wrong"
# subject = "token.ISEOF(x: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/token.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: token.ISEOF(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from token import ISEOF
try:
    ISEOF("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/token/ISNONTERMINAL__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_token_ISNONTERMINAL__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "type"
# case = "ISNONTERMINAL__x_as_int_wrong"
# subject = "token.ISNONTERMINAL(x: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/token.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: token.ISNONTERMINAL(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from token import ISNONTERMINAL
try:
    ISNONTERMINAL("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/token/ISTERMINAL__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_token_ISTERMINAL__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "token"
# dimension = "type"
# case = "ISTERMINAL__x_as_int_wrong"
# subject = "token.ISTERMINAL(x: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/token.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: token.ISTERMINAL(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from token import ISTERMINAL
try:
    ISTERMINAL("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
