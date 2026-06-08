# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_incomplete_read_error_is_present"
# subject = "asyncio.IncompleteReadError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.IncompleteReadError: api_incomplete_read_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "IncompleteReadError")
print("api_incomplete_read_error_is_present OK")
