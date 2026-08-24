use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/concurrent_interpreters/ExecutionFailed__init__excinfo_as_SimpleNamespace_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_interpreters_ExecutionFailed__init__excinfo_as_SimpleNamespace_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "ExecutionFailed__init__excinfo_as_SimpleNamespace_wrong"
# subject = "concurrent.interpreters.ExecutionFailed.__init__(excinfo: SimpleNamespace)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.ExecutionFailed.__init__(excinfo: SimpleNamespace); call it with the wrong type.

typeshed contract: excinfo is SimpleNamespace. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import ExecutionFailed
try:
    ExecutionFailed(_W())  # excinfo: SimpleNamespace <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_interpreters/Interpreter____new____id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_interpreters_Interpreter____new____id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "Interpreter____new____id_as_int_wrong"
# subject = "concurrent.interpreters.Interpreter.__new__(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.Interpreter.__new__(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from concurrent.interpreters import Interpreter
obj = object.__new__(Interpreter)
try:
    obj.__new__("not_an_int")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_interpreters/Interpreter__prepare_main__ns_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_interpreters_Interpreter__prepare_main__ns_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "Interpreter__prepare_main__ns_as_typed_wrong"
# subject = "concurrent.interpreters.Interpreter.prepare_main(ns: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.Interpreter.prepare_main(ns: typed); call it with the wrong type.

typeshed contract: ns is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import Interpreter
obj = object.__new__(Interpreter)
try:
    obj.prepare_main(_W())  # ns: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
