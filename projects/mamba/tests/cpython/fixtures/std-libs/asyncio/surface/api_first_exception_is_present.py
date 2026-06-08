# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_first_exception_is_present"
# subject = "asyncio.FIRST_EXCEPTION"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.FIRST_EXCEPTION: api_first_exception_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "FIRST_EXCEPTION")
print("api_first_exception_is_present OK")
