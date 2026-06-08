# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "decimal"
# dimension = "surface"
# case = "context_has_rounding"
# subject = "decimal.Context"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""decimal.Context: context_has_rounding (surface)."""
import decimal

assert hasattr(decimal.Context, "rounding")
print("context_has_rounding OK")
