use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/enum/EnumMeta____call____names_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_EnumMeta____call____names_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "EnumMeta____call____names_as_typed_wrong"
# subject = "enum.EnumMeta.__call__(names: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.EnumMeta.__call__(names: typed); call it with the wrong type.

typeshed contract: names is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import EnumMeta
obj = object.__new__(EnumMeta)
try:
    obj.__call__(None, _W())  # names: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/EnumMeta____call____value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_EnumMeta____call____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "EnumMeta____call____value_as_str_wrong"
# subject = "enum.EnumMeta.__call__(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.EnumMeta.__call__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import EnumMeta
obj = object.__new__(EnumMeta)
try:
    obj.__call__(12345, None)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/EnumMeta____getitem____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_EnumMeta____getitem____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "EnumMeta____getitem____name_as_str_wrong"
# subject = "enum.EnumMeta.__getitem__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.EnumMeta.__getitem__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import EnumMeta
obj = object.__new__(EnumMeta)
try:
    obj.__getitem__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/EnumMeta____new____cls_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_EnumMeta____new____cls_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "EnumMeta____new____cls_as_str_wrong"
# subject = "enum.EnumMeta.__new__(cls: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.EnumMeta.__new__(cls: str); call it with the wrong type.

typeshed contract: cls is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import EnumMeta
obj = object.__new__(EnumMeta)
try:
    obj.__new__(12345, None, None)  # cls: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/EnumMeta____prepare____cls_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_EnumMeta____prepare____cls_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "EnumMeta____prepare____cls_as_str_wrong"
# subject = "enum.EnumMeta.__prepare__(cls: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.EnumMeta.__prepare__(cls: str); call it with the wrong type.

typeshed contract: cls is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import EnumMeta
try:
    EnumMeta.__prepare__(12345, None)  # cls: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/Enum____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_Enum____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Enum____format____format_spec_as_str_wrong"
# subject = "enum.Enum.__format__(format_spec: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Enum.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import Enum
obj = object.__new__(Enum)
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/Flag____and____other_as_Self_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_Flag____and____other_as_Self_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Flag____and____other_as_Self_wrong"
# subject = "enum.Flag.__and__(other: Self)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Flag.__and__(other: Self); call it with the wrong type.

typeshed contract: other is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import Flag
obj = object.__new__(Flag)
try:
    obj.__and__(_W())  # other: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/Flag____contains____other_as_Self_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_Flag____contains____other_as_Self_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Flag____contains____other_as_Self_wrong"
# subject = "enum.Flag.__contains__(other: Self)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Flag.__contains__(other: Self); call it with the wrong type.

typeshed contract: other is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import Flag
obj = object.__new__(Flag)
try:
    obj.__contains__(_W())  # other: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/Flag____or____other_as_Self_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_Flag____or____other_as_Self_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Flag____or____other_as_Self_wrong"
# subject = "enum.Flag.__or__(other: Self)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Flag.__or__(other: Self); call it with the wrong type.

typeshed contract: other is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import Flag
obj = object.__new__(Flag)
try:
    obj.__or__(_W())  # other: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/Flag____xor____other_as_Self_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_Flag____xor____other_as_Self_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "Flag____xor____other_as_Self_wrong"
# subject = "enum.Flag.__xor__(other: Self)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.Flag.__xor__(other: Self); call it with the wrong type.

typeshed contract: other is Self. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import Flag
obj = object.__new__(Flag)
try:
    obj.__xor__(_W())  # other: Self <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/IntEnum____new____value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_IntEnum____new____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntEnum____new____value_as_int_wrong"
# subject = "enum.IntEnum.__new__(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntEnum.__new__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntEnum
obj = object.__new__(IntEnum)
try:
    obj.__new__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/IntFlag____and____other_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_IntFlag____and____other_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntFlag____and____other_as_int_wrong"
# subject = "enum.IntFlag.__and__(other: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntFlag.__and__(other: int); call it with the wrong type.

typeshed contract: other is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntFlag
obj = object.__new__(IntFlag)
try:
    obj.__and__("not_an_int")  # other: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/IntFlag____new____value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_IntFlag____new____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntFlag____new____value_as_int_wrong"
# subject = "enum.IntFlag.__new__(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntFlag.__new__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntFlag
obj = object.__new__(IntFlag)
try:
    obj.__new__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/IntFlag____or____other_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_IntFlag____or____other_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntFlag____or____other_as_int_wrong"
# subject = "enum.IntFlag.__or__(other: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntFlag.__or__(other: int); call it with the wrong type.

typeshed contract: other is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntFlag
obj = object.__new__(IntFlag)
try:
    obj.__or__("not_an_int")  # other: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/IntFlag____xor____other_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_IntFlag____xor____other_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "IntFlag____xor____other_as_int_wrong"
# subject = "enum.IntFlag.__xor__(other: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.IntFlag.__xor__(other: int); call it with the wrong type.

typeshed contract: other is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import IntFlag
obj = object.__new__(IntFlag)
try:
    obj.__xor__("not_an_int")  # other: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/StrEnum____new____value_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_StrEnum____new____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "StrEnum____new____value_as_str_wrong"
# subject = "enum.StrEnum.__new__(value: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.StrEnum.__new__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import StrEnum
obj = object.__new__(StrEnum)
try:
    obj.__new__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/bin__num_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_bin__num_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "bin__num_as_SupportsIndex_wrong"
# subject = "enum.bin(num: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.bin(num: SupportsIndex); call it with the wrong type.

typeshed contract: num is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import bin
try:
    bin(_W())  # num: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/global_enum__cls_as__EnumerationT_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_global_enum__cls_as__EnumerationT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "global_enum__cls_as__EnumerationT_wrong"
# subject = "enum.global_enum(cls: _EnumerationT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.global_enum(cls: _EnumerationT); call it with the wrong type.

typeshed contract: cls is _EnumerationT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import global_enum
try:
    global_enum(_W())  # cls: _EnumerationT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/global_enum_repr__self_as_Enum_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_global_enum_repr__self_as_Enum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "global_enum_repr__self_as_Enum_wrong"
# subject = "enum.global_enum_repr(self: Enum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.global_enum_repr(self: Enum); call it with the wrong type.

typeshed contract: self is Enum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import global_enum_repr
try:
    global_enum_repr(_W())  # self: Enum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/global_flag_repr__self_as_Flag_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_global_flag_repr__self_as_Flag_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "global_flag_repr__self_as_Flag_wrong"
# subject = "enum.global_flag_repr(self: Flag)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.global_flag_repr(self: Flag); call it with the wrong type.

typeshed contract: self is Flag. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import global_flag_repr
try:
    global_flag_repr(_W())  # self: Flag <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/global_str__self_as_Enum_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_global_str__self_as_Enum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "global_str__self_as_Enum_wrong"
# subject = "enum.global_str(self: Enum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.global_str(self: Enum); call it with the wrong type.

typeshed contract: self is Enum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import global_str
try:
    global_str(_W())  # self: Enum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/pickle_by_global_name__self_as_Enum_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_pickle_by_global_name__self_as_Enum_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "pickle_by_global_name__self_as_Enum_wrong"
# subject = "enum.pickle_by_global_name(self: Enum)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.pickle_by_global_name(self: Enum); call it with the wrong type.

typeshed contract: self is Enum. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import pickle_by_global_name
try:
    pickle_by_global_name(_W(), 0)  # self: Enum <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/show_flag_values__value_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_show_flag_values__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "show_flag_values__value_as_int_wrong"
# subject = "enum.show_flag_values(value: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.show_flag_values(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from enum import show_flag_values
try:
    show_flag_values("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/enum/unique__enumeration_as__EnumerationT_wrong.py`.
#[test]
fn test_gen_type_std_libs_enum_unique__enumeration_as__EnumerationT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "type"
# case = "unique__enumeration_as__EnumerationT_wrong"
# subject = "enum.unique(enumeration: _EnumerationT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/enum.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: enum.unique(enumeration: _EnumerationT); call it with the wrong type.

typeshed contract: enumeration is _EnumerationT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from enum import unique
try:
    unique(_W())  # enumeration: _EnumerationT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
