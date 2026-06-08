# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "value_is_callable"
# subject = "multiprocessing.Value"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.Value: value_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.Value)
print("value_is_callable OK")
