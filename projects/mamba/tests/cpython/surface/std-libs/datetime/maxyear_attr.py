# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "maxyear_attr"
# subject = "datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: maxyear_attr (surface)."""
import datetime

assert hasattr(datetime, "MAXYEAR")
print("maxyear_attr OK")
