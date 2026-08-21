# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "main_thread_is_callable"
# subject = "threading.main_thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.main_thread: main_thread_is_callable (surface)."""
import threading

assert callable(threading.main_thread)
print("main_thread_is_callable OK")
