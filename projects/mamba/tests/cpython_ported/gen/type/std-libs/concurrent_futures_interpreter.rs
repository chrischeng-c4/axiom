use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/InterpreterPoolExecutor__init__max_workers_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_InterpreterPoolExecutor__init__max_workers_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "InterpreterPoolExecutor__init__max_workers_as_typed_wrong"
# subject = "concurrent.futures.interpreter.InterpreterPoolExecutor.__init__(max_workers: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.InterpreterPoolExecutor.__init__(max_workers: typed); call it with the wrong type.

typeshed contract: max_workers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import InterpreterPoolExecutor
try:
    InterpreterPoolExecutor(_W())  # max_workers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/InterpreterPoolExecutor__prepare_context__initializer_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_InterpreterPoolExecutor__prepare_context__initializer_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "InterpreterPoolExecutor__prepare_context__initializer_as_Callable_wrong"
# subject = "concurrent.futures.interpreter.InterpreterPoolExecutor.prepare_context(initializer: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.InterpreterPoolExecutor.prepare_context(initializer: Callable); call it with the wrong type.

typeshed contract: initializer is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import InterpreterPoolExecutor
try:
    InterpreterPoolExecutor.prepare_context(_W(), None)  # initializer: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/WorkerContext__init__initdata_as__Task_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_WorkerContext__init__initdata_as__Task_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "WorkerContext__init__initdata_as__Task_wrong"
# subject = "concurrent.futures.interpreter.WorkerContext.__init__(initdata: _Task)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.WorkerContext.__init__(initdata: _Task); call it with the wrong type.

typeshed contract: initdata is _Task. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import WorkerContext
try:
    WorkerContext(_W())  # initdata: _Task <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/WorkerContext__prepare__initializer_as_Callable_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_WorkerContext__prepare__initializer_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "WorkerContext__prepare__initializer_as_Callable_wrong"
# subject = "concurrent.futures.interpreter.WorkerContext.prepare(initializer: Callable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.WorkerContext.prepare(initializer: Callable); call it with the wrong type.

typeshed contract: initializer is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import WorkerContext
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

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/WorkerContext__run__task_as__Task_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_WorkerContext__run__task_as__Task_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "WorkerContext__run__task_as__Task_wrong"
# subject = "concurrent.futures.interpreter.WorkerContext.run(task: _Task)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.WorkerContext.run(task: _Task); call it with the wrong type.

typeshed contract: task is _Task. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import WorkerContext
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

/// Ported from `tests/cpython/type/std-libs/concurrent_futures_interpreter/do_call__results_as_Queue_wrong.py`.
#[test]
fn test_gen_type_std_libs_concurrent_futures_interpreter_do_call__results_as_Queue_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_interpreter"
# dimension = "type"
# case = "do_call__results_as_Queue_wrong"
# subject = "concurrent.futures.interpreter.do_call(results: Queue)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/interpreter.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.interpreter.do_call(results: Queue); call it with the wrong type.

typeshed contract: results is Queue. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.interpreter import do_call
try:
    do_call(_W(), None, None, None)  # results: Queue <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
