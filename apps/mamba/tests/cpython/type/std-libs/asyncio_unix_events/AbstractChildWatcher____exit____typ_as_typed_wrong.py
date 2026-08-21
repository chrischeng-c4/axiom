# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_unix_events"
# dimension = "type"
# case = "AbstractChildWatcher____exit____typ_as_typed_wrong"
# subject = "asyncio.unix_events.AbstractChildWatcher.__exit__(typ: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typ"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/unix_events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed typ
# mamba-strict-type: TypeError
"""Type wall: asyncio.unix_events.AbstractChildWatcher.__exit__(typ: typed); call it with the wrong type.

typeshed contract: typ is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.unix_events import AbstractChildWatcher
obj = object.__new__(AbstractChildWatcher)
try:
    obj.__exit__(_W(), None, None)  # typ: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
