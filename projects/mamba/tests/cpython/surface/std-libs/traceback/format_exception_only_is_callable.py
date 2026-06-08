# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "format_exception_only_is_callable"
# subject = "traceback.format_exception_only"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception_only: format_exception_only_is_callable (surface)."""
import traceback

assert callable(traceback.format_exception_only)
print("format_exception_only_is_callable OK")
