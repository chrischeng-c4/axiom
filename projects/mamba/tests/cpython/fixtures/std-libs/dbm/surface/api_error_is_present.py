# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dbm"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "dbm.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""dbm.error: api_error_is_present (surface)."""
import dbm

assert hasattr(dbm, "error")
print("api_error_is_present OK")
