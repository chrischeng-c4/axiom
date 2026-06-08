# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "cpu_count_is_callable"
# subject = "multiprocessing.cpu_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.cpu_count: cpu_count_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.cpu_count)
print("cpu_count_is_callable OK")
