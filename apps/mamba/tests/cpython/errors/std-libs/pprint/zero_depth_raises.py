# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "zero_depth_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: zero_depth_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(depth=0)
except ValueError:
    _raised = True
assert _raised, "zero_depth_raises: expected ValueError"
print("zero_depth_raises OK")
