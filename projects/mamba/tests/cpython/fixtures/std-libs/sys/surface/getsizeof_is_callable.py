# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "getsizeof_is_callable"
# subject = "sys.getsizeof"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getsizeof: getsizeof_is_callable (surface)."""
import sys

assert callable(sys.getsizeof)
print("getsizeof_is_callable OK")
