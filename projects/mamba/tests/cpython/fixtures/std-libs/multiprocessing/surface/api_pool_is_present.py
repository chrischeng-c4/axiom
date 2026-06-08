# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_pool_is_present"
# subject = "multiprocessing.Pool"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Pool: api_pool_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Pool")
print("api_pool_is_present OK")
