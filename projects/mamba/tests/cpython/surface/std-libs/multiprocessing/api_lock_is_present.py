# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_lock_is_present"
# subject = "multiprocessing.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Lock: api_lock_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Lock")
print("api_lock_is_present OK")
