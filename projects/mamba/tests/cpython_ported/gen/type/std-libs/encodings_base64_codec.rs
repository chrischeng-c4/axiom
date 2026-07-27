use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/Codec__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_Codec__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "Codec__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.Codec.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.Codec.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import Codec
obj = object.__new__(Codec)
try:
    obj.decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/Codec__encode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_Codec__encode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "Codec__encode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.Codec.encode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.Codec.encode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import Codec
obj = object.__new__(Codec)
try:
    obj.encode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/IncrementalDecoder__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_IncrementalDecoder__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "IncrementalDecoder__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.IncrementalDecoder.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.IncrementalDecoder.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import IncrementalDecoder
obj = object.__new__(IncrementalDecoder)
try:
    obj.decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/IncrementalEncoder__encode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_IncrementalEncoder__encode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.IncrementalEncoder.encode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.IncrementalEncoder.encode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import IncrementalEncoder
obj = object.__new__(IncrementalEncoder)
try:
    obj.encode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/base64_decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_base64_decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "base64_decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.base64_decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.base64_decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import base64_decode
try:
    base64_decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_base64_codec/base64_encode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_base64_codec_base64_encode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_base64_codec"
# dimension = "type"
# case = "base64_encode__input_as_ReadableBuffer_wrong"
# subject = "encodings.base64_codec.base64_encode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/base64_codec.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.base64_codec.base64_encode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.base64_codec import base64_encode
try:
    base64_encode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
