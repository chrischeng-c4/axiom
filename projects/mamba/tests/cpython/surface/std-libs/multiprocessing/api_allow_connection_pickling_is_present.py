# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing"
# dimension = "surface"
# case = "api_allow_connection_pickling_is_present"
# subject = "multiprocessing.allow_connection_pickling"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""multiprocessing.allow_connection_pickling: api_allow_connection_pickling_is_present (surface)."""
import multiprocessing

assert hasattr(multiprocessing, "allow_connection_pickling")
print("api_allow_connection_pickling_is_present OK")
