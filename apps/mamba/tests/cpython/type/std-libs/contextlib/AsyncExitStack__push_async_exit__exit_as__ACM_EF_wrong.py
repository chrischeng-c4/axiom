# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "AsyncExitStack__push_async_exit__exit_as__ACM_EF_wrong"
# subject = "contextlib.AsyncExitStack.push_async_exit(exit: _ACM_EF)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exit"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exit
# mamba-strict-type: TypeError
"""Type wall: contextlib.AsyncExitStack.push_async_exit(exit: _ACM_EF); call it with the wrong type.

typeshed contract: exit is _ACM_EF. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import AsyncExitStack
obj = object.__new__(AsyncExitStack)
try:
    obj.push_async_exit(_W())  # exit: _ACM_EF <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
