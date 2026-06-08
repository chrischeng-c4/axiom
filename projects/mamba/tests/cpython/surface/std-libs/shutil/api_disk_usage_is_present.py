# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "api_disk_usage_is_present"
# subject = "shutil.disk_usage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""shutil.disk_usage: api_disk_usage_is_present (surface)."""
import shutil

assert hasattr(shutil, "disk_usage")
print("api_disk_usage_is_present OK")
