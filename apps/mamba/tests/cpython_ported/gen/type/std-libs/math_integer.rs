use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/math_integer/comb__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_integer_comb__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_integer"
# dimension = "type"
# case = "comb__n_as_SupportsIndex_wrong"
# subject = "math.integer.comb(n: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math/integer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.integer.comb(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math.integer import comb
try:
    comb(_W(), None)  # n: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math_integer/factorial__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_integer_factorial__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_integer"
# dimension = "type"
# case = "factorial__n_as_SupportsIndex_wrong"
# subject = "math.integer.factorial(n: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math/integer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.integer.factorial(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math.integer import factorial
try:
    factorial(_W())  # n: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math_integer/isqrt__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_integer_isqrt__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_integer"
# dimension = "type"
# case = "isqrt__n_as_SupportsIndex_wrong"
# subject = "math.integer.isqrt(n: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math/integer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.integer.isqrt(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math.integer import isqrt
try:
    isqrt(_W())  # n: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math_integer/perm__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_integer_perm__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math_integer"
# dimension = "type"
# case = "perm__n_as_SupportsIndex_wrong"
# subject = "math.integer.perm(n: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math/integer.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.integer.perm(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math.integer import perm
try:
    perm(_W())  # n: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
