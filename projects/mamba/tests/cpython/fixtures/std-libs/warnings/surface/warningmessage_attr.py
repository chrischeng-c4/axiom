# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "warningmessage_attr"
# subject = "warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: warningmessage_attr (surface)."""
import warnings

assert hasattr(warnings, "WarningMessage")
print("warningmessage_attr OK")
