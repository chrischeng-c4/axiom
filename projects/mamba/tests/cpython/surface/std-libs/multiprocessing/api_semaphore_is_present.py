# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_semaphore_is_present"
# subject = "multiprocessing.Semaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Semaphore: api_semaphore_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Semaphore")
print("api_semaphore_is_present OK")
