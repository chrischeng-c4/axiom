# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "minyear_attr"
# subject = "datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: minyear_attr (surface)."""
import datetime

assert hasattr(datetime, "MINYEAR")
print("minyear_attr OK")
