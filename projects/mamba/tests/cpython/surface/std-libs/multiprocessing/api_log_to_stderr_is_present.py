# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_log_to_stderr_is_present"
# subject = "multiprocessing.log_to_stderr"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.log_to_stderr: api_log_to_stderr_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "log_to_stderr")
print("api_log_to_stderr_is_present OK")
