# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "surface"
# case = "disk_usage_is_callable"
# subject = "shutil.disk_usage"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""shutil.disk_usage: disk_usage_is_callable (surface)."""
import shutil

assert callable(shutil.disk_usage)
print("disk_usage_is_callable OK")
