use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/distutils_spawn/find_executable__executable_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_spawn_find_executable__executable_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_spawn"
# dimension = "type"
# case = "find_executable__executable_as_str_wrong"
# subject = "distutils.spawn.find_executable(executable: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/spawn.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.spawn.find_executable(executable: str); call it with the wrong type.

typeshed contract: executable is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.spawn import find_executable
try:
    find_executable(12345)  # executable: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/distutils_spawn/spawn__cmd_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_distutils_spawn_spawn__cmd_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_spawn"
# dimension = "type"
# case = "spawn__cmd_as_Iterable_wrong"
# subject = "distutils.spawn.spawn(cmd: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/spawn.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.spawn.spawn(cmd: Iterable); call it with the wrong type.

typeshed contract: cmd is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.spawn import spawn
try:
    spawn(_W())  # cmd: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
