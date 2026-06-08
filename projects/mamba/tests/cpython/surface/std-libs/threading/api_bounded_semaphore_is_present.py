# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_bounded_semaphore_is_present"
# subject = "threading.BoundedSemaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.BoundedSemaphore: api_bounded_semaphore_is_present (surface)."""
import threading

assert hasattr(threading, "BoundedSemaphore")
print("api_bounded_semaphore_is_present OK")
