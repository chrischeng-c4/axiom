# /// script
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
