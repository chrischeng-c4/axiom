# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_blocking_io_error_is_present"
# subject = "io.BlockingIOError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.BlockingIOError: api_blocking_io_error_is_present (surface)."""
import io

assert hasattr(io, "BlockingIOError")
print("api_blocking_io_error_is_present OK")
