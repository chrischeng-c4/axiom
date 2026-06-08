# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "getcontext_callable"
# subject = "decimal.getcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.getcontext: getcontext_callable (surface)."""
import decimal

assert callable(decimal.getcontext)
print("getcontext_callable OK")
