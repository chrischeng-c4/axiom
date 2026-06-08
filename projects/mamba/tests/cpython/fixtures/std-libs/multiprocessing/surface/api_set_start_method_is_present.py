# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_set_start_method_is_present"
# subject = "multiprocessing.set_start_method"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.set_start_method: api_set_start_method_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "set_start_method")
print("api_set_start_method_is_present OK")
