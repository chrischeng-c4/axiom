# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_timeout_max_is_present"
# subject = "threading.TIMEOUT_MAX"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.TIMEOUT_MAX: api_timeout_max_is_present (surface)."""
import threading

assert hasattr(threading, "TIMEOUT_MAX")
print("api_timeout_max_is_present OK")
