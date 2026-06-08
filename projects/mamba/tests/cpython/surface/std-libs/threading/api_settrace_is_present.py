# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_settrace_is_present"
# subject = "threading.settrace"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.settrace: api_settrace_is_present (surface)."""
import threading

assert hasattr(threading, "settrace")
print("api_settrace_is_present OK")
