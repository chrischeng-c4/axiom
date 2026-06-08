# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_handle_is_present"
# subject = "asyncio.Handle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.Handle: api_handle_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "Handle")
print("api_handle_is_present OK")
