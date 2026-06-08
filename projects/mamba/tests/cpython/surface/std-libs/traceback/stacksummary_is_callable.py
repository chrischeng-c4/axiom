# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "stacksummary_is_callable"
# subject = "traceback.StackSummary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.StackSummary: stacksummary_is_callable (surface)."""
import traceback

assert callable(traceback.StackSummary)
print("stacksummary_is_callable OK")
