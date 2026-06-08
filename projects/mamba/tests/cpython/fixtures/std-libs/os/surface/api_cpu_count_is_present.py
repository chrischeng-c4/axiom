# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_cpu_count_is_present"
# subject = "os.cpu_count"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.cpu_count: api_cpu_count_is_present (surface)."""
import os

assert hasattr(os, "cpu_count")
print("api_cpu_count_is_present OK")
