use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_contextvars/ContextVar____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__contextvars_ContextVar____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "ContextVar____new____name_as_str_wrong"
# subject = "_contextvars.ContextVar.__new__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.ContextVar.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _contextvars import ContextVar
obj = object.__new__(ContextVar)
try:
    obj.__new__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_contextvars/ContextVar__reset__token_as_Token_wrong.py`.
#[test]
fn test_gen_type_std_libs__contextvars_ContextVar__reset__token_as_Token_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "ContextVar__reset__token_as_Token_wrong"
# subject = "_contextvars.ContextVar.reset(token: Token)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.ContextVar.reset(token: Token); call it with the wrong type.

typeshed contract: token is Token. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import ContextVar
obj = ContextVar("reset_token_wall")
try:
    obj.reset(_W())  # token: Token <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_contextvars/Context____getitem____key_as_ContextVar_wrong.py`.
#[test]
fn test_gen_type_std_libs__contextvars_Context____getitem____key_as_ContextVar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "Context____getitem____key_as_ContextVar_wrong"
# subject = "_contextvars.Context.__getitem__(key: ContextVar)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.Context.__getitem__(key: ContextVar); call it with the wrong type.

typeshed contract: key is ContextVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import Context
obj = Context()
try:
    obj.__getitem__(_W())  # key: ContextVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_contextvars/Context__get__key_as_ContextVar_wrong.py`.
#[test]
fn test_gen_type_std_libs__contextvars_Context__get__key_as_ContextVar_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "Context__get__key_as_ContextVar_wrong"
# subject = "_contextvars.Context.get(key: ContextVar)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.Context.get(key: ContextVar); call it with the wrong type.

typeshed contract: key is ContextVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import Context
obj = Context()
try:
    obj.get(_W())  # key: ContextVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_contextvars/Context__run__callable_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs__contextvars_Context__run__callable_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "Context__run__callable_as_Callable_wrong"
# subject = "_contextvars.Context.run(callable: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.Context.run(callable: Callable); call it with the wrong type.

typeshed contract: callable is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import Context
obj = Context()
try:
    obj.run(_W())  # callable: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
