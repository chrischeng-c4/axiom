# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "surface"
# case = "error_is_exception_subclass"
# subject = "copy.Error"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""copy.Error: error_is_exception_subclass (surface)."""
import copy

assert callable(copy.Error)
print("error_is_exception_subclass OK")
