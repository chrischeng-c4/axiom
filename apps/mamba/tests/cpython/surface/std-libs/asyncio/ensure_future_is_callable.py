# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "ensure_future_is_callable"
# subject = "asyncio.ensure_future"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.ensure_future: ensure_future_is_callable (surface)."""
import asyncio

assert callable(asyncio.ensure_future)
print("ensure_future_is_callable OK")
