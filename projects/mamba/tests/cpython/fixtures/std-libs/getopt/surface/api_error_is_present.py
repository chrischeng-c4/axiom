# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "api_error_is_present"
# subject = "getopt.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getopt.error: api_error_is_present (surface)."""
import getopt

assert hasattr(getopt, "error")
print("api_error_is_present OK")
