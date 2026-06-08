# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "defaultaction_attr"
# subject = "warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: defaultaction_attr (surface)."""
import warnings

assert hasattr(warnings, "defaultaction")
print("defaultaction_attr OK")
