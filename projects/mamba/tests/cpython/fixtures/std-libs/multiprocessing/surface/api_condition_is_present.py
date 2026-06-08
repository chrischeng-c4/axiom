# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_condition_is_present"
# subject = "multiprocessing.Condition"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.Condition: api_condition_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "Condition")
print("api_condition_is_present OK")
