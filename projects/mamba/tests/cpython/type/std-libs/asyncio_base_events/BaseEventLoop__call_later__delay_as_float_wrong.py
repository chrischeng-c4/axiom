# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_events"
# dimension = "type"
# case = "BaseEventLoop__call_later__delay_as_float_wrong"
# subject = "asyncio.base_events.BaseEventLoop.call_later(delay: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_events.BaseEventLoop.call_later(delay: float); call it with the wrong type.

typeshed contract: delay is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.base_events import BaseEventLoop
obj = object.__new__(BaseEventLoop)
try:
    obj.call_later("not_a_float", None)  # delay: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
