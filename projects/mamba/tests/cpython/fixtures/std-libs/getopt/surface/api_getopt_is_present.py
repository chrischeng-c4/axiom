# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "api_getopt_is_present"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getopt.getopt: api_getopt_is_present (surface)."""
import getopt

assert hasattr(getopt, "getopt")
print("api_getopt_is_present OK")
