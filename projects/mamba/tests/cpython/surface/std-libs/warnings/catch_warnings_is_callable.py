# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "catch_warnings_is_callable"
# subject = "warnings.catch_warnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.catch_warnings: catch_warnings_is_callable (surface)."""
import warnings

assert callable(warnings.catch_warnings)
print("catch_warnings_is_callable OK")
