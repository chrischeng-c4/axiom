# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "behavior"
# case = "event_wakes_waiter_after_set"
# subject = "asyncio.Event"
# kind = "semantic"
# xfail = "mamba asyncio shim: asyncio.Event not implemented (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.Event: an asyncio.Event wakes a waiting coroutine only after another coroutine calls set(); the waiter observes 'setting' before 'woke'"""
import asyncio


async def _main():
    _ev = asyncio.Event()
    _log = []

    async def _waiter():
        _log.append("waiting")
        await _ev.wait()
        _log.append("woke")

    async def _setter():
        await asyncio.sleep(0)
        _log.append("setting")
        _ev.set()

    await asyncio.gather(_waiter(), _setter())
    assert "waiting" in _log, "waiter started"
    assert "setting" in _log, "setter ran"
    assert "woke" in _log, "waiter woke"
    assert _log.index("setting") < _log.index("woke"), "setter before woke"


asyncio.run(_main())

print("event_wakes_waiter_after_set OK")
