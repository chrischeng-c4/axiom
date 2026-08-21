# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "rlock_is_callable"
# subject = "multiprocessing.RLock"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.RLock: rlock_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.RLock)
print("rlock_is_callable OK")
