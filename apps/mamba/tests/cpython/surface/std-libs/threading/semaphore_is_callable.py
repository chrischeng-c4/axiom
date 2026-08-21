# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "semaphore_is_callable"
# subject = "threading.Semaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Semaphore: semaphore_is_callable (surface)."""
import threading

assert callable(threading.Semaphore)
print("semaphore_is_callable OK")
