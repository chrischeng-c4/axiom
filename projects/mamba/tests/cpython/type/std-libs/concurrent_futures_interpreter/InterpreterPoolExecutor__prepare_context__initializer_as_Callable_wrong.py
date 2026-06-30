# /// script
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
