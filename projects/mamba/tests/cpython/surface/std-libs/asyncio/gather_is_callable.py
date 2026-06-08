# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "gather_is_callable"
# subject = "asyncio.gather"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""asyncio.gather: gather_is_callable (surface)."""
import asyncio

assert callable(asyncio.gather)
print("gather_is_callable OK")
