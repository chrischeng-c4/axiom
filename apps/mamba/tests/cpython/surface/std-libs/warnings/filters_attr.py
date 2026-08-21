# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "filters_attr"
# subject = "warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: filters_attr (surface)."""
import warnings

assert hasattr(warnings, "filters")
print("filters_attr OK")
