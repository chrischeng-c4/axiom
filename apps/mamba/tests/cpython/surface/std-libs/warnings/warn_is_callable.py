# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "warn_is_callable"
# subject = "warnings.warn"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn_is_callable (surface)."""
import warnings

assert callable(warnings.warn)
print("warn_is_callable OK")
