# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "surface"
# case = "getswitchinterval_is_callable"
# subject = "sys.getswitchinterval"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.getswitchinterval: getswitchinterval_is_callable (surface)."""
import sys

assert callable(sys.getswitchinterval)
print("getswitchinterval_is_callable OK")
