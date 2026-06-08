# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_enumerate_is_present"
# subject = "threading.enumerate"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.enumerate: api_enumerate_is_present (surface)."""
import threading

assert hasattr(threading, "enumerate")
print("api_enumerate_is_present OK")
