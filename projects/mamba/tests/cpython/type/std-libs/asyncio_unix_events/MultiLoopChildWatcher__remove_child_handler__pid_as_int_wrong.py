# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "MultiLoopChildWatcher__remove_child_handler__pid_as_int_wrong"
# subject = "asyncio.unix_events.MultiLoopChildWatcher.remove_child_handler(pid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.MultiLoopChildWatcher.remove_child_handler(pid: int); call it with the wrong type.

typeshed contract: pid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.unix_events import MultiLoopChildWatcher
obj = object.__new__(MultiLoopChildWatcher)
try:
    obj.remove_child_handler("not_an_int")  # pid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
