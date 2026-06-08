# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "filterwarnings_is_callable"
# subject = "warnings.filterwarnings"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.filterwarnings: filterwarnings_is_callable (surface)."""
import warnings

assert callable(warnings.filterwarnings)
print("filterwarnings_is_callable OK")
