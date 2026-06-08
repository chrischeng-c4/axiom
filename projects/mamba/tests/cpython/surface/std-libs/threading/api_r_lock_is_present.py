# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_r_lock_is_present"
# subject = "threading.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.RLock: api_r_lock_is_present (surface)."""
import threading

assert hasattr(threading, "RLock")
print("api_r_lock_is_present OK")
