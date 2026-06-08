# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "utc_alias_attr"
# subject = "datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: utc_alias_attr (surface)."""
import datetime

assert hasattr(datetime, "UTC")
print("utc_alias_attr OK")
