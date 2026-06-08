# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "process_is_callable"
# subject = "multiprocessing.Process"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Process: process_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Process)
print("process_is_callable OK")
