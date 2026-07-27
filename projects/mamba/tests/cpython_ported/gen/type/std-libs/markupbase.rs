use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_markupbase/ParserBase__parse_comment__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__markupbase_ParserBase__parse_comment__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_markupbase"
# dimension = "type"
# case = "ParserBase__parse_comment__i_as_int_wrong"
# subject = "_markupbase.ParserBase.parse_comment(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_markupbase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _markupbase.ParserBase.parse_comment(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _markupbase import ParserBase
obj = object.__new__(ParserBase)
try:
    obj.parse_comment("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_markupbase/ParserBase__parse_declaration__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__markupbase_ParserBase__parse_declaration__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_markupbase"
# dimension = "type"
# case = "ParserBase__parse_declaration__i_as_int_wrong"
# subject = "_markupbase.ParserBase.parse_declaration(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_markupbase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _markupbase.ParserBase.parse_declaration(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _markupbase import ParserBase
obj = object.__new__(ParserBase)
try:
    obj.parse_declaration("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_markupbase/ParserBase__parse_marked_section__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__markupbase_ParserBase__parse_marked_section__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_markupbase"
# dimension = "type"
# case = "ParserBase__parse_marked_section__i_as_int_wrong"
# subject = "_markupbase.ParserBase.parse_marked_section(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_markupbase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _markupbase.ParserBase.parse_marked_section(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _markupbase import ParserBase
obj = object.__new__(ParserBase)
try:
    obj.parse_marked_section("not_an_int")  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_markupbase/ParserBase__unknown_decl__data_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__markupbase_ParserBase__unknown_decl__data_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_markupbase"
# dimension = "type"
# case = "ParserBase__unknown_decl__data_as_str_wrong"
# subject = "_markupbase.ParserBase.unknown_decl(data: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_markupbase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _markupbase.ParserBase.unknown_decl(data: str); call it with the wrong type.

typeshed contract: data is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _markupbase import ParserBase
obj = object.__new__(ParserBase)
try:
    obj.unknown_decl(12345)  # data: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_markupbase/ParserBase__updatepos__i_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__markupbase_ParserBase__updatepos__i_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_markupbase"
# dimension = "type"
# case = "ParserBase__updatepos__i_as_int_wrong"
# subject = "_markupbase.ParserBase.updatepos(i: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_markupbase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _markupbase.ParserBase.updatepos(i: int); call it with the wrong type.

typeshed contract: i is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _markupbase import ParserBase
obj = object.__new__(ParserBase)
try:
    obj.updatepos("not_an_int", 0)  # i: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
