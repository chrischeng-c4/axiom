# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "context_has_prec"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Context: context_has_prec (surface)."""
import decimal

assert hasattr(decimal.Context, "prec")
print("context_has_prec OK")
