# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "datetime"
# dimension = "surface"
# case = "api_minyear_is_present"
# subject = "datetime.MINYEAR"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""datetime.MINYEAR: api_minyear_is_present (surface)."""
import datetime

assert hasattr(datetime, "MINYEAR")
print("api_minyear_is_present OK")
