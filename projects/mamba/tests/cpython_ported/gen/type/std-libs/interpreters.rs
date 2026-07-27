use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_interpreters/CrossInterpreterBufferView____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_CrossInterpreterBufferView____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "CrossInterpreterBufferView____buffer____flags_as_int_wrong"
# subject = "_interpreters.CrossInterpreterBufferView.__buffer__(flags: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.CrossInterpreterBufferView.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _interpreters import CrossInterpreterBufferView
obj = object.__new__(CrossInterpreterBufferView)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/call__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_call__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "call__id_as_SupportsIndex_wrong"
# subject = "_interpreters.call(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.call(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import call
try:
    call(_W(), None)  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/capture_exception__exc_as_BaseException_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_capture_exception__exc_as_BaseException_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "capture_exception__exc_as_BaseException_wrong"
# subject = "_interpreters.capture_exception(exc: BaseException)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.capture_exception(exc: BaseException); call it with the wrong type.

typeshed contract: exc is BaseException. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import capture_exception
try:
    capture_exception(_W())  # exc: BaseException <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/capture_exception__exc_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_capture_exception__exc_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "capture_exception__exc_as_typed_wrong"
# subject = "_interpreters.capture_exception(exc: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.capture_exception(exc: typed); call it with the wrong type.

typeshed contract: exc is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import capture_exception
try:
    capture_exception(_W())  # exc: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/create__config_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_create__config_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "create__config_as_typed_wrong"
# subject = "_interpreters.create(config: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.create(config: typed); call it with the wrong type.

typeshed contract: config is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import create
try:
    create(_W())  # config: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/decref__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_decref__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "decref__id_as_SupportsIndex_wrong"
# subject = "_interpreters.decref(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.decref(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import decref
try:
    decref(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/destroy__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_destroy__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "destroy__id_as_SupportsIndex_wrong"
# subject = "_interpreters.destroy(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.destroy(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import destroy
try:
    destroy(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/exec__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_exec__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "exec__id_as_SupportsIndex_wrong"
# subject = "_interpreters.exec(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.exec(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import exec
try:
    exec(_W(), None)  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/get_config__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_get_config__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "get_config__id_as_SupportsIndex_wrong"
# subject = "_interpreters.get_config(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.get_config(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import get_config
try:
    get_config(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/incref__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_incref__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "incref__id_as_SupportsIndex_wrong"
# subject = "_interpreters.incref(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.incref(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import incref
try:
    incref(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/is_running__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_is_running__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "is_running__id_as_SupportsIndex_wrong"
# subject = "_interpreters.is_running(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.is_running(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import is_running
try:
    is_running(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/new_config__name_as__Configs_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_new_config__name_as__Configs_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "new_config__name_as__Configs_wrong"
# subject = "_interpreters.new_config(name: _Configs)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.new_config(name: _Configs); call it with the wrong type.

typeshed contract: name is _Configs. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import new_config
try:
    new_config(_W())  # name: _Configs <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/run_func__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_run_func__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "run_func__id_as_SupportsIndex_wrong"
# subject = "_interpreters.run_func(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.run_func(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import run_func
try:
    run_func(_W(), None)  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/run_string__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_run_string__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "run_string__id_as_SupportsIndex_wrong"
# subject = "_interpreters.run_string(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.run_string(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import run_string
try:
    run_string(_W(), None)  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/set___main___attrs__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_set___main___attrs__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "set___main___attrs__id_as_SupportsIndex_wrong"
# subject = "_interpreters.set___main___attrs(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.set___main___attrs(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import set___main___attrs
try:
    set___main___attrs(_W(), None)  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_interpreters/whence__id_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs__interpreters_whence__id_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "whence__id_as_SupportsIndex_wrong"
# subject = "_interpreters.whence(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.whence(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import whence
try:
    whence(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
