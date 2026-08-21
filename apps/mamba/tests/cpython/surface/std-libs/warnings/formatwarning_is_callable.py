# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "formatwarning_is_callable"
# subject = "warnings.formatwarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.formatwarning: formatwarning_is_callable (surface)."""
import warnings

assert callable(warnings.formatwarning)
print("formatwarning_is_callable OK")
