use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/string/Formatter__convert_field__conversion_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__convert_field__conversion_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__convert_field__conversion_as_typed_wrong"
# subject = "string.Formatter.convert_field(conversion: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.convert_field(conversion: typed); call it with the wrong type.

typeshed contract: conversion is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.convert_field(None, _W())  # conversion: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Formatter__format_field__format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__format_field__format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__format_field__format_spec_as_str_wrong"
# subject = "string.Formatter.format_field(format_spec: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.format_field(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.format_field(None, 12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Formatter__get_field__field_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__get_field__field_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__get_field__field_name_as_str_wrong"
# subject = "string.Formatter.get_field(field_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.get_field(field_name: str); call it with the wrong type.

typeshed contract: field_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.get_field(12345, None, None)  # field_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Formatter__get_value__key_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__get_value__key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__get_value__key_as_typed_wrong"
# subject = "string.Formatter.get_value(key: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.get_value(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.get_value(_W(), None, None)  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Formatter__vformat__format_string_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__vformat__format_string_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__vformat__format_string_as_LiteralString_wrong"
# subject = "string.Formatter.vformat(format_string: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.vformat(format_string: LiteralString); call it with the wrong type.

typeshed contract: format_string is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.vformat(_W(), None, None)  # format_string: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Formatter__vformat__format_string_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Formatter__vformat__format_string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Formatter__vformat__format_string_as_str_wrong"
# subject = "string.Formatter.vformat(format_string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Formatter.vformat(format_string: str); call it with the wrong type.

typeshed contract: format_string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Formatter
obj = object.__new__(Formatter)
try:
    obj.vformat(12345, None, None)  # format_string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/string/Template__init__template_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_string_Template__init__template_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string"
# dimension = "type"
# case = "Template__init__template_as_str_wrong"
# subject = "string.Template.__init__(template: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/string.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: string.Template.__init__(template: str); call it with the wrong type.

typeshed contract: template is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from string import Template
try:
    Template(12345)  # template: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
