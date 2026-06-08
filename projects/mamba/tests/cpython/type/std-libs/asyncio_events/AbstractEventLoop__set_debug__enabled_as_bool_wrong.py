# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_events"
# dimension = "type"
# case = "AbstractEventLoop__set_debug__enabled_as_bool_wrong"
# subject = "asyncio.events.AbstractEventLoop.set_debug(enabled: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enabled"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed enabled
# mamba-strict-type: TypeError
"""Type wall: asyncio.events.AbstractEventLoop.set_debug(enabled: bool); call it with the wrong type.

typeshed contract: enabled is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.events import AbstractEventLoop
obj = object.__new__(AbstractEventLoop)
try:
    obj.set_debug("not_a_bool")  # enabled: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
