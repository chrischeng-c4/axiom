# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_process_is_present"
# subject = "multiprocessing.Process"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Process: api_process_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Process")
print("api_process_is_present OK")
