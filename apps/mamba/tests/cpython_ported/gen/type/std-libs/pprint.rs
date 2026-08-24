use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pprint/PrettyPrinter__format__context_as_dict_wrong.py`.
#[test]
fn test_gen_type_std_libs_pprint_PrettyPrinter__format__context_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "PrettyPrinter__format__context_as_dict_wrong"
# subject = "pprint.PrettyPrinter.format(context: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.PrettyPrinter.format(context: dict); call it with the wrong type.

typeshed contract: context is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pprint import PrettyPrinter
obj = object.__new__(PrettyPrinter)
try:
    obj.format(None, 12345, 0, 0)  # context: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pprint/PrettyPrinter__init__indent_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pprint_PrettyPrinter__init__indent_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "PrettyPrinter__init__indent_as_int_wrong"
# subject = "pprint.PrettyPrinter.__init__(indent: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.PrettyPrinter.__init__(indent: int); call it with the wrong type.

typeshed contract: indent is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pprint import PrettyPrinter
try:
    PrettyPrinter("not_an_int")  # indent: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pprint/pformat__indent_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_pprint_pformat__indent_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "pformat__indent_as_int_wrong"
# subject = "pprint.pformat(indent: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.pformat(indent: int); call it with the wrong type.

typeshed contract: indent is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pprint import pformat
try:
    pformat(None, "not_an_int")  # indent: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pprint/pp__stream_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pprint_pp__stream_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "pp__stream_as_typed_wrong"
# subject = "pprint.pp(stream: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.pp(stream: typed); call it with the wrong type.

typeshed contract: stream is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pprint import pp
try:
    pp(None, _W())  # stream: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pprint/pprint__stream_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pprint_pprint__stream_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "type"
# case = "pprint__stream_as_typed_wrong"
# subject = "pprint.pprint(stream: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pprint.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pprint.pprint(stream: typed); call it with the wrong type.

typeshed contract: stream is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pprint import pprint
try:
    pprint(None, _W())  # stream: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
