# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures__base"
# dimension = "type"
# case = "Executor__submit__fn_as_Callable_wrong"
# subject = "concurrent.futures._base.Executor.submit(fn: Callable)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fn"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/_base.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed fn
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures._base.Executor.submit(fn: Callable); call it with the wrong type.

typeshed contract: fn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures._base import Executor
obj = object.__new__(Executor)
try:
    obj.submit(_W())  # fn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
