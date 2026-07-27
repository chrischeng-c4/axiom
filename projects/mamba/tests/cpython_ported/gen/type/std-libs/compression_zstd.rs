use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/compression_zstd/FrameInfo__init__decompressed_size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_FrameInfo__init__decompressed_size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "FrameInfo__init__decompressed_size_as_int_wrong"
# subject = "compression.zstd.FrameInfo.__init__(decompressed_size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.FrameInfo.__init__(decompressed_size: int); call it with the wrong type.

typeshed contract: decompressed_size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compression.zstd import FrameInfo
try:
    FrameInfo("not_an_int", 0)  # decompressed_size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression_zstd/compress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_compress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "compress__data_as_ReadableBuffer_wrong"
# subject = "compression.zstd.compress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.compress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import compress
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd/decompress__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_decompress__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "decompress__data_as_ReadableBuffer_wrong"
# subject = "compression.zstd.decompress(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.decompress(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import decompress
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd/finalize_dict__zstd_dict_as_ZstdDict_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_finalize_dict__zstd_dict_as_ZstdDict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "finalize_dict__zstd_dict_as_ZstdDict_wrong"
# subject = "compression.zstd.finalize_dict(zstd_dict: ZstdDict)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.finalize_dict(zstd_dict: ZstdDict); call it with the wrong type.

typeshed contract: zstd_dict is ZstdDict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import finalize_dict
try:
    finalize_dict(_W(), None, 0, 0)  # zstd_dict: ZstdDict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compression_zstd/get_frame_info__frame_buffer_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_get_frame_info__frame_buffer_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "get_frame_info__frame_buffer_as_ReadableBuffer_wrong"
# subject = "compression.zstd.get_frame_info(frame_buffer: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.get_frame_info(frame_buffer: ReadableBuffer); call it with the wrong type.

typeshed contract: frame_buffer is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import get_frame_info
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

/// Ported from `tests/cpython/type/std-libs/compression_zstd/train_dict__samples_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_compression_zstd_train_dict__samples_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compression_zstd"
# dimension = "type"
# case = "train_dict__samples_as_Iterable_wrong"
# subject = "compression.zstd.train_dict(samples: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compression/zstd.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compression.zstd.train_dict(samples: Iterable); call it with the wrong type.

typeshed contract: samples is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compression.zstd import train_dict
try:
    train_dict(_W(), 0)  # samples: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
