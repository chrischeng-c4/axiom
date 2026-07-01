# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "type"
# case = "Task__init__coro_as__TaskCompatibleCoro_wrong"
# subject = "_asyncio.Task.__init__(coro: _TaskCompatibleCoro)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_asyncio.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _asyncio.Task.__init__(coro: _TaskCompatibleCoro); call it with the wrong type.

typeshed contract: coro is _TaskCompatibleCoro. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _asyncio import Task
try:
    Task(_W())  # coro: _TaskCompatibleCoro <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
