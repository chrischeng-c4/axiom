# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "zero_width_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""pprint.PrettyPrinter: zero_width_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(width=0)
except ValueError:
    _raised = True
assert _raised, "zero_width_raises: expected ValueError"
print("zero_width_raises OK")
