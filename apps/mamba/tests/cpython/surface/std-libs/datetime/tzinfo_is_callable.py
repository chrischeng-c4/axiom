# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "tzinfo_is_callable"
# subject = "datetime.tzinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.tzinfo: tzinfo_is_callable (surface)."""
import datetime

assert callable(datetime.tzinfo)
print("tzinfo_is_callable OK")
