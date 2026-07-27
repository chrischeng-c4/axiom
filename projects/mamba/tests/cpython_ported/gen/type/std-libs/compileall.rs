use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/compileall/compile_dir__dir_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_compileall_compile_dir__dir_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "type"
# case = "compile_dir__dir_as_StrPath_wrong"
# subject = "compileall.compile_dir(dir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compileall.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compileall.compile_dir(dir: StrPath); call it with the wrong type.

typeshed contract: dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compileall import compile_dir
try:
    compile_dir(_W())  # dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compileall/compile_file__fullname_as_StrPath_wrong.py`.
#[test]
fn test_gen_type_std_libs_compileall_compile_file__fullname_as_StrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "type"
# case = "compile_file__fullname_as_StrPath_wrong"
# subject = "compileall.compile_file(fullname: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compileall.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compileall.compile_file(fullname: StrPath); call it with the wrong type.

typeshed contract: fullname is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compileall import compile_file
try:
    compile_file(_W())  # fullname: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/compileall/compile_path__skip_curdir_as_bool_wrong.py`.
#[test]
fn test_gen_type_std_libs_compileall_compile_path__skip_curdir_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "type"
# case = "compile_path__skip_curdir_as_bool_wrong"
# subject = "compileall.compile_path(skip_curdir: bool)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compileall.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compileall.compile_path(skip_curdir: bool); call it with the wrong type.

typeshed contract: skip_curdir is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from compileall import compile_path
try:
    compile_path("not_a_bool")  # skip_curdir: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
