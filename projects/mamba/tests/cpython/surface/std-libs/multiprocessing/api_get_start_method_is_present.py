# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_get_start_method_is_present"
# subject = "multiprocessing.get_start_method"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.get_start_method: api_get_start_method_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "get_start_method")
print("api_get_start_method_is_present OK")
