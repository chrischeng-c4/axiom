# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters"
# dimension = "type"
# case = "Interpreter__call_in_thread__callable_as_Callable_wrong"
# subject = "concurrent.interpreters.Interpreter.call_in_thread(callable: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callable"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed callable
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters.Interpreter.call_in_thread(callable: Callable); call it with the wrong type.

typeshed contract: callable is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters import Interpreter
obj = object.__new__(Interpreter)
try:
    obj.call_in_thread(_W())  # callable: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
