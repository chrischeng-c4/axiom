# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "intern_is_callable"
# subject = "sys.intern"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.intern: intern_is_callable (surface)."""
import sys

assert callable(sys.intern)
print("intern_is_callable OK")
