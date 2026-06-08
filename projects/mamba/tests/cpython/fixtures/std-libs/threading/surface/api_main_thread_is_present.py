# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "api_main_thread_is_present"
# subject = "threading.main_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""threading.main_thread: api_main_thread_is_present (surface)."""
import threading

assert hasattr(threading, "main_thread")
print("api_main_thread_is_present OK")
