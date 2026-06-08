# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "current_process_is_callable"
# subject = "multiprocessing.current_process"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""multiprocessing.current_process: current_process_is_callable (surface)."""
import multiprocessing

assert callable(multiprocessing.current_process)
print("current_process_is_callable OK")
