use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zlib/adler32__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_adler32__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "adler32__data_as_ReadableBuffer_wrong"
# subject = "zlib.adler32(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.adler32(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zlib import adler32
try:
    adler32(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zlib/adler32_combine__adler1_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_adler32_combine__adler1_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "adler32_combine__adler1_as_int_wrong"
# subject = "zlib.adler32_combine(adler1: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.adler32_combine(adler1: int); call it with the wrong type.

typeshed contract: adler1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import adler32_combine
try:
    adler32_combine("not_an_int", 0, 0)  # adler1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zlib/compress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_compress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "compress__data_as_ReadableBuffer_wrong"
# subject = "zlib.compress(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zlib import compress
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

/// Ported from `tests/cpython/type/std-libs/zlib/compressobj__level_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_compressobj__level_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "compressobj__level_as_int_wrong"
# subject = "zlib.compressobj(level: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.compressobj(level: int); call it with the wrong type.

typeshed contract: level is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import compressobj
try:
    compressobj("not_an_int")  # level: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zlib/crc32__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_crc32__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "crc32__data_as_ReadableBuffer_wrong"
# subject = "zlib.crc32(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.crc32(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zlib import crc32
try:
    crc32(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zlib/crc32_combine__crc1_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_crc32_combine__crc1_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "crc32_combine__crc1_as_int_wrong"
# subject = "zlib.crc32_combine(crc1: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.crc32_combine(crc1: int); call it with the wrong type.

typeshed contract: crc1 is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import crc32_combine
try:
    crc32_combine("not_an_int", 0, 0)  # crc1: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zlib/decompress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_decompress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "decompress__data_as_ReadableBuffer_wrong"
# subject = "zlib.decompress(data: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.decompress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zlib import decompress
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

/// Ported from `tests/cpython/type/std-libs/zlib/decompressobj__wbits_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_zlib_decompressobj__wbits_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zlib"
# dimension = "type"
# case = "decompressobj__wbits_as_int_wrong"
# subject = "zlib.decompressobj(wbits: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zlib.decompressobj(wbits: int); call it with the wrong type.

typeshed contract: wbits is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from zlib import decompressobj
try:
    decompressobj("not_an_int")  # wbits: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
