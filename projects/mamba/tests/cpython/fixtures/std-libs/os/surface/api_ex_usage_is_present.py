# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_ex_usage_is_present"
# subject = "os.EX_USAGE"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.EX_USAGE: api_ex_usage_is_present (surface)."""
import os

assert hasattr(os, "EX_USAGE")
print("api_ex_usage_is_present OK")
