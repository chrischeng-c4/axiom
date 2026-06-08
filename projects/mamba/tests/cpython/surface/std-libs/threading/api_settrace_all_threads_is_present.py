# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_settrace_all_threads_is_present"
# subject = "threading.settrace_all_threads"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.settrace_all_threads: api_settrace_all_threads_is_present (surface)."""
import threading

assert hasattr(threading, "settrace_all_threads")
print("api_settrace_all_threads_is_present OK")
