# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "AsyncExitStack__enter_async_context__cm_as_AbstractAsyncContextManager_wrong"
# subject = "contextlib.AsyncExitStack.enter_async_context(cm: AbstractAsyncContextManager)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cm"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cm
# mamba-strict-type: TypeError
"""Type wall: contextlib.AsyncExitStack.enter_async_context(cm: AbstractAsyncContextManager); call it with the wrong type.

typeshed contract: cm is AbstractAsyncContextManager. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import AsyncExitStack
obj = object.__new__(AsyncExitStack)
try:
    obj.enter_async_context(_W())  # cm: AbstractAsyncContextManager <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
