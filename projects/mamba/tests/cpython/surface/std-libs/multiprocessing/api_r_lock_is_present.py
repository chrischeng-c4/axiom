# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_r_lock_is_present"
# subject = "multiprocessing.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.RLock: api_r_lock_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "RLock")
print("api_r_lock_is_present OK")
