use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/pty/slave_open__tty_name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_pty_slave_open__tty_name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "type"
# case = "slave_open__tty_name_as_str_wrong"
# subject = "pty.slave_open(tty_name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pty.slave_open(tty_name: str); call it with the wrong type.

typeshed contract: tty_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from pty import slave_open
try:
    slave_open(12345)  # tty_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/pty/spawn__argv_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_pty_spawn__argv_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pty"
# dimension = "type"
# case = "spawn__argv_as_typed_wrong"
# subject = "pty.spawn(argv: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/pty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: pty.spawn(argv: typed); call it with the wrong type.

typeshed contract: argv is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from pty import spawn
try:
    spawn(_W())  # argv: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
