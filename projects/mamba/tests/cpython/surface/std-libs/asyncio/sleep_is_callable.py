# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "sleep_is_callable"
# subject = "asyncio.sleep"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.sleep: sleep_is_callable (surface)."""
import asyncio

assert callable(asyncio.sleep)
print("sleep_is_callable OK")
