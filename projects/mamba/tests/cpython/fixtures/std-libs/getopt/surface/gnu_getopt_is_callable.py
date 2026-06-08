# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getopt"
# dimension = "surface"
# case = "gnu_getopt_is_callable"
# subject = "getopt.gnu_getopt"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""getopt.gnu_getopt: gnu_getopt_is_callable (surface)."""
import getopt

assert callable(getopt.gnu_getopt)
print("gnu_getopt_is_callable OK")
