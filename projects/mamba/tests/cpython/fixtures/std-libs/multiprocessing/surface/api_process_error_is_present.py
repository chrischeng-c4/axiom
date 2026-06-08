# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_process_error_is_present"
# subject = "multiprocessing.ProcessError"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.ProcessError: api_process_error_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "ProcessError")
print("api_process_error_is_present OK")
