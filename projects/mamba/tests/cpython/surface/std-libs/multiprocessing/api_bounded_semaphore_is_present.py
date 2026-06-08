# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_bounded_semaphore_is_present"
# subject = "multiprocessing.BoundedSemaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.BoundedSemaphore: api_bounded_semaphore_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "BoundedSemaphore")
print("api_bounded_semaphore_is_present OK")
