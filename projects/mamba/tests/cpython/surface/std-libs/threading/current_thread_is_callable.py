# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "current_thread_is_callable"
# subject = "threading.current_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.current_thread: current_thread_is_callable (surface)."""
import threading

assert callable(threading.current_thread)
print("current_thread_is_callable OK")
