use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/ParseError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_ParseError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "ParseError__init__msg_as_str_wrong"
# subject = "lib2to3.pgen2.parse.ParseError.__init__(msg: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.ParseError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.parse import ParseError
try:
    ParseError(12345, 0, None, None)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__addtoken__type_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__addtoken__type_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__addtoken__type_as_int_wrong"
# subject = "lib2to3.pgen2.parse.Parser.addtoken(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.addtoken(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.parse import Parser
obj = object.__new__(Parser)
try:
    obj.addtoken("not_an_int", None, None)  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__classify__type_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__classify__type_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__classify__type_as_int_wrong"
# subject = "lib2to3.pgen2.parse.Parser.classify(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.classify(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.parse import Parser
obj = object.__new__(Parser)
try:
    obj.classify("not_an_int", None, None)  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__init__grammar_as_Grammar_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__init__grammar_as_Grammar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__init__grammar_as_Grammar_wrong"
# subject = "lib2to3.pgen2.parse.Parser.__init__(grammar: Grammar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.__init__(grammar: Grammar); call it with the wrong type.

typeshed contract: grammar is Grammar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.parse import Parser
try:
    Parser(_W())  # grammar: Grammar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__push__type_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__push__type_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__push__type_as_int_wrong"
# subject = "lib2to3.pgen2.parse.Parser.push(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.push(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.parse import Parser
obj = object.__new__(Parser)
try:
    obj.push("not_an_int", None, 0, None)  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__setup__start_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__setup__start_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__setup__start_as_typed_wrong"
# subject = "lib2to3.pgen2.parse.Parser.setup(start: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.setup(start: typed); call it with the wrong type.

typeshed contract: start is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from lib2to3.pgen2.parse import Parser
obj = object.__new__(Parser)
try:
    obj.setup(_W())  # start: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/lib2to3_pgen2_parse/Parser__shift__type_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_lib2to3_pgen2_parse_Parser__shift__type_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "lib2to3_pgen2_parse"
# dimension = "type"
# case = "Parser__shift__type_as_int_wrong"
# subject = "lib2to3.pgen2.parse.Parser.shift(type: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/lib2to3/pgen2/parse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: lib2to3.pgen2.parse.Parser.shift(type: int); call it with the wrong type.

typeshed contract: type is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from lib2to3.pgen2.parse import Parser
obj = object.__new__(Parser)
try:
    obj.shift("not_an_int", None, 0, None)  # type: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
