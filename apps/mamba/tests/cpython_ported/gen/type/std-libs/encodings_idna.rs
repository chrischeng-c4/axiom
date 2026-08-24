use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/encodings_idna/Codec__decode__input_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_idna_Codec__decode__input_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "Codec__decode__input_as_typed_wrong"
# subject = "encodings.idna.Codec.decode(input: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.Codec.decode(input: typed); call it with the wrong type.

typeshed contract: input is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.idna import Codec
obj = object.__new__(Codec)
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

/// Ported from `tests/cpython/type/std-libs/encodings_idna/Codec__encode__input_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_idna_Codec__encode__input_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "Codec__encode__input_as_str_wrong"
# subject = "encodings.idna.Codec.encode(input: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.Codec.encode(input: str); call it with the wrong type.

typeshed contract: input is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.idna import Codec
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

/// Ported from `tests/cpython/type/std-libs/encodings_idna/ToASCII__label_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_idna_ToASCII__label_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "ToASCII__label_as_str_wrong"
# subject = "encodings.idna.ToASCII(label: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.ToASCII(label: str); call it with the wrong type.

typeshed contract: label is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.idna import ToASCII
try:
    ToASCII(12345)  # label: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_idna/ToUnicode__label_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_idna_ToUnicode__label_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "ToUnicode__label_as_typed_wrong"
# subject = "encodings.idna.ToUnicode(label: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.ToUnicode(label: typed); call it with the wrong type.

typeshed contract: label is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from encodings.idna import ToUnicode
try:
    ToUnicode(_W())  # label: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/encodings_idna/nameprep__label_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_encodings_idna_nameprep__label_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "encodings_idna"
# dimension = "type"
# case = "nameprep__label_as_str_wrong"
# subject = "encodings.idna.nameprep(label: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/encodings/idna.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: encodings.idna.nameprep(label: str); call it with the wrong type.

typeshed contract: label is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from encodings.idna import nameprep
try:
    nameprep(12345)  # label: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
