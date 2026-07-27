use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/clear_tool_id__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_clear_tool_id__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "clear_tool_id__tool_id_as_int_wrong"
# subject = "sys._monitoring.clear_tool_id(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.clear_tool_id(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import clear_tool_id
try:
    clear_tool_id("not_an_int")  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/free_tool_id__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_free_tool_id__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "free_tool_id__tool_id_as_int_wrong"
# subject = "sys._monitoring.free_tool_id(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.free_tool_id(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import free_tool_id
try:
    free_tool_id("not_an_int")  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/get_events__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_get_events__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "get_events__tool_id_as_int_wrong"
# subject = "sys._monitoring.get_events(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.get_events(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import get_events
try:
    get_events("not_an_int")  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/get_local_events__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_get_local_events__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "get_local_events__tool_id_as_int_wrong"
# subject = "sys._monitoring.get_local_events(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.get_local_events(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import get_local_events
try:
    get_local_events("not_an_int", None)  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/get_tool__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_get_tool__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "get_tool__tool_id_as_int_wrong"
# subject = "sys._monitoring.get_tool(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.get_tool(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import get_tool
try:
    get_tool("not_an_int")  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/register_callback__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_register_callback__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "register_callback__tool_id_as_int_wrong"
# subject = "sys._monitoring.register_callback(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.register_callback(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import register_callback
try:
    register_callback("not_an_int", 0, None)  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/set_events__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_set_events__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "set_events__tool_id_as_int_wrong"
# subject = "sys._monitoring.set_events(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.set_events(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import set_events
try:
    set_events("not_an_int", 0)  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/set_local_events__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_set_local_events__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "set_local_events__tool_id_as_int_wrong"
# subject = "sys._monitoring.set_local_events(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.set_local_events(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import set_local_events
try:
    set_local_events("not_an_int", None, 0)  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/sys__monitoring/use_tool_id__tool_id_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_sys__monitoring_use_tool_id__tool_id_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys__monitoring"
# dimension = "type"
# case = "use_tool_id__tool_id_as_int_wrong"
# subject = "sys._monitoring.use_tool_id(tool_id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sys/_monitoring.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sys._monitoring.use_tool_id(tool_id: int); call it with the wrong type.

typeshed contract: tool_id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sys._monitoring import use_tool_id
try:
    use_tool_id("not_an_int", "")  # tool_id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
