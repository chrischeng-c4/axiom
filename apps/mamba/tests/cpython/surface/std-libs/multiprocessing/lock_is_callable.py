# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "lock_is_callable"
# subject = "multiprocessing.Lock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Lock: lock_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Lock)
print("lock_is_callable OK")
