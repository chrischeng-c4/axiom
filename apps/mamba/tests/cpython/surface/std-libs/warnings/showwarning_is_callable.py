# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "showwarning_is_callable"
# subject = "warnings.showwarning"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.showwarning: showwarning_is_callable (surface)."""
import warnings

assert callable(warnings.showwarning)
print("showwarning_is_callable OK")
