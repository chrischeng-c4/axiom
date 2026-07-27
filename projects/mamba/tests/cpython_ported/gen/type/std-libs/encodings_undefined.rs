use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_undefined/Codec__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_undefined_Codec__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_undefined"
# dimension = "type"
# case = "Codec__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.undefined.Codec.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/undefined.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.undefined.Codec.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.undefined import Codec
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

/// Ported from `tests/cpython/type/std-libs/encodings_undefined/Codec__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_undefined_Codec__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_undefined"
# dimension = "type"
# case = "Codec__encode__input_as_str_wrong"
# subject = "encodings.undefined.Codec.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/undefined.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.undefined.Codec.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.undefined import Codec
obj = object.__new__(Codec)
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

/// Ported from `tests/cpython/type/std-libs/encodings_undefined/IncrementalDecoder__decode__input_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_undefined_IncrementalDecoder__decode__input_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_undefined"
# dimension = "type"
# case = "IncrementalDecoder__decode__input_as_ReadableBuffer_wrong"
# subject = "encodings.undefined.IncrementalDecoder.decode(input: ReadableBuffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/undefined.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.undefined.IncrementalDecoder.decode(input: ReadableBuffer); call it with the wrong type.

typeshed contract: input is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.undefined import IncrementalDecoder
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

/// Ported from `tests/cpython/type/std-libs/encodings_undefined/IncrementalEncoder__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_undefined_IncrementalEncoder__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_undefined"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_str_wrong"
# subject = "encodings.undefined.IncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/undefined.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.undefined.IncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.undefined import IncrementalEncoder
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
