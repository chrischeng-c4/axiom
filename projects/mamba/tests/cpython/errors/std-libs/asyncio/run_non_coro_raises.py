# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "errors"
# case = "run_non_coro_raises"
# subject = "asyncio.run"
# kind = "mechanical"
# xfail = "mamba asyncio shim: asyncio.run(42) returns silently instead of raising (mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.run: run_non_coro_raises (errors)."""
import asyncio

_raised = False
try:
    asyncio.run(42)
except ValueError:
    _raised = True
assert _raised, "run_non_coro_raises: expected ValueError"
print("run_non_coro_raises OK")
