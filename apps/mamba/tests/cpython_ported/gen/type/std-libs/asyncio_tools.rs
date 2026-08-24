use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/asyncio_tools/build_async_tree__result_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_tools_build_async_tree__result_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "build_async_tree__result_as_Iterable_wrong"
# subject = "asyncio.tools.build_async_tree(result: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.build_async_tree(result: Iterable); call it with the wrong type.

typeshed contract: result is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import build_async_tree
try:
    build_async_tree(_W())  # result: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_tools/build_task_table__result_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_tools_build_task_table__result_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "build_task_table__result_as_Iterable_wrong"
# subject = "asyncio.tools.build_task_table(result: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.build_task_table(result: Iterable); call it with the wrong type.

typeshed contract: result is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import build_task_table
try:
    build_task_table(_W())  # result: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_tools/display_awaited_by_tasks_table__pid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_tools_display_awaited_by_tasks_table__pid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "display_awaited_by_tasks_table__pid_as_SupportsIndex_wrong"
# subject = "asyncio.tools.display_awaited_by_tasks_table(pid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.display_awaited_by_tasks_table(pid: SupportsIndex); call it with the wrong type.

typeshed contract: pid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import display_awaited_by_tasks_table
try:
    display_awaited_by_tasks_table(_W())  # pid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_tools/display_awaited_by_tasks_tree__pid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_tools_display_awaited_by_tasks_tree__pid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "display_awaited_by_tasks_tree__pid_as_SupportsIndex_wrong"
# subject = "asyncio.tools.display_awaited_by_tasks_tree(pid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.display_awaited_by_tasks_tree(pid: SupportsIndex); call it with the wrong type.

typeshed contract: pid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import display_awaited_by_tasks_tree
try:
    display_awaited_by_tasks_tree(_W())  # pid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/asyncio_tools/get_all_awaited_by__pid_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_std_libs_asyncio_tools_get_all_awaited_by__pid_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "get_all_awaited_by__pid_as_SupportsIndex_wrong"
# subject = "asyncio.tools.get_all_awaited_by(pid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.get_all_awaited_by(pid: SupportsIndex); call it with the wrong type.

typeshed contract: pid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import get_all_awaited_by
try:
    get_all_awaited_by(_W())  # pid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
