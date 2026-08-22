use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_raw_unicode_escape/Codec__decode__data_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_raw_unicode_escape_Codec__decode__data_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_raw_unicode_escape"
# dimension = "type"
# case = "Codec__decode__data_as_typed_wrong"
# subject = "encodings.raw_unicode_escape.Codec.decode(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/raw_unicode_escape.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.raw_unicode_escape.Codec.decode(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.raw_unicode_escape import Codec
try:
    Codec.decode(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_raw_unicode_escape/Codec__encode__str_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_raw_unicode_escape_Codec__encode__str_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_raw_unicode_escape"
# dimension = "type"
# case = "Codec__encode__str_as_str_wrong"
# subject = "encodings.raw_unicode_escape.Codec.encode(str: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/raw_unicode_escape.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.raw_unicode_escape.Codec.encode(str: str); call it with the wrong type.

typeshed contract: str is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.raw_unicode_escape import Codec
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

/// Ported from `tests/cpython/type/std-libs/encodings_raw_unicode_escape/IncrementalEncoder__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_raw_unicode_escape_IncrementalEncoder__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_raw_unicode_escape"
# dimension = "type"
# case = "IncrementalEncoder__encode__input_as_str_wrong"
# subject = "encodings.raw_unicode_escape.IncrementalEncoder.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/raw_unicode_escape.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.raw_unicode_escape.IncrementalEncoder.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.raw_unicode_escape import IncrementalEncoder
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

/// Ported from `tests/cpython/type/std-libs/encodings_raw_unicode_escape/StreamReader__decode__input_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_raw_unicode_escape_StreamReader__decode__input_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_raw_unicode_escape"
# dimension = "type"
# case = "StreamReader__decode__input_as_typed_wrong"
# subject = "encodings.raw_unicode_escape.StreamReader.decode(input: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/raw_unicode_escape.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.raw_unicode_escape.StreamReader.decode(input: typed); call it with the wrong type.

typeshed contract: input is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.raw_unicode_escape import StreamReader
obj = object.__new__(StreamReader)
try:
    obj.decode(_W())  # input: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
