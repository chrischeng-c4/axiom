# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "api_struct_time_is_present"
# subject = "time.struct_time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""time.struct_time: api_struct_time_is_present (surface)."""
import time

assert hasattr(time, "struct_time")
print("api_struct_time_is_present OK")
