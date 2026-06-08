# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "surface"
# case = "api_reporting_flags_is_present"
# subject = "doctest.REPORTING_FLAGS"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""doctest.REPORTING_FLAGS: api_reporting_flags_is_present (surface)."""
import doctest

assert hasattr(doctest, "REPORTING_FLAGS")
print("api_reporting_flags_is_present OK")
