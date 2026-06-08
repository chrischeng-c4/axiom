# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "BaseChildWatcher__attach_loop__loop_as_typed_wrong"
# subject = "asyncio.unix_events.BaseChildWatcher.attach_loop(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.BaseChildWatcher.attach_loop(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import BaseChildWatcher
obj = object.__new__(BaseChildWatcher)
try:
    obj.attach_loop(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
