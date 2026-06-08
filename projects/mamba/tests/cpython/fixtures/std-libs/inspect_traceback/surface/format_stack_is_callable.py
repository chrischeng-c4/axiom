# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect_traceback"
# dimension = "surface"
# case = "format_stack_is_callable"
# subject = "traceback.format_stack"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.format_stack: format_stack_is_callable (surface)."""
import traceback

assert callable(traceback.format_stack)
print("format_stack_is_callable OK")
