# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_parent_process_is_present"
# subject = "multiprocessing.parent_process"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.parent_process: api_parent_process_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "parent_process")
print("api_parent_process_is_present OK")
