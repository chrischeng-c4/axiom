# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_times_result_is_present"
# subject = "os.times_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.times_result: api_times_result_is_present (surface)."""
import os

assert hasattr(os, "times_result")
print("api_times_result_is_present OK")
