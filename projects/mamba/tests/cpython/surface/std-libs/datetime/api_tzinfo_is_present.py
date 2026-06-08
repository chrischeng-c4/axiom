# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_tzinfo_is_present"
# subject = "datetime.tzinfo"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.tzinfo: api_tzinfo_is_present (surface)."""
import datetime

assert hasattr(datetime, "tzinfo")
print("api_tzinfo_is_present OK")
