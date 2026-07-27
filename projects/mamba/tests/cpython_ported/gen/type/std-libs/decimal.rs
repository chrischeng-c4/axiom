use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_decimal/IEEEContext__bits_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__decimal_IEEEContext__bits_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_decimal"
# dimension = "type"
# case = "IEEEContext__bits_as_int_wrong"
# subject = "_decimal.IEEEContext(bits: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _decimal.IEEEContext(bits: int); call it with the wrong type.

typeshed contract: bits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _decimal import IEEEContext
try:
    IEEEContext("not_an_int")  # bits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_decimal/localcontext__ctx_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__decimal_localcontext__ctx_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_decimal"
# dimension = "type"
# case = "localcontext__ctx_as_typed_wrong"
# subject = "_decimal.localcontext(ctx: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _decimal.localcontext(ctx: typed); call it with the wrong type.

typeshed contract: ctx is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _decimal import localcontext
try:
    localcontext(_W())  # ctx: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_decimal/setcontext__context_as_Context_wrong.py`.
#[test]
fn test_gen_type_std_libs__decimal_setcontext__context_as_Context_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_decimal"
# dimension = "type"
# case = "setcontext__context_as_Context_wrong"
# subject = "_decimal.setcontext(context: Context)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _decimal.setcontext(context: Context); call it with the wrong type.

typeshed contract: context is Context. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _decimal import setcontext
try:
    setcontext(_W())  # context: Context <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__abs__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__abs__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__abs__x_as__Decimal_wrong"
# subject = "decimal.Context.abs(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.abs(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.abs(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__add__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__add__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__add__x_as__Decimal_wrong"
# subject = "decimal.Context.add(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.add(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.add(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__canonical__x_as_Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__canonical__x_as_Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__canonical__x_as_Decimal_wrong"
# subject = "decimal.Context.canonical(x: Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.canonical(x: Decimal); call it with the wrong type.

typeshed contract: x is Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.canonical(_W())  # x: Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__compare__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__compare__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__compare__x_as__Decimal_wrong"
# subject = "decimal.Context.compare(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.compare(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.compare(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__compare_signal__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__compare_signal__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__compare_signal__x_as__Decimal_wrong"
# subject = "decimal.Context.compare_signal(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.compare_signal(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.compare_signal(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__compare_total__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__compare_total__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__compare_total__x_as__Decimal_wrong"
# subject = "decimal.Context.compare_total(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.compare_total(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.compare_total(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__compare_total_mag__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__compare_total_mag__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__compare_total_mag__x_as__Decimal_wrong"
# subject = "decimal.Context.compare_total_mag(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.compare_total_mag(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.compare_total_mag(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__copy_abs__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__copy_abs__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__copy_abs__x_as__Decimal_wrong"
# subject = "decimal.Context.copy_abs(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.copy_abs(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.copy_abs(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__copy_decimal__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__copy_decimal__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__copy_decimal__x_as__Decimal_wrong"
# subject = "decimal.Context.copy_decimal(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.copy_decimal(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.copy_decimal(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__copy_negate__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__copy_negate__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__copy_negate__x_as__Decimal_wrong"
# subject = "decimal.Context.copy_negate(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.copy_negate(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.copy_negate(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__copy_sign__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__copy_sign__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__copy_sign__x_as__Decimal_wrong"
# subject = "decimal.Context.copy_sign(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.copy_sign(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.copy_sign(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__create_decimal__num_as__DecimalNew_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__create_decimal__num_as__DecimalNew_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__create_decimal__num_as__DecimalNew_wrong"
# subject = "decimal.Context.create_decimal(num: _DecimalNew)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.create_decimal(num: _DecimalNew); call it with the wrong type.

typeshed contract: num is _DecimalNew. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.create_decimal(_W())  # num: _DecimalNew <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__create_decimal_from_float__f_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__create_decimal_from_float__f_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__create_decimal_from_float__f_as_float_wrong"
# subject = "decimal.Context.create_decimal_from_float(f: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.create_decimal_from_float(f: float); call it with the wrong type.

typeshed contract: f is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from decimal import Context
obj = object.__new__(Context)
try:
    obj.create_decimal_from_float("not_a_float")  # f: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__divide__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__divide__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__divide__x_as__Decimal_wrong"
# subject = "decimal.Context.divide(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.divide(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.divide(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__divide_int__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__divide_int__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__divide_int__x_as__Decimal_wrong"
# subject = "decimal.Context.divide_int(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.divide_int(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.divide_int(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__divmod__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__divmod__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__divmod__x_as__Decimal_wrong"
# subject = "decimal.Context.divmod(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.divmod(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.divmod(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__exp__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__exp__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__exp__x_as__Decimal_wrong"
# subject = "decimal.Context.exp(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.exp(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.exp(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__fma__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__fma__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__fma__x_as__Decimal_wrong"
# subject = "decimal.Context.fma(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.fma(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.fma(_W(), None, None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__init__prec_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__init__prec_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__init__prec_as_typed_wrong"
# subject = "decimal.Context.__init__(prec: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.__init__(prec: typed); call it with the wrong type.

typeshed contract: prec is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
try:
    Context(_W())  # prec: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_canonical__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_canonical__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_canonical__x_as__Decimal_wrong"
# subject = "decimal.Context.is_canonical(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_canonical(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_canonical(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_finite__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_finite__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_finite__x_as__Decimal_wrong"
# subject = "decimal.Context.is_finite(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_finite(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_finite(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_infinite__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_infinite__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_infinite__x_as__Decimal_wrong"
# subject = "decimal.Context.is_infinite(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_infinite(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_infinite(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_nan__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_nan__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_nan__x_as__Decimal_wrong"
# subject = "decimal.Context.is_nan(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_nan(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_nan(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_normal__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_normal__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_normal__x_as__Decimal_wrong"
# subject = "decimal.Context.is_normal(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_normal(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_normal(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_qnan__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_qnan__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_qnan__x_as__Decimal_wrong"
# subject = "decimal.Context.is_qnan(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_qnan(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_qnan(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_signed__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_signed__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_signed__x_as__Decimal_wrong"
# subject = "decimal.Context.is_signed(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_signed(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_signed(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_snan__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_snan__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_snan__x_as__Decimal_wrong"
# subject = "decimal.Context.is_snan(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_snan(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_snan(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_subnormal__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_subnormal__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_subnormal__x_as__Decimal_wrong"
# subject = "decimal.Context.is_subnormal(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_subnormal(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_subnormal(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__is_zero__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__is_zero__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__is_zero__x_as__Decimal_wrong"
# subject = "decimal.Context.is_zero(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.is_zero(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.is_zero(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__ln__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__ln__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__ln__x_as__Decimal_wrong"
# subject = "decimal.Context.ln(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.ln(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.ln(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__log10__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__log10__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__log10__x_as__Decimal_wrong"
# subject = "decimal.Context.log10(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.log10(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.log10(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__logb__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__logb__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__logb__x_as__Decimal_wrong"
# subject = "decimal.Context.logb(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.logb(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.logb(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__logical_and__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__logical_and__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__logical_and__x_as__Decimal_wrong"
# subject = "decimal.Context.logical_and(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.logical_and(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.logical_and(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__logical_invert__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__logical_invert__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__logical_invert__x_as__Decimal_wrong"
# subject = "decimal.Context.logical_invert(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.logical_invert(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.logical_invert(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__logical_or__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__logical_or__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__logical_or__x_as__Decimal_wrong"
# subject = "decimal.Context.logical_or(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.logical_or(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.logical_or(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__logical_xor__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__logical_xor__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__logical_xor__x_as__Decimal_wrong"
# subject = "decimal.Context.logical_xor(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.logical_xor(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.logical_xor(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__max__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__max__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__max__x_as__Decimal_wrong"
# subject = "decimal.Context.max(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.max(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.max(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__max_mag__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__max_mag__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__max_mag__x_as__Decimal_wrong"
# subject = "decimal.Context.max_mag(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.max_mag(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.max_mag(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__min__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__min__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__min__x_as__Decimal_wrong"
# subject = "decimal.Context.min(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.min(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.min(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__min_mag__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__min_mag__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__min_mag__x_as__Decimal_wrong"
# subject = "decimal.Context.min_mag(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.min_mag(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.min_mag(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__minus__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__minus__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__minus__x_as__Decimal_wrong"
# subject = "decimal.Context.minus(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.minus(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.minus(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__multiply__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__multiply__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__multiply__x_as__Decimal_wrong"
# subject = "decimal.Context.multiply(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.multiply(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.multiply(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__next_minus__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__next_minus__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__next_minus__x_as__Decimal_wrong"
# subject = "decimal.Context.next_minus(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.next_minus(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.next_minus(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__next_plus__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__next_plus__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__next_plus__x_as__Decimal_wrong"
# subject = "decimal.Context.next_plus(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.next_plus(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.next_plus(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__next_toward__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__next_toward__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__next_toward__x_as__Decimal_wrong"
# subject = "decimal.Context.next_toward(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.next_toward(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.next_toward(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__normalize__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__normalize__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__normalize__x_as__Decimal_wrong"
# subject = "decimal.Context.normalize(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.normalize(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.normalize(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__number_class__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__number_class__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__number_class__x_as__Decimal_wrong"
# subject = "decimal.Context.number_class(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.number_class(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.number_class(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__plus__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__plus__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__plus__x_as__Decimal_wrong"
# subject = "decimal.Context.plus(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.plus(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.plus(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__power__a_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__power__a_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__power__a_as__Decimal_wrong"
# subject = "decimal.Context.power(a: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.power(a: _Decimal); call it with the wrong type.

typeshed contract: a is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.power(_W(), None)  # a: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__quantize__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__quantize__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__quantize__x_as__Decimal_wrong"
# subject = "decimal.Context.quantize(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.quantize(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.quantize(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__remainder__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__remainder__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__remainder__x_as__Decimal_wrong"
# subject = "decimal.Context.remainder(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.remainder(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.remainder(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__remainder_near__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__remainder_near__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__remainder_near__x_as__Decimal_wrong"
# subject = "decimal.Context.remainder_near(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.remainder_near(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.remainder_near(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__rotate__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__rotate__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__rotate__x_as__Decimal_wrong"
# subject = "decimal.Context.rotate(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.rotate(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.rotate(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__same_quantum__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__same_quantum__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__same_quantum__x_as__Decimal_wrong"
# subject = "decimal.Context.same_quantum(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.same_quantum(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.same_quantum(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__scaleb__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__scaleb__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__scaleb__x_as__Decimal_wrong"
# subject = "decimal.Context.scaleb(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.scaleb(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.scaleb(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__shift__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__shift__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__shift__x_as__Decimal_wrong"
# subject = "decimal.Context.shift(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.shift(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.shift(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__sqrt__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__sqrt__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__sqrt__x_as__Decimal_wrong"
# subject = "decimal.Context.sqrt(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.sqrt(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.sqrt(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__subtract__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__subtract__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__subtract__x_as__Decimal_wrong"
# subject = "decimal.Context.subtract(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.subtract(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.subtract(_W(), None)  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__to_eng_string__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__to_eng_string__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__to_eng_string__x_as__Decimal_wrong"
# subject = "decimal.Context.to_eng_string(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.to_eng_string(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.to_eng_string(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__to_integral__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__to_integral__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__to_integral__x_as__Decimal_wrong"
# subject = "decimal.Context.to_integral(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.to_integral(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.to_integral(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__to_integral_exact__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__to_integral_exact__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__to_integral_exact__x_as__Decimal_wrong"
# subject = "decimal.Context.to_integral_exact(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.to_integral_exact(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.to_integral_exact(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__to_integral_value__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__to_integral_value__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__to_integral_value__x_as__Decimal_wrong"
# subject = "decimal.Context.to_integral_value(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.to_integral_value(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.to_integral_value(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Context__to_sci_string__x_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Context__to_sci_string__x_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Context__to_sci_string__x_as__Decimal_wrong"
# subject = "decimal.Context.to_sci_string(x: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Context.to_sci_string(x: _Decimal); call it with the wrong type.

typeshed contract: x is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Context
obj = object.__new__(Context)
try:
    obj.to_sci_string(_W())  # x: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____add____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____add____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____add____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__add__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__add__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__add__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____divmod____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____divmod____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____divmod____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__divmod__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__divmod__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__divmod__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____floordiv____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____floordiv____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____floordiv____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__floordiv__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__floordiv__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__floordiv__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____format____specifier_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____format____specifier_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____format____specifier_as_str_wrong"
# subject = "decimal.Decimal.__format__(specifier: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__format__(specifier: str); call it with the wrong type.

typeshed contract: specifier is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__format__(12345)  # specifier: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____ge____value_as__ComparableNum_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____ge____value_as__ComparableNum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____ge____value_as__ComparableNum_wrong"
# subject = "decimal.Decimal.__ge__(value: _ComparableNum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__ge__(value: _ComparableNum); call it with the wrong type.

typeshed contract: value is _ComparableNum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__ge__(_W())  # value: _ComparableNum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____gt____value_as__ComparableNum_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____gt____value_as__ComparableNum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____gt____value_as__ComparableNum_wrong"
# subject = "decimal.Decimal.__gt__(value: _ComparableNum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__gt__(value: _ComparableNum); call it with the wrong type.

typeshed contract: value is _ComparableNum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__gt__(_W())  # value: _ComparableNum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____le____value_as__ComparableNum_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____le____value_as__ComparableNum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____le____value_as__ComparableNum_wrong"
# subject = "decimal.Decimal.__le__(value: _ComparableNum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__le__(value: _ComparableNum); call it with the wrong type.

typeshed contract: value is _ComparableNum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__le__(_W())  # value: _ComparableNum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____lt____value_as__ComparableNum_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____lt____value_as__ComparableNum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____lt____value_as__ComparableNum_wrong"
# subject = "decimal.Decimal.__lt__(value: _ComparableNum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__lt__(value: _ComparableNum); call it with the wrong type.

typeshed contract: value is _ComparableNum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__lt__(_W())  # value: _ComparableNum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____mod____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____mod____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____mod____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__mod__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__mod__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__mod__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____mul____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____mul____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____mul____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__mul__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__mul__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__mul__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____new____value_as__DecimalNew_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____new____value_as__DecimalNew_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____new____value_as__DecimalNew_wrong"
# subject = "decimal.Decimal.__new__(value: _DecimalNew)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__new__(value: _DecimalNew); call it with the wrong type.

typeshed contract: value is _DecimalNew. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__new__(_W())  # value: _DecimalNew <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____pow____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____pow____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____pow____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__pow__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__pow__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__pow__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____radd____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____radd____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____radd____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__radd__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__radd__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__radd__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rdivmod____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rdivmod____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rdivmod____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rdivmod__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rdivmod__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rdivmod__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rfloordiv____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rfloordiv____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rfloordiv____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rfloordiv__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rfloordiv__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rfloordiv__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rmod____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rmod____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rmod____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rmod__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rmod__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rmod__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rmul____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rmul____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rmul____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rmul__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rmul__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rmul__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____round____ndigits_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____round____ndigits_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____round____ndigits_as_int_wrong"
# subject = "decimal.Decimal.__round__(ndigits: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__round__(ndigits: int); call it with the wrong type.

typeshed contract: ndigits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from decimal import Decimal
obj = Decimal("1.25")
try:
    obj.__round__("not_an_int")  # ndigits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rpow____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rpow____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rpow____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rpow__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rpow__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rpow__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rsub____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rsub____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rsub____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rsub__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rsub__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rsub__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____rtruediv____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____rtruediv____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____rtruediv____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__rtruediv__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__rtruediv__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__rtruediv__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____sub____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____sub____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____sub____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__sub__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__sub__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__sub__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal____truediv____value_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal____truediv____value_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal____truediv____value_as__Decimal_wrong"
# subject = "decimal.Decimal.__truediv__(value: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.__truediv__(value: _Decimal); call it with the wrong type.

typeshed contract: value is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.__truediv__(_W())  # value: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__compare__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__compare__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__compare__other_as__Decimal_wrong"
# subject = "decimal.Decimal.compare(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.compare(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.compare(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__compare_signal__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__compare_signal__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__compare_signal__other_as__Decimal_wrong"
# subject = "decimal.Decimal.compare_signal(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.compare_signal(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.compare_signal(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__compare_total__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__compare_total__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__compare_total__other_as__Decimal_wrong"
# subject = "decimal.Decimal.compare_total(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.compare_total(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.compare_total(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__compare_total_mag__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__compare_total_mag__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__compare_total_mag__other_as__Decimal_wrong"
# subject = "decimal.Decimal.compare_total_mag(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.compare_total_mag(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.compare_total_mag(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__copy_sign__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__copy_sign__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__copy_sign__other_as__Decimal_wrong"
# subject = "decimal.Decimal.copy_sign(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.copy_sign(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.copy_sign(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__exp__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__exp__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__exp__context_as_typed_wrong"
# subject = "decimal.Decimal.exp(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.exp(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.exp(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__fma__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__fma__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__fma__other_as__Decimal_wrong"
# subject = "decimal.Decimal.fma(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.fma(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.fma(_W(), None)  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__from_float__f_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__from_float__f_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__from_float__f_as_float_wrong"
# subject = "decimal.Decimal.from_float(f: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.from_float(f: float); call it with the wrong type.

typeshed contract: f is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from decimal import Decimal
try:
    Decimal.from_float("not_a_float")  # f: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__from_number__number_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__from_number__number_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__from_number__number_as_typed_wrong"
# subject = "decimal.Decimal.from_number(number: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.from_number(number: typed); call it with the wrong type.

typeshed contract: number is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
try:
    Decimal.from_number(_W())  # number: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__is_normal__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__is_normal__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__is_normal__context_as_typed_wrong"
# subject = "decimal.Decimal.is_normal(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.is_normal(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.is_normal(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__is_subnormal__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__is_subnormal__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__is_subnormal__context_as_typed_wrong"
# subject = "decimal.Decimal.is_subnormal(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.is_subnormal(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.is_subnormal(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__ln__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__ln__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__ln__context_as_typed_wrong"
# subject = "decimal.Decimal.ln(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.ln(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.ln(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__log10__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__log10__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__log10__context_as_typed_wrong"
# subject = "decimal.Decimal.log10(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.log10(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.log10(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__logb__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__logb__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__logb__context_as_typed_wrong"
# subject = "decimal.Decimal.logb(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.logb(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.logb(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__logical_and__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__logical_and__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__logical_and__other_as__Decimal_wrong"
# subject = "decimal.Decimal.logical_and(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.logical_and(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.logical_and(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__logical_invert__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__logical_invert__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__logical_invert__context_as_typed_wrong"
# subject = "decimal.Decimal.logical_invert(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.logical_invert(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.logical_invert(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__logical_or__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__logical_or__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__logical_or__other_as__Decimal_wrong"
# subject = "decimal.Decimal.logical_or(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.logical_or(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.logical_or(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__logical_xor__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__logical_xor__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__logical_xor__other_as__Decimal_wrong"
# subject = "decimal.Decimal.logical_xor(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.logical_xor(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.logical_xor(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__max__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__max__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__max__other_as__Decimal_wrong"
# subject = "decimal.Decimal.max(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.max(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.max(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__max_mag__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__max_mag__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__max_mag__other_as__Decimal_wrong"
# subject = "decimal.Decimal.max_mag(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.max_mag(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.max_mag(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__min__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__min__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__min__other_as__Decimal_wrong"
# subject = "decimal.Decimal.min(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.min(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.min(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__min_mag__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__min_mag__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__min_mag__other_as__Decimal_wrong"
# subject = "decimal.Decimal.min_mag(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.min_mag(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.min_mag(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__next_minus__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__next_minus__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__next_minus__context_as_typed_wrong"
# subject = "decimal.Decimal.next_minus(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.next_minus(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.next_minus(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__next_plus__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__next_plus__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__next_plus__context_as_typed_wrong"
# subject = "decimal.Decimal.next_plus(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.next_plus(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.next_plus(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__next_toward__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__next_toward__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__next_toward__other_as__Decimal_wrong"
# subject = "decimal.Decimal.next_toward(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.next_toward(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.next_toward(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__normalize__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__normalize__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__normalize__context_as_typed_wrong"
# subject = "decimal.Decimal.normalize(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.normalize(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.normalize(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__number_class__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__number_class__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__number_class__context_as_typed_wrong"
# subject = "decimal.Decimal.number_class(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.number_class(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.number_class(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__quantize__exp_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__quantize__exp_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__quantize__exp_as__Decimal_wrong"
# subject = "decimal.Decimal.quantize(exp: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.quantize(exp: _Decimal); call it with the wrong type.

typeshed contract: exp is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.quantize(_W())  # exp: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__remainder_near__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__remainder_near__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__remainder_near__other_as__Decimal_wrong"
# subject = "decimal.Decimal.remainder_near(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.remainder_near(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.remainder_near(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__rotate__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__rotate__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__rotate__other_as__Decimal_wrong"
# subject = "decimal.Decimal.rotate(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.rotate(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.rotate(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__same_quantum__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__same_quantum__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__same_quantum__other_as__Decimal_wrong"
# subject = "decimal.Decimal.same_quantum(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.same_quantum(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.same_quantum(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__scaleb__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__scaleb__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__scaleb__other_as__Decimal_wrong"
# subject = "decimal.Decimal.scaleb(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.scaleb(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.scaleb(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__shift__other_as__Decimal_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__shift__other_as__Decimal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__shift__other_as__Decimal_wrong"
# subject = "decimal.Decimal.shift(other: _Decimal)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.shift(other: _Decimal); call it with the wrong type.

typeshed contract: other is _Decimal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.shift(_W())  # other: _Decimal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__sqrt__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__sqrt__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__sqrt__context_as_typed_wrong"
# subject = "decimal.Decimal.sqrt(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.sqrt(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.sqrt(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__to_eng_string__context_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__to_eng_string__context_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__to_eng_string__context_as_typed_wrong"
# subject = "decimal.Decimal.to_eng_string(context: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.to_eng_string(context: typed); call it with the wrong type.

typeshed contract: context is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.to_eng_string(_W())  # context: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__to_integral__rounding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__to_integral__rounding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__to_integral__rounding_as_typed_wrong"
# subject = "decimal.Decimal.to_integral(rounding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.to_integral(rounding: typed); call it with the wrong type.

typeshed contract: rounding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.to_integral(_W())  # rounding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__to_integral_exact__rounding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__to_integral_exact__rounding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__to_integral_exact__rounding_as_typed_wrong"
# subject = "decimal.Decimal.to_integral_exact(rounding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.to_integral_exact(rounding: typed); call it with the wrong type.

typeshed contract: rounding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.to_integral_exact(_W())  # rounding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/decimal/Decimal__to_integral_value__rounding_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_decimal_Decimal__to_integral_value__rounding_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "type"
# case = "Decimal__to_integral_value__rounding_as_typed_wrong"
# subject = "decimal.Decimal.to_integral_value(rounding: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: decimal.Decimal.to_integral_value(rounding: typed); call it with the wrong type.

typeshed contract: rounding is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from decimal import Decimal
obj = object.__new__(Decimal)
try:
    obj.to_integral_value(_W())  # rounding: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
