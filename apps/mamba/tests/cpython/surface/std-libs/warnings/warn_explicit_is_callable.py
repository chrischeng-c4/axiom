# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "warn_explicit_is_callable"
# subject = "warnings.warn_explicit"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn_explicit: warn_explicit_is_callable (surface)."""
import warnings

assert callable(warnings.warn_explicit)
print("warn_explicit_is_callable OK")
