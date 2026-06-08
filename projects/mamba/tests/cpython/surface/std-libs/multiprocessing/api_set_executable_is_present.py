# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_set_executable_is_present"
# subject = "multiprocessing.set_executable"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.set_executable: api_set_executable_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "set_executable")
print("api_set_executable_is_present OK")
