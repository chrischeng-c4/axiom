use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/xdrlib/Error__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Error__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Error__init__msg_as_str_wrong"
# subject = "xdrlib.Error.__init__(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Error.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Error
try:
    Error(12345)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_double__x_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_double__x_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_double__x_as_float_wrong"
# subject = "xdrlib.Packer.pack_double(x: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_double(x: float); call it with the wrong type.

typeshed contract: x is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_double("not_a_float")  # x: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_enum__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_enum__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_enum__x_as_int_wrong"
# subject = "xdrlib.Packer.pack_enum(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_enum(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_enum("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_farray__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_farray__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_farray__n_as_int_wrong"
# subject = "xdrlib.Packer.pack_farray(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_farray(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_farray("not_an_int", None, None)  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_float__x_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_float__x_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_float__x_as_float_wrong"
# subject = "xdrlib.Packer.pack_float(x: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_float(x: float); call it with the wrong type.

typeshed contract: x is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_float("not_a_float")  # x: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_fopaque__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_fopaque__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_fopaque__n_as_int_wrong"
# subject = "xdrlib.Packer.pack_fopaque(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_fopaque(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_fopaque("not_an_int", b"")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_fstring__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_fstring__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_fstring__n_as_int_wrong"
# subject = "xdrlib.Packer.pack_fstring(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_fstring(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_fstring("not_an_int", b"")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_hyper__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_hyper__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_hyper__x_as_int_wrong"
# subject = "xdrlib.Packer.pack_hyper(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_hyper(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_hyper("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_int__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_int__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_int__x_as_int_wrong"
# subject = "xdrlib.Packer.pack_int(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_int(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_int("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_uhyper__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_uhyper__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_uhyper__x_as_int_wrong"
# subject = "xdrlib.Packer.pack_uhyper(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_uhyper(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_uhyper("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Packer__pack_uint__x_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Packer__pack_uint__x_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Packer__pack_uint__x_as_int_wrong"
# subject = "xdrlib.Packer.pack_uint(x: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Packer.pack_uint(x: int); call it with the wrong type.

typeshed contract: x is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Packer
obj = object.__new__(Packer)
try:
    obj.pack_uint("not_an_int")  # x: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Unpacker__set_position__position_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Unpacker__set_position__position_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__set_position__position_as_int_wrong"
# subject = "xdrlib.Unpacker.set_position(position: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.set_position(position: int); call it with the wrong type.

typeshed contract: position is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.set_position("not_an_int")  # position: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Unpacker__unpack_farray__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Unpacker__unpack_farray__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__unpack_farray__n_as_int_wrong"
# subject = "xdrlib.Unpacker.unpack_farray(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.unpack_farray(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.unpack_farray("not_an_int", None)  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Unpacker__unpack_fopaque__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Unpacker__unpack_fopaque__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__unpack_fopaque__n_as_int_wrong"
# subject = "xdrlib.Unpacker.unpack_fopaque(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.unpack_fopaque(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.unpack_fopaque("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/xdrlib/Unpacker__unpack_fstring__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_xdrlib_Unpacker__unpack_fstring__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "xdrlib"
# dimension = "type"
# case = "Unpacker__unpack_fstring__n_as_int_wrong"
# subject = "xdrlib.Unpacker.unpack_fstring(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/xdrlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: xdrlib.Unpacker.unpack_fstring(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from xdrlib import Unpacker
obj = object.__new__(Unpacker)
try:
    obj.unpack_fstring("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
