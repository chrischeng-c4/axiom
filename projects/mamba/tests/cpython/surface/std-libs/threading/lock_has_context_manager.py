# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "surface"
# case = "lock_has_context_manager"
# subject = "threading.Lock()"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""threading.Lock(): lock_has_context_manager (surface)."""
import threading

assert hasattr(threading.Lock(), "__enter__")
print("lock_has_context_manager OK")
