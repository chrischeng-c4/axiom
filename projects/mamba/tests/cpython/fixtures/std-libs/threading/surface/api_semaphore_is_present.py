# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_semaphore_is_present"
# subject = "threading.Semaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.Semaphore: api_semaphore_is_present (surface)."""
import threading

assert hasattr(threading, "Semaphore")
print("api_semaphore_is_present OK")
