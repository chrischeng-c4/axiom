# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "resetwarnings_is_callable"
# subject = "warnings.resetwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.resetwarnings: resetwarnings_is_callable (surface)."""
import warnings

assert callable(warnings.resetwarnings)
print("resetwarnings_is_callable OK")
