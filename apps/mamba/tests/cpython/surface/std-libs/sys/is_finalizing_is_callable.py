# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "is_finalizing_is_callable"
# subject = "sys.is_finalizing"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.is_finalizing: is_finalizing_is_callable (surface)."""
import sys

assert callable(sys.is_finalizing)
print("is_finalizing_is_callable OK")
