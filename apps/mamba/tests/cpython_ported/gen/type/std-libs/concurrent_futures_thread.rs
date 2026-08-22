use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_thread/ThreadPoolExecutor__init__max_workers_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_thread_ThreadPoolExecutor__init__max_workers_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "ThreadPoolExecutor__init__max_workers_as_typed_wrong"
# subject = "concurrent.futures.thread.ThreadPoolExecutor.__init__(max_workers: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.ThreadPoolExecutor.__init__(max_workers: typed); call it with the wrong type.

typeshed contract: max_workers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import ThreadPoolExecutor
try:
    ThreadPoolExecutor(_W())  # max_workers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_thread/ThreadPoolExecutor__prepare_context__initializer_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_thread_ThreadPoolExecutor__prepare_context__initializer_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "ThreadPoolExecutor__prepare_context__initializer_as_Callable_wrong"
# subject = "concurrent.futures.thread.ThreadPoolExecutor.prepare_context(initializer: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.ThreadPoolExecutor.prepare_context(initializer: Callable); call it with the wrong type.

typeshed contract: initializer is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import ThreadPoolExecutor
try:
    ThreadPoolExecutor.prepare_context(_W(), None)  # initializer: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_thread/WorkerContext__init__initializer_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_thread_WorkerContext__init__initializer_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "WorkerContext__init__initializer_as_Callable_wrong"
# subject = "concurrent.futures.thread.WorkerContext.__init__(initializer: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.WorkerContext.__init__(initializer: Callable); call it with the wrong type.

typeshed contract: initializer is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import WorkerContext
try:
    WorkerContext(_W(), None)  # initializer: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_thread/WorkerContext__prepare__initializer_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_thread_WorkerContext__prepare__initializer_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "WorkerContext__prepare__initializer_as_Callable_wrong"
# subject = "concurrent.futures.thread.WorkerContext.prepare(initializer: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.WorkerContext.prepare(initializer: Callable); call it with the wrong type.

typeshed contract: initializer is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import WorkerContext
try:
    WorkerContext.prepare(_W(), None)  # initializer: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_thread/WorkerContext__run__task_as__Task_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_thread_WorkerContext__run__task_as__Task_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "WorkerContext__run__task_as__Task_wrong"
# subject = "concurrent.futures.thread.WorkerContext.run(task: _Task)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.WorkerContext.run(task: _Task); call it with the wrong type.

typeshed contract: task is _Task. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import WorkerContext
obj = object.__new__(WorkerContext)
try:
    obj.run(_W())  # task: _Task <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
