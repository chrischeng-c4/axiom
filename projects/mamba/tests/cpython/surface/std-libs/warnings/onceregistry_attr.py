# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "onceregistry_attr"
# subject = "warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings: onceregistry_attr (surface)."""
import warnings

assert hasattr(warnings, "onceregistry")
print("onceregistry_attr OK")
