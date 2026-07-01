# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "type"
# case = "Future__remove_done_callback__fn_as_Callable_wrong"
# subject = "_asyncio.Future.remove_done_callback(fn: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_asyncio.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _asyncio.Future.remove_done_callback(fn: Callable); call it with the wrong type.

typeshed contract: fn is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _asyncio import Future
obj = Future()
try:
    obj.remove_done_callback(_W())  # fn: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
