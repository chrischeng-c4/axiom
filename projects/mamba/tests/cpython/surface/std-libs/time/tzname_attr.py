# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "time"
# dimension = "surface"
# case = "tzname_attr"
# subject = "time"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""time: tzname_attr (surface)."""
import time

assert hasattr(time, "tzname")
print("tzname_attr OK")
