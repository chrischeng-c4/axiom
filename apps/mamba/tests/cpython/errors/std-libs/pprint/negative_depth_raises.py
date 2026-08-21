# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "errors"
# case = "negative_depth_raises"
# subject = "pprint.PrettyPrinter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.PrettyPrinter: negative_depth_raises (errors)."""
import pprint

_raised = False
try:
    pprint.PrettyPrinter(depth=-1)
except ValueError:
    _raised = True
assert _raised, "negative_depth_raises: expected ValueError"
print("negative_depth_raises OK")
