use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/contextlib/AbstractAsyncContextManager____aexit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_AbstractAsyncContextManager____aexit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "AbstractAsyncContextManager____aexit____exc_type_as_typed_wrong"
# subject = "contextlib.AbstractAsyncContextManager.__aexit__(exc_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.AbstractAsyncContextManager.__aexit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import AbstractAsyncContextManager
obj = object.__new__(AbstractAsyncContextManager)
try:
    obj.__aexit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/AbstractContextManager____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_AbstractContextManager____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "AbstractContextManager____exit____exc_type_as_typed_wrong"
# subject = "contextlib.AbstractContextManager.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.AbstractContextManager.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import AbstractContextManager
obj = object.__new__(AbstractContextManager)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/aclosing__init__thing_as__SupportsAcloseT_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_aclosing__init__thing_as__SupportsAcloseT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "aclosing__init__thing_as__SupportsAcloseT_wrong"
# subject = "contextlib.aclosing.__init__(thing: _SupportsAcloseT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.aclosing.__init__(thing: _SupportsAcloseT); call it with the wrong type.

typeshed contract: thing is _SupportsAcloseT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import aclosing
try:
    aclosing(_W())  # thing: _SupportsAcloseT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/asynccontextmanager__func_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_asynccontextmanager__func_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "asynccontextmanager__func_as_Callable_wrong"
# subject = "contextlib.asynccontextmanager(func: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.asynccontextmanager(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import asynccontextmanager
try:
    asynccontextmanager(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/chdir__init__path_as__T_fd_or_any_path_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_chdir__init__path_as__T_fd_or_any_path_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "chdir__init__path_as__T_fd_or_any_path_wrong"
# subject = "contextlib.chdir.__init__(path: _T_fd_or_any_path)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.chdir.__init__(path: _T_fd_or_any_path); call it with the wrong type.

typeshed contract: path is _T_fd_or_any_path. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import chdir
try:
    chdir(_W())  # path: _T_fd_or_any_path <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/closing__init__thing_as__SupportsCloseT_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_closing__init__thing_as__SupportsCloseT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "closing__init__thing_as__SupportsCloseT_wrong"
# subject = "contextlib.closing.__init__(thing: _SupportsCloseT)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.closing.__init__(thing: _SupportsCloseT); call it with the wrong type.

typeshed contract: thing is _SupportsCloseT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import closing
try:
    closing(_W())  # thing: _SupportsCloseT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/contextlib/contextmanager__func_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_contextlib_contextmanager__func_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "contextmanager__func_as_Callable_wrong"
# subject = "contextlib.contextmanager(func: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.contextmanager(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import contextmanager
try:
    contextmanager(_W())  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
