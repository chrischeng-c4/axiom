# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_get_all_start_methods_is_present"
# subject = "multiprocessing.get_all_start_methods"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.get_all_start_methods: api_get_all_start_methods_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "get_all_start_methods")
print("api_get_all_start_methods_is_present OK")
