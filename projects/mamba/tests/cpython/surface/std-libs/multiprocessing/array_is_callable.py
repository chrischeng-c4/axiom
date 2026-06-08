# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "array_is_callable"
# subject = "multiprocessing.Array"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Array: array_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Array)
print("array_is_callable OK")
