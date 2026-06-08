# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_stat_result_is_present"
# subject = "os.stat_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.stat_result: api_stat_result_is_present (surface)."""
import os

assert hasattr(os, "stat_result")
print("api_stat_result_is_present OK")
