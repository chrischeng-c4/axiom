# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_queue"
# dimension = "type"
# case = "SimpleQueue__put__item_as__T_wrong"
# subject = "_queue.SimpleQueue.put(item: _T)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _queue.SimpleQueue.put(item: _T); call it with the wrong type.

typeshed contract: item is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _queue import SimpleQueue
obj = object.__new__(SimpleQueue)
try:
    obj.put(_W())  # item: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
