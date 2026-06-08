# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "format_exc_is_callable"
# subject = "traceback.format_exc"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_exc: format_exc_is_callable (surface)."""
import traceback

assert callable(traceback.format_exc)
print("format_exc_is_callable OK")
