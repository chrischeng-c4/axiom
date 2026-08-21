# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "pool_is_callable"
# subject = "multiprocessing.Pool"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Pool: pool_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Pool)
print("pool_is_callable OK")
