# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "localcontext_callable"
# subject = "decimal.localcontext"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.localcontext: localcontext_callable (surface)."""
import decimal

assert callable(decimal.localcontext)
print("localcontext_callable OK")
