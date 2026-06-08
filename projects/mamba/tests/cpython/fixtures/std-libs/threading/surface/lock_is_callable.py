# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "lock_is_callable"
# subject = "threading.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock: lock_is_callable (surface)."""
import threading

assert callable(threading.Lock)
print("lock_is_callable OK")
