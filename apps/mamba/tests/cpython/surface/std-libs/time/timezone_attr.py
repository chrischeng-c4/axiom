# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "timezone_attr"
# subject = "time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time: timezone_attr (surface)."""
import time

assert hasattr(time, "timezone")
print("timezone_attr OK")
