use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMACompressor____new____format_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMACompressor____new____format_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMACompressor____new____format_as_int_wrong"
# subject = "_lzma.LZMACompressor.__new__(format: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMACompressor.__new__(format: int); call it with the wrong type.

typeshed contract: format is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lzma import LZMACompressor
obj = object.__new__(LZMACompressor)
try:
    obj.__new__("not_an_int")  # format: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMACompressor__compress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMACompressor__compress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMACompressor__compress__data_as_ReadableBuffer_wrong"
# subject = "_lzma.LZMACompressor.compress(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMACompressor.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _lzma import LZMACompressor
obj = object.__new__(LZMACompressor)
try:
    obj.compress(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMACompressor__init__format_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMACompressor__init__format_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMACompressor__init__format_as_int_wrong"
# subject = "_lzma.LZMACompressor.__init__(format: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMACompressor.__init__(format: int); call it with the wrong type.

typeshed contract: format is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lzma import LZMACompressor
try:
    LZMACompressor("not_an_int")  # format: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMADecompressor____new____format_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMADecompressor____new____format_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMADecompressor____new____format_as_int_wrong"
# subject = "_lzma.LZMADecompressor.__new__(format: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMADecompressor.__new__(format: int); call it with the wrong type.

typeshed contract: format is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lzma import LZMADecompressor
obj = object.__new__(LZMADecompressor)
try:
    obj.__new__("not_an_int")  # format: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMADecompressor__decompress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMADecompressor__decompress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMADecompressor__decompress__data_as_ReadableBuffer_wrong"
# subject = "_lzma.LZMADecompressor.decompress(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMADecompressor.decompress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _lzma import LZMADecompressor
obj = object.__new__(LZMADecompressor)
try:
    obj.decompress(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/LZMADecompressor__init__format_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_LZMADecompressor__init__format_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "LZMADecompressor__init__format_as_int_wrong"
# subject = "_lzma.LZMADecompressor.__init__(format: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.LZMADecompressor.__init__(format: int); call it with the wrong type.

typeshed contract: format is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lzma import LZMADecompressor
try:
    LZMADecompressor("not_an_int")  # format: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_lzma/is_check_supported__check_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__lzma_is_check_supported__check_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_lzma"
# dimension = "type"
# case = "is_check_supported__check_id_as_int_wrong"
# subject = "_lzma.is_check_supported(check_id: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _lzma.is_check_supported(check_id: int); call it with the wrong type.

typeshed contract: check_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _lzma import is_check_supported
try:
    is_check_supported("not_an_int")  # check_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__init__filename_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__init__filename_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__init__filename_as_typed_wrong"
# subject = "lzma.LZMAFile.__init__(filename: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.__init__(filename: typed); call it with the wrong type.

typeshed contract: filename is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import LZMAFile
try:
    LZMAFile(_W())  # filename: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__peek__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__peek__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__peek__size_as_int_wrong"
# subject = "lzma.LZMAFile.peek(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.peek(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lzma import LZMAFile
obj = object.__new__(LZMAFile)
try:
    obj.peek("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__read1__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__read1__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__read1__size_as_int_wrong"
# subject = "lzma.LZMAFile.read1(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.read1(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lzma import LZMAFile
obj = object.__new__(LZMAFile)
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

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__read__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__read__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__read__size_as_typed_wrong"
# subject = "lzma.LZMAFile.read(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.read(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import LZMAFile
obj = object.__new__(LZMAFile)
try:
    obj.read(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__readline__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__readline__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__readline__size_as_typed_wrong"
# subject = "lzma.LZMAFile.readline(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.readline(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import LZMAFile
obj = object.__new__(LZMAFile)
try:
    obj.readline(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__seek__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__seek__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__seek__offset_as_int_wrong"
# subject = "lzma.LZMAFile.seek(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lzma import LZMAFile
obj = object.__new__(LZMAFile)
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

/// Ported from `tests/cpython/type/std-libs/lzma/LZMAFile__write__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_LZMAFile__write__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "LZMAFile__write__data_as_ReadableBuffer_wrong"
# subject = "lzma.LZMAFile.write(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.LZMAFile.write(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import LZMAFile
obj = object.__new__(LZMAFile)
try:
    obj.write(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/compress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_compress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "compress__data_as_ReadableBuffer_wrong"
# subject = "lzma.compress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import compress
try:
    compress(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/decompress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_decompress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "decompress__data_as_ReadableBuffer_wrong"
# subject = "lzma.decompress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.decompress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import decompress
try:
    decompress(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/open__filename_as_StrOrBytesPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_open__filename_as_StrOrBytesPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "open__filename_as_StrOrBytesPath_wrong"
# subject = "lzma.open(filename: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.open(filename: StrOrBytesPath); call it with the wrong type.

typeshed contract: filename is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import open
try:
    open(_W(), None)  # filename: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lzma/open__filename_as__PathOrFile_wrong.py`.
#[test]
fn test_gen_type_std_libs_lzma_open__filename_as__PathOrFile_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lzma"
# dimension = "type"
# case = "open__filename_as__PathOrFile_wrong"
# subject = "lzma.open(filename: _PathOrFile)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lzma.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lzma.open(filename: _PathOrFile); call it with the wrong type.

typeshed contract: filename is _PathOrFile. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lzma import open
try:
    open(_W())  # filename: _PathOrFile <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
