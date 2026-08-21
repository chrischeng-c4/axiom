# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "rlock_is_callable"
# subject = "threading.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.RLock: rlock_is_callable (surface)."""
import threading

assert callable(threading.RLock)
print("rlock_is_callable OK")
