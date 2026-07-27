use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_utf_16_le/IncrementalEncoder__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_utf_16_le_IncrementalEncoder__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_utf_16_le"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_str_wrong"
# subject = "encodings.utf_16_le.IncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/utf_16_le.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.utf_16_le.IncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.utf_16_le import IncrementalEncoder
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

/// Ported from `tests/cpython/type/std-libs/encodings_utf_16_le/StreamReader__decode__data_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_utf_16_le_StreamReader__decode__data_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_utf_16_le"
# dimension = "type"
# case = "StreamReader__decode__data_as_ReadableBuffer_wrong"
# subject = "encodings.utf_16_le.StreamReader.decode(data: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/utf_16_le.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.utf_16_le.StreamReader.decode(data: ReadableBuffer); call it with the wrong type.

typeshed contract: data is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.utf_16_le import StreamReader
try:
    StreamReader.decode(_W())  # data: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_utf_16_le/StreamWriter__encode__str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_utf_16_le_StreamWriter__encode__str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_utf_16_le"
# dimension = "type"
# case = "StreamWriter__encode__str_as_str_wrong"
# subject = "encodings.utf_16_le.StreamWriter.encode(str: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/utf_16_le.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.utf_16_le.StreamWriter.encode(str: str); call it with the wrong type.

typeshed contract: str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.utf_16_le import StreamWriter
try:
    StreamWriter.encode(12345)  # str: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_utf_16_le/decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_utf_16_le_decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_utf_16_le"
# dimension = "type"
# case = "decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.utf_16_le.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/utf_16_le.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.utf_16_le.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.utf_16_le import decode
try:
    decode(_W())  # input: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
