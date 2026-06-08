# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio"
# dimension = "surface"
# case = "api_sendfile_not_available_error_is_present"
# subject = "asyncio.SendfileNotAvailableError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""asyncio.SendfileNotAvailableError: api_sendfile_not_available_error_is_present (surface)."""
import asyncio

assert hasattr(asyncio, "SendfileNotAvailableError")
print("api_sendfile_not_available_error_is_present OK")
