# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "wait_for_is_callable"
# subject = "asyncio.wait_for"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.wait_for: wait_for_is_callable (surface)."""
import asyncio

assert callable(asyncio.wait_for)
print("wait_for_is_callable OK")
