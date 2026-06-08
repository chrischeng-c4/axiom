# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_barrier_is_present"
# subject = "multiprocessing.Barrier"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Barrier: api_barrier_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Barrier")
print("api_barrier_is_present OK")
