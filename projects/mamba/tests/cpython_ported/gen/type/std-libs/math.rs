use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/math/acos__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_acos__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "acos__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.acos(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.acos(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import acos
try:
    acos(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/acosh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_acosh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "acosh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.acosh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.acosh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import acosh
try:
    acosh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/asin__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_asin__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "asin__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.asin(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.asin(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import asin
try:
    asin(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/asinh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_asinh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "asinh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.asinh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.asinh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import asinh
try:
    asinh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/atan2__y_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_atan2__y_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "atan2__y_as__SupportsFloatOrIndex_wrong"
# subject = "math.atan2(y: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.atan2(y: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: y is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import atan2
try:
    atan2(_W(), None)  # y: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/atan__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_atan__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "atan__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.atan(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.atan(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import atan
try:
    atan(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/atanh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_atanh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "atanh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.atanh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.atanh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import atanh
try:
    atanh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/cbrt__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_cbrt__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "cbrt__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.cbrt(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.cbrt(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import cbrt
try:
    cbrt(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/ceil__x_as__SupportsCeil_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_ceil__x_as__SupportsCeil_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "ceil__x_as__SupportsCeil_wrong"
# subject = "math.ceil(x: _SupportsCeil)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.ceil(x: _SupportsCeil); call it with the wrong type.

typeshed contract: x is _SupportsCeil. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import ceil
try:
    ceil(_W())  # x: _SupportsCeil <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/ceil__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_ceil__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "ceil__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.ceil(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.ceil(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import ceil
try:
    ceil(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/comb__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_comb__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "comb__n_as_SupportsIndex_wrong"
# subject = "math.comb(n: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.comb(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import comb
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

/// Ported from `tests/cpython/type/std-libs/math/copysign__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_copysign__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "copysign__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.copysign(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.copysign(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import copysign
try:
    copysign(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/cos__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_cos__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "cos__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.cos(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.cos(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import cos
try:
    cos(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/cosh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_cosh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "cosh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.cosh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.cosh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import cosh
try:
    cosh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/degrees__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_degrees__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "degrees__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.degrees(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.degrees(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import degrees
try:
    degrees(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/dist__p_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_dist__p_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "dist__p_as_Iterable_wrong"
# subject = "math.dist(p: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.dist(p: Iterable); call it with the wrong type.

typeshed contract: p is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import dist
try:
    dist(_W(), None)  # p: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/erf__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_erf__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "erf__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.erf(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.erf(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import erf
try:
    erf(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/erfc__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_erfc__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "erfc__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.erfc(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.erfc(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import erfc
try:
    erfc(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/exp2__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_exp2__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "exp2__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.exp2(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.exp2(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import exp2
try:
    exp2(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/exp__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_exp__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "exp__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.exp(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.exp(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import exp
try:
    exp(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/expm1__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_expm1__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "expm1__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.expm1(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.expm1(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import expm1
try:
    expm1(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fabs__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fabs__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fabs__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.fabs(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fabs(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fabs
try:
    fabs(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/factorial__x_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_factorial__x_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "factorial__x_as_SupportsIndex_wrong"
# subject = "math.factorial(x: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.factorial(x: SupportsIndex); call it with the wrong type.

typeshed contract: x is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import factorial
try:
    factorial(_W())  # x: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/floor__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_floor__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "floor__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.floor(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.floor(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import floor
try:
    floor(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/floor__x_as__SupportsFloor_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_floor__x_as__SupportsFloor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "floor__x_as__SupportsFloor_wrong"
# subject = "math.floor(x: _SupportsFloor)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.floor(x: _SupportsFloor); call it with the wrong type.

typeshed contract: x is _SupportsFloor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import floor
try:
    floor(_W())  # x: _SupportsFloor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fma__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fma__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fma__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.fma(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fma(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fma
try:
    fma(_W(), None, None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fmax__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fmax__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fmax__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.fmax(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fmax(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fmax
try:
    fmax(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fmin__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fmin__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fmin__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.fmin(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fmin(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fmin
try:
    fmin(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fmod__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fmod__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fmod__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.fmod(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fmod(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fmod
try:
    fmod(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/frexp__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_frexp__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "frexp__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.frexp(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.frexp(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import frexp
try:
    frexp(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/fsum__seq_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_fsum__seq_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "fsum__seq_as_Iterable_wrong"
# subject = "math.fsum(seq: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.fsum(seq: Iterable); call it with the wrong type.

typeshed contract: seq is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import fsum
try:
    fsum(_W())  # seq: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/gamma__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_gamma__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "gamma__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.gamma(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.gamma(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import gamma
try:
    gamma(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isclose__a_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isclose__a_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isclose__a_as__SupportsFloatOrIndex_wrong"
# subject = "math.isclose(a: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isclose(a: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: a is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isclose
try:
    isclose(_W(), None)  # a: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isfinite__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isfinite__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isfinite__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.isfinite(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isfinite(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isfinite
try:
    isfinite(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isinf__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isinf__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isinf__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.isinf(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isinf(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isinf
try:
    isinf(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isnan__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isnan__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isnan__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.isnan(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isnan(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isnan
try:
    isnan(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isnormal__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isnormal__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isnormal__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.isnormal(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isnormal(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isnormal
try:
    isnormal(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/isqrt__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_isqrt__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "isqrt__n_as_SupportsIndex_wrong"
# subject = "math.isqrt(n: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.isqrt(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import isqrt
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

/// Ported from `tests/cpython/type/std-libs/math/issubnormal__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_issubnormal__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "issubnormal__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.issubnormal(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.issubnormal(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import issubnormal
try:
    issubnormal(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/ldexp__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_ldexp__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "ldexp__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.ldexp(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.ldexp(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import ldexp
try:
    ldexp(_W(), 0)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/lgamma__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_lgamma__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "lgamma__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.lgamma(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.lgamma(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import lgamma
try:
    lgamma(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/log10__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_log10__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "log10__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.log10(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.log10(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import log10
try:
    log10(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/log1p__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_log1p__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "log1p__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.log1p(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.log1p(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import log1p
try:
    log1p(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/log2__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_log2__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "log2__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.log2(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.log2(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import log2
try:
    log2(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/log__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_log__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "log__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.log(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.log(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import log
try:
    log(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/modf__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_modf__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "modf__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.modf(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.modf(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import modf
try:
    modf(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/nextafter__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_nextafter__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "nextafter__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.nextafter(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.nextafter(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import nextafter
try:
    nextafter(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/perm__n_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_perm__n_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "perm__n_as_SupportsIndex_wrong"
# subject = "math.perm(n: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.perm(n: SupportsIndex); call it with the wrong type.

typeshed contract: n is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import perm
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

/// Ported from `tests/cpython/type/std-libs/math/pow__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_pow__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "pow__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.pow(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.pow(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import pow
try:
    pow(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/radians__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_radians__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "radians__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.radians(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.radians(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import radians
try:
    radians(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/remainder__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_remainder__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "remainder__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.remainder(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.remainder(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import remainder
try:
    remainder(_W(), None)  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/signbit__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_signbit__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "signbit__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.signbit(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.signbit(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import signbit
try:
    signbit(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/sin__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_sin__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "sin__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.sin(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.sin(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import sin
try:
    sin(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/sinh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_sinh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "sinh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.sinh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.sinh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import sinh
try:
    sinh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/sqrt__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_sqrt__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "sqrt__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.sqrt(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.sqrt(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import sqrt
try:
    sqrt(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/sumprod__p_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_sumprod__p_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "sumprod__p_as_Iterable_wrong"
# subject = "math.sumprod(p: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.sumprod(p: Iterable); call it with the wrong type.

typeshed contract: p is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import sumprod
try:
    sumprod(_W(), None)  # p: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/tan__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_tan__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "tan__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.tan(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.tan(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import tan
try:
    tan(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/tanh__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_tanh__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "tanh__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.tanh(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.tanh(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import tanh
try:
    tanh(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/trunc__x_as__SupportsTrunc_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_trunc__x_as__SupportsTrunc_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "trunc__x_as__SupportsTrunc_wrong"
# subject = "math.trunc(x: _SupportsTrunc)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.trunc(x: _SupportsTrunc); call it with the wrong type.

typeshed contract: x is _SupportsTrunc. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import trunc
try:
    trunc(_W())  # x: _SupportsTrunc <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/math/ulp__x_as__SupportsFloatOrIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_math_ulp__x_as__SupportsFloatOrIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "math"
# dimension = "type"
# case = "ulp__x_as__SupportsFloatOrIndex_wrong"
# subject = "math.ulp(x: _SupportsFloatOrIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/math.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: math.ulp(x: _SupportsFloatOrIndex); call it with the wrong type.

typeshed contract: x is _SupportsFloatOrIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from math import ulp
try:
    ulp(_W())  # x: _SupportsFloatOrIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
