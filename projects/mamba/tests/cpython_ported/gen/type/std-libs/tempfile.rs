use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__init__max_size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__init__max_size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__init__max_size_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.__init__(max_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.__init__(max_size: int); call it with the wrong type.

typeshed contract: max_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
try:
    SpooledTemporaryFile("not_an_int")  # max_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__read1__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__read1__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__read1__size_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.read1(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.read1(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.read1("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__read__n_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__read__n_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__read__n_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.read(n: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.read(n: int); call it with the wrong type.

typeshed contract: n is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.read("not_an_int")  # n: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__readinto1__b_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__readinto1__b_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__readinto1__b_as_WriteableBuffer_wrong"
# subject = "tempfile.SpooledTemporaryFile.readinto1(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.readinto1(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.readinto1(_W())  # b: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__readinto__b_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__readinto__b_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__readinto__b_as_WriteableBuffer_wrong"
# subject = "tempfile.SpooledTemporaryFile.readinto(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.readinto(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.readinto(_W())  # b: WriteableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__readline__limit_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__readline__limit_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__readline__limit_as_typed_wrong"
# subject = "tempfile.SpooledTemporaryFile.readline(limit: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.readline(limit: typed); call it with the wrong type.

typeshed contract: limit is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.readline(_W())  # limit: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__readlines__hint_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__readlines__hint_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__readlines__hint_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.readlines(hint: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.readlines(hint: int); call it with the wrong type.

typeshed contract: hint is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.readlines("not_an_int")  # hint: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__seek__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__seek__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__seek__offset_as_int_wrong"
# subject = "tempfile.SpooledTemporaryFile.seek(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.seek("not_an_int")  # offset: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/SpooledTemporaryFile__truncate__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_SpooledTemporaryFile__truncate__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "SpooledTemporaryFile__truncate__size_as_typed_wrong"
# subject = "tempfile.SpooledTemporaryFile.truncate(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.SpooledTemporaryFile.truncate(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tempfile import SpooledTemporaryFile
obj = object.__new__(SpooledTemporaryFile)
try:
    obj.truncate(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tempfile/mktemp__suffix_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_tempfile_mktemp__suffix_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "type"
# case = "mktemp__suffix_as_str_wrong"
# subject = "tempfile.mktemp(suffix: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tempfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tempfile.mktemp(suffix: str); call it with the wrong type.

typeshed contract: suffix is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from tempfile import mktemp
try:
    mktemp(12345)  # suffix: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
