use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/Codec__decode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_Codec__decode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "Codec__decode__data_as_ReadableBuffer_wrong"
# subject = "encodings.latin_1.Codec.decode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.Codec.decode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.latin_1 import Codec
try:
    Codec.decode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/Codec__encode__str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_Codec__encode__str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "Codec__encode__str_as_str_wrong"
# subject = "encodings.latin_1.Codec.encode(str: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.Codec.encode(str: str); call it with the wrong type.

typeshed contract: str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.latin_1 import Codec
try:
    Codec.encode(12345)  # str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/IncrementalDecoder__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_IncrementalDecoder__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "IncrementalDecoder__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.latin_1.IncrementalDecoder.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.IncrementalDecoder.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.latin_1 import IncrementalDecoder
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

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/IncrementalEncoder__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_IncrementalEncoder__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_str_wrong"
# subject = "encodings.latin_1.IncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.IncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.latin_1 import IncrementalEncoder
obj = object.__new__(IncrementalEncoder)
try:
    obj.encode(12345)  # input: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/StreamConverter__decode__str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_StreamConverter__decode__str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "StreamConverter__decode__str_as_str_wrong"
# subject = "encodings.latin_1.StreamConverter.decode(str: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.StreamConverter.decode(str: str); call it with the wrong type.

typeshed contract: str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.latin_1 import StreamConverter
try:
    StreamConverter.decode(12345)  # str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_latin_1/StreamConverter__encode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_latin_1_StreamConverter__encode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_latin_1"
# dimension = "type"
# case = "StreamConverter__encode__data_as_ReadableBuffer_wrong"
# subject = "encodings.latin_1.StreamConverter.encode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/latin_1.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.latin_1.StreamConverter.encode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.latin_1 import StreamConverter
try:
    StreamConverter.encode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
