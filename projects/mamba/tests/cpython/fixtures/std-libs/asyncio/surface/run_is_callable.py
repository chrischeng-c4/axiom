# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "run_is_callable"
# subject = "asyncio.run"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.run: run_is_callable (surface)."""
import asyncio

assert callable(asyncio.run)
print("run_is_callable OK")
