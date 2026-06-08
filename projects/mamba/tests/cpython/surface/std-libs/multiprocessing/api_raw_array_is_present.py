# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_raw_array_is_present"
# subject = "multiprocessing.RawArray"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.RawArray: api_raw_array_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "RawArray")
print("api_raw_array_is_present OK")
