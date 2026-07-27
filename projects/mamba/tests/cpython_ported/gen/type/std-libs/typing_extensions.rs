use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/typing_extensions/Buffer____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_Buffer____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "Buffer____buffer____flags_as_int_wrong"
# subject = "typing_extensions.Buffer.__buffer__(flags: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.Buffer.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import Buffer
obj = object.__new__(Buffer)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/Doc__init__documentation_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_Doc__init__documentation_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "Doc__init__documentation_as_str_wrong"
# subject = "typing_extensions.Doc.__init__(documentation: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.Doc.__init__(documentation: str); call it with the wrong type.

typeshed contract: documentation is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import Doc
try:
    Doc(12345)  # documentation: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/IntVar__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_IntVar__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "IntVar__name_as_str_wrong"
# subject = "typing_extensions.IntVar(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.IntVar(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import IntVar
try:
    IntVar(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/NewType__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_NewType__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "NewType__init__name_as_str_wrong"
# subject = "typing_extensions.NewType.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.NewType.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import NewType
try:
    NewType(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/ParamSpec__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_ParamSpec__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "ParamSpec__init__name_as_str_wrong"
# subject = "typing_extensions.ParamSpec.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.ParamSpec.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import ParamSpec
try:
    ParamSpec(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/Reader__read__size_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_Reader__read__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "Reader__read__size_as_int_wrong"
# subject = "typing_extensions.Reader.read(size: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.Reader.read(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import Reader
obj = object.__new__(Reader)
try:
    obj.read("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/Sentinel__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_Sentinel__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "Sentinel__init__name_as_str_wrong"
# subject = "typing_extensions.Sentinel.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.Sentinel.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import Sentinel
try:
    Sentinel(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/TypeAliasType__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_TypeAliasType__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "TypeAliasType__init__name_as_str_wrong"
# subject = "typing_extensions.TypeAliasType.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.TypeAliasType.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import TypeAliasType
try:
    TypeAliasType(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/TypeVarTuple__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_TypeVarTuple__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "TypeVarTuple__init__name_as_str_wrong"
# subject = "typing_extensions.TypeVarTuple.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.TypeVarTuple.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from typing_extensions import TypeVarTuple
try:
    TypeVarTuple(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/typing_extensions/deprecated__init__message_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_typing_extensions_deprecated__init__message_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing_extensions"
# dimension = "type"
# case = "deprecated__init__message_as_LiteralString_wrong"
# subject = "typing_extensions.deprecated.__init__(message: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing_extensions.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: typing_extensions.deprecated.__init__(message: LiteralString); call it with the wrong type.

typeshed contract: message is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing_extensions import deprecated
try:
    deprecated(_W())  # message: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
