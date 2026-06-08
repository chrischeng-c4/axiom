# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "exc_info_is_callable"
# subject = "sys.exc_info"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.exc_info: exc_info_is_callable (surface)."""
import sys

assert callable(sys.exc_info)
print("exc_info_is_callable OK")
