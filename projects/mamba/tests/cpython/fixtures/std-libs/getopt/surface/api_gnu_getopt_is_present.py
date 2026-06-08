# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "api_gnu_getopt_is_present"
# subject = "getopt.gnu_getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""getopt.gnu_getopt: api_gnu_getopt_is_present (surface)."""
import getopt

assert hasattr(getopt, "gnu_getopt")
print("api_gnu_getopt_is_present OK")
