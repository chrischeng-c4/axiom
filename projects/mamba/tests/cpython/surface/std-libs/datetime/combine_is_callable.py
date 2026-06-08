# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "combine_is_callable"
# subject = "datetime.datetime.combine"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.combine: combine_is_callable (surface)."""
import datetime

assert callable(datetime.datetime.combine)
print("combine_is_callable OK")
