# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_broken_barrier_error_is_present"
# subject = "threading.BrokenBarrierError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.BrokenBarrierError: api_broken_barrier_error_is_present (surface)."""
import threading

assert hasattr(threading, "BrokenBarrierError")
print("api_broken_barrier_error_is_present OK")
