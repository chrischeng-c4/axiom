# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_value_is_present"
# subject = "multiprocessing.Value"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Value: api_value_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Value")
print("api_value_is_present OK")
