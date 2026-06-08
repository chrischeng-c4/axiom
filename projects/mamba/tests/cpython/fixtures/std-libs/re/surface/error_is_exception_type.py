# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "surface"
# case = "error_is_exception_type"
# subject = "re.error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""re.error: error_is_exception_type (surface)."""
import re

assert hasattr(re.error, "__cause__")
print("error_is_exception_type OK")
