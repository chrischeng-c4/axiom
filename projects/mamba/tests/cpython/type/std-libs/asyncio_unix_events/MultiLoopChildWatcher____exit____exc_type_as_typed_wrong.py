# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "MultiLoopChildWatcher____exit____exc_type_as_typed_wrong"
# subject = "asyncio.unix_events.MultiLoopChildWatcher.__exit__(exc_type: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed exc_type
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.MultiLoopChildWatcher.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import MultiLoopChildWatcher
obj = object.__new__(MultiLoopChildWatcher)
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
