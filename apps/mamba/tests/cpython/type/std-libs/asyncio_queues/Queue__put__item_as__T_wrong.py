# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_queues"
# dimension = "type"
# case = "Queue__put__item_as__T_wrong"
# subject = "asyncio.queues.Queue.put(item: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed item"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/queues.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed item
# mamba-strict-type: TypeError
"""Type wall: asyncio.queues.Queue.put(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.queues import Queue
obj = object.__new__(Queue)
try:
    obj.put(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
