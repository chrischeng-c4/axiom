use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/plistlib/InvalidFileException__init__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_InvalidFileException__init__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "InvalidFileException__init__message_as_str_wrong"
# subject = "plistlib.InvalidFileException.__init__(message: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.InvalidFileException.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from plistlib import InvalidFileException
try:
    InvalidFileException(12345)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/UID__init__data_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_UID__init__data_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "UID__init__data_as_int_wrong"
# subject = "plistlib.UID.__init__(data: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.UID.__init__(data: int); call it with the wrong type.

typeshed contract: data is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from plistlib import UID
try:
    UID("not_an_int")  # data: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/dump__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_dump__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "dump__value_as_typed_wrong"
# subject = "plistlib.dump(value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.dump(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import dump
try:
    dump(_W(), None)  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/dumps__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_dumps__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "dumps__value_as_typed_wrong"
# subject = "plistlib.dumps(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.dumps(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import dumps
try:
    dumps(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/load__fp_as_IO_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_load__fp_as_IO_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "load__fp_as_IO_wrong"
# subject = "plistlib.load(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.load(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import load
try:
    load(_W())  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/loads__value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_loads__value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "loads__value_as_ReadableBuffer_wrong"
# subject = "plistlib.loads(value: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.loads(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import loads
try:
    loads(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/plistlib/loads__value_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_plistlib_loads__value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "loads__value_as_typed_wrong"
# subject = "plistlib.loads(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.loads(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import loads
try:
    loads(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
