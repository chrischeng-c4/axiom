use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__flush__mode_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__flush__mode_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__flush__mode_as_typed_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.flush(mode: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.flush(mode: typed); call it with the wrong type.

typeshed contract: mode is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
try:
    obj.flush(_W())  # mode: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__peek__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__peek__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__peek__size_as_int_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.peek(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.peek(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__read1__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__read1__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__read1__size_as_typed_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.read1(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.read1(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
try:
    obj.read1(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__read__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__read__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__read__size_as_typed_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.read(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.read(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__readinto1__b_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__readinto1__b_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__readinto1__b_as_WriteableBuffer_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.readinto1(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.readinto1(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__readinto__b_as_WriteableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__readinto__b_as_WriteableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__readinto__b_as_WriteableBuffer_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.readinto(b: WriteableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.readinto(b: WriteableBuffer); call it with the wrong type.

typeshed contract: b is WriteableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__readline__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__readline__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__readline__size_as_typed_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.readline(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.readline(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__seek__offset_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__seek__offset_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__seek__offset_as_int_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.seek(offset: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.seek(offset: int); call it with the wrong type.

typeshed contract: offset is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd__zstdfile/ZstdFile__write__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd__zstdfile_ZstdFile__write__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd__zstdfile"
# dimension = "type"
# case = "ZstdFile__write__data_as_ReadableBuffer_wrong"
# subject = "compression.zstd._zstdfile.ZstdFile.write(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd/_zstdfile.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd._zstdfile.ZstdFile.write(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd._zstdfile import ZstdFile
obj = object.__new__(ZstdFile)
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
