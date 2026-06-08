# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "bounded_semaphore_is_callable"
# subject = "threading.BoundedSemaphore"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.BoundedSemaphore: bounded_semaphore_is_callable (surface)."""
import threading

assert callable(threading.BoundedSemaphore)
print("bounded_semaphore_is_callable OK")
