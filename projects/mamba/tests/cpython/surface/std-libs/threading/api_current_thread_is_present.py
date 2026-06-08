# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_current_thread_is_present"
# subject = "threading.current_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.current_thread: api_current_thread_is_present (surface)."""
import threading

assert hasattr(threading, "current_thread")
print("api_current_thread_is_present OK")
