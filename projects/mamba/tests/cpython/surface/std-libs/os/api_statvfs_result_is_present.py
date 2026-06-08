# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "surface"
# case = "api_statvfs_result_is_present"
# subject = "os.statvfs_result"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""os.statvfs_result: api_statvfs_result_is_present (surface)."""
import os

assert hasattr(os, "statvfs_result")
print("api_statvfs_result_is_present OK")
