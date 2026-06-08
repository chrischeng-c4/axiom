# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_events"
# dimension = "type"
# case = "TimerHandle____lt____other_as_TimerHandle_wrong"
# subject = "asyncio.events.TimerHandle.__lt__(other: TimerHandle)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.events.TimerHandle.__lt__(other: TimerHandle); call it with the wrong type.

typeshed contract: other is TimerHandle. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.events import TimerHandle
obj = object.__new__(TimerHandle)
try:
    obj.__lt__(_W())  # other: TimerHandle <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
