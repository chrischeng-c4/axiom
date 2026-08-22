use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/codeop/CommandCompiler____call____source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_codeop_CommandCompiler____call____source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "type"
# case = "CommandCompiler____call____source_as_str_wrong"
# subject = "codeop.CommandCompiler.__call__(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codeop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codeop.CommandCompiler.__call__(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codeop import CommandCompiler
obj = object.__new__(CommandCompiler)
try:
    obj.__call__(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/codeop/Compile____call____source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_codeop_Compile____call____source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "type"
# case = "Compile____call____source_as_str_wrong"
# subject = "codeop.Compile.__call__(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codeop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codeop.Compile.__call__(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codeop import Compile
obj = object.__new__(Compile)
try:
    obj.__call__(12345, "", "")  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/codeop/compile_command__source_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_codeop_compile_command__source_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codeop"
# dimension = "type"
# case = "compile_command__source_as_str_wrong"
# subject = "codeop.compile_command(source: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/codeop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: codeop.compile_command(source: str); call it with the wrong type.

typeshed contract: source is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from codeop import compile_command
try:
    compile_command(12345)  # source: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
