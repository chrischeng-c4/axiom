# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "fromisoformat_is_callable"
# subject = "datetime.date.fromisoformat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime.date.fromisoformat: fromisoformat_is_callable (surface)."""
import datetime

assert callable(datetime.date.fromisoformat)
print("fromisoformat_is_callable OK")
