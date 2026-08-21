# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "import_datetime"
# subject = "datetime"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""datetime: import_datetime (surface)."""
import datetime

assert hasattr(datetime, "date")
print("import_datetime OK")
