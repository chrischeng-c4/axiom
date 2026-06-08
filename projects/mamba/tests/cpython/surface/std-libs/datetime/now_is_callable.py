# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "now_is_callable"
# subject = "datetime.datetime.now"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.datetime.now: now_is_callable (surface)."""
import datetime

assert callable(datetime.datetime.now)
print("now_is_callable OK")
