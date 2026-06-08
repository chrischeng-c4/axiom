# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_buffer_too_short_is_present"
# subject = "multiprocessing.BufferTooShort"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.BufferTooShort: api_buffer_too_short_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "BufferTooShort")
print("api_buffer_too_short_is_present OK")
