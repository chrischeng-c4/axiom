# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "getopt_is_callable"
# subject = "getopt.getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""getopt.getopt: getopt_is_callable (surface)."""
import getopt

assert callable(getopt.getopt)
print("getopt_is_callable OK")
