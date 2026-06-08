# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_interpreters__queues"
# dimension = "type"
# case = "Queue____new____id_as_int_wrong"
# subject = "concurrent.interpreters._queues.Queue.__new__(id: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/interpreters/_queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: concurrent.interpreters._queues.Queue.__new__(id: int); call it with the wrong type.

typeshed contract: id is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from concurrent.interpreters._queues import Queue
obj = object.__new__(Queue)
try:
    obj.__new__("not_an_int")  # id: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
