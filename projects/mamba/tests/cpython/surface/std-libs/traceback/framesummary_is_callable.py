# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "traceback"
# dimension = "surface"
# case = "framesummary_is_callable"
# subject = "traceback.FrameSummary"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""traceback.FrameSummary: framesummary_is_callable (surface)."""
import traceback

assert callable(traceback.FrameSummary)
print("framesummary_is_callable OK")
