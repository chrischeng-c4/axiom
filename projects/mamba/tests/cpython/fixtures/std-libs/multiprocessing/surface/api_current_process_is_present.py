# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_current_process_is_present"
# subject = "multiprocessing.current_process"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.current_process: api_current_process_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "current_process")
print("api_current_process_is_present OK")
