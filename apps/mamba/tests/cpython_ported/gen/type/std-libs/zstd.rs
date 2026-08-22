use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdCompressor____new____level_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdCompressor____new____level_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdCompressor____new____level_as_typed_wrong"
# subject = "_zstd.ZstdCompressor.__new__(level: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdCompressor.__new__(level: typed); call it with the wrong type.

typeshed contract: level is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdCompressor
obj = object.__new__(ZstdCompressor)
try:
    obj.__new__(_W())  # level: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdCompressor__compress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdCompressor__compress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdCompressor__compress__data_as_ReadableBuffer_wrong"
# subject = "_zstd.ZstdCompressor.compress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdCompressor.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdCompressor
obj = object.__new__(ZstdCompressor)
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

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdCompressor__flush__mode_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdCompressor__flush__mode_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdCompressor__flush__mode_as_typed_wrong"
# subject = "_zstd.ZstdCompressor.flush(mode: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdCompressor.flush(mode: typed); call it with the wrong type.

typeshed contract: mode is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdCompressor
obj = object.__new__(ZstdCompressor)
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

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdCompressor__set_pledged_input_size__size_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdCompressor__set_pledged_input_size__size_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdCompressor__set_pledged_input_size__size_as_typed_wrong"
# subject = "_zstd.ZstdCompressor.set_pledged_input_size(size: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdCompressor.set_pledged_input_size(size: typed); call it with the wrong type.

typeshed contract: size is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdCompressor
obj = object.__new__(ZstdCompressor)
try:
    obj.set_pledged_input_size(_W())  # size: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdDecompressor__decompress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdDecompressor__decompress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdDecompressor__decompress__data_as_ReadableBuffer_wrong"
# subject = "_zstd.ZstdDecompressor.decompress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdDecompressor.decompress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdDecompressor
obj = object.__new__(ZstdDecompressor)
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

/// Ported from `tests/cpython/type/std-libs/_zstd/ZstdDict____new____dict_content_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_ZstdDict____new____dict_content_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "ZstdDict____new____dict_content_as_ReadableBuffer_wrong"
# subject = "_zstd.ZstdDict.__new__(dict_content: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.ZstdDict.__new__(dict_content: ReadableBuffer); call it with the wrong type.

typeshed contract: dict_content is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import ZstdDict
obj = object.__new__(ZstdDict)
try:
    obj.__new__(_W())  # dict_content: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_zstd/get_frame_info__frame_buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_get_frame_info__frame_buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "get_frame_info__frame_buffer_as_ReadableBuffer_wrong"
# subject = "_zstd.get_frame_info(frame_buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.get_frame_info(frame_buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: frame_buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import get_frame_info
try:
    get_frame_info(_W())  # frame_buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_zstd/get_frame_size__frame_buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_get_frame_size__frame_buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "get_frame_size__frame_buffer_as_ReadableBuffer_wrong"
# subject = "_zstd.get_frame_size(frame_buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.get_frame_size(frame_buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: frame_buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import get_frame_size
try:
    get_frame_size(_W())  # frame_buffer: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_zstd/get_param_bounds__parameter_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__zstd_get_param_bounds__parameter_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "get_param_bounds__parameter_as_int_wrong"
# subject = "_zstd.get_param_bounds(parameter: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _zstd.get_param_bounds(parameter: int); call it with the wrong type.

typeshed contract: parameter is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _zstd import get_param_bounds
try:
    get_param_bounds("not_an_int", True)  # parameter: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
