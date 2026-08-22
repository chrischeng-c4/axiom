use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/cmath/acos__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_acos__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "acos__z_as__C_wrong"
# subject = "cmath.acos(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.acos(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import acos
try:
    acos(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/acosh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_acosh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "acosh__z_as__C_wrong"
# subject = "cmath.acosh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.acosh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import acosh
try:
    acosh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/asin__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_asin__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "asin__z_as__C_wrong"
# subject = "cmath.asin(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.asin(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import asin
try:
    asin(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/asinh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_asinh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "asinh__z_as__C_wrong"
# subject = "cmath.asinh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.asinh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import asinh
try:
    asinh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/atan__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_atan__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "atan__z_as__C_wrong"
# subject = "cmath.atan(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.atan(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import atan
try:
    atan(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/atanh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_atanh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "atanh__z_as__C_wrong"
# subject = "cmath.atanh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.atanh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import atanh
try:
    atanh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/cos__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_cos__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "cos__z_as__C_wrong"
# subject = "cmath.cos(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.cos(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import cos
try:
    cos(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/cosh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_cosh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "cosh__z_as__C_wrong"
# subject = "cmath.cosh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.cosh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import cosh
try:
    cosh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/exp__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_exp__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "exp__z_as__C_wrong"
# subject = "cmath.exp(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.exp(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import exp
try:
    exp(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/isclose__a_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_isclose__a_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "isclose__a_as__C_wrong"
# subject = "cmath.isclose(a: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.isclose(a: _C); call it with the wrong type.

typeshed contract: a is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import isclose
try:
    isclose(_W(), None)  # a: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/isfinite__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_isfinite__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "isfinite__z_as__C_wrong"
# subject = "cmath.isfinite(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.isfinite(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import isfinite
try:
    isfinite(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/isinf__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_isinf__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "isinf__z_as__C_wrong"
# subject = "cmath.isinf(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.isinf(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import isinf
try:
    isinf(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/isnan__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_isnan__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "isnan__z_as__C_wrong"
# subject = "cmath.isnan(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.isnan(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import isnan
try:
    isnan(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/log10__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_log10__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "log10__z_as__C_wrong"
# subject = "cmath.log10(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.log10(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import log10
try:
    log10(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/log__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_log__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "log__z_as__C_wrong"
# subject = "cmath.log(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.log(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import log
try:
    log(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/phase__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_phase__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "phase__z_as__C_wrong"
# subject = "cmath.phase(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.phase(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import phase
try:
    phase(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/polar__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_polar__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "polar__z_as__C_wrong"
# subject = "cmath.polar(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.polar(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import polar
try:
    polar(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/rect__r_as__F_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_rect__r_as__F_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "rect__r_as__F_wrong"
# subject = "cmath.rect(r: _F)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.rect(r: _F); call it with the wrong type.

typeshed contract: r is _F. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import rect
try:
    rect(_W(), None)  # r: _F <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/sin__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_sin__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "sin__z_as__C_wrong"
# subject = "cmath.sin(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.sin(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import sin
try:
    sin(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/sinh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_sinh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "sinh__z_as__C_wrong"
# subject = "cmath.sinh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.sinh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import sinh
try:
    sinh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/sqrt__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_sqrt__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "sqrt__z_as__C_wrong"
# subject = "cmath.sqrt(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.sqrt(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import sqrt
try:
    sqrt(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/tan__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_tan__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "tan__z_as__C_wrong"
# subject = "cmath.tan(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.tan(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import tan
try:
    tan(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/cmath/tanh__z_as__C_wrong.py`.
#[test]
fn test_gen_type_std_libs_cmath_tanh__z_as__C_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmath"
# dimension = "type"
# case = "tanh__z_as__C_wrong"
# subject = "cmath.tanh(z: _C)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cmath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cmath.tanh(z: _C); call it with the wrong type.

typeshed contract: z is _C. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cmath import tanh
try:
    tanh(_W())  # z: _C <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
