use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/tty/cfmakecbreak__mode_as__Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_tty_cfmakecbreak__mode_as__Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "cfmakecbreak__mode_as__Attr_wrong"
# subject = "tty.cfmakecbreak(mode: _Attr)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.cfmakecbreak(mode: _Attr); call it with the wrong type.

typeshed contract: mode is _Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import cfmakecbreak
try:
    cfmakecbreak(_W())  # mode: _Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tty/cfmakeraw__mode_as__Attr_wrong.py`.
#[test]
fn test_gen_type_std_libs_tty_cfmakeraw__mode_as__Attr_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "cfmakeraw__mode_as__Attr_wrong"
# subject = "tty.cfmakeraw(mode: _Attr)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.cfmakeraw(mode: _Attr); call it with the wrong type.

typeshed contract: mode is _Attr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import cfmakeraw
try:
    cfmakeraw(_W())  # mode: _Attr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tty/setcbreak__fd_as__FD_wrong.py`.
#[test]
fn test_gen_type_std_libs_tty_setcbreak__fd_as__FD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "setcbreak__fd_as__FD_wrong"
# subject = "tty.setcbreak(fd: _FD)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.setcbreak(fd: _FD); call it with the wrong type.

typeshed contract: fd is _FD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import setcbreak
try:
    setcbreak(_W())  # fd: _FD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/tty/setraw__fd_as__FD_wrong.py`.
#[test]
fn test_gen_type_std_libs_tty_setraw__fd_as__FD_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "setraw__fd_as__FD_wrong"
# subject = "tty.setraw(fd: _FD)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.setraw(fd: _FD); call it with the wrong type.

typeshed contract: fd is _FD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import setraw
try:
    setraw(_W())  # fd: _FD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
