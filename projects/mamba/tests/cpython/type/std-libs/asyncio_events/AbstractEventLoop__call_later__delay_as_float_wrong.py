# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_events"
# dimension = "type"
# case = "AbstractEventLoop__call_later__delay_as_float_wrong"
# subject = "asyncio.events.AbstractEventLoop.call_later(delay: float)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed delay"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed delay
# mamba-strict-type: TypeError
"""Type wall: asyncio.events.AbstractEventLoop.call_later(delay: float); call it with the wrong type.

typeshed contract: delay is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.events import AbstractEventLoop
obj = object.__new__(AbstractEventLoop)
try:
    obj.call_later("not_a_float", None)  # delay: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
