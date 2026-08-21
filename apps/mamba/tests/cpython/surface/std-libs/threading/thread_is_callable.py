# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "thread_is_callable"
# subject = "threading.Thread"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Thread: thread_is_callable (surface)."""
import threading

assert callable(threading.Thread)
print("thread_is_callable OK")
