# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "surface"
# case = "format_exception_is_callable"
# subject = "traceback.format_exception"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exception: format_exception_is_callable (surface)."""
import traceback

assert callable(traceback.format_exception)
print("format_exception_is_callable OK")
