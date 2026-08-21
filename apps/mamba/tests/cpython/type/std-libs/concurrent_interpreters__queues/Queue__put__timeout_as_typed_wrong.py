# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters__queues"
# dimension = "type"
# case = "Queue__put__timeout_as_typed_wrong"
# subject = "concurrent.interpreters._queues.Queue.put(timeout: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timeout"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters/_queues.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed timeout
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters._queues.Queue.put(timeout: typed); call it with the wrong type.

typeshed contract: timeout is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.interpreters._queues import Queue
obj = object.__new__(Queue)
try:
    obj.put(None, _W())  # timeout: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
