use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pickletools/ArgumentDescriptor__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_ArgumentDescriptor__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "ArgumentDescriptor__init__name_as_str_wrong"
# subject = "pickletools.ArgumentDescriptor.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.ArgumentDescriptor.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickletools import ArgumentDescriptor
try:
    ArgumentDescriptor(12345, 0, None, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/OpcodeInfo__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_OpcodeInfo__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "OpcodeInfo__init__name_as_str_wrong"
# subject = "pickletools.OpcodeInfo.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.OpcodeInfo.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickletools import OpcodeInfo
try:
    OpcodeInfo(12345, "", None, None, None, 0, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/StackObject__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_StackObject__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "StackObject__init__name_as_str_wrong"
# subject = "pickletools.StackObject.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.StackObject.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pickletools import StackObject
try:
    StackObject(12345, None, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/dis__pickle_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_dis__pickle_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "dis__pickle_as_typed_wrong"
# subject = "pickletools.dis(pickle: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.dis(pickle: typed); call it with the wrong type.

typeshed contract: pickle is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import dis
try:
    dis(_W())  # pickle: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/genops__pickle_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_genops__pickle_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "genops__pickle_as_typed_wrong"
# subject = "pickletools.genops(pickle: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.genops(pickle: typed); call it with the wrong type.

typeshed contract: pickle is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import genops
try:
    genops(_W())  # pickle: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/optimize__p_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_optimize__p_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "optimize__p_as_typed_wrong"
# subject = "pickletools.optimize(p: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.optimize(p: typed); call it with the wrong type.

typeshed contract: p is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import optimize
try:
    optimize(_W())  # p: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_bytes1__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_bytes1__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_bytes1__f_as_IO_wrong"
# subject = "pickletools.read_bytes1(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_bytes1(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_bytes1
try:
    read_bytes1(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_bytes4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_bytes4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_bytes4__f_as_IO_wrong"
# subject = "pickletools.read_bytes4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_bytes4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_bytes4
try:
    read_bytes4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_bytes8__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_bytes8__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_bytes8__f_as_IO_wrong"
# subject = "pickletools.read_bytes8(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_bytes8(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_bytes8
try:
    read_bytes8(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_decimalnl_long__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_decimalnl_long__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_decimalnl_long__f_as_IO_wrong"
# subject = "pickletools.read_decimalnl_long(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_decimalnl_long(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_decimalnl_long
try:
    read_decimalnl_long(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_decimalnl_short__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_decimalnl_short__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_decimalnl_short__f_as_IO_wrong"
# subject = "pickletools.read_decimalnl_short(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_decimalnl_short(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_decimalnl_short
try:
    read_decimalnl_short(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_float8__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_float8__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_float8__f_as_IO_wrong"
# subject = "pickletools.read_float8(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_float8(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_float8
try:
    read_float8(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_floatnl__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_floatnl__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_floatnl__f_as_IO_wrong"
# subject = "pickletools.read_floatnl(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_floatnl(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_floatnl
try:
    read_floatnl(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_int4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_int4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_int4__f_as_IO_wrong"
# subject = "pickletools.read_int4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_int4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_int4
try:
    read_int4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_long1__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_long1__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_long1__f_as_IO_wrong"
# subject = "pickletools.read_long1(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_long1(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_long1
try:
    read_long1(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_long4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_long4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_long4__f_as_IO_wrong"
# subject = "pickletools.read_long4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_long4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_long4
try:
    read_long4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_string1__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_string1__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_string1__f_as_IO_wrong"
# subject = "pickletools.read_string1(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_string1(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_string1
try:
    read_string1(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_string4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_string4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_string4__f_as_IO_wrong"
# subject = "pickletools.read_string4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_string4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_string4
try:
    read_string4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_stringnl__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_stringnl__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_stringnl__f_as_IO_wrong"
# subject = "pickletools.read_stringnl(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_stringnl(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_stringnl
try:
    read_stringnl(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_stringnl_noescape__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_stringnl_noescape__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_stringnl_noescape__f_as_IO_wrong"
# subject = "pickletools.read_stringnl_noescape(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_stringnl_noescape(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_stringnl_noescape
try:
    read_stringnl_noescape(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_stringnl_noescape_pair__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_stringnl_noescape_pair__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_stringnl_noescape_pair__f_as_IO_wrong"
# subject = "pickletools.read_stringnl_noescape_pair(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_stringnl_noescape_pair(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_stringnl_noescape_pair
try:
    read_stringnl_noescape_pair(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_uint1__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_uint1__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_uint1__f_as_IO_wrong"
# subject = "pickletools.read_uint1(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_uint1(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_uint1
try:
    read_uint1(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_uint2__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_uint2__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_uint2__f_as_IO_wrong"
# subject = "pickletools.read_uint2(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_uint2(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_uint2
try:
    read_uint2(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_uint4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_uint4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_uint4__f_as_IO_wrong"
# subject = "pickletools.read_uint4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_uint4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_uint4
try:
    read_uint4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_uint8__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_uint8__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_uint8__f_as_IO_wrong"
# subject = "pickletools.read_uint8(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_uint8(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_uint8
try:
    read_uint8(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_unicodestring1__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_unicodestring1__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_unicodestring1__f_as_IO_wrong"
# subject = "pickletools.read_unicodestring1(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_unicodestring1(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_unicodestring1
try:
    read_unicodestring1(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_unicodestring4__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_unicodestring4__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_unicodestring4__f_as_IO_wrong"
# subject = "pickletools.read_unicodestring4(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_unicodestring4(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_unicodestring4
try:
    read_unicodestring4(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_unicodestring8__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_unicodestring8__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_unicodestring8__f_as_IO_wrong"
# subject = "pickletools.read_unicodestring8(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_unicodestring8(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_unicodestring8
try:
    read_unicodestring8(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pickletools/read_unicodestringnl__f_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_pickletools_read_unicodestringnl__f_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "type"
# case = "read_unicodestringnl__f_as_IO_wrong"
# subject = "pickletools.read_unicodestringnl(f: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pickletools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pickletools.read_unicodestringnl(f: IO); call it with the wrong type.

typeshed contract: f is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pickletools import read_unicodestringnl
try:
    read_unicodestringnl(_W())  # f: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
