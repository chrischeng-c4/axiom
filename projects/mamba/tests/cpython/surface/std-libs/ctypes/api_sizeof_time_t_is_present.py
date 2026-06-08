# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ctypes"
# dimension = "surface"
# case = "api_sizeof_time_t_is_present"
# subject = "ctypes.SIZEOF_TIME_T"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""ctypes.SIZEOF_TIME_T: api_sizeof_time_t_is_present (surface)."""
import ctypes

assert hasattr(ctypes, "SIZEOF_TIME_T")
print("api_sizeof_time_t_is_present OK")
