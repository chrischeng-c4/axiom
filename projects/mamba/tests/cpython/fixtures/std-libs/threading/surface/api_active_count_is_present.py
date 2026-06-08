# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_active_count_is_present"
# subject = "threading.active_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.active_count: api_active_count_is_present (surface)."""
import threading

assert hasattr(threading, "active_count")
print("api_active_count_is_present OK")
