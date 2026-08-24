use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_a1__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_a1__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_a1__code_as_str_wrong"
# subject = "stringprep.in_table_a1(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_a1(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_a1
try:
    in_table_a1(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_b1__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_b1__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_b1__code_as_str_wrong"
# subject = "stringprep.in_table_b1(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_b1(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_b1
try:
    in_table_b1(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c11__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c11__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c11__code_as_str_wrong"
# subject = "stringprep.in_table_c11(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c11(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c11
try:
    in_table_c11(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c11_c12__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c11_c12__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c11_c12__code_as_str_wrong"
# subject = "stringprep.in_table_c11_c12(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c11_c12(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c11_c12
try:
    in_table_c11_c12(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c12__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c12__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c12__code_as_str_wrong"
# subject = "stringprep.in_table_c12(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c12(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c12
try:
    in_table_c12(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c21__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c21__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c21__code_as_str_wrong"
# subject = "stringprep.in_table_c21(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c21(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c21
try:
    in_table_c21(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c21_c22__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c21_c22__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c21_c22__code_as_str_wrong"
# subject = "stringprep.in_table_c21_c22(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c21_c22(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c21_c22
try:
    in_table_c21_c22(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c22__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c22__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c22__code_as_str_wrong"
# subject = "stringprep.in_table_c22(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c22(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c22
try:
    in_table_c22(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c3__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c3__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c3__code_as_str_wrong"
# subject = "stringprep.in_table_c3(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c3(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c3
try:
    in_table_c3(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c4__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c4__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c4__code_as_str_wrong"
# subject = "stringprep.in_table_c4(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c4(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c4
try:
    in_table_c4(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c5__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c5__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c5__code_as_str_wrong"
# subject = "stringprep.in_table_c5(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c5(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c5
try:
    in_table_c5(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c6__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c6__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c6__code_as_str_wrong"
# subject = "stringprep.in_table_c6(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c6(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c6
try:
    in_table_c6(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c7__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c7__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c7__code_as_str_wrong"
# subject = "stringprep.in_table_c7(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c7(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c7
try:
    in_table_c7(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c8__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c8__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c8__code_as_str_wrong"
# subject = "stringprep.in_table_c8(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c8(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c8
try:
    in_table_c8(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_c9__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_c9__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_c9__code_as_str_wrong"
# subject = "stringprep.in_table_c9(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_c9(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_c9
try:
    in_table_c9(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_d1__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_d1__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_d1__code_as_str_wrong"
# subject = "stringprep.in_table_d1(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_d1(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_d1
try:
    in_table_d1(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/in_table_d2__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_in_table_d2__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "in_table_d2__code_as_str_wrong"
# subject = "stringprep.in_table_d2(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.in_table_d2(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import in_table_d2
try:
    in_table_d2(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/map_table_b2__a_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_map_table_b2__a_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "map_table_b2__a_as_str_wrong"
# subject = "stringprep.map_table_b2(a: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.map_table_b2(a: str); call it with the wrong type.

typeshed contract: a is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import map_table_b2
try:
    map_table_b2(12345)  # a: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/stringprep/map_table_b3__code_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_stringprep_map_table_b3__code_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "type"
# case = "map_table_b3__code_as_str_wrong"
# subject = "stringprep.map_table_b3(code: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/stringprep.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: stringprep.map_table_b3(code: str); call it with the wrong type.

typeshed contract: code is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from stringprep import map_table_b3
try:
    map_table_b3(12345)  # code: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
