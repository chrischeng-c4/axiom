# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "surface"
# case = "simplefilter_is_callable"
# subject = "warnings.simplefilter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.simplefilter: simplefilter_is_callable (surface)."""
import warnings

assert callable(warnings.simplefilter)
print("simplefilter_is_callable OK")
