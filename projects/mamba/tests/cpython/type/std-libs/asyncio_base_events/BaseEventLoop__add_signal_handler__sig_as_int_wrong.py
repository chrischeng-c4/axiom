# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_events"
# dimension = "type"
# case = "BaseEventLoop__add_signal_handler__sig_as_int_wrong"
# subject = "asyncio.base_events.BaseEventLoop.add_signal_handler(sig: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sig"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_events.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed sig
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_events.BaseEventLoop.add_signal_handler(sig: int); call it with the wrong type.

typeshed contract: sig is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.base_events import BaseEventLoop
obj = object.__new__(BaseEventLoop)
try:
    obj.add_signal_handler("not_an_int", None)  # sig: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
